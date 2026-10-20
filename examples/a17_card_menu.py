"""The menu on a card's header icon.

Three nodes are linked in a chain: `a` feeds `b`, which feeds `show`. Try
these:

* Click the three bars at the right end of a card's title bar. A small menu
  opens with one entry, `Delete node`. The card is selected.
* Click `Delete node`: the node goes, and so do the links that touched it.
* Press Ctrl+Z (Cmd+Z on a Mac) to bring it back with its links.
* Click anywhere else, or press Escape, to close the menu without deleting.

Build the browser layer first: `uv run python tools/build_wasm.py`.
"""

import element_51


@element_51.node
def add(a: float = 1.0, b: float = 2.0) -> float:
    return a + b


@element_51.node
def show(value: float = 0.0) -> float:
    return value


def build_app() -> element_51.App:
    """Build the app with a chain of three linked nodes.

    Returns
    -------
    element_51.App
        An app whose last node shows 5.0.
    """
    app = element_51.App([add, show])
    first = app.add_node(add, x=60.0, y=80.0)
    second = app.add_node(add, x=340.0, y=80.0)
    last = app.add_node(show, x=620.0, y=80.0)
    app.add_link(first, "out", second, "a")
    app.add_link(second, "out", last, "value")
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
