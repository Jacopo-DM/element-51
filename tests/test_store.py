"""Tests for saving and loading a graph as versioned JSON."""

import json
import tempfile
from pathlib import Path
from typing import TYPE_CHECKING

import pytest
from hypothesis import example, given
from hypothesis import strategies as st
from sample_nodes import add, show

from element_51.edits import SetValue
from element_51.model import Graph, Link, PlacedNode, PortRef
from element_51.session import App
from element_51.store import (
    FORMAT_VERSION,
    StoreError,
    load_graph,
    save_graph,
)

if TYPE_CHECKING:
    from element_51.protocol import Snapshot


def built_app() -> App:
    """Build an app with two linked nodes at known positions.

    Returns
    -------
    App
        An app holding `add` at (10, 20) linked into `show` at (300, 40).
    """
    app = App([add, show])
    first = app.add_node(add, x=10.0, y=20.0)
    second = app.add_node(show, x=300.0, y=40.0)
    app.add_link(first, "out", second, "value")
    return app


def test_save_writes_a_version_nodes_links_and_positions(
    tmp_path: Path,
) -> None:
    """The file is plain JSON a person can read."""
    path = tmp_path / "g.json"
    built_app().save(path)
    data = json.loads(path.read_text("utf-8"))
    assert data["format_version"] == FORMAT_VERSION
    assert [
        (n["id"], n["node_type"], n["x"], n["y"]) for n in data["nodes"]
    ] == [
        ("n1", "add", 10.0, 20.0),
        ("n2", "show", 300.0, 40.0),
    ]
    assert data["links"] == [
        {
            "source": {"node": "n1", "port": "out"},
            "target": {"node": "n2", "port": "value"},
        }
    ]


def test_a_saved_graph_loads_back_the_same(tmp_path: Path) -> None:
    """Round trip into a fresh app with the same node types."""
    path = tmp_path / "g.json"
    saved = built_app()
    saved.save(path)
    fresh = App([add, show])
    fresh.load(path)
    assert fresh.graph == saved.graph


def test_load_replaces_the_current_graph_and_tells_listeners(
    tmp_path: Path,
) -> None:
    """A page that is open learns of the loaded graph at once."""
    path = tmp_path / "g.json"
    built_app().save(path)
    app = App([add, show])
    app.add_node(show, x=0.0, y=0.0)
    heard: list[Snapshot] = []
    app.subscribe(heard.append)
    app.load(path)
    assert [n.id for n in app.graph.nodes] == ["n1", "n2"]
    assert heard == [app.snapshot()]


def test_a_version_this_library_does_not_read_is_refused(
    tmp_path: Path,
) -> None:
    """The error names both versions, and the app is left alone."""
    path = tmp_path / "g.json"
    path.write_text(
        json.dumps({"format_version": 99, "nodes": [], "links": []}), "utf-8"
    )
    app = built_app()
    before = app.graph
    with pytest.raises(StoreError, match=r"version 99.*versions 1 and 2"):
        app.load(path)
    assert app.graph == before


@pytest.mark.parametrize(
    ("text", "expected"),
    [
        ("not json", "not valid JSON"),
        ("[]", "not a graph file"),
        ('{"nodes": [], "links": []}', "format_version"),
        ('{"format_version": 1, "nodes": 3, "links": []}', "nodes"),
        (
            '{"format_version": 1, "nodes": [], "links": [], "extra": 1}',
            "extra",
        ),
    ],
)
def test_a_file_that_is_not_a_graph_gives_a_clear_error(
    tmp_path: Path, text: str, expected: str
) -> None:
    """Each kind of damage gets a message that points at the problem."""
    path = tmp_path / "g.json"
    path.write_text(text, "utf-8")
    with pytest.raises(StoreError, match=expected):
        App([add]).load(path)


def test_a_graph_that_breaks_the_link_rules_is_refused(tmp_path: Path) -> None:
    """Loading replays every node and link through the same rules as edits."""
    path = tmp_path / "g.json"
    built_app().save(path)
    app = App([add])  # does not know the `show` type
    app.add_node(add, x=1.0, y=1.0)
    before = app.graph
    with pytest.raises(StoreError, match="show"):
        app.load(path)
    assert app.graph == before


def test_a_missing_file_raises_the_usual_error(tmp_path: Path) -> None:
    """`FileNotFoundError` is not wrapped: it already says what happened."""
    with pytest.raises(FileNotFoundError):
        App([add]).load(tmp_path / "nope.json")


