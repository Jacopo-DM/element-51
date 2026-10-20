"""The light theme: the same editor on a light grey canvas.

Run it, open http://127.0.0.1:8765 and compare with the default. The
cards stay dark and the canvas, grid and selection colour change.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Click a wire: it turns dark, so it shows on the light canvas.
    * Click a node: it gets a light outline.
    * In `main`, use `theme="purple-synthwave"` (or no `theme`) for the
        default, or `theme="antimony"` for the dark Antimony look.
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
        The app with two nodes and one link, in the light theme.
    """
    app = element_51.App([number, halve])
    first = app.add_node(number, x=60.0, y=80.0)
    second = app.add_node(halve, x=340.0, y=80.0)
    app.add_link(first, "out", second, "value")
    app.theme = "antimony-light"
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app(), theme="antimony-light")


if __name__ == "__main__":  # pragma: no cover
    main()
