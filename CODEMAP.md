# Code map

Generated from the code on 2026-10-19. Generated and downloaded paths are in
the ignore list in CLAUDE.md.

## Entry points

- `element_51.serve(app, port=8765)`: run the editor for an `App` on 127.0.0.1
  until interrupted.
- `element_51.asgi_app(app)`: an ASGI app to mount in an existing server.
- `uv run python tools/build_wasm.py`: build the browser layer.
- `examples/a1_serve.py`: three node types, an empty graph, served.
- Browser: `element_51_web::start()` runs when the page loads, opens `/ws`, and
  wires the canvas, the right-click menu and the window.

## Python, `src/element_51/`

| Module | Purpose | Public interface |
| --- | --- | --- |
| `__init__` | package surface | `App`, `NodeFunction`, `node`, `asgi_app`, `serve` |
| `model` | immutable records for node types and graphs | `Model`, `PortType`, `can_link`, `NodeId`, `Port`, `NodeType`, `PortRef`, `Link`, `Node`, `Graph` (with `next_id()`) |
| `edits` | edits and the rules that allow or refuse them | `AddNode`, `AddLink`, `Edit`, `apply_edit`, `EditError`, `EditErrorKind` |
| `nodes` | `@node` turns a typed function into a node type | `node`, `NodeFunction`, `Registered` (protocol), `port_type` |
| `protocol` | messages in both directions | `Snapshot`, `EditRequest`, `Ack`, `Reject` |
| `session` | the live graph and the node types | `App(node_functions)`, `App.add_node`, `App.apply`, `App.snapshot`, `App.graph` |
| `server` | page, WASM files and `/ws` through starlette and uvicorn | `asgi_app(app)`, `serve(app, port)`, `reply_to`, `WEB_DIR`, `HOST` |

Import direction: `server` imports `session`, `protocol`, `edits`; `session`
imports `nodes`, `protocol`, `edits`, `model`; `protocol` and `edits` import
`model`; `nodes` imports `model`. Nothing imports
`server` except `__init__`. `web/` holds the built page and WASM (generated).

## Rust, `rust/`

| Crate | Purpose | Public interface |
| --- | --- | --- |
| `element-51-core` | records, messages, edit rules, the browser's graph copy, box layout; native tests | `PortType`, `Port`, `NodeType`, `PortRef`, `Link`, `Node`, `Graph` (`next_id`), `Snapshot`, `Edit`, `EditRequest`, `Ack`, `Reject`, `Message` (`from_json`, `to_json`), `can_link`, `apply_edit`, `EditError` (`kind`), `Replica` (`new`, `reset`, `graph`, `node_types`, `submit`, `ack`, `reject`, `pending_len`, `next_node_id`), `layout::{node_rect, input_anchor, output_anchor, anchor_of, port_at, PortHit, Side, Rect, constants}` |
| `element-51-web` | WASM layer: websocket, canvas drawing (`draw`), mouse and menu handling, error banner | `start()` (runs on load) |

Import direction: `element-51-web` imports `element-51-core`. The reverse never
happens. `element-51-web/static/index.html` is copied into `src/element_51/web/`.

## Shared fixtures, `tests/fixtures/`

Each is read by a Python test and a Rust test, so the two sides cannot drift:
`snapshot.json` (the snapshot message), `messages.json` (edit request, ack,
reject), `link_cases.json` (a graph, candidate links and the expected verdicts).

## Not built yet

The engine, save and load, node moving and deleting, value editing, undo, the
theme, the nested menu. The planned layout is in DESIGN.md under Specs.
