"""Undo and redo every change.

Try these, with the mouse and the keyboard (Cmd on a Mac, Ctrl elsewhere):

* Drag a node elsewhere, then press Cmd+Z: it jumps back. One drag is one step.
* Cmd+Shift+Z (or Ctrl+Y) does it again: redo.
* Select a node, press Delete, then Cmd+Z: it returns with links and values.
* A node you delete comes back drawn on top of the others.
* Add a node, link two ports, type a value: each is one step to walk back.
* Make a new change after an undo, and what you undid can no longer be redone.
* Typing in a box has its own undo. Click outside the box first.

Undo sends its edits to Python like any others, so a reload keeps the result.

Build the browser layer first: `uv run python tools/build_wasm.py`.
"""

import element_51


@element_51.node
def number(value: float = 3.0) -> float:
    return value


@element_51.node
def double(value: float) -> float:
    return value * 2


@element_51.node
def add(a: float, b: float = 1.0) -> float:
    return a + b


def build_app() -> element_51.App:
    """Build `number` into `double` into `add`, with nothing to undo yet.

    Returns
    -------
    element_51.App
        The app with three nodes and two links.
    """
    app = element_51.App([number, double, add])
    first = app.add_node(number, x=60.0, y=80.0)
    second = app.add_node(double, x=340.0, y=80.0)
    third = app.add_node(add, x=620.0, y=80.0)
    app.add_link(first, "out", second, "value")
    app.add_link(second, "out", third, "a")
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
