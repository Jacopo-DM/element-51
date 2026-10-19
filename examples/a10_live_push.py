"""A script that edits the graph while the page is open.

Start it, then open http://127.0.0.1:8765 and watch: every two seconds the
script adds a `one` node and an `add` node and links them into a running
total. Nothing needs a reload. Press Run at any point to see the total grow
(2.0, 3.0, 4.0 and so on). Open a second tab and add a node by hand: the first
tab shows it at once.

Build the browser layer first: `uv run python tools/build_wasm.py`.
"""

import threading
import time

import element_51

STEPS = 5
PAUSE_SECONDS = 2.0


@element_51.node
def one() -> float:
    return 1.0


@element_51.node
def add(a: float, b: float) -> float:
    return a + b


def build_app() -> element_51.App:
    """Build the starting graph: a single `one` node.

    Returns
    -------
    element_51.App
        The app, with one node.
    """
    app = element_51.App([one, add])
    app.add_node(one, x=60.0, y=100.0)
    return app


def keep_adding(app: element_51.App, *, steps: int, pause: float) -> None:
    """Add a `one` node and an `add` node to the running total, `steps` times.

    Parameters
    ----------
    app
        The app to edit; open pages see each step as it happens.
    steps
        How many pairs of nodes to add.
    pause
        Seconds to wait before each step.
    """
    total = "n1"
    for step in range(steps):
        time.sleep(pause)
        y = 100.0 + 120.0 * (step + 1)
        number = app.add_node(one, x=60.0, y=y)
        new_total = app.add_node(add, x=320.0, y=y)
        app.add_link(total, "out", new_total, "a")
        app.add_link(number, "out", new_total, "b")
        total = new_total


def main() -> None:
    """Entry point."""
    app = build_app()
    worker = threading.Thread(
        target=keep_adding,
        kwargs={"app": app, "steps": STEPS, "pause": PAUSE_SECONDS},
        daemon=True,
    )
    worker.start()
    element_51.serve(app)


if __name__ == "__main__":  # pragma: no cover
    main()
