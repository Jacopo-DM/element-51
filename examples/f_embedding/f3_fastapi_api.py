"""A typed JSON API over the graph, with FastAPI's own docs page.

FastAPI hosts the editor at `/editor` and a small API at `/api`. Every request
and reply is a pydantic model, so FastAPI checks each request, refuses a bad
one with a clear message and writes a docs page at `/docs` that you can click
through. All calls work on one `App`, so what a script posts shows up in the
open editor at once.

Run it with one worker only: the `App` lives in this process, and a second
worker would hold a second, separate graph.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Run the script, open http://127.0.0.1:8765/editor/ in one tab and
        http://127.0.0.1:8765/docs in another. The editor shows one `add`
        node.
    * In the docs page open `POST /api/nodes`, press `Try it out` and send
        `{"node_type": "multiply", "x": 400, "y": 120}`: the reply is the new
        node, `n2`, and it appears in the editor with no reload.
    * Open `POST /api/links` and send `{"source": "n1", "source_port": "out",
        "target": "n2", "target_port": "a"}`: the editor draws the link.
    * Send the same link again: the reply is 409 and says why, because an
        input takes one link only.
    * Send a node type that does not exist, `"node_type": "divide"`: the
        reply is 404.
    * Open `POST /api/run` and send it: every node's status and outputs
        come back, and a node that fails shows its error instead of
        breaking the call.

"""

# Third-party libraries
import uvicorn
from fastapi import FastAPI, HTTPException, status
from rich.console import Console
from rich.traceback import install

# Local libraries
import element_51
from element_51.edits import EditError
from element_51.engine import RunResult

# --- Globals --- #
PORT = 8765
FIRST_X = 60.0
FIRST_Y = 80.0

# --- Warnings --- #

# --- Typing --- #

# --- TERMINAL OUTPUT SETUP --- #
install(show_locals=False)
console = Console()


class NewNode(element_51.Model):
    """The body of `POST /api/nodes`: what to place and where."""

    node_type: str
    x: float = FIRST_X
    y: float = FIRST_Y


class NewLink(element_51.Model):
    """The body of `POST /api/links`: an output port and an input port."""

    source: str
    source_port: str
    target: str
    target_port: str


class NodeInfo(element_51.Model):
    """One placed node, as the API tells it."""

    id: str
    node_type: str
    x: float
    y: float


@element_51.node
def add(a: float = 1.0, b: float = 2.0) -> float:
    return a + b


@element_51.node
def multiply(a: float = 1.0, b: float = 2.0) -> float:
    return a * b


def build_app() -> element_51.App:
    """Build the app with one `add` node placed.

    Returns
    -------
    element_51.App
        An app with `add` and `multiply` registered and one `add` node, `n1`,
        that shows 3.0 when run.
    """
    app = element_51.App([add, multiply])
    app.add_node(add, x=FIRST_X, y=FIRST_Y)
    return app


def build_host(app: element_51.App) -> FastAPI:
    """Build the FastAPI app: the editor at `/editor`, the API at `/api`.

    Parameters
    ----------
    app
        The session the editor and the API share.

    Returns
    -------
    FastAPI
        The host. FastAPI's docs page is at `/docs`.
    """
    host = FastAPI(title="element-51 API")

    @host.get("/api/node-types")
    def node_types() -> list[str]:
        return sorted(app.node_functions)

    @host.get("/api/nodes")
    def nodes() -> list[NodeInfo]:
        return [
            NodeInfo(id=n.id, node_type=n.node_type, x=n.x, y=n.y)
            for n in app.graph.nodes
        ]

    @host.post("/api/nodes", status_code=status.HTTP_201_CREATED)
    def add_node(body: NewNode) -> NodeInfo:
        function = app.node_functions.get(body.node_type)
        if function is None:
            raise HTTPException(
                status_code=status.HTTP_404_NOT_FOUND,
                detail=f"there is no node type {body.node_type!r}",
            )
        node_id = app.add_node(function, x=body.x, y=body.y)
        return NodeInfo(
            id=node_id, node_type=body.node_type, x=body.x, y=body.y
        )

    @host.post("/api/links", status_code=status.HTTP_201_CREATED)
    def add_link(body: NewLink) -> NewLink:
        try:
            app.add_link(
                body.source, body.source_port, body.target, body.target_port
            )
        except EditError as error:
            raise HTTPException(
                status_code=status.HTTP_409_CONFLICT, detail=error.reason
            ) from error
        return body

    @host.post("/api/run")
    def run_graph() -> RunResult:
        return app.execute()

    host.mount("/editor", element_51.asgi_app(app))
    return host


def main() -> None:
    """Entry point."""
    uvicorn.run(build_host(build_app()), host="127.0.0.1", port=PORT)


if __name__ == "__main__":  # pragma: no cover
    main()
