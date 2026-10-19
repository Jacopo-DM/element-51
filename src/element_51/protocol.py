"""Messages between Python and the browser, as JSON."""

from typing import Literal

from element_51.edits import Edit
from element_51.model import Graph, Model, NodeType


class Snapshot(Model):
    """The whole state, sent when a browser connects."""

    op: Literal["snapshot"] = "snapshot"
    version: Literal[1] = 1
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
