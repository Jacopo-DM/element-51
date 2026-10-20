"""The editor inside a NiceGUI page, with a side panel written in Python.

NiceGUI runs inside a FastAPI app, and the editor is mounted on the same app
at `/editor`, so everything is one process and one `App`. The page is a
splitter: the editor in an iframe on the left, NiceGUI widgets on the right.

Run it with one worker only: the `App` lives in this process.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Run the script and open http://127.0.0.1:8765 if it does not open for you.
    * Click `Run`: the outputs appear in the panel as a NiceGUI label.
    * Click `Add node`: a node appears in the editor at once.
    * Drag the splitter bar to resize the editor.

"""

# Third-party libraries
import uvicorn
from fastapi import FastAPI
from nicegui import ui
from rich.console import Console
from rich.traceback import install

# Local libraries
import element_51

# --- Globals --- #
PORT = 8765

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
    """Build the FastAPI app that hosts NiceGUI and the editor.

    Parameters
    ----------
    app
        The session the editor and the panel share.

    Returns
    -------
    FastAPI
        The host, with the editor mounted at `/editor` and NiceGUI at `/`.
    """
    host = FastAPI()
    host.mount("/editor", element_51.asgi_app(app))

    @ui.page("/")
    def page() -> None:
        def run() -> None:
            result.set_text(str(app.run()))

        def add_node() -> None:
            x = 60.0 + 40.0 * len(app.graph.nodes)
            app.add_node(add, x=x, y=240.0)

        with ui.splitter(value=75).classes("w-full h-screen") as splitter:
            with splitter.before:
                ui.element("iframe").props("src=/editor/").style(
                    "width: 100%; height: 100%; border: 0"
                )
            with splitter.after, ui.column().classes("p-4"):
                ui.label("Side panel").classes("text-lg")
                with ui.row():
                    ui.button("Run", on_click=run)
                    ui.button("Add node", on_click=add_node)
                result = ui.label()

    ui.run_with(host, mount_path="/")
    return host


def main() -> None:
    """Entry point."""
    uvicorn.run(build_host(build_app()), host="127.0.0.1", port=PORT)


if __name__ == "__main__":  # pragma: no cover
    main()
