"""All four port types: float, int, str and bool, and which links are allowed.

Port colours show the type: orange for numbers, green for text, blue for
true/false. Press Run, then try these drags:

* `flag` (bool) onto `describe.flag`: allowed, same type.
* `how_many` (int) onto `describe.count`: allowed.
* `how_many` (int) onto `describe.ratio` (float): allowed, int may feed float.
* `ratio` (float) onto `describe.count` (int): refused, float to int.
* `flag` (bool) onto `describe.count` (int): refused, a bool is not an int.
* Any output onto `describe.label` (str) other than `label`: refused.

While you drag, the input port under the pointer turns green if the link would
be accepted and red if it would be refused.

Build the browser layer first: `uv run python tools/build_wasm.py`.
"""

import element_51


@element_51.node
def flag() -> bool:
    return True


@element_51.node
def how_many() -> int:
    return 4


@element_51.node
def ratio() -> float:
    return 0.5


@element_51.node
def label() -> str:
    return "items"


@element_51.node
def describe(
    label: str, count: int, ratio: float, *, flag: bool = False
) -> str:
    verdict = "yes" if flag else "no"
    return f"{count} {label}, ratio {ratio}, flag {verdict}"


def build_app() -> element_51.App:
    """Build one source per port type, all feeding `describe` except `flag`.

    Returns
    -------
    element_51.App
        The app, with `flag` placed but left unlinked.
    """
    app = element_51.App([flag, how_many, ratio, label, describe])
    app.add_node(flag, x=60.0, y=60.0)
    n_count = app.add_node(how_many, x=60.0, y=180.0)
    n_ratio = app.add_node(ratio, x=60.0, y=300.0)
    n_label = app.add_node(label, x=60.0, y=420.0)
    n_describe = app.add_node(describe, x=420.0, y=200.0)
    app.add_link(n_count, "out", n_describe, "count")
    app.add_link(n_ratio, "out", n_describe, "ratio")
    app.add_link(n_label, "out", n_describe, "label")
    return app


def main() -> None:
    """Entry point."""
    element_51.serve(build_app())


if __name__ == "__main__":  # pragma: no cover
    main()
