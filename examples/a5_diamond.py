"""One value, two paths, joined again: 2 squared plus 2 negated.

Press Run: `start` gives 2.0, `square` gives 4.0, `negate` gives -2.0 and `add`
gives 2.0. One output can feed several inputs. An input takes only one link, so
dragging a second link onto `add.a` is refused ("already has a link"), and a
link from `add` back to `start` is refused too ("would make a cycle").

The `count` node returns an int. It may feed a float input (`half`), because an
int can stand in for a float, but a float cannot feed an int input.

Build the browser layer first: `uv run python tools/build_wasm.py`.
"""

import element_51


@element_51.node
def start() -> float:
    return 2.0


@element_51.node
def square(x: float) -> float:
    return x * x


@element_51.node
def negate(x: float) -> float:
    return -x


@element_51.node
def add(a: float, b: float) -> float:
    return a + b


@element_51.node
def count() -> int:
    return 5


@element_51.node
def half(x: float) -> float:
    return x / 2


def build_app() -> element_51.App:
    """Build the diamond, plus an int feeding a float.

    Returns
    -------
    element_51.App
        The app.
    """
    app = element_51.App([start, square, negate, add, count, half])
    n_start = app.add_node(start, x=60.0, y=160.0)
    n_square = app.add_node(square, x=320.0, y=60.0)
    n_negate = app.add_node(negate, x=320.0, y=260.0)
    n_add = app.add_node(add, x=580.0, y=160.0)
    n_count = app.add_node(count, x=60.0, y=420.0)
    n_half = app.add_node(half, x=320.0, y=420.0)
    app.add_link(n_start, "out", n_square, "x")
    app.add_link(n_start, "out", n_negate, "x")
    app.add_link(n_square, "out", n_add, "a")
    app.add_link(n_negate, "out", n_add, "b")
    app.add_link(n_count, "out", n_half, "x")
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
