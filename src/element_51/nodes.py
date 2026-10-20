"""Turn a typed function, or a `Node` subclass, into a node type."""

import inspect
from collections.abc import Callable, Mapping
from typing import ClassVar, Protocol, cast, get_type_hints, overload

from pydantic import BaseModel

from element_51.model import Model, NodeType, Port, PortValue, value_fits
from element_51.ports import NO_DEFAULT, input_port, output_ports


class NamedCall(Protocol):
    """A function called with every input by name."""

    def __call__(self, **inputs: PortValue) -> object:
        """Call the function.

        Parameters
        ----------
        **inputs
            One value per input, by parameter name.
        """
        ...


class Registered(Protocol):
    """What the session and the engine need from a node function or class."""

    @property
    def node_type(self) -> NodeType:
        """The node type built from the hints."""
        ...

    @property
    def defaults(self) -> Mapping[str, PortValue]:
        """The Python defaults, by input name, for inputs left unlinked."""
        ...

    @property
    def output_model(self) -> type[BaseModel] | None:
        """The model that holds the outputs, or None for a single `out`."""
        ...

    def call(self, inputs: Mapping[str, PortValue]) -> object:
        """Call the node with every input by name.

        Parameters
        ----------
        inputs
            One value per input.
        """
        ...


class NodeFunction[**P, R]:
    """A function together with the node type built from its hints.

    Calling it calls the function, with the same signature. This holds
    behaviour, not just fields, which is why it is a class and not a pydantic
    record.
    """

    def __init__(
        self,
        func: Callable[P, R],
        node_type: NodeType,
        defaults: Mapping[str, PortValue],
        output_model: type[BaseModel] | None,
    ) -> None:
        self.func = func
        self.node_type = node_type
        self.defaults = defaults
        self.output_model = output_model

    def call(self, inputs: Mapping[str, PortValue]) -> object:
        """Call the function with every input by name.

        `node` has checked that every parameter is a keyword parameter with a
        port type, so the function accepts exactly these names and types.

        Parameters
        ----------
        inputs
            One value per input.

        Returns
        -------
        object
            Whatever the function returns.
        """
        return cast("NamedCall", self.func)(**inputs)

    def __call__(self, *args: P.args, **kwargs: P.kwargs) -> R:
        """Call the wrapped function.

        Parameters
        ----------
        *args
            Positional arguments.
        **kwargs
            Keyword arguments.

        Returns
        -------
        R
            Whatever the function returns.
        """
        return self.func(*args, **kwargs)


class Node(Model):
    """Base class for a node written as a class.

    Fields are the inputs, with their defaults. `run` computes the outputs: a
    plain return type gives one output `out`, and a pydantic model gives one
    output per field. The class name is the node type name.
    """

    node_type: ClassVar[NodeType]
    category: ClassVar[str] = ""
    defaults: ClassVar[Mapping[str, PortValue]]
    output_model: ClassVar[type[BaseModel] | None]

    @classmethod
    def __pydantic_init_subclass__(cls, **kwargs: object) -> None:  # ruff: ignore[bad-dunder-method-name]  # the hook name pydantic defines
        """Build the node type from the fields and the hint on `run`.

        Parameters
        ----------
        **kwargs
            Class keyword arguments, passed on to pydantic.

        Raises
        ------
        TypeError
            If a field or the return hint is not a supported port type.
        """
        super().__pydantic_init_subclass__(**kwargs)
        name = cls.__name__
        inputs: list[Port] = []
        defaults: dict[str, PortValue] = {}
        for field_name, info in cls.model_fields.items():
            where = f"{name}, field {field_name!r}"
            if info.default_factory is not None:
                msg = f"{where}: default_factory is not supported"
                raise TypeError(msg)
            default = NO_DEFAULT if info.is_required() else info.default
            port = input_port(field_name, info.annotation, default, where)
            inputs.append(port)
            if value_fits(default, port.kind):
                defaults[field_name] = default
        hints = get_type_hints(cls.run)
        if "return" not in hints:
            msg = f"{name}, return: no type hint"
            raise TypeError(msg)
        outputs, output_model = output_ports(name, hints["return"])
        cls.node_type = NodeType(
            name=name,
            inputs=tuple(inputs),
            outputs=outputs,
            defaults=defaults,
            category=_check_category(cls.category, name),
        )
        cls.defaults = defaults
        cls.output_model = output_model

    def run(self) -> object:
        """Compute the outputs from the inputs. Subclasses override this.

        Raises
        ------
        NotImplementedError
            Always, here: this is the placeholder a subclass replaces.
        """
        raise NotImplementedError

    @classmethod
    def call(cls, inputs: Mapping[str, PortValue]) -> object:
        """Build the node from its inputs and run it.

        Parameters
        ----------
        inputs
            One value per input.

        Returns
        -------
        object
            Whatever `run` returns.
        """
        return cls(**inputs).run()


