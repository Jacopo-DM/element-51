"""Helpers for the server tests: a running server, GET and POST."""

import http.client
import json
import os
import socket
import subprocess  # ruff: ignore[suspicious-subprocess-import]  # runs our own code
import sys
import threading
from collections.abc import Generator
from contextlib import contextmanager
from pathlib import Path

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


def free_port() -> int:
    """Find a TCP port nobody is listening on.

    Returns
    -------
    int
        A port that was free a moment ago.
    """
    with socket.socket() as probe:
        probe.bind((server.HOST, 0))
        port: int = probe.getsockname()[1]
    return port


def post(port: int, path: str) -> object:
    """POST nothing to `path` on the local server.

    Returns
    -------
    object
        The JSON the server answered with.
    """
    connection = http.client.HTTPConnection(server.HOST, port, timeout=10)
    try:
        connection.request("POST", path)
        return json.loads(connection.getresponse().read())
    finally:
        connection.close()


def post_json(port: int, path: str, body: object) -> tuple[int, object]:
    """POST `body` as JSON to `path` on the local server.

    Returns
    -------
    tuple[int, object]
        The status and the JSON the server answered with (`None` if it sent
        nothing, as for a 204).
    """
    connection = http.client.HTTPConnection(server.HOST, port, timeout=10)
    try:
        connection.request(
            "POST",
            path,
            body=json.dumps(body),
            headers={"Content-Type": "application/json"},
        )
        response = connection.getresponse()
        answer = response.read()
        return response.status, json.loads(answer) if answer else None
    finally:
        connection.close()


def delete(port: int, path: str) -> int:
    """Send a DELETE for `path` to the local server.

    Returns
    -------
    int
        The status the server answered with.
    """
    connection = http.client.HTTPConnection(server.HOST, port, timeout=10)
    try:
        connection.request("DELETE", path)
        response = connection.getresponse()
        response.read()
        return response.status
    finally:
        connection.close()


EXAMPLES = Path(__file__).parent.parent / "examples"

CHILD = """
import importlib.util
import sys
from pathlib import Path

from element_51 import server
from server_support import running

path, mode, folder, web = sys.argv[1:5]
server.WEB_DIR = Path(web)
spec = importlib.util.spec_from_file_location("example", path)
module = importlib.util.module_from_spec(spec)
sys.modules["example"] = module
spec.loader.exec_module(module)
host = module.build_host(
    module.build_app() if mode == "app" else Path(folder)
)
with running(host) as port:
    print(port, flush=True)
    sys.stdin.readline()
"""


@contextmanager
def running_example(name: str, *, folder: Path | None = None) -> Generator[int]:
    """Run an example's host in a child Python and yield its port.

    NiceGUI keeps one global app that cannot be started twice in one process,
    so an example that uses it runs in a process of its own. The child tells
    its port on its first line and stops when its input closes. It serves the
    web folder of this process (the `web_dir` fixture's fake one, or the real
    one), so it needs no built browser layer.

    Parameters
    ----------
    name
        The example's path under `examples/`.
    folder
        Where a host that saves files keeps them. `None` for a host that is
        built from the example's `build_app()`.

    Yields
    ------
    int
        The port the example's host listens on.
    """
    mode = "app" if folder is None else "folder"
    child = subprocess.Popen(  # ruff: ignore[subprocess-without-shell-equals-true]  # our own code
        [
            sys.executable,
            "-c",
            CHILD,
            str(EXAMPLES / name),
            mode,
            str(folder or ""),
            str(server.WEB_DIR),
        ],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        text=True,
        env={**os.environ, "PYTHONPATH": str(Path(__file__).parent)},
    )
    assert child.stdin is not None
    assert child.stdout is not None
    try:
        yield int(child.stdout.readline())
    finally:
        child.stdin.close()
        child.stdout.close()
        child.wait(timeout=30)
