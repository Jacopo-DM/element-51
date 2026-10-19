"""Edits to the graph, and the rules that decide whether they are allowed.

The browser applies an edit at once and asks Python to confirm it. Both sides
run the same rules; `tests/fixtures/link_cases.json` is read by the tests of
both, so they cannot drift apart unnoticed.
"""

from collections.abc import Mapping
from enum import StrEnum
from typing import Annotated, Literal, assert_never

from pydantic import Field

from element_51.model import (
    Graph,
    Link,
    Model,
    NodeId,
    NodeType,
    PlacedNode,
    Port,
    PortRef,
    PortValue,
    can_link,
    value_fits,
)


class EditErrorKind(StrEnum):
    """Why an edit was refused. The values are part of the wire format."""

    UNKNOWN_NODE_TYPE = "unknown_node_type"
    ID_IN_USE = "id_in_use"
    UNKNOWN_NODE = "unknown_node"
    UNKNOWN_PORT = "unknown_port"
    TYPE_MISMATCH = "type_mismatch"
    INPUT_TAKEN = "input_taken"
    CYCLE = "cycle"
    UNKNOWN_LINK = "unknown_link"
    BAD_VALUE = "bad_value"


class EditError(Exception):
    """An edit the rules refuse.

    Parameters
    ----------
    kind
        The category of refusal.
    reason
        A sentence for the person who sees it.
    """

    def __init__(self, kind: EditErrorKind, reason: str) -> None:
        super().__init__(reason)
        self.kind = kind
        self.reason = reason


class AddNode(Model):
    """Place a node. The browser proposes the id."""

    kind: Literal["add_node"] = "add_node"
    id: NodeId
    node_type: str
    x: float
    y: float


class AddLink(Model):
    """Link an output port to an input port."""

    kind: Literal["add_link"] = "add_link"
    source: PortRef
    target: PortRef


class MoveNode(Model):
    """Put a node at a new position."""

    kind: Literal["move_node"] = "move_node"
    id: NodeId
    x: float
    y: float


class DeleteNode(Model):
    """Remove a node, and every link that touches it."""

    kind: Literal["delete_node"] = "delete_node"
    id: NodeId


class DeleteLink(Model):
    """Remove one link."""

    kind: Literal["delete_link"] = "delete_link"
    source: PortRef
    target: PortRef


class SetValue(Model):
    """Give an input its own value, or take it away with `None`."""

    kind: Literal["set_value"] = "set_value"
    node: NodeId
    port: str
    value: PortValue | None


type AnyEdit = AddNode | AddLink | MoveNode | DeleteNode | DeleteLink | SetValue
type Edit = Annotated[AnyEdit, Field(discriminator="kind")]


def apply_edit(
    graph: Graph, node_types: Mapping[str, NodeType], edit: AnyEdit
) -> Graph:
    """Return the graph after `edit`, or refuse it.

    Parameters
    ----------
    graph
        The graph before the edit.
    node_types
        Every node type, by name.
    edit
        The edit to apply.

    Returns
    -------
    Graph
        A new graph. The old one is unchanged.
    """
    match edit:
        case AddNode():
            return _add_node(graph, node_types, edit)
        case AddLink():
            return _add_link(graph, node_types, edit)
        case MoveNode():
            return _move_node(graph, edit)
        case DeleteNode():
            return _delete_node(graph, edit)
        case DeleteLink():
            return _delete_link(graph, edit)
        case SetValue():
            return _set_value(graph, node_types, edit)
        case _:  # pragma: no cover  # unreachable: the match is exhaustive
            assert_never(edit)


def _add_node(
    graph: Graph, node_types: Mapping[str, NodeType], edit: AddNode
) -> Graph:
    if edit.node_type not in node_types:
        msg = f"unknown node type {edit.node_type!r}"
        raise EditError(EditErrorKind.UNKNOWN_NODE_TYPE, msg)
    if any(node.id == edit.id for node in graph.nodes):
        msg = f"node id {edit.id} is already in use"
        raise EditError(EditErrorKind.ID_IN_USE, msg)
    node = PlacedNode(id=edit.id, node_type=edit.node_type, x=edit.x, y=edit.y)
    return Graph(nodes=(*graph.nodes, node), links=graph.links)


