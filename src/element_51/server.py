"""Serve the editor page, its WASM files and the websocket."""

from pathlib import Path

import uvicorn
from pydantic import ValidationError
from starlette.applications import Starlette
from starlette.routing import Mount, WebSocketRoute
from starlette.staticfiles import StaticFiles
from starlette.websockets import WebSocket

from element_51.edits import EditError
from element_51.protocol import Ack, EditRequest, Reject
from element_51.session import App

WEB_DIR = Path(__file__).parent / "web"
HOST = "127.0.0.1"
INVALID_PAYLOAD = 1007  # websocket close code: data could not be parsed


def reply_to(app: App, text: str) -> Ack | Reject | None:
    """Apply the edit in a browser message and say how it went.

    Parameters
    ----------
    app
        The session to edit.
    text
        The message text from the browser.

    Returns
    -------
    Ack | Reject | None
        `Ack` if the edit was applied, `Reject` with a reason if the rules
        refused it, `None` if the message is not a valid edit request.
    """
    try:
        request = EditRequest.model_validate_json(text)
    except ValidationError:
        return None
    try:
        app.apply(request.edit)
    except EditError as error:
        return Reject(id=request.id, reason=error.reason)
    return Ack(id=request.id)


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

    async def connect(websocket: WebSocket) -> None:
        await websocket.accept()
        await websocket.send_text(app.snapshot().model_dump_json())
        async for text in websocket.iter_text():
            reply = reply_to(app, text)
            if reply is None:
                await websocket.close(
                    code=INVALID_PAYLOAD, reason="malformed message"
                )
                return
            await websocket.send_text(reply.model_dump_json())

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