def test_node_values_survive_a_round_trip(tmp_path: Path) -> None:
    """What the user typed into an input is saved and comes back."""
    path = tmp_path / "g.json"
    app = built_app()
    app.apply(SetValue(node="n1", port="a", value=4.5))
    app.save(path)
    fresh = App([add, show])
    fresh.load(path)
    assert fresh.graph.nodes[0].values == {"a": 4.5}


def test_a_version_1_file_without_values_still_loads(tmp_path: Path) -> None:
    """Files written before node values existed read as having none."""
    path = tmp_path / "g.json"
    path.write_text(
        json.dumps({
            "format_version": 1,
            "nodes": [{"id": "n1", "node_type": "add", "x": 1.0, "y": 2.0}],
            "links": [],
        }),
        "utf-8",
    )
    app = App([add])
    app.load(path)
    assert app.graph.nodes[0].values == {}


def test_a_value_that_does_not_fit_its_port_is_refused(tmp_path: Path) -> None:
    """A hand-edited file with text in a float input does not load."""
    path = tmp_path / "g.json"
    path.write_text(
        json.dumps({
            "format_version": 2,
            "nodes": [
                {
                    "id": "n1",
                    "node_type": "add",
                    "x": 0.0,
                    "y": 0.0,
                    "values": {"a": "text"},
                }
            ],
            "links": [],
        }),
        "utf-8",
    )
    with pytest.raises(StoreError, match="not a float"):
        App([add]).load(path)


NODE_TYPES = {"add": add.node_type, "show": show.node_type}
SAVED_GRAPH = json.dumps({
    "format_version": FORMAT_VERSION,
    "nodes": [
        {"id": "n1", "node_type": "add", "x": 1.0, "y": 2.0},
        {"id": "n2", "node_type": "show", "x": 3.0, "y": 4.0},
    ],
    "links": [
        {
            "source": {"node": "n1", "port": "out"},
            "target": {"node": "n2", "port": "value"},
        }
    ],
})


@given(st.text())
@example(SAVED_GRAPH)
@example(json.dumps({"format_version": 1, "nodes": [], "links": []}))
def test_load_graph_boundary(text: str) -> None:
    """Any file text is a graph or a StoreError, nothing else."""
    with tempfile.TemporaryDirectory() as folder:
        path = Path(folder) / "g.json"
        path.write_text(text, "utf-8")
        try:
            graph = load_graph(path, NODE_TYPES)
        except StoreError:
            return
    assert isinstance(graph, Graph)


@st.composite
def graphs(draw: st.DrawFn) -> Graph:
    """Draw a graph of `add` and `show` nodes that follows the link rules.

    Returns
    -------
    Graph
        Nodes at finite positions, some with typed values, and links from
        `add` outputs to `show` inputs.
    """
    coordinate = st.floats(allow_nan=False, allow_infinity=False)
    value = st.one_of(
        st.floats(allow_nan=False, allow_infinity=False), st.integers()
    )
    adds = draw(st.integers(min_value=1, max_value=4))
    shows = draw(st.integers(min_value=0, max_value=3))
    nodes: list[PlacedNode] = []
    for index in range(adds):
        values = draw(
            st.dictionaries(st.sampled_from(["a", "b"]), value, max_size=2)
        )
        nodes.append(
            PlacedNode(
                id=f"n{index + 1}",
                node_type="add",
                x=draw(coordinate),
                y=draw(coordinate),
                values=values,
            )
        )
    links: list[Link] = []
    for index in range(shows):
        shown_id = f"n{adds + index + 1}"
        nodes.append(
            PlacedNode(
                id=shown_id,
                node_type="show",
                x=draw(coordinate),
                y=draw(coordinate),
            )
        )
        if draw(st.booleans()):
            source = f"n{draw(st.integers(min_value=1, max_value=adds))}"
            links.append(
                Link(
                    source=PortRef(node=source, port="out"),
                    target=PortRef(node=shown_id, port="value"),
                )
            )
    return Graph(nodes=tuple(nodes), links=tuple(links))


@given(graphs())
def test_graph_file_round_trip(graph: Graph) -> None:
    """A saved graph loads back as the same graph, values and links included."""
    with tempfile.TemporaryDirectory() as folder:
        path = Path(folder) / "g.json"
        save_graph(graph, path)
        assert load_graph(path, NODE_TYPES) == graph
