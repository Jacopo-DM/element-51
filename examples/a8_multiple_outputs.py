"""A node with two outputs: 17 divided by 5 gives a quotient and a remainder.

A function that returns a pydantic model gets one output port per field, in
the order the fields are declared. Here `div_mod` has the ports `q` and `r`.
Press Run: after the node's name you see `q 3` and `r 2` on its two output
rows. The `Describe` node is written as a class: its fields are its inputs and
its `run` method makes the output.

Drag `q` to one input and `r` to the other. Try swapping them, or linking the
same output twice. An output may feed several inputs, but an input takes one
link.

Build the browser layer first: `uv run python tools/build_wasm.py`.

"""

# Standard library
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


class QuotientAndRemainder(element_51.Model):
    q: int
    r: int


@element_51.node
def numerator() -> int:
    return 17


@element_51.node
def denominator() -> int:
    return 5


@element_51.node
def div_mod(a: int, b: int) -> QuotientAndRemainder:
    quotient, remainder = divmod(a, b)
    return QuotientAndRemainder(q=quotient, r=remainder)


class Describe(element_51.Node):
    quotient: int = 0
    remainder: int = 0

    @override
    def run(self) -> str:
        return f"{self.quotient} remainder {self.remainder}"


def build_app() -> element_51.App:
    """Build `Describe(div_mod(17, 5))`.

    Returns
    -------
    element_51.App
        The app, with four nodes and four links.
    """
    app = element_51.App([numerator, denominator, div_mod, Describe])
    n_numerator = app.add_node(numerator, x=60.0, y=80.0)
    n_denominator = app.add_node(denominator, x=60.0, y=220.0)
    n_div_mod = app.add_node(div_mod, x=320.0, y=120.0)
    n_describe = app.add_node(Describe, x=600.0, y=120.0)
    app.add_link(n_numerator, "out", n_div_mod, "a")
    app.add_link(n_denominator, "out", n_div_mod, "b")
    app.add_link(n_div_mod, "q", n_describe, "quotient")
    app.add_link(n_div_mod, "r", n_describe, "remainder")
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
