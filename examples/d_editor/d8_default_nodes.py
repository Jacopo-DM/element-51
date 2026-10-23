"""The nodes every editor starts with, and how to turn them off.

Right-click the canvas. The menu has two folders: `Default nodes`, which holds
`Output nodes` and the `Plot` inside it, and `User defined nodes`, which holds
`add` and `show`, the two nodes this script lists. You did not list `Plot`:
every editor offers it.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Open `Default nodes`, then `Output nodes`, and click `Plot` to add one.
    * Open `User defined nodes`: your nodes are in there, not at the top level.
    * Type `plot` in the search box: the search finds `Plot` by its name.
    * Stop the server, set `DEFAULTS = False` below and start it again. The menu
        shows only `add` and `show`, at the top level, as in version 0.3.0.

"""

# Third-party libraries
from rich.console import Console
from rich.traceback import install

# Local libraries
import element_51

# --- Globals --- #
DEFAULTS = True

# --- Warnings --- #

# --- Typing --- #

# --- TERMINAL OUTPUT SETUP --- #
install(show_locals=False)
console = Console()


@element_51.node
def add(a: float = 1.0, b: float = 2.0) -> float:
    return a + b


@element_51.node
def show(value: float = 0.0) -> float:
    return value


def build_app(*, defaults: bool = DEFAULTS) -> element_51.App:
    """Build the app with `add` placed and, if asked, the default nodes.

    Parameters
    ----------
    defaults
        False leaves the default nodes out of the editor.

    Returns
    -------
    element_51.App
        An app that offers `add` and `show`, and `Plot` when `defaults` is true.
    """
    app = element_51.App([add, show], defaults=defaults)
    app.add_node(add, x=80.0, y=80.0)
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
