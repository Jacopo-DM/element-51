"""Try a Coolors palette: paste a link and the editor takes its colours.

Run it, open http://127.0.0.1:8765, press the menu button at the top right,
choose Load palette from Coolors link and paste a link such as
`https://coolors.co/acadbc-9b9ece-6665dd-473bf0-000500`.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * The colours are sorted by lightness: the darkest becomes the canvas, the
        lightest the text, and the most saturated the accent (the Run button).
    * Text and icon colours are nudged lighter or darker until they meet WCAG
        AA, so every palette stays readable.
    * Nothing is fetched and Python is not told: reload the page to get the
        theme Python holds back.
    * A link that is not a Coolors link, or has fewer than 2 or more than 10
        colours, shows the reason in the banner.

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
        The app with two nodes and one link, in the default theme.
    """
    app = element_51.App([number, halve])
    first = app.add_node(number, x=60.0, y=80.0)
    second = app.add_node(halve, x=340.0, y=80.0)
    app.add_link(first, "out", second, "value")
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
