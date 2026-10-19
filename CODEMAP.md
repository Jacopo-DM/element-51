# Code map

Generated from the code on 2026-10-19. Generated and downloaded paths are in
the ignore list in CLAUDE.md.

## Entry points

- `element_51.serve(app, port=8765)`: run the editor for an `App` on 127.0.0.1
  until interrupted.
- `element_51.asgi_app(app)`: an ASGI app to mount in an existing server.
- `uv run python tools/build_wasm.py`: build the browser layer.
- `examples/a1_serve.py` to `a6_port_types.py`: served graphs, one idea each
  (blank canvas, arithmetic, strings, errors, diamond and int to float, all four
  port types), `a8_multiple_outputs.py` (a function with two outputs) and
  `a9_class_nodes.py` (class-form nodes). `a7_script_api.py`: the graph from plain
  Python, no browser. Each example has `build_app()`; `tests/test_examples.py`
  runs every one.
- Browser: `element_51_web::start()` runs when the page loads, opens `/ws`, and
  wires the canvas, the right-click menu and the window.

## Python, `src/element_51/`

| Module | Purpose | Public interface |
| --- | --- | --- |
| `__init__` | package surface | `App`, `Model`, `Node`, `NodeFunction`, `node`, `asgi_app`, `serve` |
| `model` | immutable records for node types and graphs | `Model`, `PortType`, `PortValue`, `value_fits`, `can_link`, `NodeId`, `Port`, `NodeType`, `PortRef`, `Link`, `PlacedNode`, `Graph` (with `next_id()`) |
| `edits` | edits and the rules that allow or refuse them | `AddNode`, `AddLink`, `Edit`, `apply_edit`, `EditError`, `EditErrorKind` |
| `nodes` | `@node` and the `Node` base class turn a typed function or a class into a node type, with its defaults and output model | `node`, `Node` (`run`, `call`; `node_type`, `defaults`, `output_model` built per subclass), `NodeFunction` (`call`, `defaults`, `output_model`), `Registered` (protocol), `port_type` |
| `engine` | runs a graph in dependency order | `run_graph`, `RunResult`, `NodeOutcome` (`outputs` by port) |
| `protocol` | messages in both directions | `Snapshot`, `EditRequest`, `RunRequest`, `ClientMessage`, `Ack`, `Reject` (and `engine.RunResult`) |
| `session` | the live graph and the node types | `App(node_functions)`, `App.add_node`, `App.add_link`, `App.apply`, `App.execute`, `App.run` (nested: node id, then port), `App.snapshot`, `App.graph` |
| `server` | page, WASM files and `/ws` through starlette and uvicorn | `asgi_app(app)`, `serve(app, port)`, async `reply_to`, `WEB_DIR`, `HOST` |

Import direction: `server` imports `session`, `protocol`, `edits`, `engine`;
`session` imports `nodes`, `protocol`, `edits`, `engine`, `model`; `engine`
imports `nodes`, `model`; `protocol` imports `edits`, `model`; `edits` and
`nodes` import `model`. Nothing imports
`server` except `__init__`. `web/` holds the built page and WASM (generated).

## Rust, `rust/`

| Crate | Purpose | Public interface |
| --- | --- | --- |
| `element-51-core` | records, messages, edit rules, the browser's graph copy, box layout; native tests | `PortType`, `Port`, `NodeType`, `PortRef`, `Link`, `PlacedNode`, `Graph` (`next_id`), `Snapshot`, `Edit`, `EditRequest`, `Ack`, `Reject`, `RunRequest`, `RunResult` (`outcome_for`, `error_summary`), `NodeOutcome` (`outputs`, `output_text`), `Status`, `Message` (`from_json`, `to_json`), `can_link`, `apply_edit`, `EditError` (`kind`), `Replica` (`new`, `reset`, `graph`, `node_types`, `submit`, `ack`, `reject`, `pending_len`, `next_node_id`), `layout::{node_rect, input_anchor, output_anchor, anchor_of, port_at, PortHit, Side, Rect, constants}` |
| `element-51-web` | WASM layer: websocket, canvas drawing (`draw`), mouse, menu and Run button handling, error banner | `start()` (runs on load) |

Import direction: `element-51-web` imports `element-51-core`. The reverse never
happens. `element-51-web/static/index.html` is copied into `src/element_51/web/`.

## Shared fixtures, `tests/fixtures/`

Each is read by a Python test and a Rust test, so the two sides cannot drift:
`snapshot.json` (the snapshot message), `messages.json` (edit request, ack,
reject, run, run result), `link_cases.json` (a graph with a multi-output node, candidate links and the expected verdicts).

## Not built yet

Save and load, node moving and deleting, value editing, undo, the
theme, the nested menu. The planned layout is in DESIGN.md under Specs.
