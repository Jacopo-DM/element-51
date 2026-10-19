"""Serve the editor page, its WASM files and the websocket."""

from pathlib import Path

import uvicorn
from starlette.applications import Starlette
from starlette.routing import Mount, WebSocketRoute
from starlette.staticfiles import StaticFiles
from starlette.websockets import WebSocket, WebSocketDisconnect

from element_51.session import App

WEB_DIR = Path(__file__).parent / "web"
HOST = "127.0.0.1"


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
        # Messages from the browser are read and dropped until the edit
        # protocol exists (task 2).
        try:
            while True:
                await websocket.receive_text()
        except WebSocketDisconnect:
            return

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
