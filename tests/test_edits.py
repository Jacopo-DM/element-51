"""Tests for the edit rules, including the cases the Rust side also runs."""

from pathlib import Path

import pytest
from pydantic import TypeAdapter

from element_51.edits import (
    AddLink,
    AddNode,
    Edit,
    EditError,
    EditErrorKind,
    apply_edit,
)
from element_51.model import Graph, Link, Model, NodeType, PlacedNode, PortRef

FIXTURE = Path(__file__).parent / "fixtures" / "link_cases.json"


class Case(Model):
    """One candidate link and the expected outcome."""

    name: str
    source: PortRef
    target: PortRef
    result: str


class LinkCases(Model):
    """The shared fixture: a graph and the links to try on it."""

    node_types: tuple[NodeType, ...]
    graph: Graph
    cases: tuple[Case, ...]


CASES = LinkCases.model_validate_json(FIXTURE.read_text())
NODE_TYPES = {t.name: t for t in CASES.node_types}


@pytest.mark.parametrize("case", CASES.cases, ids=lambda case: case.name)
def test_link_rules_match_the_shared_cases(case: Case) -> None:
    """Python gives the same verdict the Rust side gives on each case."""
    edit = AddLink(source=case.source, target=case.target)
    if case.result == "ok":
        graph = apply_edit(CASES.graph, NODE_TYPES, edit)
        assert Link(source=case.source, target=case.target) in graph.links
        assert len(graph.links) == len(CASES.graph.links) + 1
        return
    with pytest.raises(EditError) as raised:
        apply_edit(CASES.graph, NODE_TYPES, edit)
    assert raised.value.kind == EditErrorKind(case.result)
    assert raised.value.reason


def test_add_node_places_a_node() -> None:
    """A valid add_node returns a graph that holds the new node."""
    edit = AddNode(id="n8", node_type="add", x=1.0, y=2.0)
    graph = apply_edit(CASES.graph, NODE_TYPES, edit)
    assert graph.nodes[-1] == PlacedNode(id="n8", node_type="add", x=1.0, y=2.0)
    assert graph.links == CASES.graph.links


def test_add_node_of_an_unknown_type_is_refused() -> None:
    """The browser can only place types Python announced."""
    edit = AddNode(id="n8", node_type="nope", x=0.0, y=0.0)
    with pytest.raises(EditError) as raised:
        apply_edit(CASES.graph, NODE_TYPES, edit)
    assert raised.value.kind is EditErrorKind.UNKNOWN_NODE_TYPE


def test_add_node_with_a_used_id_is_refused() -> None:
    """Two nodes never share an id."""
    edit = AddNode(id="n1", node_type="add", x=0.0, y=0.0)
    with pytest.raises(EditError) as raised:
        apply_edit(CASES.graph, NODE_TYPES, edit)
    assert raised.value.kind is EditErrorKind.ID_IN_USE


def test_node_ids_must_look_like_n_and_a_number() -> None:
    """`next_id` relies on this format, so the model enforces it."""
    with pytest.raises(ValueError, match="pattern"):
        AddNode(id="five", node_type="add", x=0.0, y=0.0)


def test_a_diamond_in_the_graph_is_searched_once() -> None:
    """Two paths to one node do not confuse the cycle check."""
    ports = (("n1", "n2", "a"), ("n1", "n3", "a"), ("n2", "n4", "a"))
    links = (
        *tuple(
            Link(
                source=PortRef(node=src, port="out"),
                target=PortRef(node=dst, port=port),
            )
            for src, dst, port in ports
        ),
        Link(
            source=PortRef(node="n3", port="out"),
            target=PortRef(node="n4", port="b"),
        ),
    )
    nodes = tuple(
        PlacedNode(id=f"n{i}", node_type="add", x=0.0, y=0.0)
        for i in range(1, 6)
    )
    graph = Graph(nodes=nodes, links=links)
    edit = AddLink(
        source=PortRef(node="n5", port="out"),
        target=PortRef(node="n1", port="a"),
    )
    assert len(apply_edit(graph, NODE_TYPES, edit).links) == len(links) + 1


EDIT_FIXTURE = Path(__file__).parent / "fixtures" / "edit_cases.json"
EDIT = TypeAdapter(Edit)


class EditCase(Model):
    """One edit, with the graph it should give or the reason it is refused."""

    name: str
    edit: dict[str, object]
    result: str
    graph: Graph | None = None


class EditCases(Model):
    """The shared fixture for move and delete edits."""

    node_types: tuple[NodeType, ...]
    graph: Graph
    cases: tuple[EditCase, ...]


EDIT_CASES = EditCases.model_validate_json(EDIT_FIXTURE.read_text())
EDIT_NODE_TYPES = {t.name: t for t in EDIT_CASES.node_types}


@pytest.mark.parametrize("case", EDIT_CASES.cases, ids=lambda case: case.name)
def test_move_and_delete_match_the_shared_cases(case: EditCase) -> None:
    """Python gives the graph, or the refusal, the Rust side gives."""
    edit = EDIT.validate_python(case.edit)
    if case.result == "ok":
        graph = apply_edit(EDIT_CASES.graph, EDIT_NODE_TYPES, edit)
        assert graph == case.graph
        return
    with pytest.raises(EditError) as raised:
        apply_edit(EDIT_CASES.graph, EDIT_NODE_TYPES, edit)
    assert raised.value.kind == EditErrorKind(case.result)
    assert raised.value.reason
