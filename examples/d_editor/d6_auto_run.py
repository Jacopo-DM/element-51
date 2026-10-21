"""Let the results follow your edits with `Run automatically`.

The box beside Run, once ticked, runs the graph 300 ms after you stop editing.
`share` divides `scale`'s result by `parts`, which starts at 0.0, so it fails
until you give it a number.

Build the browser layer first: `uv run python tools/build_wasm.py`.

Notes
-----
    * Press Run: `share` fails. Its card gets a red border and a red title bar,
        and the banner says why.
    * Tick `Run automatically`. Run turns grey: the box runs the graph for you.
    * In the `parts` box of `share`, type `4` and press Enter. Without pressing
        Run, `scale` and `share` show 6.0 and 1.5, the red border goes and the
        banner clears.
    * Type `0` in `parts` again: the failure comes back by itself.
    * Change `value` of `scale` three times in a row, quickly: the graph runs
        once, when you stop.
    * Drag a node: nothing runs, because moving changes no result.
    * Untick the box and edit again: nothing runs until you press Run.
    * Reload the page: the box is unticked again, because it is not saved.

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
def scale(value: float = 2.0, factor: float = 3.0) -> float:
    return value * factor


@element_51.node
def share(total: float, parts: float = 0.0) -> float:
    return total / parts


def build_app() -> element_51.App:
    """Build `scale` feeding `share`, which fails until `parts` is set.

    Returns
    -------
    element_51.App
        The app with two nodes and one link, from `scale` to `share`.
    """
    app = element_51.App([scale, share])
    first = app.add_node(scale, x=60.0, y=80.0)
    second = app.add_node(share, x=380.0, y=80.0)
    app.add_link(first, "out", second, "total")
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
