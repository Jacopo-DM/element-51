"""Tests for the `@node` decorator."""

from functools import partial
from typing import override

import pytest
from pydantic import Field
from sample_nodes import add, show

from element_51.model import Model, Port, PortType
from element_51.nodes import Node, node


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


class QuotientAndRemainder(Model):
    r: int
    q: int


@node
def div_mod(a: int = 7, b: int = 2) -> QuotientAndRemainder:
    quotient, remainder = divmod(a, b)
    return QuotientAndRemainder(q=quotient, r=remainder)


def test_a_function_returning_a_model_still_runs_as_a_function() -> None:
    """Decorating does not change what the function computes."""
    assert div_mod(17, 5) == QuotientAndRemainder(q=3, r=2)


def test_the_base_class_has_no_run_of_its_own() -> None:
    """A subclass must supply `run`; the placeholder says so when called."""
    with pytest.raises(NotImplementedError):
        Node().run()


def test_a_function_returning_a_model_has_one_output_per_field() -> None:
    """The ports are the model's fields, in the order they are declared."""
    assert div_mod.node_type.outputs == (
        Port(name="r", kind=PortType.INT),
        Port(name="q", kind=PortType.INT),
    )
    assert div_mod.output_model is QuotientAndRemainder
    assert add.output_model is None


def test_a_model_with_an_unsupported_field_is_refused() -> None:
    """Every output field must be a float, int, str or bool."""

    class Bad(Model):
        items: list[int]

    def bad(
        a: float,
    ) -> Bad:  # pragma: no cover  # never called; only its hints are read
        return Bad(items=[int(a)])

    with pytest.raises(TypeError, match=r"bad, return field 'items'"):
        node(bad)


class Add(Node):
    a: float = 0.0
    b: float = 0.0

    def run(self) -> float:
        return self.a + self.b


class Split(Node):
    whole: int
    parts: int = 2

    def run(self) -> QuotientAndRemainder:
        quotient, remainder = divmod(self.whole, self.parts)
        return QuotientAndRemainder(q=quotient, r=remainder)


def test_a_class_builds_the_same_node_type_as_a_function() -> None:
    """Fields are inputs, the class name is the type name, `run` gives `out`."""
    assert Add.node_type.name == "Add"
    assert Add.node_type.inputs == add.node_type.inputs
    assert Add.node_type.outputs == add.node_type.outputs
    assert Add.defaults == {"a": 0.0, "b": 0.0}
    assert Add.output_model is None


def test_a_class_whose_run_returns_a_model_has_several_outputs() -> None:
    """A field without a default is an input with no default."""
    assert [p.name for p in Split.node_type.outputs] == ["r", "q"]
    assert Split.defaults == {"parts": 2}
    assert Split.output_model is QuotientAndRemainder


def test_calling_a_class_node_validates_and_runs() -> None:
    """The engine's call builds the model from the inputs, then runs it."""
    assert Add.call({"a": 1.0, "b": 2.0}) == Add(a=1.0, b=2.0).run()
    result = Split.call({"whole": 17, "parts": 5})
    assert result == QuotientAndRemainder(q=3, r=2)


def test_a_class_field_of_an_unsupported_type_is_refused() -> None:
    """The same port types apply to the class form."""
    with pytest.raises(TypeError, match=r"Bad, field 'items'"):

        class Bad(Node):
            items: list[int] = []  # ruff: ignore[mutable-class-default]

            @override
            def run(self) -> float:  # pragma: no cover  # never called
                return 0.0


def test_a_class_default_of_the_wrong_type_is_refused() -> None:
    """A default must be a value the port can carry."""
    with pytest.raises(TypeError, match=r"Bad, field 'x': default"):

        class Bad(Node):
            x: float = "oops"  # type: ignore[assignment]  # ty: ignore[invalid-assignment]  # the point of the test

            def run(self) -> float:  # pragma: no cover  # never called
                return self.x


def test_a_class_default_factory_is_refused() -> None:
    """A computed default has no single value to show or send."""
    with pytest.raises(TypeError, match=r"Bad, field 'x': default_factory"):

        class Bad(Node):
            x: float = Field(default_factory=float)

            def run(self) -> float:  # pragma: no cover  # never called
                return self.x


def test_a_class_run_without_a_return_hint_is_refused() -> None:
    """`run` must say what it produces."""
    with pytest.raises(TypeError, match=r"Bad, return"):

        class Bad(Node):
            x: float = 0.0

            def run(self):  # type: ignore[no-untyped-def]  # ruff: ignore[missing-return-type-private-function]  # pragma: no cover  # never called; the point of the test
                return self.x


def test_the_node_type_carries_the_python_defaults() -> None:
    """The browser can show a default because the node type sends it."""

    @node
    def scale(
        value: float, factor: float = 2.0
    ) -> float:  # pragma: no cover  # only its hints are read
        return value * factor

    assert scale.node_type.defaults == {"factor": 2.0}
