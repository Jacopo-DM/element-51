"""Save a graph to a file and load it back, with the page open.

Run it: `uv run python examples/c_python_api/c3_save_load.py`, then open
http://127.0.0.1:8765. The page shows a small graph. Add a node or two by hand,
then press Ctrl+C. Run it again: the graph you left is back, because the script
loads `c3_graph.json` (next to this file) if it exists, and saves it again on
exit. Delete that file to start over. Open it in a text editor to see the
format: a `format_version`, the nodes with their positions, and the links.

Build the browser layer first: `uv run python tools/build_wasm.py`.

"""

# Standard library
from pathlib import Path

# Third-party libraries
from rich.console import Console
from rich.traceback import install

# Local libraries
import element_51

# --- Globals --- #
FILE = Path(__file__).with_name("c3_graph.json")

# --- Warnings --- #

# --- Typing --- #

# --- TERMINAL OUTPUT SETUP --- #
install(show_locals=False)
console = Console()


@element_51.node
def number() -> float:
    return 4.0


@element_51.node
def double(value: float) -> float:
    return value * 2


def build_app(path: Path | None = None) -> element_51.App:
    """Build the app, loading the saved graph from `path` if the file exists.

    Parameters
    ----------
    path
        The file to load from, or None to start with the small starting graph.

    Returns
    -------
    element_51.App
        The app: the saved graph, or `number` linked into `double`.
    """
    app = element_51.App([number, double])
    if path is not None and path.exists():
        app.load(path)
        return app
    first = app.add_node(number, x=60.0, y=100.0)
    second = app.add_node(double, x=320.0, y=100.0)
    app.add_link(first, "out", second, "value")
    return app


def main() -> None:
    """Entry point: load, serve until Ctrl+C, then save."""
    app = build_app(FILE)
    try:
        element_51.serve(app)
    finally:
        app.save(FILE)
        print(f"saved {FILE}")  # ruff: ignore[print]


if __name__ == "__main__":  # pragma: no cover
    main()
