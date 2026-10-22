"""One graph, two front doors: a NiceGUI page and a JSON API.

FastAPI hosts everything. NiceGUI draws a page at `/` with the editor beside
a form, for people, and FastAPI answers `/api/...`, for scripts. Both doors
call the same three plain functions, `place`, `node_rows` and `result_rows`,
which know nothing about either framework. That is the point of the example:
put the behaviour in functions that take the `App`, and each framework only
has to call them. A node placed from the page shows up in the API's list, and
one posted to the API shows up in the page and in the editor.

Run it with one worker only: the `App` lives in this process.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Run the script and open http://127.0.0.1:8765 if it does not open for you.
    * Choose `multiply` in the form and press `Add node`: the node appears in
        the editor and in the node table.
    * Press `Run`: the results table shows each node's status and outputs.
    * In a terminal, `curl http://127.0.0.1:8765/api/nodes` lists the same
        nodes as the table.
    * Post a node from the terminal:
        `curl -X POST http://127.0.0.1:8765/api/nodes -H 'Content-Type:
        application/json' -d '{"node_type": "add"}'`. It appears in the
        editor and, within a second, in the table.
    * Post an unknown type, `{"node_type": "divide"}`: the reply is 404 and the
        page is unchanged.
    * Drag a node in the editor and press `Run` again: both doors see the graph
        you just edited.

"""

# Third-party libraries
import uvicorn
from fastapi import FastAPI, HTTPException, status
from nicegui import ui
from rich.console import Console
from rich.traceback import install

# Local libraries
import element_51

# --- Globals --- #
PORT = 8765
FIRST_X = 60.0
FIRST_Y = 80.0
STEP_X = 220.0
ROW_Y = 240.0
POLL_SECONDS = 1.0

# --- Warnings --- #

# --- Typing --- #

# --- TERMINAL OUTPUT SETUP --- #
install(show_locals=False)
console = Console()


class NewNode(element_51.Model):
    """The body of `POST /api/nodes`: the type to place."""

    node_type: str


class NodeRow(element_51.Model):
    """One placed node, as both doors show it."""

    id: str
    node_type: str
    x: float
    y: float


class ResultRow(element_51.Model):
    """What happened to one node in a run, as both doors show it."""

    node: str
    status: str
    text: str


class UnknownTypeError(LookupError):
    """There is no node type with the name asked for."""


@element_51.node
def add(a: float = 1.0, b: float = 2.0) -> float:
    return a + b


@element_51.node
def multiply(a: float = 1.0, b: float = 2.0) -> float:
    return a * b


def place(app: element_51.App, node_type: str) -> NodeRow:
    """Place a node of a registered type next to the ones already there.

    Parameters
    ----------
    app
        The session to add the node to.
    node_type
        The name of a node type registered with `app`.

    Returns
    -------
    NodeRow
        The new node.

    Raises
    ------
    UnknownTypeError
        If `app` has no node type of that name.
    """
    function = app.node_functions.get(node_type)
    if function is None:
        msg = f"there is no node type {node_type!r}"
        raise UnknownTypeError(msg)
    x = FIRST_X + STEP_X * len(app.graph.nodes)
    node_id = app.add_node(function, x=x, y=ROW_Y)
    return NodeRow(id=node_id, node_type=node_type, x=x, y=ROW_Y)


def node_rows(app: element_51.App) -> list[NodeRow]:
    """List the nodes of the graph as they are now.

    Parameters
    ----------
    app
        The session to read.

    Returns
    -------
    list[NodeRow]
        One row per node, in the order they were placed.
    """
    return [
        NodeRow(id=n.id, node_type=n.node_type, x=n.x, y=n.y)
        for n in app.graph.nodes
    ]


def result_rows(app: element_51.App) -> list[ResultRow]:
    """Run the graph and say what happened to each node.

    Parameters
    ----------
    app
        The session to run.

    Returns
    -------
    list[ResultRow]
        One row per node: its status, and its outputs or its error.
    """
    return [
        ResultRow(
            node=outcome.node,
            status=outcome.status,
            text=outcome.error or str(dict(outcome.outputs)),
        )
        for outcome in app.execute().outcomes
    ]


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
    """Build the FastAPI app that hosts NiceGUI, the editor and the API.

    Parameters
    ----------
    app
        The session every door shares.

    Returns
    -------
    FastAPI
        The host: NiceGUI at `/`, the editor at `/editor` and the API at
        `/api`.
    """
    host = FastAPI()

    @host.get("/api/nodes")
    def nodes() -> list[NodeRow]:
        return node_rows(app)

    @host.post("/api/nodes", status_code=status.HTTP_201_CREATED)
    def add_node(body: NewNode) -> NodeRow:
        try:
            return place(app, body.node_type)
        except UnknownTypeError as error:
            raise HTTPException(
                status_code=status.HTTP_404_NOT_FOUND, detail=str(error)
            ) from error

    @host.post("/api/results")
    def results() -> list[ResultRow]:
        return result_rows(app)

    host.mount("/editor", element_51.asgi_app(app))

    @ui.page("/")
    def page() -> None:
        ui.add_css("html { color-scheme: light; }")

        def add_chosen() -> None:
            place(app, str(chooser.value))
            refresh()

        def run() -> None:
            table_of_results.rows = [r.model_dump() for r in result_rows(app)]
            table_of_results.update()

        def refresh() -> None:
            table_of_nodes.rows = [r.model_dump() for r in node_rows(app)]
            table_of_nodes.update()

        with ui.splitter(value=65).classes("w-full h-screen") as splitter:
            with splitter.before:
                ui.element("iframe").props("src=/editor/").style(
                    "width: 100%; height: 100%; border: 0"
                )
            with splitter.after, ui.column().classes("p-4 w-full"):
                ui.label("Form").classes("text-lg")
                with ui.row():
                    chooser = ui.select(sorted(app.node_functions), value="add")
                    ui.button("Add node", on_click=add_chosen)
                    ui.button("Run", on_click=run)
                table_of_nodes = ui.table(
                    columns=[
                        {"name": name, "label": name, "field": name}
                        for name in ("id", "node_type", "x", "y")
                    ],
                    rows=[],
                    row_key="id",
                ).classes("w-full")
                table_of_results = ui.table(
                    columns=[
                        {"name": name, "label": name, "field": name}
                        for name in ("node", "status", "text")
                    ],
                    rows=[],
                    row_key="node",
                ).classes("w-full")
        refresh()
        ui.timer(POLL_SECONDS, refresh)

    ui.run_with(host, mount_path="/")
    return host


def main() -> None:
    """Entry point."""
    uvicorn.run(build_host(build_app()), host="127.0.0.1", port=PORT)


if __name__ == "__main__":  # pragma: no cover
    main()
