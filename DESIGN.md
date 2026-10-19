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
  Both build the same node type. An `App([...])` holds the node functions and
  the authoritative graph; `app.add_node(fn, x=, y=)` places a node.
- Execution: on demand (`app.run()` or a Run button), in dependency order. A
  node that raises is marked and shows its message with a Dismiss button;
  nodes downstream of it are skipped; `app.run()` raises the same error.
- Undo and redo: every edit (add, delete, move, link, typed value) is one
  reversible step; a node drag is one step. Undo lives in the browser and
  reaches Python as ordinary edits.
- Canvas: pan and zoom, move, select, delete, edit values in nodes through real
  HTML inputs placed over the canvas.
- Palette: right-click menu.
- Server: starlette and uvicorn. `asgi_app(app)` and `serve(app)` are both
  public, so the editor can be mounted in an existing ASGI app. They are module
  functions that take the `App`, so `session` never imports `server`.
- Save and load: `app.save("g.json")` and `app.load("g.json")`, versioned JSON
  holding nodes, links, positions and values.

Architecture:

- Python package `src/element_51/`: `model` (Graph, Node, Port, Link, PortType;
  pydantic, frozen), `nodes`, `engine`, `protocol`, `session`, `server`,
  `store`, and `web/` (built WASM, glue JS, index.html; git-ignored output).
  Import direction: `server` imports `session`, `store`, `protocol`; `session`
  imports `engine`, `model`; `engine`, `store`, `nodes` import only `model`.
  Nothing imports `server`.
- Rust workspace `rust/`: `element-51-core` (graph, port types, link rules,
  message types, undo log, layout, hit-testing, theme; plain `cargo test`) and
  `element-51-web` (wasm-bindgen layer: canvas drawing, DOM inputs, websocket).
  `element-51-web` imports `element-51-core`, never the reverse.
- Build: `tools/build_wasm.py` calls `rustup run stable cargo build --target
  wasm32-unknown-unknown --release`, then `wasm-bindgen`, and copies the output
  into `src/element_51/web/`. The wheel bundles the output; users need no Rust.
- Browser side: Rust draws everything except text boxes and dropdowns, which
  are real HTML elements Rust places and reads. A small loader in JS and no
  framework, no npm.

Testing: Python at 100% coverage. `element-51-core` under native `cargo test`.
`element-51-web` cannot reach 100% automatically; it is checked through the
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
- Renamed from `patchbay` to `element-51` on 2026-10-19: `patchbay` is taken on
  PyPI and crates.io, and `element-51` is free on both. PyPI name `element-51`,
  import `element_51`, crates `element-51-core` and `element-51-web`, licence
  Apache-2.0 (copyright 2026 jmdm). The repo folder is still named `patchbay`:
  renaming it would break the link to this project's Claude session history, so
  that is the user's call. Earlier entries in this file were corrected to the
  new names, not left as history.
- JSON field names (task 1): ports carry `kind`, nodes carry `node_type`,
  messages carry `op` and `version`. Avoids `type`, which shadows a Python
  builtin, and keeps `op` for message kinds. Links are `source` and `target`
  port references.
- Node ids are `n<number>`, one above the highest in use, so a deleted id is
  never reused.
- Both sides refuse unknown fields (pydantic `extra="forbid"`, serde
  `deny_unknown_fields`) and a protocol version they do not read.
- The browser shows errors in a banner that stays until the user presses
  Dismiss; the same pattern serves the rollback and run-error messages later.
- `@node` accepts only plain functions: a `functools.partial` has no name to
  give the node type.
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

| Date | Change | Result |
| --- | --- | --- |
| 2026-10-19 | task 1: node types reach the browser | `@node` hints become ports; snapshot over `/ws`; Rust parses it and draws both nodes; 27 Python tests at 100%, 13 Rust tests; one JSON fixture parsed by both sides |
| 2026-10-19 | skeleton: Python serves, WASM draws, core logic runs | page shows `float to str link allowed: false`; 4 Python tests at 100%, 4 Rust tests; WASM 706 KB unoptimised |

## Gotchas

- The 22 transitive Rust crates come from wasm-bindgen (`futures-*`, `slab`,
  `bumpalo`, `syn` 3.0.9 and friends) and serde (`serde_derive`).
