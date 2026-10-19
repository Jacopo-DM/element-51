"""Tests for the editor server."""

import http.client
import socket
import threading
from collections.abc import Generator
from contextlib import contextmanager
from http import HTTPStatus
from pathlib import Path

import pytest
import uvicorn
from sample_nodes import add
from starlette.types import ASGIApp
from websockets.exceptions import ConnectionClosed
from websockets.sync.client import connect

from element_51 import server
from element_51.protocol import Ack, Reject, Snapshot
from element_51.session import App

WASM_MAGIC = b"\x00asm\x01\x00\x00\x00"
REQUEST_ID_7 = 7
REQUEST_ID_8 = 8
INVALID_PAYLOAD = 1007


class _Server(uvicorn.Server):
    """A uvicorn server that signals when it is listening."""

    def __init__(self, config: uvicorn.Config) -> None:
        super().__init__(config)
        self.ready = threading.Event()

    async def startup(self, sockets: list[socket.socket] | None = None) -> None:
        await super().startup(sockets)
        self.ready.set()


@contextmanager
def running(app: ASGIApp) -> Generator[int]:
    """Run `app` on a free local port.

    Yields
    ------
    int
        The port the server listens on.
    """
    config = uvicorn.Config(app, host=server.HOST, port=0, log_level="warning")
    instance = _Server(config)
    thread = threading.Thread(target=instance.run)
    thread.start()
    assert instance.ready.wait(timeout=10)
    try:
        yield instance.servers[0].sockets[0].getsockname()[1]
    finally:
        instance.should_exit = True
        thread.join()


def fetch(port: int, path: str) -> tuple[int, str, bytes]:
    """GET `path` from the local server.

    Returns
    -------
    tuple[int, str, bytes]
        Status, content type and body.
    """
    connection = http.client.HTTPConnection(server.HOST, port, timeout=10)
    try:
        connection.request("GET", path)
        response = connection.getresponse()
        return (
            response.status,
            response.getheader("content-type") or "",
            response.read(),
        )
    finally:
        connection.close()


@pytest.fixture
def web_dir(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    """Build a fake output folder in place of `src/element_51/web/`.

    Returns
    -------
    Path
        The folder, holding `index.html` and one `.wasm` file.
    """
    (tmp_path / "index.html").write_text("<p>element-51</p>")
    (tmp_path / "element_51_web_bg.wasm").write_bytes(WASM_MAGIC)
    monkeypatch.setattr(server, "WEB_DIR", tmp_path)
    return tmp_path


def test_index_page_is_served_at_root(web_dir: Path) -> None:
    """The page loads at `/` over a real local server."""
    assert web_dir.exists()
    with running(server.asgi_app(App([]))) as port:
        status, content_type, body = fetch(port, "/")
    assert status == HTTPStatus.OK
    assert content_type.startswith("text/html")
    assert body == b"<p>element-51</p>"


def test_wasm_is_served_with_the_wasm_type(web_dir: Path) -> None:
    """Browsers only stream-compile `.wasm` sent as application/wasm."""
    assert web_dir.exists()
    with running(server.asgi_app(App([]))) as port:
        status, content_type, body = fetch(port, "/element_51_web_bg.wasm")
    assert (status, content_type, body) == (
        HTTPStatus.OK,
        "application/wasm",
        WASM_MAGIC,
    )


def test_unknown_path_is_404(web_dir: Path) -> None:
    """A path outside the build output is not found."""
    assert web_dir.exists()
    with running(server.asgi_app(App([]))) as port:
        status, _, _ = fetch(port, "/nope.txt")
    assert status == HTTPStatus.NOT_FOUND


def test_serve_listens_on_localhost_only(
    web_dir: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """`serve` binds 127.0.0.1 and passes the port through."""
    assert web_dir.exists()
    calls: list[tuple[str, int]] = []

    def fake_run(app: ASGIApp, *, host: str, port: int) -> None:
        assert app is not None
        calls.append((host, port))

    monkeypatch.setattr(uvicorn, "run", fake_run)
    server.serve(App([]), port=9000)
    assert calls == [("127.0.0.1", 9000)]


def test_websocket_sends_a_snapshot_on_connect(web_dir: Path) -> None:
    """A browser that connects gets the node types and the graph."""
    assert web_dir.exists()
    app = App([add])
    app.add_node(add, x=5.0, y=6.0)
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        message = websocket.recv()
    snapshot = Snapshot.model_validate_json(message)
    assert [t.name for t in snapshot.node_types] == ["add"]
    assert [(n.id, n.x, n.y) for n in snapshot.graph.nodes] == [
        ("n1", 5.0, 6.0)
    ]


def test_every_new_connection_gets_the_current_graph(web_dir: Path) -> None:
    """A reload sees the edits made since the last connection."""
    assert web_dir.exists()
    app = App([add])
    with running(server.asgi_app(app)) as port:
        url = f"ws://{server.HOST}:{port}/ws"
        with connect(url) as first:
            before = Snapshot.model_validate_json(first.recv())
        app.add_node(add, x=1.0, y=1.0)
        with connect(url) as second:
            after = Snapshot.model_validate_json(second.recv())
    assert len(before.graph.nodes) == 0
    assert len(after.graph.nodes) == 1


def test_an_edit_from_the_browser_is_applied_and_acknowledged(
    web_dir: Path,
) -> None:
    """The browser adds a node; Python acks it and `app.graph` has it."""
    assert web_dir.exists()
    app = App([add])
    request = (
        '{"op": "edit", "id": 7, "edit": {"kind": "add_node", "id": "n1",'
        ' "node_type": "add", "x": 3.0, "y": 4.0}}'
    )
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        websocket.recv()  # the snapshot
        websocket.send(request)
        reply = Ack.model_validate_json(websocket.recv())
    assert reply.id == REQUEST_ID_7
    assert [(n.id, n.x, n.y) for n in app.graph.nodes] == [("n1", 3.0, 4.0)]


def test_a_bad_edit_is_rejected_with_a_reason(web_dir: Path) -> None:
    """A link to a node that does not exist gets `reject`, not a crash."""
    assert web_dir.exists()
    app = App([add])
    request = (
        '{"op": "edit", "id": 8, "edit": {"kind": "add_link",'
        ' "source": {"node": "n1", "port": "out"},'
        ' "target": {"node": "n2", "port": "a"}}}'
    )
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        websocket.recv()
        websocket.send(request)
        reply = Reject.model_validate_json(websocket.recv())
        websocket.send(request)  # the connection is still usable
        again = Reject.model_validate_json(websocket.recv())
    assert reply.id == REQUEST_ID_8
    assert "n1" in reply.reason
    assert again.id == reply.id
    assert app.graph.links == ()


def test_a_malformed_message_closes_the_connection(web_dir: Path) -> None:
    """Garbage is not guessed at: the socket closes with 1007 and a reason."""
    assert web_dir.exists()
    with (
        running(server.asgi_app(App([add]))) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        websocket.recv()
        websocket.send("not json")
        with pytest.raises(ConnectionClosed) as closed:
            websocket.recv()
    assert closed.value.rcvd is not None
    assert closed.value.rcvd.code == INVALID_PAYLOAD
    assert "malformed" in closed.value.rcvd.reason
