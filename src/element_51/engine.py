"""Run a graph: call each node's function in dependency order."""

from collections import deque
from collections.abc import Mapping, Sequence
from typing import Literal

from element_51.model import (
    Graph,
    Link,
    Model,
    PlacedNode,
    PortValue,
)
from element_51.nodes import Registered
from element_51.visuals import Carried, Image, carries


class NodeOutcome(Model):
    """What happened to one node in a run.

    `outputs` holds the plain values by output port name. `shown` holds what
    the card displays, by port name: the pictures a node produced. The page
    reads `shown` for the card and `outputs` for the boxes beside the ports.
    """

    node: str
    status: Literal["ok", "error", "skipped"]
    outputs: Mapping[str, PortValue] = {}
    shown: Mapping[str, Image] = {}
    error: str | None = None


class RunResult(Model):
    """The outcome of every node, in the order they ran."""

    op: Literal["run_result"] = "run_result"
    outcomes: tuple[NodeOutcome, ...]


def run_graph(
    graph: Graph,
    functions: Mapping[str, Registered],
    *,
    raise_errors: bool,
) -> RunResult:
    """Run every node once, upstream nodes first.

    A node takes each input from the link into it, or else from the value the
    user gave the node, or else from its function's default. A node that fails
    is marked `error`; every node downstream of it is `skipped`; independent
    nodes still run.

    Parameters
    ----------
    graph
        The graph to run.
    functions
        The node functions, by node type name.
    raise_errors
        If true, the first failure is raised, with a note naming the node,
        instead of being recorded.

    Returns
    -------
    RunResult
        One outcome per node.

    Raises
    ------
    ValueError
        If `raise_errors` is true and the graph has a cycle or a node has an
        input with no value.
    """
    order = _order(graph)
    if len(order) < len(graph.nodes):
        if raise_errors:
            msg = "the graph has a cycle"
            raise ValueError(msg)
        return RunResult(
            outcomes=tuple(
                NodeOutcome(
                    node=node.id,
                    status="error",
                    error="the graph has a cycle",
                )
                for node in graph.nodes
            )
        )
    outcomes: dict[str, NodeOutcome] = {}
    outputs_by_node: dict[str, Mapping[str, Carried]] = {}
    # Indexed once, so each node finds its links without scanning them all.
    into: dict[str, list[Link]] = {}
    for link in graph.links:
        into.setdefault(link.target.node, []).append(link)
    for node in order:
        incoming = into.get(node.id, ())
        upstream = (link.source.node for link in incoming)
        if any(outcomes[source].status != "ok" for source in upstream):
            outcomes[node.id] = NodeOutcome(node=node.id, status="skipped")
            continue
        try:
            outputs = _evaluate(
                node, incoming, functions[node.node_type], outputs_by_node
            )
        except Exception as error:  # node code can raise anything
            if raise_errors:
                error.add_note(f"in node {node.id} ({node.node_type})")
                raise
            outcomes[node.id] = NodeOutcome(
                node=node.id,
                status="error",
                error=_describe(error),
            )
            continue
        outputs_by_node[node.id] = outputs
        outcomes[node.id] = _outcome(node.id, outputs)
    return RunResult(outcomes=tuple(outcomes.values()))


def _outcome(node: str, outputs: Mapping[str, Carried]) -> NodeOutcome:
    """Split a node's outputs into plain values and pictures.

    Returns
    -------
    NodeOutcome
        An `ok` outcome.
    """
    plain: dict[str, PortValue] = {}
    shown: dict[str, Image] = {}
    for name, value in outputs.items():
        if isinstance(value, Image):
            shown[name] = value
        else:
            plain[name] = value
    return NodeOutcome(node=node, status="ok", outputs=plain, shown=shown)


class _MissingInputError(ValueError):
    """An input has neither a link nor a default."""


class _BadReturnError(TypeError):
    """A node function returned a value its output port cannot carry."""


def _describe(error: Exception) -> str:
    """Return an error as one line.

    The engine's own errors are already worded for a person. Anything a node
    function raised gets its type name in front.

    Returns
    -------
    str
        The error, worded for a person.
    """
    if isinstance(error, _MissingInputError | _BadReturnError):
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
) -> Mapping[str, Carried]:
    """Call one node's function and check what comes back.

    Returns
    -------
    Mapping[str, Carried]
        The value of each of the node's output ports.

    Raises
    ------
    _MissingInputError
        If an input has neither a link nor a default.
    """
    inputs: dict[str, Carried] = {}
    for port in function.node_type.inputs:
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
        else:
            msg = f"input {port.name!r} has no value"
            raise _MissingInputError(msg)
    return _collect_outputs(function, function.call(inputs))


def _collect_outputs(
    function: Registered, result: object
) -> Mapping[str, Carried]:
    """Check what a node returned and turn it into a value per output port.

    Returns
    -------
    Mapping[str, Carried]
        The value of each output port, by port name.

    Raises
    ------
    _BadReturnError
        If the node returned something its output ports cannot carry.
    """
    model = function.output_model
    if model is None:
        port = function.node_type.outputs[0]
        if carries(result, port.kind):
            return {port.name: result}
        msg = f"returned {type(result).__name__}, expected {port.kind}"
        raise _BadReturnError(msg)
    if not isinstance(result, model):
        msg = f"returned {type(result).__name__}, expected {model.__name__}"
        raise _BadReturnError(msg)
    outputs: dict[str, Carried] = {}
    for port in function.node_type.outputs:
        value = getattr(result, port.name)
        if not carries(value, port.kind):
            msg = (
                f"output {port.name!r} is {type(value).__name__}, "
                f"expected {port.kind}"
            )
            raise _BadReturnError(msg)
        outputs[port.name] = value
    return outputs
