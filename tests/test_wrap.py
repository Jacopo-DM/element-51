"""Tests for the scan of a module's functions (`nodes_from_module`)."""

import pytest
import sample_wrap
import sample_wrap_all

from element_51 import App, nodes_from_module
from element_51.edits import SetValue
from element_51.model import Port, PortType
from element_51.wrap import Scan, Skipped


def _names(scan: Scan) -> list[str]:
    return [node.node_type.name for node in scan.nodes]


def test_typed_functions_become_nodes_in_definition_order() -> None:
    """The fitting public functions come back in the order they are written."""
    scan = nodes_from_module(sample_wrap)
    assert _names(scan) == ["zscore", "shout", "doubled", "Greeter"]


def test_ports_come_from_the_hints_as_with_node() -> None:
    """A scanned function gets the ports `@node` would give it."""
    zscore = nodes_from_module(sample_wrap).nodes[0]
    assert zscore.node_type.inputs == (
        Port(name="x", kind=PortType.FLOAT),
        Port(name="mean", kind=PortType.FLOAT),
        Port(name="std", kind=PortType.FLOAT),
    )
    assert zscore.node_type.outputs == (Port(name="out", kind=PortType.FLOAT),)


def test_the_category_goes_to_scanned_functions_only() -> None:
    """A function already decorated keeps its own category."""
    scan = nodes_from_module(sample_wrap, category="Stats")
    categories = {n.node_type.name: n.node_type.category for n in scan.nodes}
    assert categories == {
        "zscore": "Stats",
        "shout": "Stats",
        "doubled": "Own",
        "Greeter": "",
    }


def test_a_bad_category_is_an_error_not_a_skip() -> None:
    """The caller's mistake is raised, not blamed on each function."""
    with pytest.raises(ValueError, match="category"):
        nodes_from_module(sample_wrap, category=" Stats")


def test_a_function_that_does_not_fit_is_skipped_with_its_reason() -> None:
    """Each skip names the function and says what is wrong."""
    scan = nodes_from_module(sample_wrap)
    reasons = {skip.name: skip.reason for skip in scan.skipped}
    assert list(reasons) == ["untyped", "spread", "positional"]
    assert "no type hint" in reasons["untyped"]
    assert "*args" in reasons["spread"]
    assert "positional-only" in reasons["positional"]


def test_private_imported_and_other_names_are_left_out() -> None:
    """`_hidden`, the imported node, the alias, a constant and a class."""
    scan = nodes_from_module(sample_wrap)
    seen = _names(scan) + [skip.name for skip in scan.skipped]
    for left_out in (
        "_hidden",
        "add",
        "imported",
        "plus",
        "THRESHOLD",
        "Helper",
    ):
        assert left_out not in seen


def test_all_limits_the_scan_to_the_names_it_lists() -> None:
    """A module with `__all__` is scanned for those names only."""
    scan = nodes_from_module(sample_wrap_all)
    assert _names(scan) == ["listed"]
    assert scan.skipped == ()


def test_a_second_function_with_a_taken_name_is_reported() -> None:
    """Two different functions named alike: the later one is a skip."""
    module = type(sample_wrap)("clash")

    def same(x: float) -> float:  # pragma: no cover  # only read by the scan
        return x

    def other(x: float) -> float:  # pragma: no cover  # only read by the scan
        return x - 1.0

    other.__name__ = "same"
    same.__module__ = other.__module__ = "clash"
    vars(module).update(same=same, other=other)
    scan = nodes_from_module(module)
    assert _names(scan) == ["same"]
    assert scan.skipped == (
        Skipped(
            name="other", reason="another function is already named 'same'"
        ),
    )


def test_the_scan_feeds_the_app_as_it_is() -> None:
    """`App` takes the nodes, and a scanned function runs."""
    scan = nodes_from_module(sample_wrap)
    app = App(scan.nodes)
    zscore = app.add_node(scan.nodes[0], x=0.0, y=0.0)
    for port, value in (("x", 5.0), ("mean", 3.0), ("std", 2.0)):
        app.apply(SetValue(node=zscore, port=port, value=value))
    assert app.run() == {zscore: {"out": 1.0}}
