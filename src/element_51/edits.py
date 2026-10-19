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
    Node,
    NodeId,
    NodeType,
    Port,
    PortRef,
    can_link,
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


type Edit = Annotated[AddNode | AddLink, Field(discriminator="kind")]


def apply_edit(
    graph: Graph, node_types: Mapping[str, NodeType], edit: AddNode | AddLink
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
    node = Node(id=edit.id, node_type=edit.node_type, x=edit.x, y=edit.y)
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
