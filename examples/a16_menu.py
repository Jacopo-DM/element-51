"""A nested add-node menu with search.

Right-click the canvas. The menu has two submenus, `Math` (with `Trig` inside
it) and `Text`, and two nodes at the top level.

The categories come from `@node(category="Math/Trig")` and, for a class, a
`category = "Text"` line. A slash separates the levels.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Move the pointer over `Math`, then `Trig`, and click `sine` to add it.
    * Type `sin` in the search box: every match shows at once, with its path.
    * Type `math`: the search looks at the whole path, so all of Math matches.
    * Press Enter to add the first match, or Escape to close the menu.
    * Right-click near the right edge: the submenus open to the left instead.

"""

# Standard library
import math
from typing import override

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


@element_51.node(category="Math/Trig")
def sine(angle: float = 0.0) -> float:
    return math.sin(angle)


@element_51.node(category="Math/Trig")
def cosine(angle: float = 0.0) -> float:
    return math.cos(angle)


@element_51.node(category="Math")
def square_root(value: float = 4.0) -> float:
    return math.sqrt(value)


class Shout(element_51.Node):
    category = "Text"
    text: str = "hello"

    @override
    def run(self) -> str:
        return self.text.upper()


@element_51.node
def add(a: float = 1.0, b: float = 2.0) -> float:
    return a + b


@element_51.node
def show(value: float = 0.0) -> float:
    return value


def build_app() -> element_51.App:
    """Build the app with the menu's nodes and one `add` placed.

    Returns
    -------
    element_51.App
        An app that offers six node types in three places in the menu.
    """
    app = element_51.App([sine, cosine, square_root, Shout, add, show])
    app.add_node(add, x=80.0, y=80.0)
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
