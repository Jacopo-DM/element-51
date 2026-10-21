"""Save a graph to a JSON file and read it back.

The file holds a `format_version`, the nodes with their positions, and the
links. Reading replays every node and link through the same rules as an edit,
so a file that breaks the rules is refused instead of loaded.
"""

import json
from collections.abc import Mapping
from pathlib import Path

from pydantic import ConfigDict, ValidationError

from element_51.edits import AddLink, AddNode, EditError, SetValue, apply_edit
from element_51.model import Graph, Link, Model, NodeType, PlacedNode

FORMAT_VERSION = 2
# Version 1 had no node values; it reads as version 2 with none.
READABLE_VERSIONS = frozenset({1, FORMAT_VERSION})


class StoreError(Exception):
    """A graph file that cannot be loaded. The message says why."""


class _Header(Model):
    """The part of a file every version has: which version it is."""

    model_config = ConfigDict(frozen=True, extra="ignore")

    format_version: int


class _SavedGraph(Model):
    """A graph file, as version 2 writes it and version 1 reads."""

    format_version: int
    nodes: tuple[PlacedNode, ...]
    links: tuple[Link, ...]


def save_graph(graph: Graph, path: Path) -> None:
    """Write `graph` to `path` as JSON, replacing any file there.

    Parameters
    ----------
    graph
        The graph to write.
    path
        The file to write.
    """
    saved = _SavedGraph(
        format_version=FORMAT_VERSION, nodes=graph.nodes, links=graph.links
    )
    path.write_text(saved.model_dump_json(indent=2) + "\n", "utf-8")


def load_graph(path: Path, node_types: Mapping[str, NodeType]) -> Graph:
    """Read a graph from `path` and check it against `node_types`.

    Parameters
    ----------
    path
        The file to read.
    node_types
        Every node type the graph may use, by name.

    Returns
    -------
    Graph
        The graph in the file.

    Raises
    ------
    StoreError
        If the file is not UTF-8 text, is not JSON, is not a graph file, has
        a format version this library does not read, or holds a graph the
        link rules refuse.
    """
    try:
        text = path.read_text("utf-8")
    except UnicodeDecodeError as error:
        msg = f"{path} is not a graph file: it is not UTF-8 text"
        raise StoreError(msg) from error
    try:
        data: object = json.loads(text)
    except json.JSONDecodeError as error:
        msg = f"{path} is not valid JSON: {error}"
        raise StoreError(msg) from error
    try:
        header = _Header.model_validate(data)
        if header.format_version not in READABLE_VERSIONS:
            msg = (
                f"{path} is format version {header.format_version}; "
                "this library reads versions 1 and 2"
            )
            raise StoreError(msg)
        saved = _SavedGraph.model_validate(data)
    except ValidationError as error:
        msg = f"{path} is not a graph file: {error}"
        raise StoreError(msg) from error
    graph = Graph(nodes=(), links=())
    try:
        for placed in saved.nodes:
            graph = apply_edit(
                graph,
                node_types,
                AddNode(
                    id=placed.id,
                    node_type=placed.node_type,
                    x=placed.x,
                    y=placed.y,
                ),
            )
        for placed in saved.nodes:
            for port, value in placed.values.items():
                graph = apply_edit(
                    graph,
                    node_types,
                    SetValue(node=placed.id, port=port, value=value),
                )
        for link in saved.links:
            graph = apply_edit(
                graph,
                node_types,
                AddLink(source=link.source, target=link.target),
            )
    except EditError as error:
        msg = f"{path} holds a graph the rules refuse: {error.reason}"
        raise StoreError(msg) from error
    return graph
