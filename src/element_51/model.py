"""Records for node types and graphs.

Every record is immutable and refuses unknown fields. The JSON these produce
is the format the browser layer reads, so field names are a contract with
`rust/element-51-core/src/model.rs`.
"""

from enum import StrEnum
from typing import Annotated, TypeGuard

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


def can_link(source: PortType, target: PortType) -> bool:
    """Whether an output of type `source` may feed an input of type `target`.

    Equal types link. An int output may also feed a float input.

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


class NodeType(Model):
    """The shape of a node: its name and its ports, in drawing order."""

    name: str
    inputs: tuple[Port, ...]
    outputs: tuple[Port, ...]


class PortRef(Model):
    """One end of a link: a port on a node in the graph."""

    node: str
    port: str


class Link(Model):
    """A link from an output port to an input port."""

    source: PortRef
    target: PortRef


class PlacedNode(Model):
    """A node placed in the graph."""

    id: NodeId
    node_type: str
    x: float
    y: float


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
