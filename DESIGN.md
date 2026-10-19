# Design

## Specs

### skeleton (approved 2026-10-19)

Product: a Python library for browser dataflow / node-editor GUIs. A Rust
program compiled to WASM draws the canvas and owns the graph while you edit.
Python defines node types, serves the page, holds the authoritative copy, and
runs the nodes.

Users: Python authors who want a node-editor GUI for their own node types.

Acceptance criteria: the six tests listed under goal `skeleton` in PLAN.md.

Non-goals: multi-user live editing, static export without Python, hosted or
remote deployment (binds to 127.0.0.1 only), mobile and touch, a binary
channel for large data (JSON only; the message format carries a version and
leaves room for one), live re-run on every edit.

Behaviour:

- Graph owner: the browser edits first and sends numbered edits. Python replies
  `ok N` or `rejected N: reason`. On a rejection the browser rolls the edit
  back and shows a message with a Dismiss button. On connect or reload Python
  sends a full snapshot.
- Port types: float, int, str, bool. An int output may feed a float input. Any
  other pairing is refused by the Rust engine at drag time. A link that would
  make a cycle is refused.
- Ports: inputs come from the function parameters or class fields. A function
  returning a plain type has one output port named `out`. A function returning
  a pydantic model has one output port per field, in field order, named after
  the field.
- Python API: `@node` on a typed function, and a pydantic `Node` subclass.
  Both build the same node type.
- Execution: on demand (`app.run()` or a Run button), in dependency order. A
  node that raises is marked and shows its message with a Dismiss button;
  nodes downstream of it are skipped; `app.run()` raises the same error.
- Undo and redo: every edit (add, delete, move, link, typed value) is one
  reversible step; a node drag is one step. Undo lives in the browser and
  reaches Python as ordinary edits.
- Canvas: pan and zoom, move, select, delete, edit values in nodes through real
  HTML inputs placed over the canvas.
- Palette: right-click menu.
- Server: starlette and uvicorn. `bay.asgi_app()` and `bay.serve()` are both
  public, so the editor can be mounted in an existing ASGI app.
- Save and load: `app.save("g.json")` and `app.load("g.json")`, versioned JSON
  holding nodes, links, positions and values.

Architecture:

- Python package `src/patchbay/`: `model` (Graph, Node, Port, Link, PortType;
  pydantic, frozen), `nodes`, `engine`, `protocol`, `session`, `server`,
  `store`, and `web/` (built WASM, glue JS, index.html; git-ignored output).
  Import direction: `server` imports `session`, `store`, `protocol`; `session`
  imports `engine`, `model`; `engine`, `store`, `nodes` import only `model`.
  Nothing imports `server`.
- Rust workspace `rust/`: `patchbay-core` (graph, port types, link rules,
  message types, undo log, layout, hit-testing, theme; plain `cargo test`) and
  `patchbay-web` (wasm-bindgen layer: canvas drawing, DOM inputs, websocket).
  `patchbay-web` imports `patchbay-core`, never the reverse.
- Build: `tools/build_wasm.py` calls `rustup run stable cargo build --target
  wasm32-unknown-unknown --release`, then `wasm-bindgen`, and copies the output
  into `src/patchbay/web/`. The wheel bundles the output; users need no Rust.
- Browser side: Rust draws everything except text boxes and dropdowns, which
  are real HTML elements Rust places and reads. A small loader in JS and no
  framework, no npm.

Testing: Python at 100% coverage. `patchbay-core` under native `cargo test`.
`patchbay-web` cannot reach 100% automatically; it is checked through the
in-app browser against the running example.

## Decisions

- Product shape and Rust/WASM role (renderer and engine): user choice,
  2026-10-19.
- Python API offers both the decorator and the pydantic class: user choice.
- Python serves, browser edits: user choice.
- JSON for the wire format, binary where it matters later: user choice.
- Rust owns the graph while editing, synced to Python: user choice.
- On-demand execution: user choice.
- Fixed small port type set: user choice.
- Two Rust crates in a workspace: user choice.
- Multiple outputs in the skeleton, as pydantic model fields: user choice.
- starlette and uvicorn rather than websockets alone: the user wants the editor
  mountable in an existing ASGI app, which websockets alone cannot do.
- Hybrid canvas: Rust draws, real HTML inputs for text, because a canvas text
  box has no copy-paste, input-method support or screen-reader access.
- One wheel with the WASM bundled, so users need no Rust toolchain.
- Rust built with `rustup run stable cargo`: the Homebrew `rustc` on PATH has no
  WASM target.
- wasm-bindgen CLI is the prebuilt release, pinned to the crate version
  (0.2.129): the two must match exactly.
- Servers tested with a real uvicorn server and the websockets client, to avoid
  adding httpx as a dev dependency.
- Reference screenshots of Antimony stay outside the repo; they are not ours to
  redistribute.

## Rejected approaches

- Python owns the graph and the browser asks first: every drag waits for a
  round trip, and a stuck Python script freezes the editor.
- A single Rust crate: link-rule tests would need a browser test runner.
- A JS framework around the canvas: adds npm and a second language.
- Rust runs the node code in the browser: makes Python only a description
  language, against the goal.
- websockets alone as the server: smallest, but cannot be mounted in an ASGI
  app.
- aiohttp: heaviest option, no benefit here.
- Committing the built WASM: stale builds and large binaries in every commit.

## Results

None yet.

## Gotchas

None yet.
