"""Helpers for the server tests: a running server, GET and POST."""

import http.client
import json
import socket
import threading
from collections.abc import Generator
from contextlib import contextmanager

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
