"""Serve a graph that shows every state the editor's page can be in.

The graph has nodes that run, fail and are skipped, a long text value, a
tick box, and node types in several menu categories. With `--graph display` it
is a picture, its negative, two series and a Plot card instead (the states of
`tools/ui_keys.js`' `uiKeysDisplay`). With `--graph job` it is three job nodes
and a Plot (the states of `uiKeysJob`: a slow job to stop, a quick one that
finishes into the Plot, and one that fails). With `--graph slider` it is a
slider linked to a `sine` node (`uiKeysSlider`). Open the page, then put
it in a state with `tools/ui_states.js` (which opens the menus, runs the graph
and shows the error banner). Run it from the repo root: `uv run python
tools/ui_states.py [--theme antimony-light] [--graph display] [--port 8768]`.
Ctrl+C stops it.

Notes
-----
    * It serves on http://127.0.0.1:8768 unless `--port` says otherwise, not the
        examples' port; `tools/run_browser_tests.py` starts it on a free port.
    * The graph is fixed, so two captures of the same state are identical.
"""

# Standard library
import argparse
import math
import time
from typing import cast, override

# Local libraries
from element_51 import (
    App,
    Image,
    JobNode,
    Model,
    PlotNode,
    Report,
    Series,
    Slider,
    node,
    serve,
)
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


@node
def ramp(side: int = 28) -> Image:
    """Make a square grey picture that brightens from left to right.

    Returns
    -------
    Image
        `side` by `side` pixels.
    """
    row = bytes(column * 255 // max(side - 1, 1) for column in range(side))
    return Image(width=side, height=side, channels=1, pixels=row * side)


@node
def invert(source: Image) -> Image:
    """Swap light and dark.

    Returns
    -------
    Image
        The same size as `source`.
    """
    return Image(
        width=source.width,
        height=source.height,
        channels=source.channels,
        pixels=bytes(255 - pixel for pixel in source.pixels),
    )


class Curves(Model):
    """The two series `training` makes."""

    loss: Series
    accuracy: Series


@node
def training(steps: int = 60) -> Curves:
    """Make a falling loss and a rising accuracy.

    Returns
    -------
    Curves
        `steps` values each.
    """
    return Curves(
        loss=Series(values=tuple(math.exp(-i / 20) for i in range(steps))),
        accuracy=Series(
            values=tuple(1 - math.exp(-i / 25) for i in range(steps))
        ),
    )


def build_display_app() -> App:
    """Build the graph with a picture, a negative, and a Plot of two lines.

    Returns
    -------
    App
        An app whose first two nodes show pictures, whose fourth makes two
        series, and whose fifth is a Plot with two rows linked and one empty.
    """
    app = App([ramp, invert, training])
    first = app.add_node(ramp, x=60.0, y=60.0)
    second = app.add_node(invert, x=320.0, y=60.0)
    curves = app.add_node(training, x=60.0, y=400.0)
    chart = app.add_node(PlotNode, x=320.0, y=400.0)
    app.add_link(first, "out", second, "source")
    app.add_link(curves, "loss", chart, "series_1")
    app.add_link(curves, "accuracy", chart, "series_2")
    return app


class Loss(Model):
    """The series a job makes."""

    loss: Series


class Slow(JobNode[None]):
    """A step job that takes ten seconds: forty steps of a quarter second."""

    @override
    def steps(self) -> int:
        return 40

    @override
    def begin(self) -> None:
        return None

    @override
    def step(self, state: None, i: int) -> Loss:
        del state
        time.sleep(0.25)
        return Loss(loss=Series(values=(math.exp(-i / 10),)))


class Quick(JobNode[None]):
    """A step job of three short steps, linked to a Plot."""

    @override
    def steps(self) -> int:
        return 3

    @override
    def begin(self) -> None:
        return None

    @override
    def step(self, state: None, i: int) -> Loss:
        del state
        time.sleep(0.05)
        return Loss(loss=Series(values=(1.0 / (i + 1),)))


class Fail(JobNode[None]):
    """A work job that fails after a moment."""

    @override
    def work(self, report: Report) -> int:
        del report
        time.sleep(0.2)
        msg = "boom"
        raise RuntimeError(msg)


def build_job_app() -> App:
    """Build the graph: a slow job, a quick one into a Plot, a failing one.

    Returns
    -------
    App
        An app whose first node is `Slow` (n1), the second `Quick` (n2), the
        third `Fail` (n3) and the fourth a Plot with `Quick`'s loss linked.
    """
    app = App([Slow, Quick, Fail])
    app.add_node(Slow, x=60.0, y=60.0)
    quick = app.add_node(Quick, x=320.0, y=60.0)
    app.add_node(Fail, x=60.0, y=300.0)
    chart = app.add_node(PlotNode, x=320.0, y=300.0)
    app.add_link(quick, "loss", chart, "series_1")
    return app


def build_slider_app() -> App:
    """Build the graph: a slider at 1.0 linked to a `sine` node.

    Returns
    -------
    App
        An app whose first node is the `Slider` (n1) and the second `sine` (n2).
    """
    app = App([sine])
    slider = app.add_node(Slider, x=60.0, y=60.0)
    sine_node = app.add_node(sine, x=420.0, y=60.0)
    app.add_link(slider, "out", sine_node, "angle")
    app.apply(SetValue(node=slider, port="value", value=1.0))
    return app


def main() -> None:
    """Serve the graph."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--theme", default="antimony")
    parser.add_argument("--port", type=int, default=PORT)
    parser.add_argument(
        "--graph",
        choices=["standard", "display", "job", "slider"],
        default="standard",
    )
    args = parser.parse_args()
    builders = {
        "standard": build_app,
        "display": build_display_app,
        "job": build_job_app,
        "slider": build_slider_app,
    }
    app = builders[args.graph]()
    serve(app, port=args.port, theme=args.theme)


if __name__ == "__main__":
    main()
