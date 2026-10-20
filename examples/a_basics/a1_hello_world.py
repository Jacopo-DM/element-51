"""One node: the smallest editor, opened in your browser.

Run it: `uv run examples/a_basics/a1_hello_world.py`. The page opens by
itself. Stop it with Ctrl+C in the terminal; it shuts down and the command
ends.

The graph holds a single `hello` node, written as a function.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Press Run: `hello` gives "hello, world".
    * Right-click the canvas and add another node.
    * Next, `a2_two_nodes.py` links two nodes.

"""

# Third-party libraries
from rich.console import Console
from rich.traceback import install

# Local libraries
import element_51

# --- Globals --- #

# --- Warnings --- #

# --- Typing --- #

# --- TERMINAL OUTPUT SETUP --- #
install(show_locals=False)
console = Console()


@element_51.node
def hello() -> str:
    return "hello, world"


def build_app() -> element_51.App:
    """Build the app: one `hello` node.

    Returns
    -------
    element_51.App
        An app that offers `hello`, with one placed.
    """
    app = element_51.App([hello])
    app.add_node(hello, x=120.0, y=100.0)
    return app


def main() -> None:
    """Entry point: serve the editor and open it in the browser."""
    element_51.serve(build_app(), open_browser=True)


if __name__ == "__main__":  # pragma: no cover
    main()
