"""A picture and a plot on a card.

Five nodes. `ramp` makes a grey picture whose brightness rises from left to
right, `invert` flips its brightness, and `pixels` counts its pixels. A picture
is a value like a number, with its own port (a diamond) that links only to
other picture ports. `training` makes two series, a list of numbers each (a
circle port that links only to other series ports), and the built-in `Plot` node
draws the series linked to it as lines on its card.

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
    * The `Plot` card shows the loss and the accuracy as two lines with a scale
        on both axes, each named after its source port (`loss`, `accuracy`)
        with its last value at its end. Its inputs grow: every link adds a
        row, and a new empty row waits below for the next line.
    * Open the card menu (the three bars) of `ramp` and choose Save as PNG: the
        browser downloads `n1-out.png`, the picture enlarged to fill a page
        with crisp pixels. On the `Plot` card the same entry saves
        `n5-chart.png`, a figure ready for a paper: white page, serif text, blue
        and red lines with markers, 300 dpi. Before you press Run there is
        nothing to save, and the page says so.
    * Drag the `accuracy` link off its row (or delete it with the link's
        card menu or the Delete key): the row goes and the card shrinks.
    * Change `steps` in `training` to 300 and press Run: the lines get longer
        and the x axis grows. Tick `blow_up`: the loss turns to NaN after
        step 40, and its line stops there, with a gap, not an error.

"""

# Standard library
import math

# Third-party libraries
from rich.console import Console
from rich.traceback import install

# Local libraries
import element_51

# --- Globals --- #
MAX_BRIGHTNESS = 255
BLOW_UP_AT = 40

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


class Curves(element_51.Model):
    """The two series `training` makes."""

    loss: element_51.Series
    accuracy: element_51.Series


@element_51.node
def training(steps: int = 60, *, blow_up: bool = False) -> Curves:
    """Make a falling loss with a ripple and a rising accuracy.

    Returns
    -------
    Curves
        `steps` values each; with `blow_up`, the loss is NaN from step 40 on.
    """
    loss = tuple(
        math.nan
        if blow_up and step >= BLOW_UP_AT
        else math.exp(-step / 20) + 0.05 * math.sin(step)
        for step in range(steps)
    )
    accuracy = tuple(1 - math.exp(-step / 25) for step in range(steps))
    return Curves(
        loss=element_51.Series(values=loss),
        accuracy=element_51.Series(values=accuracy),
    )


def build_app() -> element_51.App:
    """Build the app with a picture, its negative, a pixel count and a plot.

    Returns
    -------
    element_51.App
        An app whose first two nodes show pictures and whose last shows a
        chart.
    """
    app = element_51.App([
        ramp,
        invert,
        pixels,
        training,
        element_51.PlotNode,
    ])
    first = app.add_node(ramp, x=60.0, y=60.0)
    second = app.add_node(invert, x=360.0, y=60.0)
    third = app.add_node(pixels, x=660.0, y=60.0)
    app.add_link(first, "out", second, "source")
    app.add_link(first, "out", third, "source")
    curves = app.add_node(training, x=60.0, y=400.0)
    chart = app.add_node(element_51.PlotNode, x=360.0, y=400.0)
    app.add_link(curves, "loss", chart, "series_1")
    app.add_link(curves, "accuracy", chart, "series_2")
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
