"""Run a graph: call each node's function in dependency order."""

from collections import deque
from collections.abc import Iterable, Mapping, Sequence
from collections.abc import Set as AbstractSet
from typing import Literal

from element_51.model import (
    Graph,
    Link,
    Model,
    NodeType,
    PlacedNode,
    Port,
    PortType,
    PortValue,
    variadic_index,
)
from element_51.nodes import Registered
from element_51.visuals import Carried, Image, Series, Shown, carries


class NodeOutcome(Model):
    """What happened to one node in a run.

    `outputs` holds the plain values by output port name. `shown` holds what
    the card displays, by port name: the pictures a node produced, or the
    series linked to a chart node by input port name. The page reads `shown`
    for the card and `outputs` for the boxes beside the ports.
    """

    node: str
    status: Literal["ok", "error", "skipped"]
    outputs: Mapping[str, PortValue] = {}
    shown: Mapping[str, Shown] = {}
    error: str | None = None


class RunResult(Model):
    """The outcome of every node, in the order they ran.

    A `partial` result covers only some nodes (the ones after a job that
    ended): the page merges it into its last result, node by node, instead of
    replacing that result.
    """

    op: Literal["run_result"] = "run_result"
    outcomes: tuple[NodeOutcome, ...]
    partial: bool = False


NO_OUTPUT = "no output yet: press Start"


def run_graph(
    graph: Graph,
    functions: Mapping[str, Registered],
    *,
    raise_errors: bool,
    only: AbstractSet[str] | None = None,
    fixed: Mapping[str, Mapping[str, Carried]] | None = None,
) -> RunResult:
    """Run every node once, upstream nodes first.

    A node takes each input from the link into it, or else from the value the
    user gave the node, or else from its function's default. A node that fails
    is marked `error`; every node downstream of it is `skipped`; independent
    nodes still run.

    A job node is never run here. If `fixed` holds outputs for it they are used
    as if it had run; otherwise it is `skipped`, with the reason in `error`.

    Parameters
    ----------
    graph
        The graph to run.
    functions
        The node functions, by node type name.
    raise_errors
        If true, the first failure is raised, with a note naming the node,
        instead of being recorded.
    only
        Run just these nodes, and report just these. It must hold everything
        its nodes need, or those nodes must be in `fixed` (see `closure`).
        None runs the whole graph.
    fixed
        Nodes that are not run, with their outputs by port name. Nodes that
        link from them take these values.

    Returns
    -------
    RunResult
        One outcome per node.

    With `raise_errors`, a cycle, an input with no value or a job node with no
    outputs in `fixed` raises a `ValueError`.
    """
    fixed = fixed or {}
    order = _order(graph)
    if len(order) < len(graph.nodes):
        return _cycle(graph, raise_errors=raise_errors)
    outcomes: dict[str, NodeOutcome] = {}
    outputs_by_node: dict[str, Mapping[str, Carried]] = {}
    # Indexed once, so each node finds its links without scanning them all.
    into: dict[str, list[Link]] = {}
    for link in graph.links:
        into.setdefault(link.target.node, []).append(link)
    for node in _visited(order, only, fixed):
        if node.id in fixed:
            outputs_by_node[node.id] = fixed[node.id]
            outcomes[node.id] = _outcome(
                node.id,
                functions[node.node_type].node_type,
                (fixed[node.id], {}),
            )
            continue
        incoming = into.get(node.id, ())
        upstream = (link.source.node for link in incoming)
        if any(outcomes[source].status != "ok" for source in upstream):
            outcomes[node.id] = NodeOutcome(node=node.id, status="skipped")
            continue
        if (
            node.node_type in functions
            and functions[node.node_type].node_type.job
        ):
            outcomes[node.id] = _no_output(node, raise_errors=raise_errors)
            continue
        try:
            function = functions[node.node_type]
            outputs, inputs = _evaluate(
                node, incoming, function, outputs_by_node
            )
        except Exception as error:  # node code can raise anything
            if raise_errors:
                error.add_note(f"in node {node.id} ({node.node_type})")
                raise
            outcomes[node.id] = NodeOutcome(
                node=node.id,
                status="error",
                error=describe_error(error),
            )
            continue
        outputs_by_node[node.id] = outputs
        outcomes[node.id] = _outcome(
            node.id, function.node_type, (outputs, inputs)
        )
    return RunResult(
        outcomes=tuple(
            outcome
            for node_id, outcome in outcomes.items()
            if only is None or node_id in only
        )
    )


