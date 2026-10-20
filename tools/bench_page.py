"""Serve a graph of 200 nodes for timing the page in a browser.

Builds 10 rows of 20 chained nodes and serves it on http://127.0.0.1:8767
(not the examples' port, so both can run). Open the page, then run
`tools/bench_page.js` in the browser's console, or have Claude run it in the
browser pane. Run it from the repo root: `uv run python tools/bench_page.py`.
Ctrl+C stops it.
"""

# Local libraries
from element_51 import App, node, serve

ROWS = 10
COLUMNS = 20
PORT = 8767


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


def main() -> None:
    """Serve the graph."""
    serve(build_app(), port=PORT)


if __name__ == "__main__":
    main()
