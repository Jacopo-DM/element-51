"""Things that go wrong, and what the editor shows for each.

Press Run. One graph, four outcomes:

The red messages appear in a banner at the bottom that stays until you press
Dismiss. Link a node into `needs_input` and run again to fix that one.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * `divide` raises ZeroDivisionError: its header turns red.
    * `double` sits after it and is skipped (dimmed): it never runs.
    * `needs_input` has no link and no Python default: red, "has no value".
    * `broken` returns a str from a float output: red, "returned str".
    * The `ten` -> `double` branch is independent, so it still runs and shows
        20.0.

"""

# Standard library
from typing import cast

# Third-party libraries
from rich.console import Console
from rich.traceback import install

# Local libraries
import element_51

# --- Globals --- #

# --- Warnings --- #

# --- Typing --- #

# --- TERMINAL OUTPUT SETUP --- #
install(show_locals=False)
console = Console()


@element_51.node
def ten() -> float:
    return 10.0


@element_51.node
def zero() -> float:
    return 0.0


@element_51.node
def divide(a: float, b: float) -> float:
    return a / b


@element_51.node
def double(x: float) -> float:
    return x * 2


@element_51.node
def needs_input(x: float) -> float:
    return x


@element_51.node
def broken() -> float:
    return cast("float", "this is not a number")


def build_app() -> element_51.App:
    """Build a graph where three nodes fail, one is skipped and one is fine.

    Returns
    -------
    element_51.App
        The app.
    """
    app = element_51.App([ten, zero, divide, double, needs_input, broken])
    n_ten = app.add_node(ten, x=60.0, y=60.0)
    n_zero = app.add_node(zero, x=60.0, y=200.0)
    n_divide = app.add_node(divide, x=320.0, y=100.0)
    n_after = app.add_node(double, x=580.0, y=100.0)
    n_healthy = app.add_node(double, x=320.0, y=300.0)
    app.add_node(needs_input, x=320.0, y=440.0)
    app.add_node(broken, x=580.0, y=300.0)
    app.add_link(n_ten, "out", n_divide, "a")
    app.add_link(n_zero, "out", n_divide, "b")
    app.add_link(n_divide, "out", n_after, "x")
    app.add_link(n_ten, "out", n_healthy, "x")
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
