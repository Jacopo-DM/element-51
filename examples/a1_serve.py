"""A blank canvas with three node types, served on http://127.0.0.1:8765.

Try it:

* Right-click the canvas and pick a node to add it.
* Drag from an output port (right side of a node) to an input port (left side).
* Drag a number output onto the `shout` text input: the editor refuses it.
* Press Run. A node with no link uses its Python default.
* So `add` fails with "input 'a' has no value", and `shout` shows "HI".

Build the browser layer first: `uv run python tools/build_wasm.py`.
Stop with Ctrl+C.
"""

import element_51


@element_51.node
def add(a: float, b: float) -> float:
    return a + b


@element_51.node
def show(value: float = 0.0) -> float:
    return value


@element_51.node
def shout(text: str = "hi") -> str:
    return text.upper()


def build_app() -> element_51.App:
    """Build the app with no nodes placed.

    Returns
    -------
    element_51.App
        An app that offers `add`, `show` and `shout`.
    """
    return element_51.App([add, show, shout])


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
