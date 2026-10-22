"""Tests for the scan of numpy's ufuncs (`nodes_from_ufuncs`)."""

import math
import warnings
from types import ModuleType

import numpy as np
import pytest
from hypothesis import given
from hypothesis import strategies as st

from element_51 import App, nodes_from_ufuncs
from element_51.edits import SetValue
from element_51.model import Port, PortType
from element_51.nodes import Registered
from element_51.wrap import Scan

KNOWN_SKIPS = {"matmul", "matvec", "vecdot", "vecmat", "isnat"}
HYPOT_3_4 = 5.0
LDEXP_1_5_BY_3 = 12.0
GCD_12_18 = 6
MANY_ONE = 30
MANY_TWO = 10


@pytest.fixture(scope="module")
def scan() -> Scan:
    """Scan numpy once.

    Returns
    -------
    Scan
        The nodes and skips for `numpy`.
    """
    return nodes_from_ufuncs(np, category="Numpy")


def _node(scan: Scan, name: str) -> Registered:
    return next(n for n in scan.nodes if n.node_type.name == name)


def _call(scan: Scan, name: str, **inputs: float) -> object:
    return _node(scan, name).call(inputs)


def test_every_distinct_ufunc_is_a_node_or_a_skip(scan: Scan) -> None:
    """Nothing is lost: a ufunc is a node or is skipped with a reason."""
    distinct = {id(u) for u in vars(np).values() if isinstance(u, np.ufunc)}
    names = [n.node_type.name for n in scan.nodes]
    names += [skip.name for skip in scan.skipped]
    assert len(names) == len(distinct)
    assert len(set(names)) == len(names)
    assert all(skip.reason for skip in scan.skipped)


def test_the_known_skips_are_named_with_their_reason(scan: Scan) -> None:
    """The generalised ufuncs need arrays; `isnat` has only date loops."""
    reasons = {skip.name: skip.reason for skip in scan.skipped}
    assert set(reasons) == KNOWN_SKIPS
    assert "whole arrays" in reasons["matmul"]
    assert "no float, int or bool loop" in reasons["isnat"]


def test_an_alias_gives_one_node_named_by_the_ufunc(scan: Scan) -> None:
    """`np.acos` and `np.arccos` are one ufunc: one node, `arccos`."""
    names = {n.node_type.name for n in scan.nodes}
    assert "arccos" in names
    assert "acos" not in names
    assert "absolute" in names
    assert "abs" not in names


def test_the_category_is_given_to_every_node(scan: Scan) -> None:
    """`category="Numpy"` puts the nodes in one flyout."""
    assert {n.node_type.category for n in scan.nodes} == {"Numpy"}


def test_a_bad_category_is_an_error() -> None:
    """The caller's mistake is raised, as for `nodes_from_module`."""
    with pytest.raises(ValueError, match="category"):
        nodes_from_ufuncs(np, category="Numpy/ ")


def test_ports_follow_the_arity_and_the_loop(scan: Scan) -> None:
    """One input is `x`, two are `x1` and `x2`; types come from the loop."""
    sin = _node(scan, "sin").node_type
    assert sin.inputs == (Port(name="x", kind=PortType.FLOAT),)
    assert sin.outputs == (Port(name="out", kind=PortType.FLOAT),)
    hypot = _node(scan, "hypot").node_type
    assert [p.name for p in hypot.inputs] == ["x1", "x2"]
    isnan = _node(scan, "isnan").node_type
    assert isnan.outputs == (Port(name="out", kind=PortType.BOOL),)
    ldexp = _node(scan, "ldexp").node_type
    assert [p.kind for p in ldexp.inputs] == [PortType.FLOAT, PortType.INT]
    gcd = _node(scan, "gcd").node_type
    assert [p.kind for p in gcd.inputs] == [PortType.INT, PortType.INT]


def test_two_outputs_get_two_ports(scan: Scan) -> None:
    """`modf` and `frexp` have two outputs, `out1` and `out2`."""
    modf = _node(scan, "modf").node_type
    assert [p.name for p in modf.outputs] == ["out1", "out2"]
    frexp = _node(scan, "frexp").node_type
    assert [p.kind for p in frexp.outputs] == [PortType.FLOAT, PortType.INT]


def test_results_are_plain_python_values(scan: Scan) -> None:
    """A float, an int and a bool come back as the builtin types."""
    assert _call(scan, "sin", x=0.0) == pytest.approx(0.0)
    assert type(_call(scan, "sin", x=0.0)) is float
    assert _call(scan, "hypot", x1=3.0, x2=4.0) == HYPOT_3_4
    assert _call(scan, "isnan", x=math.nan) is True
    assert _call(scan, "gcd", x1=12, x2=18) == GCD_12_18
    assert type(_call(scan, "gcd", x1=12, x2=18)) is int
    assert _call(scan, "ldexp", x1=1.5, x2=3) == LDEXP_1_5_BY_3


