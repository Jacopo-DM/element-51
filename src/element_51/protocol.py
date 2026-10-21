"""Messages between Python and the browser, as JSON."""

from typing import Annotated, Literal

from pydantic import Field, TypeAdapter

from element_51.edits import Edit
from element_51.model import Graph, Model, NodeType
from element_51.theme import ThemeChanges


class Snapshot(Model):
    """The whole state, sent when a browser connects."""

    op: Literal["snapshot"] = "snapshot"
    version: Literal[2] = 2
    theme: str
    theme_changes: ThemeChanges = ThemeChanges()
    node_types: tuple[NodeType, ...]
    graph: Graph


class EditRequest(Model):
    """The browser asks Python to confirm an edit it already applied."""

    op: Literal["edit"] = "edit"
    id: int
    edit: Edit


class Ack(Model):
    """Python confirms the edit with this request id."""

    op: Literal["ack"] = "ack"
    id: int


class Reject(Model):
    """Python refuses the edit with this request id."""

    op: Literal["reject"] = "reject"
    id: int
    reason: str


class RunRequest(Model):
    """The browser asks Python to run the graph."""

    op: Literal["run"] = "run"


class SourceRequest(Model):
    """The browser asks for the source code of one node's type."""

    op: Literal["source"] = "source"
    node: str


class SourceReply(Model):
    """Python answers a `SourceRequest`: the code, or why there is none.

    Exactly one of `source` and `reason` is set. The reply goes to the page
    that asked and to no other.
    """

    op: Literal["source_reply"] = "source_reply"
    node: str
    source: str | None = None
    reason: str | None = None


ClientMessage: TypeAdapter[EditRequest | RunRequest | SourceRequest] = (
    TypeAdapter(
        Annotated[
            EditRequest | RunRequest | SourceRequest,
            Field(discriminator="op"),
        ]
    )
)
