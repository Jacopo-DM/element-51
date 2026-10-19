"""Serve the editor on http://127.0.0.1:8765 until Ctrl+C.

Build the browser layer first: `uv run python tools/build_wasm.py`.
"""

import element_51


def main() -> None:
    """Entry point."""
    element_51.serve()


if __name__ == "__main__":  # pragma: no cover
    main()
