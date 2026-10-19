# Code map

Generated from the code on 2026-10-19. Generated and downloaded paths are in
the ignore list in CLAUDE.md.

## Entry points

- `element_51.serve(app, port=8765)`: run the editor for an `App` on 127.0.0.1
  until interrupted.
- `element_51.asgi_app(app)`: an ASGI app to mount in an existing server.
- `uv run python tools/build_wasm.py`: build the browser layer.
- `examples/a1_two_nodes.py` to `a6_port_types.py`: served graphs, one idea each
  (two nodes and a link, which opens the browser, arithmetic, strings, errors, diamond and int to float, all four
  port types), `a8_multiple_outputs.py` (a function with two outputs) and
  `a9_class_nodes.py` (class-form nodes), `a10_live_push.py` (a script edits the
  graph while the page is open), `a11_save_load.py` (save and load a file), `a12_big_graph.py` (pan, zoom, move, select, delete), `a13_values.py` (typing values into inputs), `a14_undo_redo.py` (undo and redo). `a7_script_api.py`: the graph from plain
  Python, no browser. `a0_hello_world.py` is the project template, not an
  example of the library. Each example has `build_app()`; `tests/test_examples.py`
  runs every one.
- Browser: `element_51_web::start()` runs when the page loads, opens `/ws`, and
  wires the canvas, the right-click menu and the window.

## Python, `src/element_51/`

| Module | Purpose | Public interface |
| --- | --- | --- |
| `__init__` | package surface | `App`, `Model`, `Node`, `NodeFunction`, `node`, `asgi_app`, `serve`, `StoreError` |
| `model` | immutable records for node types and graphs | `Model`, `PortType`, `PortValue`, `value_fits`, `can_link`, `NodeId`, `Port`, `NodeType` (`defaults`), `PortRef`, `Link`, `PlacedNode` (`values`), `Graph` (with `next_id()`) |
| `edits` | edits and the rules that allow or refuse them | `AddNode`, `AddLink`, `MoveNode`, `DeleteNode`, `DeleteLink`, `SetValue`, `AnyEdit`, `Edit`, `apply_edit`, `EditError`, `EditErrorKind` |
| `nodes` | `@node` and the `Node` base class turn a typed function or a class into a node type, with its defaults and output model | `node`, `Node` (`run`, `call`; `node_type`, `defaults`, `output_model` built per subclass), `NodeFunction` (`call`, `defaults`, `output_model`), `Registered` (protocol), `port_type` |
| `engine` | runs a graph in dependency order; an input takes its link, then the node's own value, then the default | `run_graph`, `RunResult`, `NodeOutcome` (`outputs` by port) |
| `protocol` | messages in both directions | `Snapshot`, `EditRequest`, `RunRequest`, `ClientMessage`, `Ack`, `Reject` (and `engine.RunResult`) |
| `session` | the live graph and the node types | `App(node_functions)`, `App.add_node`, `App.add_link`, `App.subscribe`, `App.save`, `App.load`, `App.apply` (silent), `App.execute`, `App.run` (nested: node id, then port), `App.snapshot`, `App.graph` |
| `store` | save and load a graph as versioned JSON | `FORMAT_VERSION` (2; reads 1 and 2), `StoreError`, `save_graph`, `load_graph` |
| `server` | page, WASM files and `/ws` through starlette and uvicorn | `asgi_app(app)`, `serve(app, port, open_browser)`, `PageServer`, async `reply_to`, `Hub` (open pages, `broadcast`), `Connection` (one page's outbox, `send`, `pump`), `Sender`, `WEB_DIR`, `HOST` |

Import direction: `server` imports `session`, `protocol`, `edits`, `engine`;
`session` imports `nodes`, `protocol`, `edits`, `engine`, `store`, `model`; `store` imports `edits`, `model`; `engine`
imports `nodes`, `model`; `protocol` imports `edits`, `model`; `edits` and
`nodes` import `model`. Nothing imports
`server` except `__init__`. `web/` holds the built page and WASM (generated).

## Rust, `rust/`

| Crate | Purpose | Public interface |
| --- | --- | --- |
| `element-51-core` | records, messages, edit rules, the browser's graph copy, box layout; native tests | `PortType`, `Port`, `NodeType`, `PortRef`, `Link`, `PlacedNode`, `Graph` (`next_id`, `is_linked`), `Snapshot`, `Edit`, `EditRequest`, `Ack`, `Reject`, `RunRequest`, `RunResult` (`outcome_for`, `error_summary`), `NodeOutcome` (`outputs`, `output_text`), `Status`, `Message` (`from_json`, `to_json`), `can_link`, `apply_edit`, `value_fits`, `EditError` (`kind`), `fields::{parse_input, format_value}`, `history::History` (`record`, `undo`, `redo`, `can_undo`, `can_redo`), `Replica` (`new`, `reset` (keeps pending edits that still apply), `graph`, `node_types`, `submit`, `ack`, `reject`, `pending_len`, `next_node_id`), `layout::{node_rect, input_anchor, output_anchor, output_row, anchor_of, port_at, node_at, link_at, wire_controls, PortHit, Side, Rect}` (every one takes the theme's `Metrics`; a card's rows are its inputs, then its outputs), `theme::{Theme, Colours, Metrics, Fonts, THEME_NAMES}` (`builtin`, `css_variables`; built-in themes are JSON in `element-51-core/themes/`), `view::{View, MIN_ZOOM, MAX_ZOOM}` (`pan_by`, `zoom_at`, `to_screen`, `to_graph`) |
| `element-51-web` | WASM layer: websocket, canvas drawing (`draw`, one `Frame` per frame, which carries the `Theme`; `columns` and `field_box` place the labels and value boxes; the dot grid, cards, wires and ports are drawn in that order), `apply_theme` (CSS custom properties from the theme), the HTML input boxes over the canvas (`sync_fields`), undo and redo keys (`step_history`), mouse, wheel, key, menu and Run button handling (a `Gesture` is a link drag, a node move or a pan), error banner | `start()` (runs on load) |

Import direction: `element-51-web` imports `element-51-core`. The reverse never
happens. `element-51-web/static/index.html` is copied into `src/element_51/web/`.

## Shared fixtures, `tests/fixtures/`

Each is read by a Python test and a Rust test, so the two sides cannot drift:
`snapshot.json` (the snapshot message), `messages.json` (edit request, ack,
reject, run, run result), `link_cases.json` (a graph with a multi-output node, candidate links and the expected verdicts), `edit_cases.json` (move, delete and set-value edits with the expected graphs).

## Tests

Python, `tests/`, one file per module: `test_model.py`, `test_edits.py`,
`test_nodes.py`, `test_engine.py`, `test_protocol.py`, `test_session.py`,
`test_server.py` (a real uvicorn server and the `websockets` client),
`test_examples.py` (loads every `examples/a1` to `a14` file and runs its graph).
`sample_nodes.py` holds node functions shared by several test files.

Rust, `rust/element-51-core/tests/`: `snapshot.rs`, `messages.rs`, `rules.rs`
(the shared fixtures), `replica.rs`, `layout.rs`, `run.rs`; unit tests for
`can_link` sit in `src/lib.rs`. `element-51-web` has no automated tests.

## Not built yet

The theme, the nested menu. The
planned layout is in DESIGN.md under Specs.
