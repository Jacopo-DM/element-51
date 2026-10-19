"""Tests for running a graph."""

from typing import cast

import pytest

from element_51.engine import run_graph
from element_51.model import Graph, Link, PlacedNode, PortRef
from element_51.nodes import node


@node
def one() -> float:
    return 1.0


@node
def two() -> float:
    return 2.0


@node
def add(a: float, b: float) -> float:
    return a + b


@node
def double(x: float = 0.0) -> float:
    return x * 2


@node
def divide(a: float, b: float = 0.0) -> float:
    return a / b


@node
def shout(text: str = "hi") -> str:
    return text.upper()


@node
def wrong_type(x: float = 1.0) -> float:
    return cast("float", f"not a number: {x}")


@node
def bool_for_int(*, flag: bool = True) -> int:
    return cast("int", flag)


@node
def int_for_float(n: int = 3) -> float:
    return cast("float", n)


FUNCTIONS = {
    f.node_type.name: f
    for f in (
        one,
        two,
        add,
        double,
        divide,
        shout,
        wrong_type,
        bool_for_int,
        int_for_float,
    )
}


def graph_of(
    types: dict[str, str], links: list[tuple[str, str, str, str]]
) -> Graph:
    """Build a graph from `{id: node type}` and `(src, out, dst, in)` links.

    Returns
    -------
    Graph
        The graph, with every node at the origin.
    """
    return Graph(
        nodes=tuple(
            PlacedNode(id=node_id, node_type=name, x=0.0, y=0.0)
            for node_id, name in types.items()
        ),
        links=tuple(
            Link(
                source=PortRef(node=src, port=out),
                target=PortRef(node=dst, port=inp),
            )
            for src, out, dst, inp in links
        ),
    )


def test_values_flow_along_links_in_dependency_order() -> None:
    """The consumer is listed after what it depends on, whatever the order."""
    graph = graph_of(
        {"n3": "add", "n1": "one", "n2": "two"},
        [("n1", "out", "n3", "a"), ("n2", "out", "n3", "b")],
    )
    result = run_graph(graph, FUNCTIONS, raise_errors=False)
    assert [(o.node, o.status, o.value) for o in result.outcomes] == [
        ("n1", "ok", 1.0),
        ("n2", "ok", 2.0),
        ("n3", "ok", 3.0),
    ]


def test_an_unlinked_input_uses_the_functions_default() -> None:
    """`double` has no link in, so it runs with its default 0.0."""
    result = run_graph(
        graph_of({"n1": "double"}, []), FUNCTIONS, raise_errors=False
    )
    assert result.outcomes[0].value == pytest.approx(0.0)


def test_an_unlinked_input_without_a_default_is_an_error() -> None:
    """The message names the input, and what depends on it is skipped."""
    graph = graph_of({"n1": "add", "n2": "double"}, [("n1", "out", "n2", "x")])
    first, second = run_graph(graph, FUNCTIONS, raise_errors=False).outcomes
    assert first.status == "error"
    assert first.error == "input 'a' has no value"
    assert second.status == "skipped"
    assert second.value is None


def test_a_node_that_raises_is_marked_and_its_dependents_are_skipped() -> None:
    """Independent branches still run; the chain after the failure stops."""
    graph = graph_of(
        {"n1": "one", "n2": "divide", "n3": "double", "n4": "double"},
        [
            ("n1", "out", "n2", "a"),
            ("n2", "out", "n3", "x"),
            ("n1", "out", "n4", "x"),
        ],
    )
    outcomes = run_graph(graph, FUNCTIONS, raise_errors=False).outcomes
    by_id = {o.node: o for o in outcomes}
    assert by_id["n2"].status == "error"
    assert by_id["n2"].error == "ZeroDivisionError: float division by zero"
    assert by_id["n3"].status == "skipped"
    assert by_id["n4"].status == "ok"
    expected = 2.0
    assert by_id["n4"].value == expected


def test_skipping_passes_down_a_chain() -> None:
    """A node two steps after the failure is skipped too."""
    graph = graph_of(
        {"n1": "divide", "n2": "double", "n3": "double"},
        [("n1", "out", "n2", "x"), ("n2", "out", "n3", "x")],
    )
    statuses = [
        o.status
        for o in run_graph(graph, FUNCTIONS, raise_errors=False).outcomes
    ]
    assert statuses == ["error", "skipped", "skipped"]


def test_a_value_of_the_wrong_type_is_an_error() -> None:
    """A node that returns a str where it declared float is caught."""
    result = run_graph(
        graph_of({"n1": "wrong_type"}, []), FUNCTIONS, raise_errors=False
    )
    outcome = result.outcomes[0]
    assert outcome.status == "error"
    assert outcome.error == "returned str, expected float"


def test_bool_is_not_an_int_but_an_int_may_be_a_float() -> None:
    """The port types are strict where Python is not."""
    graph = graph_of({"n1": "bool_for_int", "n2": "int_for_float"}, [])
    first, second = run_graph(graph, FUNCTIONS, raise_errors=False).outcomes
    assert first.error == "returned bool, expected int"
    assert second.status == "ok"
    expected = 3
    assert second.value == expected


def test_string_values_pass_through() -> None:
    """A str port carries a str."""
    result = run_graph(
        graph_of({"n1": "shout"}, []), FUNCTIONS, raise_errors=False
    )
    assert result.outcomes[0].value == "HI"


def test_raise_mode_raises_the_nodes_own_exception_with_a_note() -> None:
    """`app.run()` shows the real error and says which node it came from."""
    graph = graph_of({"n1": "one", "n2": "divide"}, [("n1", "out", "n2", "a")])
    with pytest.raises(ZeroDivisionError) as raised:
        run_graph(graph, FUNCTIONS, raise_errors=True)
    assert any("n2" in note for note in raised.value.__notes__)


def test_raise_mode_raises_for_a_missing_input_and_a_wrong_type() -> None:
    """The engine's own errors raise in the same way."""
    with pytest.raises(ValueError, match="input 'a' has no value"):
        run_graph(graph_of({"n1": "add"}, []), FUNCTIONS, raise_errors=True)
    with pytest.raises(TypeError, match="returned str, expected float"):
        run_graph(
            graph_of({"n1": "wrong_type"}, []), FUNCTIONS, raise_errors=True
        )


def test_a_cycle_that_got_into_the_graph_is_reported_not_looped() -> None:
    """Edits refuse cycles, but a hand-built graph must not hang the engine."""
    graph = graph_of(
        {"n1": "double", "n2": "double"},
        [("n1", "out", "n2", "x"), ("n2", "out", "n1", "x")],
    )
    outcomes = run_graph(graph, FUNCTIONS, raise_errors=False).outcomes
    assert [(o.status, o.error) for o in outcomes] == [
        ("error", "the graph has a cycle"),
        ("error", "the graph has a cycle"),
    ]
    with pytest.raises(ValueError, match="cycle"):
        run_graph(graph, FUNCTIONS, raise_errors=True)
