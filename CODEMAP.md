# Code map

Generated from the code on 2026-10-19. Generated and downloaded paths are in
the ignore list in CLAUDE.md.

## Entry points

- `element_51.serve(app, port=8765)`: run the editor for an `App` on 127.0.0.1
  until interrupted.
- `element_51.asgi_app(app)`: an ASGI app to mount in an existing server.
- `uv run python tools/build_wasm.py`: build the browser layer.
- `examples/a1_serve.py`: two node types and two nodes, served.
- Browser: `element_51_web::start()` runs when the page loads, opens `/ws`.

## Python, `src/element_51/`

| Module | Purpose | Public interface |
| --- | --- | --- |
| `__init__` | package surface | `App`, `NodeFunction`, `node`, `asgi_app`, `serve` |
| `model` | immutable records for node types and graphs | `Model`, `PortType`, `Port`, `NodeType`, `PortRef`, `Link`, `Node`, `Graph` (with `next_id()`) |
| `nodes` | `@node` turns a typed function into a node type | `node`, `NodeFunction`, `Registered` (protocol), `port_type` |
| `protocol` | messages sent to the browser | `Snapshot` |
| `session` | the live graph and the node types | `App(node_functions)`, `App.add_node`, `App.snapshot`, `App.graph` |
| `server` | page, WASM files and `/ws` through starlette and uvicorn | `asgi_app(app)`, `serve(app, port)`, `WEB_DIR`, `HOST` |

Import direction: `server` imports `session`; `session` imports `nodes`,
`protocol`, `model`; `protocol` and `nodes` import `model`. Nothing imports
`server` except `__init__`. `web/` holds the built page and WASM (generated).

## Rust, `rust/`

| Crate | Purpose | Public interface |
| --- | --- | --- |
| `element-51-core` | graph records, messages, link rule, box layout; native tests | `PortType`, `Port`, `NodeType`, `PortRef`, `Link`, `Node`, `Graph`, `Snapshot`, `Message::from_json`, `can_link`, `layout::{node_rect, Rect, NODE_WIDTH, HEADER_HEIGHT, ROW_HEIGHT}` |
| `element-51-web` | WASM layer: websocket, canvas, error banner | `start()` (runs on load) |

Import direction: `element-51-web` imports `element-51-core`. The reverse never
happens. `element-51-web/static/index.html` is copied into `src/element_51/web/`.
`tests/fixtures/snapshot.json` is parsed by both `tests/test_model.py` and
`rust/element-51-core/tests/snapshot.rs`.

## Not built yet

Edits and their acknowledgements, the engine, save and load, the theme, the
menu. The planned layout is in DESIGN.md under Specs.
