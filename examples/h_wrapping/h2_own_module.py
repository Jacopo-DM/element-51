"""Turn the typed functions of your own module into nodes with one call.

`nodes_from_module(module, category="Stats")` makes a node for every function
that has type hints on its parameters and its result. This example scans
itself. The functions need no decorator, and they still work as plain Python.
The names in `__all__` are the ones scanned, so `build_app` and `main` are left
out. `describe` is listed on purpose: it takes `*values`, which has no fixed
ports, so the scan skips it and says why when the example starts.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Press Run: `mean_of_two` shows 70.0, and `zscore` takes it as its mean and
        shows 1.5 for a score of 82 with a spread of 8. `percent` shows 75.0.
    * Right-click the canvas: the `Stats` flyout holds `mean_of_two`, `zscore`
        and `percent`, and no `describe`.
    * Change the `std` box of `zscore` to 0 and press Run: the node fails with
        ZeroDivisionError and the others still run.

"""

# Standard library
import sys

# Third-party libraries
from rich.console import Console
from rich.traceback import install

# Local libraries
import element_51

# --- Globals --- #
CATEGORY = "Stats"
__all__ = ["describe", "mean_of_two", "percent", "zscore"]

# --- Warnings --- #

# --- Typing --- #

# --- TERMINAL OUTPUT SETUP --- #
install(show_locals=False)
console = Console()


def mean_of_two(a: float = 60.0, b: float = 80.0) -> float:
    return (a + b) / 2


def zscore(x: float = 82.0, mean: float = 70.0, std: float = 8.0) -> float:
    return (x - mean) / std


def percent(part: float = 3.0, whole: float = 4.0) -> float:
    return 100.0 * part / whole


def describe(*values: float) -> str:
    return f"{len(values)} values"


def scan() -> element_51.Scan:
    """Scan this module for functions that can be nodes.

    Returns
    -------
    element_51.Scan
        The nodes, and the functions skipped.
    """
    return element_51.nodes_from_module(
        sys.modules[__name__], category=CATEGORY
    )


def build_app() -> element_51.App:
    """Build `zscore` fed by `mean_of_two`, and `percent` on its own.

    Returns
    -------
    element_51.App
        The app with three nodes and one link.
    """
    nodes = {node.node_type.name: node for node in scan().nodes}
    app = element_51.App(nodes.values())
    n_mean = app.add_node(nodes["mean_of_two"], x=60.0, y=80.0)
    n_score = app.add_node(nodes["zscore"], x=340.0, y=80.0)
    app.add_node(nodes["percent"], x=340.0, y=280.0)
    app.add_link(n_mean, "out", n_score, "mean")
    return app


def main() -> None:
    """Entry point."""
    for item in scan().skipped:
        console.print(f"skipped {item.name}: {item.reason}")
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
