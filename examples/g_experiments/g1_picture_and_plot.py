"""A picture on a card.

Three nodes: `ramp` makes a grey picture whose brightness rises from left to
right, `invert` flips its brightness, and `pixels` counts its pixels. A picture
is a value like a number, with its own port (a diamond) that links only to other
picture ports.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Press Run. The `ramp` and `invert` cards show their pictures below the
        ports, one the opposite of the other. `pixels` shows 784.
    * Change `side` in `ramp` (a whole number from 1 to 256) and press Run
        again. The picture changes size; it is always drawn with crisp pixels.
    * Drag a link from the `invert` output to the `source` input of a second
        `invert` node: pictures link only to pictures. A picture output does not
        link to a number input, and the page refuses it with the reason.
    * A picture input has no value box, because a picture cannot be typed.

"""

# Third-party libraries
from rich.console import Console
from rich.traceback import install

# Local libraries
import element_51

# --- Globals --- #
MAX_BRIGHTNESS = 255

# --- Warnings --- #

# --- Typing --- #

# --- TERMINAL OUTPUT SETUP --- #
install(show_locals=False)
console = Console()


@element_51.node
def ramp(side: int = 28) -> element_51.Image:
    """Make a square grey picture that brightens from left to right.

    Returns
    -------
    element_51.Image
        `side` by `side` pixels.
    """
    row = bytes(
        column * MAX_BRIGHTNESS // max(side - 1, 1) for column in range(side)
    )
    return element_51.Image(
        width=side, height=side, channels=1, pixels=row * side
    )


@element_51.node
def invert(source: element_51.Image) -> element_51.Image:
    """Swap light and dark.

    Returns
    -------
    element_51.Image
        The same size as `source`.
    """
    return element_51.Image(
        width=source.width,
        height=source.height,
        channels=source.channels,
        pixels=bytes(MAX_BRIGHTNESS - pixel for pixel in source.pixels),
    )


@element_51.node
def pixels(source: element_51.Image) -> int:
    """Count the pixels of a picture.

    Returns
    -------
    int
        Width times height.
    """
    return source.width * source.height


def build_app() -> element_51.App:
    """Build the app with a picture, its negative and a pixel count.

    Returns
    -------
    element_51.App
        An app whose first two nodes show pictures.
    """
    app = element_51.App([ramp, invert, pixels])
    first = app.add_node(ramp, x=60.0, y=60.0)
    second = app.add_node(invert, x=360.0, y=60.0)
    third = app.add_node(pixels, x=660.0, y=60.0)
    app.add_link(first, "out", second, "source")
    app.add_link(first, "out", third, "source")
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
