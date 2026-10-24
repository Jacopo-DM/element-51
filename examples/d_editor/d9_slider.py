"""A slider into a `show` node and into numpy's `sin`.

Right-click the canvas: `Default nodes` has an `Input nodes` folder with
`Float`, `Integer`, `Text`, `Flag` and `Slider`. You did not list any of them.
This script lists no node of its own except `sin`, which `nodes_from_ufuncs`
takes from numpy.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Tick `Run automatically`, then drag the handle of the slider on `n1`:
        `n2` shows the value and `n3` shows its sine, and both follow your hand.
    * Click the handle and press Left or Right: one step is a hundredth of the
        range. Home and End go to the ends.
    * Press Ctrl+Z after a drag: the whole drag goes back in one step.
        Shift+Ctrl+Z puts it forward again.
    * Change `maximum` to 0.5 while the slider is at 1: `n1` fails with "the
        value 1.0 must be between -10.0 and 0.5", and the nodes after it are
        skipped.
    * Right-click and add a `Float` from `Default nodes`; link it to the `x`
        of `n3` in place of the slider: a typed number feeds the same input.

"""

# Third-party libraries
import numpy as np
from rich.console import Console
from rich.traceback import install

# Local libraries
import element_51
from element_51.edits import SetValue

# --- Globals --- #
START = 1.0

# --- Warnings --- #

# --- Typing --- #

# --- TERMINAL OUTPUT SETUP --- #
install(show_locals=False)
console = Console()


@element_51.node
def show(value: float = 0.0) -> float:
    return value


def build_app() -> element_51.App:
    """Build a slider linked to `show` and to numpy's `sin`.

    Returns
    -------
    element_51.App
        An app with the slider (n1) at `START`, `show` (n2) and `sin` (n3).
    """
    sin = {n.node_type.name: n for n in element_51.nodes_from_ufuncs(np).nodes}[
        "sin"
    ]
    app = element_51.App([show, sin])
    slider = app.add_node(element_51.Slider, x=60.0, y=60.0)
    shown = app.add_node(show, x=420.0, y=60.0)
    sine = app.add_node(sin, x=420.0, y=200.0)
    app.add_link(slider, "out", shown, "value")
    app.add_link(slider, "out", sine, "x")
    app.apply(SetValue(node=slider, port="value", value=START))
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
