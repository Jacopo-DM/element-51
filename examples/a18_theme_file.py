"""Your own theme, from a file.

`my_theme.json` next to this script changes a few values of the dark theme: a
dark blue canvas, blue title bars, orange number ports, wider cards and a
larger value font. Try these:

* Run the script and look at the editor: it is two linked nodes in the
  new colours.
* Open `my_theme.json`, change `"canvas"` to another colour, stop the script
  with Ctrl+C and run it again to see it.
* Change `"colours"` to `"colors"` and run it: Python refuses the file and
  names the member that is wrong.
* Change `"canvas"` to `"canvass"` and run it: the page shows a banner that
  names the key and lists the valid ones.

Every key you can change is listed in `docs/themes.md`.

Build the browser layer first: `uv run python tools/build_wasm.py`.
"""

from pathlib import Path

import element_51

THEME_FILE = Path(__file__).with_name("my_theme.json")


@element_51.node
def add(a: float = 1.0, b: float = 2.0) -> float:
    return a + b


@element_51.node
def show(value: float = 0.0) -> float:
    return value


def build_app() -> element_51.App:
    """Build the app with two linked nodes, using the theme file.

    Returns
    -------
    element_51.App
        An app whose theme is `my_theme.json`.
    """
    app = element_51.App([add, show])
    first = app.add_node(add, x=60.0, y=80.0)
    last = app.add_node(show, x=380.0, y=80.0)
    app.add_link(first, "out", last, "value")
    app.theme = THEME_FILE
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
