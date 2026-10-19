"""Tests for saving and loading a graph as versioned JSON."""

import json
from pathlib import Path
from typing import TYPE_CHECKING

import pytest
from sample_nodes import add, show

from element_51.session import App
from element_51.store import FORMAT_VERSION, StoreError

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
    with pytest.raises(StoreError, match=r"version 99.*version 1"):
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
