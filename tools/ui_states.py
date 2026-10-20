"""Serve a graph that shows every state the editor's page can be in.

The graph has nodes that run, fail and are skipped, a long text value, a
tick box, and node types in several menu categories. Open the page, then put
it in a state with `tools/ui_states.js` (which opens the menus, runs the graph
and shows the error banner). Run it from the repo root: `uv run python
tools/ui_states.py [--theme antimony-light]`. Ctrl+C stops it.

Notes
-----
    * It serves on http://127.0.0.1:8768, not the examples' port.
    * The graph is fixed, so two captures of the same state are identical.
"""

# Standard library
import argparse
from typing import cast

# Local libraries
from element_51 import App, node, serve
from element_51.edits import SetValue

PORT = 8768
LONG_TEXT = "a very long line of text that does not fit in its box " * 3


@node(category="Math")
def ten() -> float:
    """Return ten.

    Returns
    -------
    float
        Ten.
    """
    return 10.0


@node(category="Math")
def zero() -> float:
    """Return zero.

    Returns
    -------
    float
        Zero.
    """
    return 0.0


@node(category="Math")
def divide(a: float, b: float) -> float:
    """Divide `a` by `b`.

    Parameters
    ----------
    a : float
        The dividend.
    b : float
        The divisor.

    Returns
    -------
    float
        The quotient.
    """
    return a / b


@node(category="Math/Trig")
def sine(angle: float) -> float:
    """Return the sine.

    Parameters
    ----------
    angle : float
        The angle in radians.

    Returns
    -------
    float
        Its sine.
    """
    return angle


@node(category="Text")
def shout(text: str, *, loud: bool = True) -> str:
    """Capitalise the text.

    Parameters
    ----------
    text : str
        The text.
    loud : bool
        Whether to capitalise it.

    Returns
    -------
    str
        The text, capitalised if `loud`.
    """
    return text.upper() if loud else text


@node(category="Logic")
def broken() -> float:
    """Return something that is not a number.

    Returns
    -------
    float
        Never a float.
    """
    return cast("float", "not a number")


def build_app() -> App:
    """Build the graph.

    Returns
    -------
    App
        An app with a good chain, a failing node, a skipped node and a node
        holding a long text.
    """
    app = App([ten, zero, divide, sine, shout, broken])
    n_ten = app.add_node(ten, x=60.0, y=60.0)
    n_zero = app.add_node(zero, x=60.0, y=200.0)
    n_divide = app.add_node(divide, x=320.0, y=100.0)
    n_sine = app.add_node(sine, x=580.0, y=100.0)
    app.add_node(broken, x=580.0, y=300.0)
    n_shout = app.add_node(shout, x=320.0, y=300.0)
    app.add_link(n_ten, "out", n_divide, "a")
    app.add_link(n_zero, "out", n_divide, "b")
    app.add_link(n_divide, "out", n_sine, "angle")
    app.apply(SetValue(node=n_shout, port="text", value=LONG_TEXT))
    return app


def main() -> None:
    """Serve the graph."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--theme", default="antimony")
    args = parser.parse_args()
    app = build_app()
    serve(app, port=PORT, theme=args.theme)


if __name__ == "__main__":
    main()
