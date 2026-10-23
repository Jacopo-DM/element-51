"""All of numpy's element-wise functions as nodes, with nothing written by hand.

`nodes_from_ufuncs(np, category="Numpy")` reads what each numpy ufunc says
about itself (how many inputs and outputs it has, and which types it works on)
and makes a node for it. On numpy 2.5 that is 85 nodes. The 5 it leaves out are
printed when the example starts, each with its reason.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Press Run: `hypot` has 3 and 4 typed into its two boxes and shows 5.
        `modf` splits 2.5 into 0.5 (`out1`) and 2.0 (`out2`). `exp2` takes the
        2.0 and shows 4.0.
    * Right-click the canvas: the `Numpy` flyout lists all 85 nodes. Type `sin`
        to search them.
    * Add `divide`, leave `x2` at 0 and press Run: the result is `inf`, as in
        numpy, with no warning. The page shows it as `null` (a known limit).
    * Click the menu icon of a numpy node and choose View source code: it shows
        the call, such as `numpy.hypot(x1, x2)`, because a ufunc has no source
        file.

"""

# Third-party libraries
import numpy as np
from rich.console import Console
from rich.traceback import install

# Local libraries
import element_51
from element_51.edits import SetValue

# --- Globals --- #
CATEGORY = "Numpy"

# --- Warnings --- #

# --- Typing --- #

# --- TERMINAL OUTPUT SETUP --- #
install(show_locals=False)
console = Console()


def build_app() -> element_51.App:
    """Build `hypot(3, 4)`, a `modf` of 2.5 and an `exp2` of its whole part.

    Returns
    -------
    element_51.App
        The app: every numpy ufunc node, and three nodes placed in the graph.
    """
    scan = element_51.nodes_from_ufuncs(np, category=CATEGORY)
    by_name = {node.node_type.name: node for node in scan.nodes}
    app = element_51.App(scan.nodes)
    n_hypot = app.add_node(by_name["hypot"], x=60.0, y=60.0)
    n_modf = app.add_node(by_name["modf"], x=60.0, y=240.0)
    n_exp2 = app.add_node(by_name["exp2"], x=340.0, y=240.0)
    app.add_link(n_modf, "out2", n_exp2, "x")
    app.apply(SetValue(node=n_hypot, port="x1", value=3.0))
    app.apply(SetValue(node=n_hypot, port="x2", value=4.0))
    app.apply(SetValue(node=n_modf, port="x", value=2.5))
    return app


def main() -> None:
    """Entry point."""
    skipped = element_51.nodes_from_ufuncs(np, category=CATEGORY).skipped
    for item in skipped:
        console.print(f"skipped {item.name}: {item.reason}")
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
