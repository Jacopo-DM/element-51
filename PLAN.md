# Plan

## Goals

### skeleton

- Goal: a Python author defines two node types, edits a graph of them in the
  browser, saves and loads it, and Python sees the result, all from one
  command.
- Kind: new behaviour.
- Success measure (each is a test):
    1. `@node` on a typed function and a pydantic `Node` subclass each appear
       as a node type in the browser palette.
    2. In the browser, two nodes can be added and linked; a link between
       incompatible port types (float to str) is refused by the Rust engine.
    3. After editing, `app.graph` in Python holds the same nodes and links,
       and `app.run()` returns the computed value.
    4. `uv run examples/a1_two_nodes.py` starts a local server, opens the
       page, and stops cleanly on Ctrl+C.
    5. A function returning a pydantic model (e.g. `-> DivMod` with fields `q`
       and `r`) shows one output port per field; a link from either port
       works and `app.run()` passes the right value on.
    6. `app.save("g.json")` writes nodes, links and positions with a format
       version field, and `app.load("g.json")` restores the same graph in
       Python and in the browser.
- Status: approved 2026-10-19
- Added: 2026-10-19

### antimony-theme

- Goal: the editor looks and feels like Antimony (Matt Keeter's CAD tool) by
  default, with themes switchable and the credit in the README. Depends on
  `skeleton` (needs a canvas to draw on).
- Kind: new behaviour.
- Success measure (each is a test, except 1, which you judge):
    1. Side by side with the reference screenshots (kept outside the repo),
       you accept the default dark theme: charcoal node cards with a grey
       header bar (id left, type name right, menu icon), right-aligned
       labels with dark value boxes, a greyed placeholder on a linked input,
       type-coloured port squares (orange numbers, olive green for others),
       smooth S-curve wires, a dark dot-grid canvas, a light outline on the
       selected node.
    2. All colours, sizes and fonts come from one `Theme` definition in
       `patchbay-core`; no colour literal appears in the drawing code.
    3. Two themes ship: `antimony` (dark, default) and `antimony-light`
       (light grey canvas). `bay.serve(theme="antimony-light")` shows the
       light one; an unknown name raises an error naming the valid ones.
    4. Right-click opens a nested category menu in the Antimony style; typing
       filters across all categories. `@node(category="Math")` and the
       pydantic class form both set the category.
    5. `README.md` credits Antimony and Matt Keeter with a link, and says the
       look is inspired by it. No Antimony image or code is in the repo.
- Status: approved 2026-10-19
- Added: 2026-10-19

## Current state

Phase: planning. Goals `skeleton` and `antimony-theme` approved 2026-10-19.
Next step: approve the spec and architecture for `skeleton` (proposed in chat
2026-10-19, not yet answered).

Approved 2026-10-19, not yet installed (the skeleton step installs them):

- Python runtime, PyPI: starlette 1.9.0, uvicorn 0.54.0, websockets 17.2 and
  their dependencies (anyio, h11, click, idna), about 0.8 MB.
- Rust target `wasm32-unknown-unknown` for the rustup `stable` toolchain, from
  static.rust-lang.org, 23.2 MB. The Homebrew `rustc` on PATH cannot use it;
  build with `rustup run stable cargo`.
- wasm-bindgen CLI 0.2.129, prebuilt `aarch64-apple-darwin` from the GitHub
  release, 7.9 MB, sha256 checked, kept in git-ignored `tools/bin/`.
- Rust crates, crates.io: wasm-bindgen =0.2.129, js-sys 0.3.106, web-sys
  0.3.106, serde 1.0.229, serde_json 1.0.154 (about 1.1 MB direct). Report the
  transitive list from `cargo fetch` before building.

## Baseline

No code yet.

## Tasks

None yet.

## Open questions

None yet.

## Known problems

Found by /zz-01-project-setup on the uv scaffolding (2026-10-19):

- `uv run --locked ruff check --no-fix` reports 2 errors in
  `src/patchbay/__init__.py`: `non-empty-init-module` (a function lives in
  `__init__`) and `undocumented-public-package` (no docstring).
- `uv run --locked pytest` fails: there is no `tests/` folder, so coverage is 0%
  against the 100% floor.
- `src/__init__.py` and `src/hello.py` sit beside the real package
  `src/patchbay/`. They look like leftover scaffolding ("Hello from .!").
- `README.md`, `docs/index.md` and the `pyproject.toml` description are still
  template text.
- `uv run --locked mypy .` passes (6 files).
