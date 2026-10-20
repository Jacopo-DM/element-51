"""Nodes written as classes: fields are inputs, `run` makes the outputs.

Press Run. `Greeting` has two text inputs with defaults, so with no links it
says "hello, world!", and `Shout` upper-cases it. `Numbers` returns two values,
so it has two output ports, and `Summary` turns them into a total and a mean.

Compare with `a8_multiple_outputs.py`, which uses the function form. Both build
the same kind of node. A class gets its input types checked by pydantic, and
the class name is the name you see in the right-click menu.

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


class Pair(element_51.Model):
    a: float
    b: float


class Totals(element_51.Model):
    total: float
    mean: float


class Greeting(element_51.Node):
    name: str = "world"
    punctuation: str = "!"

    @override
    def run(self) -> str:
        return f"hello, {self.name}{self.punctuation}"


class Shout(element_51.Node):
    text: str = ""

    @override
    def run(self) -> str:
        return self.text.upper()


class Numbers(element_51.Node):
    @override
    def run(self) -> Pair:
        return Pair(a=3.0, b=4.5)


class Summary(element_51.Node):
    a: float = 0.0
    b: float = 0.0

    @override
    def run(self) -> Totals:
        return Totals(total=self.a + self.b, mean=(self.a + self.b) / 2)


def build_app() -> element_51.App:
    """Build a text pipeline and a numbers pipeline.

    Returns
    -------
    element_51.App
        The app, with four nodes and three links.
    """
    app = element_51.App([Greeting, Shout, Numbers, Summary])
    n_greeting = app.add_node(Greeting, x=60.0, y=80.0)
    n_shout = app.add_node(Shout, x=340.0, y=80.0)
    n_numbers = app.add_node(Numbers, x=60.0, y=280.0)
    n_summary = app.add_node(Summary, x=340.0, y=280.0)
    app.add_link(n_greeting, "out", n_shout, "text")
    app.add_link(n_numbers, "a", n_summary, "a")
    app.add_link(n_numbers, "b", n_summary, "b")
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
