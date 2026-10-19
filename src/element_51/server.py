"""Serve the editor page and its WASM files."""

from pathlib import Path

import uvicorn
from starlette.applications import Starlette
from starlette.routing import Mount
from starlette.staticfiles import StaticFiles

WEB_DIR = Path(__file__).parent / "web"
HOST = "127.0.0.1"


def asgi_app() -> Starlette:
    """Return an ASGI app that serves the editor.

    Mount it in an existing ASGI app, or pass it to `serve`.

    Returns
    -------
    Starlette
        The app. It serves the files under `src/element_51/web/`, which
        `tools/build_wasm.py` fills.
    """
    return Starlette(
        routes=[Mount("/", StaticFiles(directory=WEB_DIR, html=True))],
    )


def serve(port: int = 8765) -> None:
    """Run the editor on this machine only, until interrupted.

    Parameters
    ----------
    port : int
        TCP port to listen on.
    """
    uvicorn.run(asgi_app(), host=HOST, port=port)
