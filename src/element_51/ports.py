"""Map type hints to ports: the part of `@node` and `Node` shared by both."""

import inspect

from pydantic import BaseModel

from element_51.model import Port, PortType, value_fits

PORT_TYPES: dict[type, PortType] = {
    float: PortType.FLOAT,
    int: PortType.INT,
    str: PortType.STR,
    bool: PortType.BOOL,
}

NO_DEFAULT = inspect.Parameter.empty


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


def input_port(
    name: str, annotation: object, default: object, where: str
) -> Port:
    """Build an input port, checking its default against its type.

    Returns
    -------
    Port
        The input port.

    Raises
    ------
    TypeError
        If the type is unsupported or the default does not fit it.
    """
    kind = port_type(annotation, where)
    if default is not NO_DEFAULT and not value_fits(default, kind):
        msg = f"{where}: default {default!r} is not a {kind}"
        raise TypeError(msg)
    return Port(name=name, kind=kind)


def output_ports(
    name: str, annotation: object
) -> tuple[tuple[Port, ...], type[BaseModel] | None]:
    """Build the output ports from a return hint.

    A pydantic model gives one port per field, in the order declared. Any
    other hint gives the single port `out`.

    Returns
    -------
    tuple[tuple[Port, ...], type[BaseModel] | None]
        The ports, and the model if there is one.
    """
    if isinstance(annotation, type) and issubclass(annotation, BaseModel):
        return (
            tuple(
                Port(
                    name=field_name,
                    kind=port_type(
                        info.annotation, f"{name}, return field {field_name!r}"
                    ),
                )
                for field_name, info in annotation.model_fields.items()
            ),
            annotation,
        )
    return (
        Port(name="out", kind=port_type(annotation, f"{name}, return")),
    ), None
