"""Tests for the `@node` decorator."""

from functools import partial

import pytest
from sample_nodes import add, show

from element_51.model import Port, PortType
from element_51.nodes import node


@node
def label(text: str, count: int, *, loud: bool) -> str:
    return (text * count).upper() if loud else text * count


def test_ports_come_from_the_type_hints() -> None:
    """Parameters become inputs; the return type becomes the output `out`."""
    assert add.node_type.name == "add"
    assert add.node_type.inputs == (
        Port(name="a", kind=PortType.FLOAT),
        Port(name="b", kind=PortType.FLOAT),
    )
    assert add.node_type.outputs == (Port(name="out", kind=PortType.FLOAT),)


def test_every_port_type_is_recognised() -> None:
    """str, int and bool map to their port kinds, and bool is not int."""
    kinds = [p.kind for p in label.node_type.inputs]
    assert kinds == [PortType.STR, PortType.INT, PortType.BOOL]
    assert label.node_type.outputs[0].kind is PortType.STR


def test_the_function_still_runs() -> None:
    """Decorating does not change what the function computes."""
    expected = 3.0
    assert add(1.0, 2.0) == expected
    assert show(expected) == expected
    assert label("ab", 2, loud=True) == "ABAB"


def test_a_parameter_without_a_hint_is_refused() -> None:
    """Every input needs a type so the browser can check links."""

    def bad(
        a: float, b: float
    ) -> float:  # pragma: no cover  # never called; only its hints are read
        return a * 2 + b

    del bad.__annotations__["a"]
    with pytest.raises(TypeError, match=r"bad.*parameter 'a'"):
        node(bad)


def test_an_unsupported_parameter_type_is_refused() -> None:
    """Only float, int, str and bool are port types."""

    def bad(
        a: list[int],
    ) -> float:  # pragma: no cover  # never called; only its hints are read
        return float(len(a))

    with pytest.raises(TypeError, match=r"bad.*parameter 'a'"):
        node(bad)


def test_a_missing_return_hint_is_refused() -> None:
    """A node must say what it produces."""

    def bad(
        a: float,
    ) -> float:  # pragma: no cover  # never called; only its hints are read
        return a

    del bad.__annotations__["return"]
    with pytest.raises(TypeError, match=r"bad.*return"):
        node(bad)


def test_an_unsupported_return_type_is_refused() -> None:
    """Multiple outputs arrive in a later task; anything else is an error."""

    def bad(
        a: float,
    ) -> list[
        float
    ]:  # pragma: no cover  # never called; only its hints are read
        return [a]

    with pytest.raises(TypeError, match=r"bad.*return"):
        node(bad)


def test_variadic_parameters_are_refused() -> None:
    """`*args` has no fixed ports."""

    def bad(
        *args: float,
    ) -> float:  # pragma: no cover  # never called; only its hints are read
        return sum(args)

    with pytest.raises(TypeError, match=r"bad.*\*args"):
        node(bad)


def test_something_that_is_not_a_plain_function_is_refused() -> None:
    """A `functools.partial` has no name to give the node type."""
    with pytest.raises(TypeError, match="plain function"):
        node(partial(add, 1.0))


@node
def with_defaults(a: float = 1.5, b: int = 2, label: str = "x") -> str:
    return f"{a} {b} {label}"


def test_defaults_are_recorded_by_input_name() -> None:
    """An input with a Python default is not an error when unlinked."""
    assert with_defaults.defaults == {"a": 1.5, "b": 2, "label": "x"}
    assert add.defaults == {}


def test_call_runs_the_function_by_input_name() -> None:
    """The engine calls with a mapping of input name to value."""
    assert with_defaults.call({"a": 1.0, "b": 3, "label": "y"}) == "1.0 3 y"


def test_a_default_of_the_wrong_type_is_refused() -> None:
    """A default must be a value the port can carry."""

    def bad(a: float = "x") -> float:  # type: ignore[assignment]  # ty: ignore[invalid-parameter-default]  # pragma: no cover  # never called; the point of the test
        return a

    with pytest.raises(TypeError, match=r"bad, parameter 'a': default"):
        node(bad)


def test_positional_only_parameters_are_refused() -> None:
    """The engine passes inputs by name, so every input needs a name."""

    def bad(
        a: float, /
    ) -> float:  # pragma: no cover  # never called; only its signature is read
        return a

    with pytest.raises(TypeError, match=r"bad.*positional-only"):
        node(bad)
