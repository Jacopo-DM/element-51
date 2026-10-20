"""The editor inside a FastAPI app, with a side panel of your own.

FastAPI hosts the page. The editor is mounted at `/editor`, the side panel is
plain HTML at `/`, and both talk to one `App` held in this process.

Run it with one worker only: the `App` lives in this process, and a second
worker would hold a second, separate graph.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Run the script. Your browser opens on the panel, with the editor on the
        left and a panel on the right.
    * Click `Run` in the panel: FastAPI calls `app.run()` and shows every node's
        outputs.
    * Click `Add node`: a node appears in the editor at once, with no reload,
        because the editor listens to the `App`.
    * Drag a node in the editor and click `Run` again: the panel sees the graph
        you just edited.

"""

# Third-party libraries
import uvicorn
from fastapi import FastAPI
from fastapi.responses import HTMLResponse
from rich.console import Console
from rich.traceback import install

# Local libraries
import element_51
from element_51.model import PortValue

# --- Globals --- #
PORT = 8765

PAGE = """<!doctype html>
<meta charset="utf-8">
<title>element-51 in FastAPI</title>
<style>
    body {
        margin: 0;
        display: flex;
        height: 100vh;
        color-scheme: light;
        font: 14px Verdana, sans-serif;
    }
    iframe { flex: 1; border: 0; }
    aside {
        width: 280px;
        padding: 16px;
        box-sizing: border-box;
        border-left: 1px solid #ccc;
    }
    button { margin-right: 8px; padding: 6px 12px; }
    pre { white-space: pre-wrap; }
</style>
<iframe src="/editor/"></iframe>
<aside>
    <h3>Side panel</h3>
    <button id="run">Run</button><button id="add">Add node</button>
    <pre id="out"></pre>
</aside>
<script>
    const out = document.getElementById("out");
    async function call(path) {
        const reply = await fetch(path, {method: "POST"});
        out.textContent = JSON.stringify(await reply.json(), null, 4);
    }
    document.getElementById("run").onclick = () => call("/api/run");
    document.getElementById("add").onclick = () => call("/api/add-node");
</script>
"""

# --- Warnings --- #

# --- Typing --- #

# --- TERMINAL OUTPUT SETUP --- #
install(show_locals=False)
console = Console()


@element_51.node
def add(a: float = 1.0, b: float = 2.0) -> float:
    return a + b


def build_app() -> element_51.App:
    """Build the app with one `add` node placed.

    Returns
    -------
    element_51.App
        An app with a single node that shows 3.0 when run.
    """
    app = element_51.App([add])
    app.add_node(add, x=60.0, y=80.0)
    return app


def build_host(app: element_51.App) -> FastAPI:
    """Build the FastAPI app that hosts the editor and the side panel.

    Parameters
    ----------
    app
        The session the editor and the panel share.

    Returns
    -------
    FastAPI
        The host, with the editor mounted at `/editor`.
    """
    host = FastAPI()

    @host.get("/")
    def index() -> HTMLResponse:
        return HTMLResponse(PAGE)

    @host.post("/api/run")
    def run_graph() -> dict[str, dict[str, PortValue]]:
        return app.run()

    @host.post("/api/add-node")
    def add_node() -> dict[str, str]:
        x = 60.0 + 40.0 * len(app.graph.nodes)
        return {"added": app.add_node(add, x=x, y=240.0)}

    host.mount("/editor", element_51.asgi_app(app))
    return host


def main() -> None:
    """Entry point."""
    uvicorn.run(build_host(build_app()), host="127.0.0.1", port=PORT)


if __name__ == "__main__":  # pragma: no cover
    main()
