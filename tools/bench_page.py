"""Serve a graph for timing the page in a browser.

The default workload is 200 nodes: 10 rows of 20 chained nodes. It is served on
http://127.0.0.1:8767 (not the examples' port, so both can run). Open the page,
then run `tools/bench_page.js` in the browser's console, or have Claude run it
in the browser pane. Run it from the repo root:
`uv run python tools/bench_page.py [--workload nodes|pictures|chart]`. Ctrl+C
stops it.

The other two workloads need Run pressed in the page before timing, so the
pictures and the chart are drawn: `pictures` is 12 picture cards in a grid that
fits a 1024 by 768 page, and `chart` is a 100,000-point series linked to a Plot.
"""

# Standard library
import argparse
import math
from typing import Literal

# Local libraries
from element_51 import App, Image, PlotNode, Series, node, serve

ROWS = 10
COLUMNS = 20
PORT = 8767
PICTURES = 12
PICTURE_COLUMNS = 4
PICTURE_SIDE = 28
MAX_BRIGHTNESS = 255
CHART_POINTS = 100_000

type Workload = Literal["nodes", "pictures", "chart"]


@node
def start() -> float:
    """Start a row of the graph.

    Returns
    -------
    float
        The first value.
    """
    return 1.0


@node
def double(value: float) -> float:
    """Double the value before it.

    Parameters
    ----------
    value : float
        The value of the node before it.

    Returns
    -------
    float
        Twice `value`.
    """
    return value * 2.0


def build_app() -> App:
    """Build `ROWS` chains of `COLUMNS` nodes.

    Returns
    -------
    App
        The app, laid out in a grid.
    """
    app = App([start, double])
    for row in range(ROWS):
        y = 60.0 + 150.0 * row
        previous = app.add_node(start, x=60.0, y=y)
        for column in range(1, COLUMNS):
            current = app.add_node(double, x=60.0 + 260.0 * column, y=y)
            app.add_link(previous, "out", current, "value")
            previous = current
    return app


@node
def ramp(side: int = PICTURE_SIDE) -> Image:
    """Make a square grey picture that brightens from left to right.

    Returns
    -------
    Image
        `side` pixels on a side. The `side` input gives the card a value box,
        as the card of the picture example has.
    """
    row = bytes(
        column * MAX_BRIGHTNESS // max(side - 1, 1) for column in range(side)
    )
    return Image(width=side, height=side, channels=1, pixels=row * side)


@node
def big() -> Series:
    """Make a long series.

    Returns
    -------
    Series
        `CHART_POINTS` values of a decaying ripple.
    """
    return Series(
        values=tuple(
            math.exp(-step / 20_000) + 0.05 * math.sin(step / 50)
            for step in range(CHART_POINTS)
        )
    )


def build_pictures_app() -> App:
    """Build `PICTURES` picture nodes in a grid.

    Returns
    -------
    App
        The app, with no links.
    """
    app = App([ramp])
    for index in range(PICTURES):
        row, column = divmod(index, PICTURE_COLUMNS)
        app.add_node(ramp, x=40.0 + 240.0 * column, y=40.0 + 240.0 * row)
    return app


def build_chart_app() -> App:
    """Build one `big` node linked to a Plot.

    Returns
    -------
    App
        The app, with two nodes and one link.
    """
    app = App([big])
    source = app.add_node(big, x=60.0, y=80.0)
    chart = app.add_node(PlotNode, x=360.0, y=80.0)
    app.add_link(source, "out", chart, "series_1")
    return app


def main() -> None:
    """Serve the graph of the workload named on the command line."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--workload", choices=("nodes", "pictures", "chart"), default="nodes"
    )
    workload: Workload = parser.parse_args().workload
    builders = {
        "nodes": build_app,
        "pictures": build_pictures_app,
        "chart": build_chart_app,
    }
    serve(builders[workload](), port=PORT)


if __name__ == "__main__":
    main()
