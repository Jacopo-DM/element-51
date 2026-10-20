"""The purple-synthwave theme: a near-black canvas and blue-violet cards.

Run it, open http://127.0.0.1:8765 and compare with the dark default. The
theme was made from a Coolors palette (`examples/e_look/e3_palette.py` shows how
to try your own) and keeps every text readable.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Press Run: both nodes show their values on the violet cards.
    * Click a node: it gets a pale outline.
    * In `main`, use `theme="antimony"` (or no `theme`) for the dark default, or
        `"purple-synthwave"` for the light one.
    * A name that does not exist raises an error listing the valid ones.

References
----------
    [1] docs/themes.md

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
def number() -> float:
    return 12.0


@element_51.node
def halve(value: float) -> float:
    return value / 2


def build_app() -> element_51.App:
    """Build `number` linked into `halve`.

    Returns
    -------
    element_51.App
        The app with two nodes and one link, in the purple-synthwave theme.
    """
    app = element_51.App([number, halve])
    first = app.add_node(number, x=60.0, y=80.0)
    second = app.add_node(halve, x=340.0, y=80.0)
    app.add_link(first, "out", second, "value")
    app.theme = "purple-synthwave"
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app(), theme="purple-synthwave")


if __name__ == "__main__":  # pragma: no cover
    main()
