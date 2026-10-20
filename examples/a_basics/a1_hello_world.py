"""One node: the smallest editor, served on an address you choose.

Run it: `uv run examples/a_basics/a1_hello_world.py`, then open
http://127.0.0.1:8765 yourself. Stop it with Ctrl+C in the terminal.

The graph holds a single `hello` node, written as a function.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Press Run: `hello` gives "hello, world".
    * Right-click the canvas and add another node.
    * Change `HOST` to `"0.0.0.0"` to reach the editor from other machines on
        your network. It has no password, so anyone who can reach the address
        can edit and run the graph. `element_51.serve` always uses `127.0.0.1`.
    * Next, `a2_two_nodes.py` links two nodes.

"""

# Third-party libraries
import uvicorn
from rich.console import Console
from rich.traceback import install

# Local libraries
import element_51

# --- Globals --- #
HOST = "127.0.0.1"
PORT = 8765

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
    """Entry point: serve the editor on `HOST` and `PORT`."""
    uvicorn.run(element_51.asgi_app(build_app()), host=HOST, port=PORT)


if __name__ == "__main__":  # pragma: no cover
    main()
