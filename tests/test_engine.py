"""Tests for running a graph."""

from typing import cast, override

import pytest

from element_51.engine import (
    NO_OUTPUT,
    ancestors,
    closure,
    descendants,
    run_graph,
)
from element_51.job_node import JobNode, Report
from element_51.model import Graph, Link, Model, PlacedNode, PortRef
from element_51.nodes import Node, node


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


class DivModResult(Model):
    q: int
    r: int


@node
def div_mod(a: int = 7, b: int = 2) -> DivModResult:
    quotient, remainder = divmod(a, b)
    return DivModResult(q=quotient, r=remainder)


@node
def not_a_model(a: int = 1) -> DivModResult:
    return cast("DivModResult", {"q": a, "r": a})


class Triple(Node):
    x: float = 1.0

    @override
    def run(self) -> float:
        return self.x * 3


class Explode(Node):
    x: float = 1.0

    @override
    def run(self) -> float:
        msg = "boom"
        raise RuntimeError(msg)


class Training(JobNode[None]):
    epochs: int = 3

    @override
    def work(
        self, report: Report
    ) -> float:  # pragma: no cover  # a job is never run here
        report(out=1.0)
        return float(self.epochs)


@node
def wrong_field(a: int = 1) -> DivModResult:
    return DivModResult.model_construct(q=cast("int", "bad"), r=a)


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
        div_mod,
        not_a_model,
        wrong_field,
        Triple,
        Explode,
        Training,
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
    assert [(o.node, o.status, dict(o.outputs)) for o in result.outcomes] == [
        ("n1", "ok", {"out": 1.0}),
        ("n2", "ok", {"out": 2.0}),
        ("n3", "ok", {"out": 3.0}),
    ]


def test_an_unlinked_input_uses_the_functions_default() -> None:
    """`double` has no link in, so it runs with its default 0.0."""
    result = run_graph(
        graph_of({"n1": "double"}, []), FUNCTIONS, raise_errors=False
    )
    assert result.outcomes[0].outputs["out"] == pytest.approx(0.0)


def test_an_unlinked_input_without_a_default_is_an_error() -> None:
    """The message names the input, and what depends on it is skipped."""
    graph = graph_of({"n1": "add", "n2": "double"}, [("n1", "out", "n2", "x")])
    first, second = run_graph(graph, FUNCTIONS, raise_errors=False).outcomes
    assert first.status == "error"
    assert first.error == "input 'a' has no value"
    assert second.status == "skipped"
    assert second.outputs == {}


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
    assert by_id["n4"].outputs == {"out": expected}


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
    assert second.outputs == {"out": expected}


def test_string_values_pass_through() -> None:
    """A str port carries a str."""
    result = run_graph(
        graph_of({"n1": "shout"}, []), FUNCTIONS, raise_errors=False
    )
    assert result.outcomes[0].outputs == {"out": "HI"}


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


def test_each_output_of_a_multi_output_node_can_feed_a_different_node() -> None:
    """`q` and `r` are separate ports with separate values."""
    graph = graph_of(
        {"n1": "div_mod", "n2": "double", "n3": "double"},
        [("n1", "q", "n2", "x"), ("n1", "r", "n3", "x")],
    )
    outcomes = run_graph(graph, FUNCTIONS, raise_errors=False).outcomes
    by_id = {o.node: dict(o.outputs) for o in outcomes}
    assert by_id == {
        "n1": {"q": 3, "r": 1},
        "n2": {"out": 6.0},
        "n3": {"out": 2.0},
    }


def test_a_multi_output_node_must_return_its_model() -> None:
    """Returning a dict where the model was declared is an error."""
    result = run_graph(
        graph_of({"n1": "not_a_model"}, []), FUNCTIONS, raise_errors=False
    )
    assert result.outcomes[0].error == "returned dict, expected DivModResult"
    assert result.outcomes[0].outputs == {}


def test_a_class_node_runs_with_its_defaults_and_its_links() -> None:
    """The class form takes inputs from fields and gives `out` from `run`."""
    graph = graph_of(
        {"n1": "Triple", "n2": "Triple"}, [("n1", "out", "n2", "x")]
    )
    result = run_graph(graph, FUNCTIONS, raise_errors=False)
    assert [dict(o.outputs) for o in result.outcomes] == [
        {"out": 3.0},
        {"out": 9.0},
    ]


def test_a_class_node_that_raises_is_marked_like_any_other() -> None:
    """The class form gets the same error handling as the function form."""
    result = run_graph(
        graph_of({"n1": "Explode"}, []), FUNCTIONS, raise_errors=False
    )
    assert result.outcomes[0].status == "error"
    assert result.outcomes[0].error == "RuntimeError: boom"