def _cycle(graph: Graph, *, raise_errors: bool) -> RunResult:
    """Report a graph with a cycle: every node is an error.

    Returns
    -------
    RunResult
        One `error` outcome per node.

    Raises
    ------
    ValueError
        If `raise_errors` is true.
    """
    msg = "the graph has a cycle"
    if raise_errors:
        raise ValueError(msg)
    return RunResult(
        outcomes=tuple(
            NodeOutcome(node=node.id, status="error", error=msg)
            for node in graph.nodes
        )
    )


def _visited(
    order: list[PlacedNode],
    only: AbstractSet[str] | None,
    fixed: Mapping[str, Mapping[str, Carried]],
) -> list[PlacedNode]:
    """Pick the nodes a run goes through: those in `only`, and the fixed ones.

    Returns
    -------
    list[PlacedNode]
        The nodes of `order` to visit, in that order.
    """
    if only is None:
        return order
    return [node for node in order if node.id in only or node.id in fixed]


def _no_output(node: PlacedNode, *, raise_errors: bool) -> NodeOutcome:
    """Say that a job node has no outputs to give.

    With `raise_errors` it raises a `ValueError` with a note naming the node.

    Returns
    -------
    NodeOutcome
        A `skipped` outcome whose `error` is the reason.
    """
    if raise_errors:
        error = ValueError(NO_OUTPUT)
        error.add_note(f"in node {node.id} ({node.node_type})")
        raise error
    return NodeOutcome(node=node.id, status="skipped", error=NO_OUTPUT)


def ancestors(graph: Graph, node: str) -> frozenset[str]:
    """Return the nodes `node` needs, directly or not.

    Returns
    -------
    frozenset[str]
        The ids of every node with a path of links to `node`, not `node`.
    """
    return _reach(graph, [node], upstream=True) - {node}


def descendants(graph: Graph, node: str) -> frozenset[str]:
    """Return the nodes that need `node`, directly or not.

    Returns
    -------
    frozenset[str]
        The ids of every node `node` has a path of links to, not `node`.
    """
    return _reach(graph, [node], upstream=False) - {node}


def closure(graph: Graph, nodes: Iterable[str]) -> frozenset[str]:
    """Return `nodes` and everything they need.

    Returns
    -------
    frozenset[str]
        The given ids and the ids of all their ancestors.
    """
    return _reach(graph, nodes, upstream=True)


def _reach(
    graph: Graph, start: Iterable[str], *, upstream: bool
) -> frozenset[str]:
    """Follow links from `start`, against their direction if `upstream`.

    Returns
    -------
    frozenset[str]
        `start` and every node reached.
    """
    next_to: dict[str, list[str]] = {}
    for link in graph.links:
        source, target = link.source.node, link.target.node
        key, value = (target, source) if upstream else (source, target)
        next_to.setdefault(key, []).append(value)
    seen = set(start)
    todo = list(seen)
    while todo:
        for found in next_to.get(todo.pop(), ()):
            if found not in seen:
                seen.add(found)
                todo.append(found)
    return frozenset(seen)


def _outcome(
    node: str,
    node_type: NodeType,
    values: tuple[Mapping[str, Carried], Mapping[str, Carried]],
) -> NodeOutcome:
    """Split a node's values into plain outputs and what its card shows.

    A node with a picture output shows its pictures. A chart node shows the
    series linked to its inputs. A series output is also sent, so the page can
    write its length and last value in the output's box; only a chart draws it.

    Parameters
    ----------
    node
        The node id.
    node_type
        The node's type, which says what its card shows.
    values
        The node's outputs and its inputs, by port name.

    Returns
    -------
    NodeOutcome
        An `ok` outcome.
    """
    outputs, inputs = values
    plain: dict[str, PortValue] = {}
    shown: dict[str, Shown] = {}
    for name, value in outputs.items():
        if isinstance(value, Image | Series):
            shown[name] = value
        else:
            plain[name] = value
    if node_type.display == "chart":
        shown.update({
            name: value
            for name, value in inputs.items()
            if isinstance(value, Series)
        })
    return NodeOutcome(node=node, status="ok", outputs=plain, shown=shown)


class MissingInputError(ValueError):
    """An input has neither a link nor a default."""


class BadReturnError(TypeError):
    """A node function returned a value its output port cannot carry."""


def describe_error(error: Exception) -> str:
    """Return an error as one line.

    The engine's own errors are already worded for a person. Anything a node
    function raised gets its type name in front.

    Returns
    -------
    str
        The error, worded for a person.
    """
    if isinstance(error, MissingInputError | BadReturnError):
        return str(error)
    return f"{type(error).__name__}: {error}"


