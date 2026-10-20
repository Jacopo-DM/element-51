# element-51

[![PyPI](https://img.shields.io/pypi/v/element-51)](https://pypi.org/project/element-51/)
[![Python](https://img.shields.io/badge/python-3.13%2B-blue)](https://pypi.org/project/element-51/)
[![License](https://img.shields.io/pypi/l/element-51)](LICENSE)
[![CI](https://github.com/Jacopo-DM/element-51/actions/workflows/ci.yml/badge.svg)](https://github.com/Jacopo-DM/element-51/actions/workflows/ci.yml)
[![Rust and WASM](https://github.com/Jacopo-DM/element-51/actions/workflows/rust.yml/badge.svg)](https://github.com/Jacopo-DM/element-51/actions/workflows/rust.yml)
[![Docs](https://github.com/Jacopo-DM/element-51/actions/workflows/docs.yml/badge.svg)](https://jacopo-dm.github.io/element-51/)
[![uv](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/uv/main/assets/badge/v0.json)](https://github.com/astral-sh/uv)
[![Ruff](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/astral-sh/ruff/main/assets/badge/v2.json)](https://github.com/astral-sh/ruff)

![The element-51 editor in the dark theme, with a graph of ten linked nodes](https://raw.githubusercontent.com/Jacopo-DM/element-51/main/docs/images/dark.jpg)

Design node-editor GUIs from Python. You write nodes as typed functions or small
classes. element-51 serves an editor in the browser where people wire them
together, and Python keeps the graph, runs it and can change it while the page
is open.

- **Nodes from type hints.** Put `@element_51.node` on a function, or subclass
    `element_51.Node`. Parameters become inputs and the return type becomes the
    output (a pydantic model gives one output per field).
- **A fast editor.** The canvas is drawn by a Rust and WebAssembly module: pan,
    zoom, drag, link, undo and redo, typed values in the node, a nested
    add-node menu with search, and refused links between ports of different
    types.
- **Python is in charge.** The graph lives in your process. Read it, run it,
    save and load it as JSON, or add nodes from a script and watch an open page
    update.
- **Yours to restyle.** Two themes ship, and a small JSON file changes any colour,
    size, font or port shape.
- **Embeddable.** It is an ASGI app, so it mounts inside FastAPI, NiceGUI or any
    other ASGI host.

## Installation

```bash
uv add element-51
```

To work on it from a clone, you also need Rust (the `stable` toolchain from
rustup, with the `wasm32-unknown-unknown` target) to build the browser layer:

```bash
uv sync
uv run python tools/build_wasm.py
```

## Example

Two nodes and a link. `number` is a function, `Double` is a class.

```python
import element_51


@element_51.node
def number() -> float:
    return 21.0


class Double(element_51.Node):
    value: float = 0.0

    def run(self) -> float:
        return self.value * 2


app = element_51.App([number, Double])
first = app.add_node(number, x=60, y=80)
second = app.add_node(Double, x=320, y=80)
app.add_link(first, "out", second, "value")
element_51.serve(app, open_browser=True)
```

Run it and your browser opens on the editor. Press Run: `number` shows 21.0 and
`Double` shows 42.0. Right-click the canvas to add more nodes. Stop the server
with Ctrl+C.

More examples, grouped by topic, are in
[`examples/`](examples): the basics, node types, using the graph from plain
Python, editor features, themes, and embedding the editor in FastAPI or NiceGUI.

## Looks

The default theme is dark. The light theme is one argument away
(`serve(app, theme="antimony-light")`).

![The same graph in the light theme](https://raw.githubusercontent.com/Jacopo-DM/element-51/main/docs/images/light.jpg)

Paste a [Coolors](https://coolors.co) palette link into the editor's menu and it
takes the colours at once, keeping the text readable (`examples/e_look/e3_palette.py`).

![The same graph after pasting a Coolors palette](https://raw.githubusercontent.com/Jacopo-DM/element-51/main/docs/images/palette.jpg)

## Documentation

The [documentation](https://jacopo-dm.github.io/element-51/) has the guides and
the API reference (the sources are in [`docs/`](docs/index.md)). To
build it, run `uv sync --group docs` and then
`uv run sphinx-build -b html docs docs/_build/html`.

## Credits

The look of the editor is inspired by
[Antimony](https://www.mattkeeter.com/projects/antimony/3/), the graph-based
CAD tool by Matt Keeter. element-51 shares no code or images with Antimony; the
colours, shapes and layout were drawn again from scratch.
