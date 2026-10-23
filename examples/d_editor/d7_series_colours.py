"""Choose the colour of each line on a Plot card.

Three nodes make a series each (`decay` falls, `rise` rises, `wave` swings) and
all three feed one `Plot`. The script gives the first two lines their colours
with `SetColour`, the same edit the page sends; the third line keeps the
theme's colour.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Press Run. The `Plot` card shows three lines. Beside each `series_N` name
        is a small colour box in the line's colour; `series_1` is orange and
        `series_2` green because the script chose them.
    * Click the box beside `series_3` and pick another colour: the line and its
        legend entry change as the browser's colour window closes. A small `x`
        appears beside the box.
    * Press the `x`: the line follows the theme again. Ctrl+Z brings your
        colour back.
    * Pick a colour that is hard to see on the chart, such as dark blue on the
        dark theme: the line is drawn as you chose it, and the box gets a red
        outline. Hover it for the contrast ratio.
    * Reload the page: the colours stay, because Python holds them. They are
        also in `app.save(...)` files, from format version 4.

"""

# Standard library
import math

# Third-party libraries
from rich.console import Console
from rich.traceback import install

# Local libraries
import element_51
from element_51.edits import SetColour

# --- Globals --- #
POINTS = 80
ORANGE = "#d95f02"
GREEN = "#1b9e77"

# --- Warnings --- #

# --- Typing --- #

# --- TERMINAL OUTPUT SETUP --- #
install(show_locals=False)
console = Console()


@element_51.node
def decay() -> element_51.Series:
    """Make a curve that falls from 1 towards 0.

    Returns
    -------
    element_51.Series
        `POINTS` numbers.
    """
    return element_51.Series(
        values=tuple(math.exp(-step / (POINTS / 5)) for step in range(POINTS))
    )


@element_51.node
def rise() -> element_51.Series:
    """Make a curve that rises from 0 towards 1.

    Returns
    -------
    element_51.Series
        `POINTS` numbers.
    """
    return element_51.Series(
        values=tuple(
            1 - math.exp(-step / (POINTS / 5)) for step in range(POINTS)
        )
    )


@element_51.node
def wave() -> element_51.Series:
    """Make a slow wave around one half.

    Returns
    -------
    element_51.Series
        `POINTS` numbers.
    """
    return element_51.Series(
        values=tuple(
            0.5 + 0.4 * math.sin(step * math.tau / POINTS)
            for step in range(POINTS)
        )
    )


def build_app() -> element_51.App:
    """Build the three series and a Plot, with two lines coloured.

    Returns
    -------
    element_51.App
        The app: `decay`, `rise` and `wave` linked to `series_1` to
        `series_3` of a `Plot`, the first two with their own colours.
    """
    app = element_51.App([decay, rise, wave])
    sources = [
        app.add_node(function, x=60.0, y=60.0 + 130.0 * index)
        for index, function in enumerate([decay, rise, wave])
    ]
    plot = app.add_node(element_51.PlotNode, x=380.0, y=60.0)
    for index, source in enumerate(sources, start=1):
        app.add_link(source, "out", plot, f"series_{index}")
    app.apply(SetColour(node=plot, port="series_1", colour=ORANGE))
    app.apply(SetColour(node=plot, port="series_2", colour=GREEN))
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
