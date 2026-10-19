"""Tests for the editor server."""

import asyncio
import http.client
import socket
import threading
from collections.abc import Generator
from contextlib import contextmanager
from http import HTTPStatus
from pathlib import Path

import pytest
import uvicorn
from sample_nodes import add, show
from starlette.types import ASGIApp
from starlette.websockets import WebSocketDisconnect
from websockets.exceptions import ConnectionClosed
from websockets.sync.client import connect

from element_51 import server
from element_51.engine import RunResult
from element_51.protocol import Ack, Reject, Snapshot
from element_51.session import App

WASM_MAGIC = b"\x00asm\x01\x00\x00\x00"
REQUEST_ID_7 = 7
REQUEST_ID_8 = 8
INVALID_PAYLOAD = 1007
SILENCE = 0.3


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


def test_edits_made_in_the_browser_are_there_after_a_reload(
    web_dir: Path,
) -> None:
    """The browser builds a graph, goes away, and a new page gets it back."""
    assert web_dir.exists()
    app = App([add])
    requests = [
        (
            '{"op": "edit", "id": 1, "edit": {"kind": "add_node", "id": "n1",'
            ' "node_type": "add", "x": 10.0, "y": 20.0}}'
        ),
        (
            '{"op": "edit", "id": 2, "edit": {"kind": "add_node", "id": "n2",'
            ' "node_type": "add", "x": 300.0, "y": 40.0}}'
        ),
        (
            '{"op": "edit", "id": 3, "edit": {"kind": "add_link",'
            ' "source": {"node": "n1", "port": "out"},'
            ' "target": {"node": "n2", "port": "a"}}}'
        ),
    ]
    with running(server.asgi_app(app)) as port:
        url = f"ws://{server.HOST}:{port}/ws"
        with connect(url) as before_reload:
            before_reload.recv()
            for request in requests:
                before_reload.send(request)
                Ack.model_validate_json(before_reload.recv())
        with connect(url) as after_reload:
            snapshot = Snapshot.model_validate_json(after_reload.recv())
    assert [(n.id, n.x, n.y) for n in snapshot.graph.nodes] == [
        ("n1", 10.0, 20.0),
        ("n2", 300.0, 40.0),
    ]
    assert [
        (link.source.node, link.target.node, link.target.port)
        for link in snapshot.graph.links
    ] == [("n1", "n2", "a")]


def test_two_open_pages_each_get_the_whole_graph(web_dir: Path) -> None:
    """A second tab is a second connection and starts from a full snapshot."""
    assert web_dir.exists()
    app = App([add])
    app.add_node(add, x=1.0, y=2.0)
    with running(server.asgi_app(app)) as port:
        url = f"ws://{server.HOST}:{port}/ws"
        with connect(url) as first, connect(url) as second:
            one = Snapshot.model_validate_json(first.recv())
            two = Snapshot.model_validate_json(second.recv())
    assert one == two
    assert len(one.graph.nodes) == 1


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


def test_a_run_request_returns_the_outcome_of_every_node(web_dir: Path) -> None:
    """The browser's Run button: every node comes back with a status."""
    assert web_dir.exists()
    app = App([add])
    app.add_node(add, x=0.0, y=0.0)
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        websocket.recv()
        websocket.send('{"op": "run"}')
        result = RunResult.model_validate_json(websocket.recv())
    assert [(o.node, o.status) for o in result.outcomes] == [("n1", "error")]
    assert result.outcomes[0].error == "input 'a' has no value"


def test_a_script_edit_reaches_a_page_that_is_already_open(
    web_dir: Path,
) -> None:
    """`app.add_node` after the page connected: the page gets a new snapshot."""
    assert web_dir.exists()
    app = App([add, show])
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        websocket.recv()
        first = app.add_node(add, x=1.0, y=2.0)
        pushed = Snapshot.model_validate_json(websocket.recv(timeout=5))
        second = app.add_node(show, x=3.0, y=4.0)
        websocket.recv(timeout=5)
        app.add_link(first, "out", second, "value")
        linked = Snapshot.model_validate_json(websocket.recv(timeout=5))
    assert [n.id for n in pushed.graph.nodes] == ["n1"]
    assert [(n.source.node, n.target.node) for n in linked.graph.links] == [
        ("n1", "n2")
    ]


