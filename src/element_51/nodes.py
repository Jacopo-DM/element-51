"""The `@node` decorator: turn a typed function into a node type."""

import inspect
from collections.abc import Callable, Mapping
from typing import Protocol, cast, get_type_hints

from element_51.model import NodeType, Port, PortType, PortValue, value_fits

PORT_TYPES: dict[type, PortType] = {
    float: PortType.FLOAT,
    int: PortType.INT,
    str: PortType.STR,
    bool: PortType.BOOL,
}


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
    """What the session and the engine need from a node function."""

    @property
    def node_type(self) -> NodeType:
        """The node type built from the function's hints."""
        ...

    @property
    def defaults(self) -> Mapping[str, PortValue]:
        """The Python defaults, by input name, for inputs left unlinked."""
        ...

    def call(self, inputs: Mapping[str, PortValue]) -> object:
        """Call the function with every input by name.

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
    ) -> None:
        self.func = func
        self.node_type = node_type
        self.defaults = defaults

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


def port_type(annotation: object, where: str) -> PortType:
    """Map a type hint to a port type.

    Parameters
    ----------
    annotation
        The hint from the function signature.
    where
        Words naming the hint in an error, for example "parameter 'a'".

    Returns
    -------
    PortType
        The port type for the hint.

    Raises
    ------
    TypeError
        If the hint is not float, int, str or bool.
    """
    if isinstance(annotation, type) and annotation in PORT_TYPES:
        return PORT_TYPES[annotation]
    msg = f"{where}: {annotation!r} is not one of float, int, str, bool"
    raise TypeError(msg)


def node[**P, R](func: Callable[P, R]) -> NodeFunction[P, R]:
    """Build a node type from a typed function.

    Each parameter becomes an input port and the return type becomes one
    output port named `out`.

    Parameters
    ----------
    func
        A function whose parameters and return value are annotated with
        float, int, str or bool.

    Returns
    -------
    NodeFunction[P, R]
        The function, with its node type attached.

    Raises
    ------
    TypeError
        If `func` is not a plain function, a parameter or the return value
        is missing a supported hint, or
        the function takes `*args` or `**kwargs`.
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
        kind = port_type(hints[parameter.name], where)
        if parameter.default is not parameter.empty:
            if not value_fits(parameter.default, kind):
                msg = f"{where}: default {parameter.default!r} is not a {kind}"
                raise TypeError(msg)
            defaults[parameter.name] = parameter.default
        inputs.append(Port(name=parameter.name, kind=kind))
    if "return" not in hints:
        msg = f"{name}, return: no type hint"
        raise TypeError(msg)
    output = Port(
        name="out", kind=port_type(hints["return"], f"{name}, return")
    )
    node_type = NodeType(name=name, inputs=tuple(inputs), outputs=(output,))
    return NodeFunction(func, node_type, defaults)
