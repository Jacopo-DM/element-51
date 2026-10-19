"""Serve the editor page, its WASM files and the websocket."""

import asyncio
from pathlib import Path
from typing import Protocol

import uvicorn
from pydantic import ValidationError
from starlette.applications import Starlette
from starlette.concurrency import run_in_threadpool
from starlette.routing import Mount, WebSocketRoute
from starlette.staticfiles import StaticFiles
from starlette.websockets import WebSocket, WebSocketDisconnect

from element_51.edits import EditError
from element_51.engine import RunResult
from element_51.protocol import Ack, ClientMessage, Reject, RunRequest
from element_51.session import App

WEB_DIR = Path(__file__).parent / "web"
HOST = "127.0.0.1"
INVALID_PAYLOAD = 1007  # websocket close code: data could not be parsed


class Sender(Protocol):
    """Anything that can send text to one browser page."""

    async def send_text(self, data: str) -> None:
        """Send `data` as one websocket text message."""


class Connection:
    """One open page: a queue of messages waiting to go out.

    Create it inside the event loop that serves the page. `send` may be called
    from any thread, so a script can push while the server runs elsewhere.
    """

    def __init__(self) -> None:
        self._loop = asyncio.get_running_loop()
        self._outbox: asyncio.Queue[str] = asyncio.Queue()

    def send(self, text: str) -> None:
        """Queue `text` for this page. Messages leave in the order queued."""
        self._loop.call_soon_threadsafe(self._outbox.put_nowait, text)

    async def pump(self, sender: Sender) -> None:
        """Send queued messages until the page leaves or this is cancelled."""
        while True:
            text = await self._outbox.get()
            try:
                await sender.send_text(text)
            except (WebSocketDisconnect, RuntimeError):
                return


class Hub:
    """The pages that are open now."""

    def __init__(self) -> None:
        self._open: set[Connection] = set()

    def add(self, connection: Connection) -> None:
        """Start sending pushes to `connection`."""
        self._open.add(connection)

    def remove(self, connection: Connection) -> None:
        """Stop sending pushes to `connection`."""
        self._open.discard(connection)

    def broadcast(self, text: str, *, skip: Connection | None = None) -> None:
        """Send `text` to every open page except `skip`."""
        for connection in tuple(self._open):
            if connection is not skip:
                connection.send(text)


async def reply_to(app: App, text: str) -> Ack | Reject | RunResult | None:
    """Act on a message from the browser and say how it went.

    Parameters
    ----------
    app
        The session to edit or run.
    text
        The message text from the browser.

    Returns
    -------
    Ack | Reject | RunResult | None
        `Ack` if an edit was applied, `Reject` with a reason if the rules
        refused it, `RunResult` for a run request, `None` if the message is
        not valid.
    """
    try:
        message = ClientMessage.validate_json(text)
    except ValidationError:
        return None
    if isinstance(message, RunRequest):
        return await run_in_threadpool(app.execute)
    try:
        app.apply(message.edit)
    except EditError as error:
        return Reject(id=message.id, reason=error.reason)
    return Ack(id=message.id)


def asgi_app(app: App) -> Starlette:
    """Return an ASGI app that serves the editor for `app`.

    Mount it in an existing ASGI app, or pass the `App` to `serve`.

    Parameters
    ----------
    app
        The session whose graph the browser edits.

    Returns
    -------
    Starlette
        The ASGI app. It serves the files under `src/element_51/web/`, which
        `tools/build_wasm.py` fills, and a websocket at `/ws`.
    """
    hub = Hub()
    app.subscribe(lambda snapshot: hub.broadcast(snapshot.model_dump_json()))

    async def connect(websocket: WebSocket) -> None:
        await websocket.accept()
        connection = Connection()
        hub.add(connection)
        connection.send(app.snapshot().model_dump_json())
        pump = asyncio.create_task(connection.pump(websocket))
        try:
            async for text in websocket.iter_text():
                reply = await reply_to(app, text)
                if reply is None:
                    pump.cancel()
                    await websocket.close(
                        code=INVALID_PAYLOAD, reason="malformed message"
                    )
                    return
                connection.send(reply.model_dump_json())
                if isinstance(reply, Ack):
                    # Other pages learn of the edit; the sender has it already.
                    hub.broadcast(
                        app.snapshot().model_dump_json(), skip=connection
                    )
        finally:
            pump.cancel()
            hub.remove(connection)

    return Starlette(
        routes=[
            WebSocketRoute("/ws", connect),
            Mount("/", StaticFiles(directory=WEB_DIR, html=True)),
        ],
    )


def serve(app: App, port: int = 8765) -> None:
    """Run the editor on this machine only, until interrupted.

    Parameters
    ----------
    app
        The session whose graph the browser edits.
    port
        TCP port to listen on.
    """
    uvicorn.run(asgi_app(app), host=HOST, port=port)
