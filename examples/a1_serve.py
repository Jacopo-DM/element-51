"""Serve the editor on http://127.0.0.1:8765 until Ctrl+C.

Right-click the canvas to add a node. Drag from an output port to an input
port to link them. Build the browser layer first:
`uv run python tools/build_wasm.py`.
"""

import element_51


@element_51.node
def add(a: float, b: float) -> float:
    return a + b


@element_51.node
def show(value: float) -> float:
    return value


@element_51.node
def shout(text: str) -> str:
    return text.upper()


def main() -> None:
    """Entry point."""
    element_51.serve(element_51.App([add, show, shout]))


if __name__ == "__main__":  # pragma: no cover
    main()
