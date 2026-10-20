"""Tests for the editor server's themes and the FastAPI host example."""

from http import HTTPStatus
from pathlib import Path

import pytest
import uvicorn
from sample_nodes import add
from server_support import fetch, post, running
from test_examples import load
from websockets.sync.client import connect

from element_51 import server
from element_51.protocol import Snapshot
from element_51.session import App


def _do_not_run(server_: uvicorn.Server) -> None:
    """Stand in for `uvicorn.Server.run`, which would block."""
    assert server_ is not None


def test_serve_chooses_the_theme_and_every_snapshot_names_it(
    web_dir: Path,
) -> None:
    """`serve(theme=...)` sets it; a page that connects is told which."""
    assert web_dir.exists()
    app = App([add])
    with pytest.MonkeyPatch.context() as patch:
        patch.setattr(uvicorn.Server, "run", _do_not_run)
        server.serve(app, port=9000, theme="antimony-light")
    assert app.theme == "antimony-light"
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        snapshot = Snapshot.model_validate_json(websocket.recv())
    assert snapshot.theme == "antimony-light"


def test_serve_with_an_unknown_theme_raises_before_starting(
    web_dir: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """The error names the valid themes, and no server starts."""
    assert web_dir.exists()
    started: list[bool] = []

    def record_run(
        self: uvicorn.Server,
    ) -> None:  # pragma: no cover  # serve must refuse first
        started.append(self is not None)

    monkeypatch.setattr(uvicorn.Server, "run", record_run)
    with pytest.raises(ValueError, match="antimony-light"):
        server.serve(App([add]), theme="nope")
    assert started == []


def test_asgi_app_takes_the_theme_too(web_dir: Path) -> None:
    """A mounted app picks its theme the same way."""
    assert web_dir.exists()
    app = App([add])
    server.asgi_app(app, theme="antimony-light")
    assert app.theme == "antimony-light"


def test_serve_takes_a_theme_file_and_the_page_is_sent_its_changes(
    web_dir: Path, tmp_path: Path
) -> None:
    """`serve(theme="my_theme.json")` sends the file's values to the page."""
    assert web_dir.exists()
    path = tmp_path / "my_theme.json"
    path.write_text('{"colours": {"canvas": "#101010"}}', encoding="utf-8")
    app = App([add])
    with pytest.MonkeyPatch.context() as patch:
        patch.setattr(uvicorn.Server, "run", _do_not_run)
        server.serve(app, port=9000, theme=str(path))
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        snapshot = Snapshot.model_validate_json(websocket.recv())
    assert snapshot.theme == "purple-synthwave"
    assert snapshot.theme_changes.colours == {"canvas": "#101010"}


def test_the_fastapi_example_hosts_the_editor_under_a_prefix(
    web_dir: Path,
) -> None:
    """The panel, the editor page and its websocket share one process.

    The editor's websocket is at `/editor/ws`; a node added by the panel's
    API reaches the page that is already open, and `Run` sees it.
    """
    assert web_dir.exists()
    example = load("f_embedding/f1_fastapi_host.py")
    host = example.build_host(example.build_app())
    with running(host) as port:
        status, _, panel = fetch(port, "/")
        assert status == HTTPStatus.OK
        assert b'<iframe src="/editor/">' in panel
        assert fetch(port, "/editor/")[2] == b"<p>element-51</p>"
        with connect(f"ws://{server.HOST}:{port}/editor/ws") as websocket:
            first = Snapshot.model_validate_json(websocket.recv())
            assert [n.id for n in first.graph.nodes] == ["n1"]
            assert post(port, "/api/add-node") == {"added": "n2"}
            pushed = Snapshot.model_validate_json(websocket.recv())
            assert [n.id for n in pushed.graph.nodes] == ["n1", "n2"]
        assert post(port, "/api/run") == {
            "n1": {"out": 3.0},
            "n2": {"out": 3.0},
        }