def test_two_outputs_come_back_as_the_result_model(scan: Scan) -> None:
    """`modf(2.5)` is 0.5 and 2.0, in the model the engine reads."""
    modf = _node(scan, "modf")
    assert modf.output_model is not None
    result = modf.call({"x": 2.5})
    assert isinstance(result, modf.output_model)
    assert result.model_dump() == {"out1": 0.5, "out2": 2.0}


def test_a_whole_number_is_made_a_float_before_the_call(scan: Scan) -> None:
    """An int on a float port cannot overflow in an integer loop."""
    big = 2**62
    assert _call(scan, "add", x1=big, x2=big) == pytest.approx(float(big) * 2)


def test_a_divide_by_zero_gives_inf_and_no_warning(scan: Scan) -> None:
    """Numpy would warn; the node returns what numpy computes, quietly."""
    with warnings.catch_warnings():
        warnings.simplefilter("error")
        assert _call(scan, "divide", x1=1.0, x2=0.0) == math.inf
        assert _is_nan(_call(scan, "log", x=-1.0))


def test_a_ufunc_node_shows_its_call_as_source(scan: Scan) -> None:
    """A ufunc has no source file; the panel shows how it is called."""
    assert _node(scan, "hypot").source_text() == "numpy.hypot(x1, x2)"
    assert _node(scan, "sin").source_text() == "numpy.sin(x)"
    assert _node(scan, "modf").source_text() == "numpy.modf(x)"


def test_ufunc_nodes_feed_the_app_and_run(scan: Scan) -> None:
    """A graph of `sin` into `hypot` runs through `App.run`."""
    app = App(scan.nodes)
    sin = app.add_node(_node(scan, "sin"), x=0.0, y=0.0)
    hypot = app.add_node(_node(scan, "hypot"), x=1.0, y=0.0)
    modf = app.add_node(_node(scan, "modf"), x=2.0, y=0.0)
    app.add_link(sin, "out", hypot, "x1")
    for node, port, value in (
        (sin, "x", 0.0),
        (hypot, "x2", 4.0),
        (modf, "x", 2.5),
    ):
        app.apply(SetValue(node=node, port=port, value=value))
    assert app.run() == {
        sin: {"out": 0.0},
        hypot: {"out": 4.0},
        modf: {"out1": 0.5, "out2": 2.0},
    }


def test_a_namespace_without_numpy_ufuncs_is_scanned_for_what_it_has() -> None:
    """A made-up ufunc with only object loops is skipped, a function ignored."""

    def twice(x: object) -> object:  # pragma: no cover  # numpy never calls it
        return x

    made = np.frompyfunc(twice, 1, 1)
    module = ModuleType("made_up")
    vars(module).update(made=made, number=3)
    found = nodes_from_ufuncs(module)
    assert found.nodes == ()
    assert [skip.name for skip in found.skipped] == ["twice (vectorized)"]
    assert "no float, int or bool loop" in found.skipped[0].reason


ONE_FLOAT = [
    node
    for node in nodes_from_ufuncs(np).nodes
    if [p.kind for p in node.node_type.inputs] == [PortType.FLOAT]
    and [p.kind for p in node.node_type.outputs] == [PortType.FLOAT]
]
TWO_FLOATS = [
    node
    for node in nodes_from_ufuncs(np).nodes
    if [p.kind for p in node.node_type.inputs] == [PortType.FLOAT] * 2
    and [p.kind for p in node.node_type.outputs] == [PortType.FLOAT]
]


def _is_nan(value: object) -> bool:
    return isinstance(value, float) and math.isnan(value)


def _same(left: object, right: object) -> bool:
    if _is_nan(left):
        return _is_nan(right)
    return left == right


def test_the_property_test_has_something_to_compare() -> None:
    """The two groups are not empty, so the property tests mean something."""
    assert len(ONE_FLOAT) > MANY_ONE
    assert len(TWO_FLOATS) > MANY_TWO


@given(st.floats())
def test_one_float_nodes_equal_numpy(x: float) -> None:
    """For any float, each one-input float node returns what numpy does."""
    for node in ONE_FLOAT:
        ufunc = getattr(np, node.node_type.name)
        with np.errstate(all="ignore"):
            expected = float(ufunc(x))
        assert _same(node.call({"x": x}), expected), node.node_type.name


@given(st.floats(), st.floats())
def test_two_float_nodes_equal_numpy(x1: float, x2: float) -> None:
    """For any two floats, each two-input float node returns what numpy does."""
    for node in TWO_FLOATS:
        ufunc = getattr(np, node.node_type.name)
        with np.errstate(all="ignore"):
            expected = float(ufunc(x1, x2))
        assert _same(node.call({"x1": x1, "x2": x2}), expected), (
            node.node_type.name
        )


def test_a_text_on_a_number_port_is_refused(scan: Scan) -> None:
    """The node takes numbers only; anything else is a clear error."""
    with pytest.raises(TypeError, match="str is not a number"):
        _node(scan, "sin").call({"x": "text"})
