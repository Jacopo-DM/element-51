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
from starlette.types import ASGIApp

from element_51 import server

WASM_MAGIC = b"\x00asm\x01\x00\x00\x00"


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
    with running(server.asgi_app()) as port:
        status, content_type, body = fetch(port, "/")
    assert status == HTTPStatus.OK
    assert content_type.startswith("text/html")
    assert body == b"<p>element-51</p>"


def test_wasm_is_served_with_the_wasm_type(web_dir: Path) -> None:
    """Browsers only stream-compile `.wasm` sent as application/wasm."""
    assert web_dir.exists()
    with running(server.asgi_app()) as port:
        status, content_type, body = fetch(port, "/element_51_web_bg.wasm")
    assert (status, content_type, body) == (
        HTTPStatus.OK,
        "application/wasm",
        WASM_MAGIC,
    )


def test_unknown_path_is_404(web_dir: Path) -> None:
    """A path outside the build output is not found."""
    assert web_dir.exists()
    with running(server.asgi_app()) as port:
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
    server.serve(port=9000)
    assert calls == [("127.0.0.1", 9000)]
