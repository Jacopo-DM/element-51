<div align="center">

<img src="docs/logo.svg" alt="element-51 logo: three linked node cards on a dark dotted canvas" width="160">

# element-51

Build browser node-editor GUIs from Python, with a Rust/WASM canvas.

[![CI][ci-shield]][ci-url]
[![Rust and WASM][rust-shield]][rust-url]
[![Docs][docs-shield]][docs-url]
[![PyPI][pypi-shield]][pypi-url]
[![License][license-shield]][license-url]
[![Last commit][commit-shield]][commit-url]

[![Python][tool-python-shield]][tool-python-url]
[![uv][tool-uv-shield]][tool-uv-url]
[![Rust][tool-rust-shield]][tool-rust-url]
[![WebAssembly][tool-wasm-shield]][tool-wasm-url]
[![Pydantic][tool-pydantic-shield]][tool-pydantic-url]
[![Starlette][tool-starlette-shield]][tool-starlette-url]

[Docs](https://jacopo-dm.github.io/element-51/) · [Report a bug][issues-url]

</div>

<p align="center">
    <img src="https://raw.githubusercontent.com/Jacopo-DM/element-51/main/docs/images/mnist.gif" alt="Training a small network on handwritten digits in the element-51 editor: a Train node learns while a Plot draws its loss and accuracy" width="100%">
</p>

Three nodes: one shows a picture from the MNIST digits, one trains a small
network on them, and a Plot draws the loss and the accuracy as they change.
The example is `examples/g_experiments/g3_mnist.py`.

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

## Usage

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
Python, editor features, themes, making nodes from your own functions or from
numpy in bulk, and embedding the editor in FastAPI or NiceGUI.

## Features

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
- **Yours to restyle.** Three themes ship, and a small JSON file changes any colour,
    size, font or port shape.
- **Embeddable.** It is an ASGI app, so it mounts inside FastAPI, NiceGUI or any
    other ASGI host.

## Looks

The picture under the header shows the `purple-synthwave` theme, the default. The dark `antimony` theme
is one argument away (`serve(app, theme="antimony")`):

![The same graph in the dark antimony theme](https://raw.githubusercontent.com/Jacopo-DM/element-51/main/docs/images/dark.jpg)

The light theme is one argument away (`serve(app, theme="antimony-light")`):

![The same graph in the light theme](https://raw.githubusercontent.com/Jacopo-DM/element-51/main/docs/images/light.jpg)

`purple-synthwave` was made from a [Coolors](https://coolors.co) palette. Paste any
palette link into the editor's menu and it takes the colours at once, keeping the
text readable (`examples/e_look/e3_palette.py`).

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

## License

Apache License 2.0; see [`LICENSE`](LICENSE).

[ci-shield]: https://img.shields.io/github/actions/workflow/status/Jacopo-DM/element-51/ci.yml?branch=main&label=CI
[ci-url]: https://github.com/Jacopo-DM/element-51/actions/workflows/ci.yml
[rust-shield]: https://img.shields.io/github/actions/workflow/status/Jacopo-DM/element-51/rust.yml?branch=main&label=Rust%20and%20WASM
[rust-url]: https://github.com/Jacopo-DM/element-51/actions/workflows/rust.yml
[docs-shield]: https://img.shields.io/github/actions/workflow/status/Jacopo-DM/element-51/docs.yml?branch=main&label=docs
[docs-url]: https://github.com/Jacopo-DM/element-51/actions/workflows/docs.yml
[pypi-shield]: https://img.shields.io/pypi/v/element-51
[pypi-url]: https://pypi.org/project/element-51/
[license-shield]: https://img.shields.io/github/license/Jacopo-DM/element-51
[license-url]: LICENSE
[commit-shield]: https://img.shields.io/github/last-commit/Jacopo-DM/element-51
[commit-url]: https://github.com/Jacopo-DM/element-51/commits
[tool-python-shield]: https://img.shields.io/badge/Python-3.13%2B-3776AB?style=for-the-badge&logo=python&logoColor=white
[tool-python-url]: https://www.python.org/
[tool-uv-shield]: https://img.shields.io/badge/uv-DE5FE9?style=for-the-badge&logo=uv&logoColor=white
[tool-uv-url]: https://docs.astral.sh/uv/
[tool-rust-shield]: https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white
[tool-rust-url]: https://www.rust-lang.org/
[tool-wasm-shield]: https://img.shields.io/badge/WebAssembly-654FF0?style=for-the-badge&logo=webassembly&logoColor=white
[tool-wasm-url]: https://webassembly.org/
[tool-pydantic-shield]: https://img.shields.io/badge/Pydantic-E92063?style=for-the-badge&logo=pydantic&logoColor=white
[tool-pydantic-url]: https://docs.pydantic.dev/
[tool-starlette-shield]: https://img.shields.io/badge/Starlette-333333?style=for-the-badge
[tool-starlette-url]: https://www.starlette.io/
[issues-url]: https://github.com/Jacopo-DM/element-51/issues
