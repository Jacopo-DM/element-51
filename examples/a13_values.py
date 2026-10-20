"""Type values into the inputs that have no link.

Each unlinked input has a box on its node, set in grey with the Python default
(`2.0`, `world`) until you type something.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * In the `value` box of `scale`, type `5` and press Enter. Run shows 15.0.
    * Empty the box: the grey default and the old result (6.0) come back.
    * Type `abc` in a number box: the page says why, and the box is put back.
    * Copy and paste work as in any text box.
    * Tick `loud` on `greet`, then Run: the text turns to capitals.
    * Set `times` on `repeat` to 3 and Run.
    * Zoom and pan: the boxes stay on their rows. Drag a node and they follow.
    * Reload the page: your values are still there, because Python holds them.

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
def scale(value: float = 2.0, factor: float = 3.0) -> float:
    return value * factor


@element_51.node
def greet(name: str = "world", *, loud: bool = False) -> str:
    text = f"hello {name}"
    return text.upper() if loud else text


@element_51.node
def repeat(text: str, times: int = 2) -> str:
    return text * times


def build_app() -> element_51.App:
    """Build `scale` and `greet` feeding `repeat`, all on their defaults.

    Returns
    -------
    element_51.App
        The app with three nodes and one link, from `greet` to `repeat`.
    """
    app = element_51.App([scale, greet, repeat])
    app.add_node(scale, x=60.0, y=60.0)
    greeting = app.add_node(greet, x=60.0, y=200.0)
    twice = app.add_node(repeat, x=380.0, y=200.0)
    app.add_link(greeting, "out", twice, "text")
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
