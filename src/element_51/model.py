"""Records for node types and graphs.

Every record is immutable and refuses unknown fields. The JSON these produce
is the format the browser layer reads, so field names are a contract with
`rust/element-51-core/src/model.rs`.
"""

from collections.abc import Mapping
from enum import StrEnum
from typing import Annotated, Literal, TypeGuard

from pydantic import BaseModel, ConfigDict, Field


class Model(BaseModel):
    """Base for every record in this project: immutable, no unknown fields."""

    model_config = ConfigDict(frozen=True, extra="forbid")


class PortType(StrEnum):
    """The type of a port. A link may join two ports only if types agree."""

    FLOAT = "float"
    INT = "int"
    STR = "str"
    BOOL = "bool"
    IMAGE = "image"
    SERIES = "series"


def can_link(source: PortType, target: PortType) -> bool:
    """Whether an output of type `source` may feed an input of type `target`.

    Equal types link. An int output may also feed a float input. A picture
    links only to a picture, and a series only to a series.

    Parameters
    ----------
    source
        The type of the output port.
    target
        The type of the input port.

    Returns
    -------
    bool
        True if the link is allowed.
    """
    return source == target or (
        source is PortType.INT and target is PortType.FLOAT
    )


type NodeId = Annotated[str, Field(pattern=r"^n[0-9]+$")]

type PortValue = float | int | str | bool


def value_fits(value: object, kind: PortType) -> TypeGuard[PortValue]:
    """Whether `value` is something a port of type `kind` can carry.

    The rules are stricter than Python's: a bool is not an int, and an int
    may stand in for a float.

    Parameters
    ----------
    value
        A value from a node function or a default.
    kind
        The port type.

    Returns
    -------
    bool
        True if the value fits.
    """
    if isinstance(value, bool):
        return kind is PortType.BOOL
    if kind is PortType.FLOAT:
        return isinstance(value, int | float)
    if kind is PortType.INT:
        return isinstance(value, int)
    return kind is PortType.STR and isinstance(value, str)


class Port(Model):
    """One input or output of a node type."""

    name: str
    kind: PortType


class Variadic(Model):
    """A node type whose inputs grow as they are linked.

    The rows are named `<prefix>_1`, `<prefix>_2`, and so on, and every row
    has the kind `kind`. A row that is not listed in `NodeType.inputs` exists
    as soon as something links to it.
    """

    prefix: str
    kind: PortType


def variadic_index(variadic: Variadic, name: str) -> int | None:
    """Read the row number from a port name.

    Parameters
    ----------
    variadic
        The row naming of the node type.
    name
        A port name.

    Returns
    -------
    int | None
        The row number if `name` is the prefix, an underscore and a positive
        whole number with no sign and no leading zero, else None.
    """
    head, separator, tail = name.rpartition("_")
    if (
        separator
        and head == variadic.prefix
        and tail.isascii()
        and tail.isdigit()
        and tail[0] != "0"
    ):
        return int(tail)
    return None


class NodeType(Model):
    """The shape of a node: its name and its ports, in drawing order.

    `defaults` holds the value an unlinked input takes when the node has no
    value of its own; an input with no default must be given one. `category`
    places the type in the add-node menu: `""` for the top level, with `/`
    between levels (`"Math/Trig"` is `Trig` inside `Math`). `display` says what
    the card shows below its ports: `"image"` for a node with a picture output,
    `"chart"` for a node that plots the series linked to it, else `"none"`.
    `variadic` makes the inputs grow: see `Variadic`. `job` is true for a node
    that is started and stopped by hand and works in the background.
    """

    name: str
    inputs: tuple[Port, ...]
    outputs: tuple[Port, ...]
    defaults: Mapping[str, PortValue] = {}
    category: str = ""
    display: Literal["none", "image", "chart"] = "none"
    variadic: Variadic | None = None
    job: bool = False


def find_input(node_type: NodeType, name: str) -> Port | None:
    """Find the input `name` of a node type, rows that grow included.

    Returns
    -------
    Port | None
        The listed input of that name, or for a variadic type a port of the
        variadic kind if `name` is a row, else None.
    """
    for port in node_type.inputs:
        if port.name == name:
            return port
    variadic = node_type.variadic
    if variadic is not None and variadic_index(variadic, name) is not None:
        return Port(name=name, kind=variadic.kind)
    return None


class PortRef(Model):
    """One end of a link: a port on a node in the graph."""

    node: str
    port: str


class Link(Model):
    """A link from an output port to an input port."""

    source: PortRef
    target: PortRef


class PlacedNode(Model):
    """A node placed in the graph.

    `values` holds the values the user typed for unlinked inputs, by input
    name. They win over the node type's defaults.
    """

    id: NodeId
    node_type: str
    x: float
    y: float
    values: Mapping[str, PortValue] = {}


class Graph(Model):
    """The nodes and links the user has built."""

    nodes: tuple[PlacedNode, ...]
    links: tuple[Link, ...]

    def next_id(self) -> str:
        """Return an unused node id, one above the highest in use.

        Returns
        -------
        str
            An id of the form `n<number>`.
        """
        used = (int(node.id.removeprefix("n")) for node in self.nodes)
        return f"n{max(used, default=0) + 1}"
