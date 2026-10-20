"""Tests for the editor server: the page, `serve` and the process."""

import signal
import subprocess  # ruff: ignore[suspicious-subprocess-import]  # runs our own script
import sys
import threading
import webbrowser
from http import HTTPStatus
from pathlib import Path

import pytest
import uvicorn
from server_support import WASM_MAGIC, fetch, free_port, running

from element_51 import server
from element_51.session import App


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

    def fake_run(self: uvicorn.Server) -> None:
        calls.append((self.config.host, self.config.port))

    monkeypatch.setattr(uvicorn.Server, "run", fake_run)
    server.serve(App([]), port=9000)
    assert calls == [("127.0.0.1", 9000)]


def test_serve_opens_the_page_when_asked_and_only_then(
    web_dir: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """The browser opens once the server is listening, and not by default."""
    assert web_dir.exists()
    opened: list[str] = []
    monkeypatch.setattr(webbrowser, "open", opened.append)
    for open_browser in (False, True):
        port = free_port()
        config = uvicorn.Config(
            server.asgi_app(App([])),
            host=server.HOST,
            port=port,
            log_level="warning",
        )
        url = f"http://{server.HOST}:{port}/" if open_browser else None
        instance = server.PageServer(config, url=url)
        thread = threading.Thread(target=instance.run)
        thread.start()
        while not instance.started:
            thread.join(timeout=0.05)
        instance.should_exit = True
        thread.join()
    assert opened == [f"http://{server.HOST}:{port}/"]


CTRL_C_SCRIPT = """
import sys
from pathlib import Path
from element_51 import server
from element_51.session import App

server.WEB_DIR = Path(sys.argv[1])
server.serve(App([]), port=int(sys.argv[2]))
"""


def test_ctrl_c_stops_the_server_cleanly(web_dir: Path) -> None:
    """A real process: SIGINT while serving exits with 0 and no traceback."""
    port = free_port()
    process = subprocess.Popen(  # ruff: ignore[subprocess-without-shell-equals-true]  # our own interpreter and script
        [sys.executable, "-c", CTRL_C_SCRIPT, str(web_dir), str(port)],
        stderr=subprocess.PIPE,
        text=True,
    )
    assert process.stderr is not None
    assert any("Uvicorn running" in line for line in process.stderr)
    process.send_signal(signal.SIGINT)
    _, stderr = process.communicate(timeout=20)
    assert process.returncode == 0
    assert "Traceback" not in stderr
