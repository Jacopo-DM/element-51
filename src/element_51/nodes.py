"""The `@node` decorator: turn a typed function into a node type."""

import inspect
from collections.abc import Callable
from typing import Protocol, get_type_hints

from element_51.model import NodeType, Port, PortType

PORT_TYPES: dict[type, PortType] = {
    float: PortType.FLOAT,
    int: PortType.INT,
    str: PortType.STR,
    bool: PortType.BOOL,
}


class Registered(Protocol):
    """What the session needs from a node function: its node type."""

    @property
    def node_type(self) -> NodeType:
        """The node type built from the function's hints."""
        ...


class NodeFunction[**P, R]:
    """A function together with the node type built from its hints.

    Calling it calls the function, with the same signature. This holds
    behaviour, not just fields, which is why it is a class and not a pydantic
    record.
    """

    def __init__(self, func: Callable[P, R], node_type: NodeType) -> None:
        self.func = func
        self.node_type = node_type

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
    for parameter in inspect.signature(func).parameters.values():
        if parameter.kind in {parameter.VAR_POSITIONAL, parameter.VAR_KEYWORD}:
            msg = f"{name}: *args and **kwargs have no fixed ports"
            raise TypeError(msg)
        where = f"{name}, parameter {parameter.name!r}"
        if parameter.name not in hints:
            msg = f"{where}: no type hint"
            raise TypeError(msg)
        kind = port_type(hints[parameter.name], where)
        inputs.append(Port(name=parameter.name, kind=kind))
    if "return" not in hints:
        msg = f"{name}, return: no type hint"
        raise TypeError(msg)
    output = Port(
        name="out", kind=port_type(hints["return"], f"{name}, return")
    )
    node_type = NodeType(name=name, inputs=tuple(inputs), outputs=(output,))
    return NodeFunction(func, node_type)
