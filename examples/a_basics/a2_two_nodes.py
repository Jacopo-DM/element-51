"""Two nodes and a link: the smallest editor, opened in your browser.

Run it: `uv run examples/a_basics/a2_two_nodes.py`. The page opens by
itself. Stop it with Ctrl+C in the terminal; it shuts down and the command
ends.

The graph holds a `number` node (written as a function) linked to a `Double`
node (written as a class).

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Press Run: `number` gives 21.0 and `Double` gives 42.0.
    * Right-click the canvas and add another node, then link it in.
    * Drag a link onto an input of the wrong type: the editor refuses it.
    * Type a value in the box on `Double` (after removing the link) and Run
        again.

"""

# Standard library
from typing import override

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
def number() -> float:
    return 21.0


class Double(element_51.Node):
    value: float = 0.0

    @override
    def run(self) -> float:
        return self.value * 2


def build_app() -> element_51.App:
    """Build the app: `number` linked into `Double`.

    Returns
    -------
    element_51.App
        An app that offers `number` and `Double`, with one of each placed and
        linked.
    """
    app = element_51.App([number, Double])
    first = app.add_node(number, x=80.0, y=100.0)
    second = app.add_node(Double, x=380.0, y=100.0)
    app.add_link(first, "out", second, "value")
    return app


def main() -> None:
    """Entry point: serve the editor and open it in the browser."""
    element_51.serve(build_app(), open_browser=True)


if __name__ == "__main__":  # pragma: no cover
    main()