def test_an_edit_in_one_page_reaches_the_other_page_only(web_dir: Path) -> None:
    """The sender gets only its ack; the other page gets a snapshot."""
    assert web_dir.exists()
    app = App([add])
    request = (
        '{"op": "edit", "id": 1, "edit": {"kind": "add_node", "id": "n1",'
        ' "node_type": "add", "x": 3.0, "y": 4.0}}'
    )
    with running(server.asgi_app(app)) as port:
        url = f"ws://{server.HOST}:{port}/ws"
        with connect(url) as sender, connect(url) as watcher:
            sender.recv()
            watcher.recv()
            sender.send(request)
            Ack.model_validate_json(sender.recv(timeout=5))
            pushed = Snapshot.model_validate_json(watcher.recv(timeout=5))
            with pytest.raises(TimeoutError):
                sender.recv(timeout=SILENCE)
    assert [n.id for n in pushed.graph.nodes] == ["n1"]


def test_a_closed_page_is_forgotten(web_dir: Path) -> None:
    """After one page closes, a script edit still reaches the other."""
    assert web_dir.exists()
    app = App([add])
    with running(server.asgi_app(app)) as port:
        url = f"ws://{server.HOST}:{port}/ws"
        with connect(url) as stays:
            stays.recv()
            with connect(url) as leaves:
                leaves.recv()
            # The server notices the close in its own time; edit until it has.
            app.add_node(add, x=0.0, y=0.0)
            pushed = Snapshot.model_validate_json(stays.recv(timeout=5))
    assert len(pushed.graph.nodes) == 1


class _GoneSender:
    """A page whose socket closed before the message could be sent."""

    def __init__(self, error: Exception) -> None:
        self.error = error

    async def send_text(self, data: str) -> None:
        assert data == "hello"
        raise self.error


@pytest.mark.parametrize(
    "error", [WebSocketDisconnect(code=1006), RuntimeError("closed")]
)
def test_the_sender_task_ends_quietly_when_the_page_is_gone(
    error: Exception,
) -> None:
    """A send to a closed socket stops the task; nothing is raised."""

    async def scenario() -> None:
        connection = server.Connection()
        connection.send("hello")
        await connection.pump(_GoneSender(error))

    asyncio.run(scenario())


def test_loading_a_file_updates_a_page_that_is_already_open(
    web_dir: Path, tmp_path: Path
) -> None:
    """`app.load` after the page connected: the page gets the loaded graph."""
    assert web_dir.exists()
    source = App([add, show])
    first = source.add_node(add, x=1.0, y=2.0)
    second = source.add_node(show, x=3.0, y=4.0)
    source.add_link(first, "out", second, "value")
    source.save(tmp_path / "g.json")
    app = App([add, show])
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        websocket.recv()
        app.load(tmp_path / "g.json")
        pushed = Snapshot.model_validate_json(websocket.recv(timeout=5))
    assert pushed.graph == source.graph


def test_move_and_delete_from_the_browser_change_the_graph(
    web_dir: Path,
) -> None:
    """A moved node and a deleted node are acknowledged and applied."""
    assert web_dir.exists()
    app = App([add])
    app.add_node(add, x=0.0, y=0.0)
    app.add_node(add, x=1.0, y=1.0)
    move = (
        '{"op": "edit", "id": 1, "edit": {"kind": "move_node",'
        ' "id": "n1", "x": 9.5, "y": 8.0}}'
    )
    delete = (
        '{"op": "edit", "id": 2, "edit": {"kind": "delete_node", "id": "n2"}}'
    )
    with (
        running(server.asgi_app(app)) as port,
        connect(f"ws://{server.HOST}:{port}/ws") as websocket,
    ):
        websocket.recv()
        for request in (move, delete):
            websocket.send(request)
            Ack.model_validate_json(websocket.recv(timeout=5))
    assert [(n.id, n.x, n.y) for n in app.graph.nodes] == [("n1", 9.5, 8.0)]


def test_a_value_set_in_the_browser_is_applied_and_pushed(
    web_dir: Path,
) -> None:
    """Typing into an input: ack to the sender, snapshot to the other page."""
    assert web_dir.exists()
    app = App([add])
    app.add_node(add, x=0.0, y=0.0)
    request = (
        '{"op": "edit", "id": 1, "edit": {"kind": "set_value",'
        ' "node": "n1", "port": "a", "value": 4.5}}'
    )
    with running(server.asgi_app(app)) as port:
        url = f"ws://{server.HOST}:{port}/ws"
        with connect(url) as sender, connect(url) as watcher:
            sender.recv()
            watcher.recv()
            sender.send(request)
            Ack.model_validate_json(sender.recv(timeout=5))
            pushed = Snapshot.model_validate_json(watcher.recv(timeout=5))
    assert app.graph.nodes[0].values == {"a": 4.5}
    assert pushed.graph.nodes[0].values == {"a": 4.5}
