"""Serve the editor on http://127.0.0.1:8765 until Ctrl+C.

Build the browser layer first: `uv run python tools/build_wasm.py`.
"""

import element_51


@element_51.node
def add(a: float, b: float) -> float:
    return a + b


@element_51.node
def show(value: float) -> float:
    return value


def main() -> None:
    """Entry point."""
    app = element_51.App([add, show])
    app.add_node(add, x=60.0, y=60.0)
    app.add_node(show, x=320.0, y=100.0)
    element_51.serve(app)


if __name__ == "__main__":  # pragma: no cover
    main()