def _build_node[**P, R](
    func: Callable[P, R], category: str
) -> NodeFunction[P, R]:
    """Build a node type from a typed function.

    Each parameter becomes an input port. A plain return type becomes one
    output port named `out`; a pydantic model becomes one output port per
    field.

    Parameters
    ----------
    func
        A function whose parameters and return value are annotated with
        float, int, str or bool, or whose return value is a pydantic model of
        such fields.
    category
        Where the node type goes in the add-node menu.

    Returns
    -------
    NodeFunction[P, R]
        The function, with its node type attached.

    Raises
    ------
    TypeError
        If `func` is not a plain function, a parameter or the return value
        is missing a supported hint, the function takes `*args` or
        `**kwargs`, or a parameter is positional-only or has a default of the
        wrong type.
    """
    if not inspect.isfunction(func):
        msg = f"@node needs a plain function, not {func!r}"
        raise TypeError(msg)
    name = func.__name__
    hints = get_type_hints(func)
    inputs: list[Port] = []
    defaults: dict[str, PortValue] = {}
    for parameter in inspect.signature(func).parameters.values():
        if parameter.kind in {parameter.VAR_POSITIONAL, parameter.VAR_KEYWORD}:
            msg = f"{name}: *args and **kwargs have no fixed ports"
            raise TypeError(msg)
        where = f"{name}, parameter {parameter.name!r}"
        if parameter.kind is parameter.POSITIONAL_ONLY:
            msg = f"{where}: positional-only parameters cannot be linked"
            raise TypeError(msg)
        if parameter.name not in hints:
            msg = f"{where}: no type hint"
            raise TypeError(msg)
        port = input_port(
            parameter.name, hints[parameter.name], parameter.default, where
        )
        inputs.append(port)
        if value_fits(parameter.default, port.kind):
            defaults[parameter.name] = parameter.default
    if "return" not in hints:
        msg = f"{name}, return: no type hint"
        raise TypeError(msg)
    outputs, output_model = output_ports(name, hints["return"])
    node_type = NodeType(
        name=name,
        inputs=tuple(inputs),
        outputs=outputs,
        defaults=defaults,
        category=_check_category(category, name),
    )
    return NodeFunction(func, node_type, defaults, output_model)


class _Decorate:
    """The decorator `node(category=...)` hands back."""

    def __init__(self, category: str) -> None:
        self._category = category

    def __call__[**P, R](self, func: Callable[P, R], /) -> NodeFunction[P, R]:
        """Build the node type from `func`.

        Parameters
        ----------
        func
            The function to turn into a node.

        Returns
        -------
        NodeFunction[P, R]
            The function, with its node type attached.
        """
        return _build_node(func, self._category)


class NodeDecorator(Protocol):
    """What `node(category=...)` returns: a decorator for a typed function."""

    def __call__[**P, R](self, func: Callable[P, R], /) -> NodeFunction[P, R]:
        """Build the node type from `func`.

        Parameters
        ----------
        func
            The function to turn into a node.
        """
        ...


@overload
def node[**P, R](func: Callable[P, R], /) -> NodeFunction[P, R]: ...


@overload
def node(*, category: str = "") -> NodeDecorator: ...


def node[**P, R](
    func: Callable[P, R] | None = None, /, *, category: str = ""
) -> NodeFunction[P, R] | NodeDecorator:
    """Turn a typed function into a node type, bare or with a category.

    `@node` and `@node()` give a top-level node; `@node(category="Math")`
    puts it in the `Math` menu, and `"Math/Trig"` in `Trig` inside `Math`.
    See `_build_node` for how the ports come from the hints.

    Parameters
    ----------
    func
        The function, when used as plain `@node`.
    category
        Where the node goes in the add-node menu.

    Returns
    -------
    NodeFunction[P, R] | NodeDecorator
        The function with its node type, or a decorator that makes one.
    """
    if func is None:
        return _Decorate(category)
    return _build_node(func, category)


def _check_category(category: str, where: str) -> str:
    """Return `category` if it is well formed.

    Returns
    -------
    str
        The same category.

    Raises
    ------
    ValueError
        If a level is empty or has a space at either end.
    """
    if category and any(
        not level or level != level.strip() for level in category.split("/")
    ):
        msg = (
            f"{where}: category {category!r} must be levels separated by '/', "
            "none empty and none with spaces at the ends"
        )
        raise ValueError(msg)
    return category
