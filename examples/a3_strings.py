"""Text nodes: join two words, shout them, and repeat the result.

Press Run to see "hello world", then "HELLO WORLD", then the repeated text.
Try dragging the number output of `count` onto a text input: it is refused,
because a float cannot feed a str. The `times` input of `repeat` is an int and
takes its value from its Python default, 2.

Build the browser layer first: `uv run python tools/build_wasm.py`.

"""

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
def greeting() -> str:
    return "hello"


@element_51.node
def name() -> str:
    return "world"


@element_51.node
def count() -> float:
    return 3.0


@element_51.node
def join(a: str, b: str) -> str:
    return f"{a} {b}"


@element_51.node
def shout(text: str) -> str:
    return text.upper()


@element_51.node
def repeat(text: str, times: int = 2) -> str:
    return " / ".join([text] * times)


def build_app() -> element_51.App:
    """Build a four-step text pipeline.

    Returns
    -------
    element_51.App
        The app, with `count` placed but left unlinked to show the refusal.
    """
    app = element_51.App([greeting, name, count, join, shout, repeat])
    n_greeting = app.add_node(greeting, x=60.0, y=60.0)
    n_name = app.add_node(name, x=60.0, y=180.0)
    app.add_node(count, x=60.0, y=380.0)
    n_join = app.add_node(join, x=320.0, y=100.0)
    n_shout = app.add_node(shout, x=580.0, y=100.0)
    n_repeat = app.add_node(repeat, x=840.0, y=100.0)
    app.add_link(n_greeting, "out", n_join, "a")
    app.add_link(n_name, "out", n_join, "b")
    app.add_link(n_join, "out", n_shout, "text")
    app.add_link(n_shout, "out", n_repeat, "text")
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