def test_a_field_value_that_skips_validation_is_still_checked() -> None:
    """A model built without validation cannot smuggle a wrong type through."""
    result = run_graph(
        graph_of({"n1": "wrong_field"}, []), FUNCTIONS, raise_errors=False
    )
    assert result.outcomes[0].error == "output 'q' is str, expected int"


def test_a_value_on_the_node_beats_the_default_and_a_link_beats_both() -> None:
    """The order is link, then the node's own value, then the default."""
    base = graph_of({"n1": "double", "n2": "double"}, [])
    nodes = (
        base.nodes[0].model_copy(update={"values": {"x": 5.0}}),
        base.nodes[1].model_copy(update={"values": {"x": 7.0}}),
    )
    graph = Graph(
        nodes=nodes,
        links=(
            Link(
                source=PortRef(node="n1", port="out"),
                target=PortRef(node="n2", port="x"),
            ),
        ),
    )
    first, second = run_graph(graph, FUNCTIONS, raise_errors=False).outcomes
    assert first.outputs["out"] == pytest.approx(10.0)
    assert second.outputs["out"] == pytest.approx(20.0)


def test_a_value_on_the_node_fills_an_input_that_has_no_default() -> None:
    """`add` has no defaults; typed values make it runnable."""
    base = graph_of({"n1": "add"}, [])
    graph = Graph(
        nodes=(
            base.nodes[0].model_copy(update={"values": {"a": 1.0, "b": 2}}),
        ),
        links=(),
    )
    (outcome,) = run_graph(graph, FUNCTIONS, raise_errors=False).outcomes
    assert outcome.outputs["out"] == pytest.approx(3.0)


def job_graph() -> Graph:
    """Build a job feeding `double`, with two unrelated nodes beside them.

    Returns
    -------
    Graph
        The graph.
    """
    return graph_of(
        {"n1": "one", "n2": "Training", "n3": "double", "n4": "two"},
        [("n2", "out", "n3", "x")],
    )


def test_a_plain_run_skips_a_job_and_what_follows_it_with_the_reason() -> None:
    """Run never starts a job; the job says why it has no value."""
    result = run_graph(job_graph(), FUNCTIONS, raise_errors=False)
    by_id = {o.node: o for o in result.outcomes}
    assert by_id["n2"].status == "skipped"
    assert by_id["n2"].error == NO_OUTPUT
    assert by_id["n3"].status == "skipped"
    assert by_id["n3"].error is None
    assert by_id["n1"].status == "ok"
    assert by_id["n4"].status == "ok"


def test_a_script_run_raises_for_a_job_with_no_output() -> None:
    """With `raise_errors` the job's missing output is an error with a note."""
    with pytest.raises(ValueError, match="no output yet") as caught:
        run_graph(job_graph(), FUNCTIONS, raise_errors=True)
    assert "in node n2 (Training)" in caught.value.__notes__


def test_fixed_outputs_stand_in_for_a_node_that_is_not_run() -> None:
    """A node after a job takes the job's latest values from `fixed`."""
    result = run_graph(
        job_graph(),
        FUNCTIONS,
        raise_errors=False,
        fixed={"n2": {"out": 5.0}},
    )
    by_id = {o.node: o for o in result.outcomes}
    assert by_id["n2"].status == "ok"
    assert by_id["n2"].outputs == {"out": 5.0}
    assert by_id["n3"].outputs == {"out": pytest.approx(10.0)}


def test_only_runs_and_reports_just_the_named_nodes() -> None:
    """A subset run leaves the rest out and takes fixed values from outside."""
    result = run_graph(
        job_graph(),
        FUNCTIONS,
        raise_errors=False,
        only={"n3"},
        fixed={"n2": {"out": 4.0}},
    )
    assert [o.node for o in result.outcomes] == ["n3"]
    assert result.outcomes[0].outputs == {"out": pytest.approx(8.0)}
    nothing = run_graph(
        job_graph(), FUNCTIONS, raise_errors=False, only=frozenset()
    )
    assert nothing.outcomes == ()


def test_ancestors_descendants_and_closure_follow_the_links() -> None:
    """Reach follows links either way and leaves the start out."""
    graph = graph_of(
        {"n1": "one", "n2": "double", "n3": "double", "n4": "two"},
        [
            ("n1", "out", "n2", "x"),
            ("n2", "out", "n3", "x"),
            ("n1", "out", "n3", "y"),
        ],
    )
    assert ancestors(graph, "n3") == {"n1", "n2"}
    assert descendants(graph, "n1") == {"n2", "n3"}
    assert ancestors(graph, "n1") == frozenset()
    assert closure(graph, ["n3", "n4"]) == {"n1", "n2", "n3", "n4"}
