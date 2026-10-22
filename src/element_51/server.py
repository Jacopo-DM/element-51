"""Serve the editor page, its WASM files and the websocket."""

import asyncio
import base64
import contextlib
import socket
import webbrowser
from enum import Enum
from pathlib import Path
from typing import Protocol, override

import uvicorn
from pydantic import ValidationError
from starlette.applications import Starlette
from starlette.concurrency import run_in_threadpool
from starlette.routing import Mount, WebSocketRoute
from starlette.staticfiles import StaticFiles
from starlette.websockets import WebSocket, WebSocketDisconnect

from element_51.edits import EditError
from element_51.engine import RunResult
from element_51.figures import chart_png, picture_png
from element_51.jobs import JobRefusedError
from element_51.protocol import (
    Ack,
    ClientMessage,
    JobRefusal,
    PngChartRequest,
    PngPictureRequest,
    PngReply,
    Reject,
    RunRequest,
    SourceReply,
    SourceRequest,
    StartRequest,
    StopRequest,
)
from element_51.session import App

WEB_DIR = Path(__file__).parent / "web"
HOST = "127.0.0.1"
INVALID_PAYLOAD = 1007  # websocket close code: data could not be parsed


class NoReply(Enum):
    """What `reply_to` returns when the message was fine and needs no answer."""

    NONE = "none"


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


type Reply = (
    Ack
    | Reject
    | RunResult
    | SourceReply
    | PngReply
    | JobRefusal
    | NoReply
    | None
)


async def reply_to(app: App, text: str) -> Reply:
    """Act on a message from the browser and say how it went.

    Parameters
    ----------
    app
        The session to edit or run.
    text
        The message text from the browser.

    Returns
    -------
    Reply
        `Ack` if an edit was applied, `Reject` with a reason if the rules
        refused it, `RunResult` for a run request, `SourceReply` for a source
        request, `PngReply` for a PNG request, `JobRefusal` if a start or stop
        was refused, `NoReply.NONE` if it was accepted (the job's updates are
        broadcast), `None` if the message is not valid.
    """
    try:
        message = ClientMessage.validate_json(text)
    except ValidationError:
        return None
    if isinstance(message, RunRequest):
        return await run_in_threadpool(app.execute)
    if isinstance(message, SourceRequest | PngChartRequest | PngPictureRequest):
        return await _asked(app, message)
    if isinstance(message, StartRequest | StopRequest):
        return await _job_request(app, message)
    try:
        app.apply(message.edit)
    except EditError as error:
        return Reject(id=message.id, reason=error.reason)
    return Ack(id=message.id)


async def _asked(
    app: App, message: SourceRequest | PngChartRequest | PngPictureRequest
) -> SourceReply | PngReply:
    """Answer a request for a node's source or for a PNG file.

    Returns
    -------
    SourceReply | PngReply
        The answer, which goes to the asking page only.
    """
    if isinstance(message, SourceRequest):
        return await run_in_threadpool(app.source, message.node)
    return await run_in_threadpool(_png_reply, message)


def _png_reply(message: PngChartRequest | PngPictureRequest) -> PngReply:
    """Draw the PNG file a request asks for.

    Returns
    -------
    PngReply
        The file as base64 text, or the reason it could not be drawn.
    """
    try:
        if isinstance(message, PngChartRequest):
            data = chart_png(message.lines)
        else:
            data = picture_png(message.picture)
    except ValueError as error:
        return PngReply(node=message.node, name=message.name, reason=str(error))
    return PngReply(
        node=message.node,
        name=message.name,
        data=base64.b64encode(data).decode("ascii"),
    )


async def _job_request(
    app: App, message: StartRequest | StopRequest
) -> JobRefusal | NoReply:
    """Start or stop a job and say whether it was refused.

    Returns
    -------
    JobRefusal | NoReply
        The refusal with its reason, or `NoReply.NONE`.
    """
    try:
        if isinstance(message, StartRequest):
            await run_in_threadpool(app.start_job, message.node)
        else:
            app.stop_job(message.node)
    except JobRefusedError as error:
        return JobRefusal(node=message.node, reason=error.reason)
    return NoReply.NONE


def asgi_app(app: App, theme: str | Path | None = None) -> Starlette:
    """Return an ASGI app that serves the editor for `app`.

    Mount it in an existing ASGI app, or pass the `App` to `serve`.

    Parameters
    ----------
    app
        The session whose graph the browser edits.
    theme
        The theme to draw it in: a built-in name or the path of a theme file,
        or None to keep the app's own.

    Returns
    -------
    Starlette
        The ASGI app. It serves the files under `src/element_51/web/`, which
        `tools/build_wasm.py` fills, and a websocket at `/ws`.
    """
    if theme is not None:
        app.theme = theme
    hub = Hub()
    app.subscribe(lambda snapshot: hub.broadcast(snapshot.model_dump_json()))
    app.subscribe_jobs(lambda event: hub.broadcast(event.model_dump_json()))

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
                if reply is NoReply.NONE:
                    continue
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


class PageServer(uvicorn.Server):
    """A uvicorn server that can open the editor page once it is listening.

    Parameters
    ----------
    config
        The uvicorn settings.
    url
        The page to open in the browser at startup, or None to open nothing.
    """

    def __init__(self, config: uvicorn.Config, *, url: str | None) -> None:
        super().__init__(config)
        self._url = url

    @override
    async def startup(self, sockets: list[socket.socket] | None = None) -> None:
        """Start listening, then open the page if one was asked for."""
        await super().startup(sockets)
        if self._url is not None and self.started:
            webbrowser.open(self._url)


def serve(
    app: App,
    port: int = 8765,
    *,
    open_browser: bool = False,
    theme: str | Path | None = None,
) -> None:
    """Run the editor on this machine only, until interrupted.

    Ctrl+C stops it cleanly.

    Parameters
    ----------
    app
        The session whose graph the browser edits.
    port
        TCP port to listen on.
    open_browser
        Open the editor page in the default browser once the server is
        listening.
    theme
        The theme to draw the editor in: `purple-synthwave` (the default),
        `antimony` (dark), `antimony-light`, or the path of a JSON theme file
        that changes some of their values (see `docs/themes.md`). Anything
        else raises `ValueError`.
    """
    config = uvicorn.Config(asgi_app(app, theme), host=HOST, port=port)
    url = f"http://{HOST}:{port}/" if open_browser else None
    # uvicorn stops gracefully on Ctrl+C, then raises KeyboardInterrupt again
    # so that a caller can tell; for a script, stopping is the normal end.
    with contextlib.suppress(KeyboardInterrupt):
        PageServer(config, url=url).run()