def _add_link(
    graph: Graph, node_types: Mapping[str, NodeType], edit: AddLink
) -> Graph:
    source = _port(graph, node_types, edit.source, output=True)
    target = _port(graph, node_types, edit.target, output=False)
    if not can_link(source.kind, target.kind):
        msg = f"cannot link {source.kind} to {target.kind}"
        raise EditError(EditErrorKind.TYPE_MISMATCH, msg)
    if any(link.target == edit.target for link in graph.links):
        msg = f"{edit.target.node}.{edit.target.port} already has a link"
        raise EditError(EditErrorKind.INPUT_TAKEN, msg)
    if _reaches(graph, edit.target.node, edit.source.node):
        msg = "that link would make a cycle"
        raise EditError(EditErrorKind.CYCLE, msg)
    link = Link(source=edit.source, target=edit.target)
    return Graph(nodes=graph.nodes, links=(*graph.links, link))


def _move_node(graph: Graph, edit: MoveNode) -> Graph:
    _require_node(graph, edit.id)
    nodes = tuple(
        node.model_copy(update={"x": edit.x, "y": edit.y})
        if node.id == edit.id
        else node
        for node in graph.nodes
    )
    return Graph(nodes=nodes, links=graph.links)


def _delete_node(graph: Graph, edit: DeleteNode) -> Graph:
    _require_node(graph, edit.id)
    return Graph(
        nodes=tuple(node for node in graph.nodes if node.id != edit.id),
        links=tuple(
            link
            for link in graph.links
            if edit.id not in {link.source.node, link.target.node}
        ),
    )


def _delete_link(graph: Graph, edit: DeleteLink) -> Graph:
    gone = Link(source=edit.source, target=edit.target)
    if gone not in graph.links:
        msg = (
            f"no link from {edit.source.node}.{edit.source.port} "
            f"to {edit.target.node}.{edit.target.port}"
        )
        raise EditError(EditErrorKind.UNKNOWN_LINK, msg)
    return Graph(
        nodes=graph.nodes,
        links=tuple(link for link in graph.links if link != gone),
    )


def _set_value(
    graph: Graph, node_types: Mapping[str, NodeType], edit: SetValue
) -> Graph:
    port = _port(
        graph, node_types, PortRef(node=edit.node, port=edit.port), output=False
    )
    if edit.value is not None and not value_fits(edit.value, port.kind):
        msg = f"{edit.value!r} is not a {port.kind} for {edit.node}.{port.name}"
        raise EditError(EditErrorKind.BAD_VALUE, msg)
    nodes = tuple(
        _with_value(node, edit) if node.id == edit.node else node
        for node in graph.nodes
    )
    return Graph(nodes=nodes, links=graph.links)


def _with_value(node: PlacedNode, edit: SetValue) -> PlacedNode:
    values = {k: v for k, v in node.values.items() if k != edit.port}
    if edit.value is not None:
        values[edit.port] = edit.value
    return node.model_copy(update={"values": values})


def _require_node(graph: Graph, node_id: str) -> None:
    if not any(node.id == node_id for node in graph.nodes):
        msg = f"unknown node {node_id}"
        raise EditError(EditErrorKind.UNKNOWN_NODE, msg)


def _port(
    graph: Graph,
    node_types: Mapping[str, NodeType],
    ref: PortRef,
    *,
    output: bool,
) -> Port:
    """Find the port `ref` names, as an output or as an input.

    Returns
    -------
    Port
        The port.

    Raises
    ------
    EditError
        If the node or the port does not exist.
    """
    node = next((n for n in graph.nodes if n.id == ref.node), None)
    if node is None:
        msg = f"unknown node {ref.node}"
        raise EditError(EditErrorKind.UNKNOWN_NODE, msg)
    node_type = node_types[node.node_type]
    ports = node_type.outputs if output else node_type.inputs
    port = next((p for p in ports if p.name == ref.port), None)
    if port is None:
        side = "output" if output else "input"
        msg = f"node {ref.node} has no {side} port {ref.port!r}"
        raise EditError(EditErrorKind.UNKNOWN_PORT, msg)
    return port


def _reaches(graph: Graph, start: str, goal: str) -> bool:
    """Whether following links forward from node `start` reaches `goal`.

    Returns
    -------
    bool
        True if `goal` is `start` or is downstream of it.
    """
    seen: set[str] = set()
    stack = [start]
    while stack:
        current = stack.pop()
        if current == goal:
            return True
        if current in seen:
            continue
        seen.add(current)
        stack.extend(
            link.target.node
            for link in graph.links
            if link.source.node == current
        )
    return False
