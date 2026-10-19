"""A small sum: (3 + 4) x 3, wired up and ready to run.

Press Run: every node shows its value in its header (7.0, 21.0 and so on).
Then right-click, add another `add` node, link it in, and run again. Any edit
clears the old values, so what you see is always from the last run.

Build the browser layer first: `uv run python tools/build_wasm.py`.
"""

import element_51


@element_51.node
def three() -> float:
    return 3.0


@element_51.node
def four() -> float:
    return 4.0


@element_51.node
def add(a: float, b: float) -> float:
    return a + b


@element_51.node
def multiply(a: float, b: float) -> float:
    return a * b


@element_51.node
def show(value: float) -> float:
    return value


def build_app() -> element_51.App:
    """Build the graph `show((three + four) * three)`.

    Returns
    -------
    element_51.App
        The app, with five nodes and five links.
    """
    app = element_51.App([three, four, add, multiply, show])
    n_three = app.add_node(three, x=60.0, y=80.0)
    n_four = app.add_node(four, x=60.0, y=220.0)
    n_add = app.add_node(add, x=320.0, y=120.0)
    n_multiply = app.add_node(multiply, x=580.0, y=120.0)
    n_show = app.add_node(show, x=840.0, y=140.0)
    app.add_link(n_three, "out", n_add, "a")
    app.add_link(n_four, "out", n_add, "b")
    app.add_link(n_add, "out", n_multiply, "a")
    app.add_link(n_three, "out", n_multiply, "b")
    app.add_link(n_multiply, "out", n_show, "value")
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
