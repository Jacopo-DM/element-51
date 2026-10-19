"""Messages between Python and the browser, as JSON."""

from typing import Literal

from element_51.model import Graph, Model, NodeType


class Snapshot(Model):
    """The whole state, sent when a browser connects."""

    op: Literal["snapshot"] = "snapshot"
    version: Literal[1] = 1
    node_types: tuple[NodeType, ...]
    graph: Graph
