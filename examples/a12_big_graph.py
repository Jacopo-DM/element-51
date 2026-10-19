"""A graph bigger than the window, to pan, zoom, move and delete in.

Try these, with the mouse:

* Drag empty canvas to pan. Turn the wheel to zoom around the pointer.
* Drag a node to move it. One drag is one change; Run keeps its values.
* Click a node to select it, then press Delete. Its links go with it.
* Click a wire to select it, then press Delete to remove just that link.
* Press Escape to let go of the selection. Run shows what is now missing.

Reload the page and the moves and deletions are still there: Python holds them.

Build the browser layer first: `uv run python tools/build_wasm.py`.
"""

import element_51

COLUMNS = 6
ROWS = 4
SPACING_X = 260.0
SPACING_Y = 150.0


@element_51.node
def start() -> float:
    return 1.0


@element_51.node
def double(value: float) -> float:
    return value * 2


def build_app() -> element_51.App:
    """Build `ROWS` chains of `double` nodes, each `COLUMNS` nodes long.

    Returns
    -------
    element_51.App
        The app. Row `r` starts at 1.0 and doubles `COLUMNS - 1` times.
    """
    app = element_51.App([start, double])
    for row in range(ROWS):
        y = 60.0 + SPACING_Y * row
        previous = app.add_node(start, x=60.0, y=y)
        for column in range(1, COLUMNS):
            current = app.add_node(double, x=60.0 + SPACING_X * column, y=y)
            app.add_link(previous, "out", current, "value")
            previous = current
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
