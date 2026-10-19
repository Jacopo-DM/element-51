"""Tests for the live session."""

import pytest
from sample_nodes import add, show

from element_51.edits import AddLink, AddNode, EditError
from element_51.model import Link, PortRef, PortType
from element_51.nodes import Registered, node
from element_51.session import App


def test_snapshot_lists_node_types_in_registration_order() -> None:
    """The browser gets every type, whether or not a node uses it."""
    app = App([add, show])
    snapshot = app.snapshot()
    assert [t.name for t in snapshot.node_types] == ["add", "show"]
    assert snapshot.node_types[0].outputs[0].kind is PortType.FLOAT
    assert snapshot.graph.nodes == ()


def test_add_node_places_a_node_with_a_fresh_id() -> None:
    """Ids count up from n1 and positions are kept."""
    app = App([add, show])
    first = app.add_node(add, x=10.0, y=20.0)
    second = app.add_node(show, x=30.0, y=40.0)
    assert (first, second) == ("n1", "n2")
    nodes = app.graph.nodes
    assert [(n.id, n.node_type, n.x, n.y) for n in nodes] == [
        ("n1", "add", 10.0, 20.0),
        ("n2", "show", 30.0, 40.0),
    ]


def test_snapshot_includes_the_graph() -> None:
    """A snapshot taken after an edit shows the edit."""
    app = App([add])
    app.add_node(add, x=1.0, y=2.0)
    assert app.snapshot().graph == app.graph
    assert len(app.snapshot().graph.nodes) == 1


def test_adding_a_node_of_an_unregistered_type_is_refused() -> None:
    """A node needs a type the browser knows about."""
    app = App([add])
    with pytest.raises(ValueError, match="show"):
        app.add_node(show, x=0.0, y=0.0)


def make_clashing_add() -> Registered:
    """Build a second node function that is also called `add`.

    Returns
    -------
    Registered
        The new node function.
    """

    @node
    def add(a: float, b: float) -> float:
        return a - b

    expected = 2.0
    assert add(3.0, 1.0) == expected  # a different function, same name
    return add


def test_two_node_types_with_one_name_are_refused() -> None:
    """The name is the identity of a type."""
    with pytest.raises(ValueError, match="add"):
        App([add, make_clashing_add()])


def test_apply_changes_the_graph_and_rejects_bad_edits() -> None:
    """`apply` is the one way an edit, from a script or a browser, lands."""
    app = App([add, show])
    app.apply(AddNode(id="n1", node_type="add", x=0.0, y=0.0))
    app.apply(AddNode(id="n2", node_type="show", x=9.0, y=9.0))
    link = AddLink(
        source=PortRef(node="n1", port="out"),
        target=PortRef(node="n2", port="value"),
    )
    app.apply(link)
    expected_nodes = 2
    assert len(app.graph.nodes) == expected_nodes
    assert app.graph.links == (Link(source=link.source, target=link.target),)
    with pytest.raises(EditError):
        app.apply(link)
    assert len(app.graph.links) == 1