def _order(graph: Graph) -> list[PlacedNode]:
    """Return the nodes, each after everything it depends on.

    Nodes that depend on nothing keep their order in the graph. Nodes on a
    cycle, or downstream of one, are left out.

    Returns
    -------
    list[PlacedNode]
        The nodes that can run, in running order.
    """
    waiting = {node.id: 0 for node in graph.nodes}
    leaving: dict[str, list[Link]] = {}
    for link in graph.links:
        waiting[link.target.node] += 1
        leaving.setdefault(link.source.node, []).append(link)
    by_id = {node.id: node for node in graph.nodes}
    ready = deque(node for node in graph.nodes if waiting[node.id] == 0)
    order: list[PlacedNode] = []
    while ready:
        node = ready.popleft()
        order.append(node)
        for link in leaving.get(node.id, ()):
            waiting[link.target.node] -= 1
            if waiting[link.target.node] == 0:
                ready.append(by_id[link.target.node])
    return order


def _evaluate(
    node: PlacedNode,
    incoming: Sequence[Link],
    function: Registered,
    outputs_by_node: Mapping[str, Mapping[str, Carried]],
) -> tuple[Mapping[str, Carried], Mapping[str, Carried]]:
    """Call one node's function and check what comes back.

    Returns
    -------
    tuple[Mapping[str, Carried], Mapping[str, Carried]]
        The value of each of the node's output ports, and of each input.
    """
    inputs = resolve_inputs(node, incoming, function, outputs_by_node)
    return collect_outputs(function, function.call(inputs)), inputs


def resolve_inputs(
    node: PlacedNode,
    incoming: Sequence[Link],
    function: Registered,
    outputs_by_node: Mapping[str, Mapping[str, Carried]],
) -> dict[str, Carried]:
    """Find the value of each of a node's inputs.

    An input takes its link, then the node's own value, then the default. A
    chart node takes no value for a series input nothing is linked to: that is
    an empty row, a place to link, not a missing value.

    Parameters
    ----------
    node
        The node in the graph.
    incoming
        The links into the node.
    function
        The node's registered type.
    outputs_by_node
        The outputs of every node that has run, by node id and port name.

    Returns
    -------
    dict[str, Carried]
        The value of each input by port name.

    Raises
    ------
    MissingInputError
        If an input has neither a link nor a default.
    """
    inputs: dict[str, Carried] = {}
    for port in _input_ports(function.node_type, incoming):
        link = next(
            (link for link in incoming if link.target.port == port.name),
            None,
        )
        if link is not None:
            inputs[port.name] = outputs_by_node[link.source.node][
                link.source.port
            ]
        elif port.name in node.values:
            inputs[port.name] = node.values[port.name]
        elif port.name in function.defaults:
            inputs[port.name] = function.defaults[port.name]
        elif (
            function.node_type.display == "chart"
            and port.kind is PortType.SERIES
        ):
            continue
        else:
            msg = f"input {port.name!r} has no value"
            raise MissingInputError(msg)
    return inputs


def _input_ports(node_type: NodeType, incoming: Sequence[Link]) -> list[Port]:
    """List the inputs a node has in this graph, rows that grew included.

    Returns
    -------
    list[Port]
        The listed inputs, then each linked row of a variadic type that is not
        listed, in order of row number.
    """
    ports = list(node_type.inputs)
    variadic = node_type.variadic
    if variadic is None:
        return ports
    known = {port.name for port in ports}
    rows = sorted(
        (index, link.target.port)
        for link in incoming
        if link.target.port not in known
        and (index := variadic_index(variadic, link.target.port)) is not None
    )
    return [*ports, *(Port(name=name, kind=variadic.kind) for _, name in rows)]


def collect_outputs(
    function: Registered, result: object
) -> Mapping[str, Carried]:
    """Check what a node returned and turn it into a value per output port.

    Returns
    -------
    Mapping[str, Carried]
        The value of each output port, by port name.

    Raises
    ------
    BadReturnError
        If the node returned something its output ports cannot carry.
    """
    model = function.output_model
    if not function.node_type.outputs:
        return {}
    if model is None:
        port = function.node_type.outputs[0]
        if carries(result, port.kind):
            return {port.name: result}
        msg = f"returned {type(result).__name__}, expected {port.kind}"
        raise BadReturnError(msg)
    if not isinstance(result, model):
        msg = f"returned {type(result).__name__}, expected {model.__name__}"
        raise BadReturnError(msg)
    outputs: dict[str, Carried] = {}
    for port in function.node_type.outputs:
        value = getattr(result, port.name)
        if not carries(value, port.kind):
            msg = (
                f"output {port.name!r} is {type(value).__name__}, "
                f"expected {port.kind}"
            )
            raise BadReturnError(msg)
        outputs[port.name] = value
    return outputs
