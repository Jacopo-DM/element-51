# Code map

Generated from the code on 2026-10-20. Generated and downloaded paths are in
the ignore list in CLAUDE.md.

## Entry points

- `element_51.serve(app, port=8765, *, open_browser=False, theme=None)`: run the
    editor for an `App` on 127.0.0.1 until interrupted; Ctrl+C ends it with exit 0.
    `theme` is a built-in name or the path of a theme file.
- `element_51.asgi_app(app, theme=None)`: an ASGI app to mount in an existing
    server.
- `uv run python tools/build_wasm.py`: build the browser layer.
- `examples/`, in group folders; the file name starts with the folder's letter and
    a number. `a_basics`: `a1_hello_world.py` (the project template, not an example
    of the library), `a2_two_nodes.py` (opens the browser), `a3_arithmetic.py`,
    `a4_strings.py`. `b_node_types`: `b1_port_types.py`, `b2_multiple_outputs.py`,
    `b3_class_nodes.py`, `b4_diamond.py`, `b5_errors.py`. `c_python_api`:
    `c1_script_api.py` (the graph from plain Python, no browser), `c2_live_push.py`
    (a script edits the graph while the page is open), `c3_save_load.py`.
    `d_editor`: `d1_big_graph.py` (pan, zoom, move, select, delete),
    `d2_values.py`, `d3_undo_redo.py`, `d4_menu.py`, `d5_card_menu.py`. `e_look`:
    `e1_light_theme.py`, `e2_theme_file.py` with `my_theme.json`. `f_embedding`:
    `f1_fastapi_host.py` and `f2_nicegui_host.py` (the editor mounted at `/editor`
    in a FastAPI or NiceGUI host, with a side panel). Each example has
    `build_app()`; `tests/test_examples.py` runs every one except the template.
- `docs/themes.md`: the theme reference. A hand-written introduction, then one
    table per theme section (`colours`, `ports`, `page`, `metrics`, `fonts`) with
    every key, its meaning from the doc comment in `theme.rs` and the built-in
    values; the native tests in `theme.rs` fail if a key has no row.
- `.github/workflows/`: `ci.yml` (Python checks, from the `rsync-sync` template),
    `rust.yml` (Rust checks and the WASM build and wheel check) and `docs.yml`.
- Browser: `element_51_web::start()` runs when the page loads, opens `/ws`, and
    wires the canvas, the right-click menu and the window.

## Python, `src/element_51/`

| Module | Purpose | Public interface |
| --- | --- | --- |
| `__init__` | package surface | `App`, `Model`, `Node`, `NodeFunction`, `node`, `asgi_app`, `serve`, `StoreError` |
| `model` | immutable records for node types and graphs | `Model`, `PortType`, `PortValue`, `value_fits`, `can_link`, `NodeId`, `Port`, `NodeType` (`defaults`, `category`), `PortRef`, `Link`, `PlacedNode` (`values`), `Graph` (with `next_id()`) |
| `edits` | edits and the rules that allow or refuse them | `AddNode`, `AddLink`, `MoveNode`, `DeleteNode`, `DeleteLink`, `SetValue`, `AnyEdit`, `Edit`, `apply_edit`, `EditError`, `EditErrorKind` |
| `nodes` | `@node` and the `Node` base class turn a typed function or a class into a node type, with its defaults and output model | `node` (bare, or `node(category=...)`), `Node` (`run`, `call`; `node_type`, `defaults`, `output_model` built per subclass), `NodeFunction` (`call`, `defaults`, `output_model`), `Registered` (protocol), `port_type` |
| `engine` | runs a graph in dependency order; an input takes its link, then the node's own value, then the default | `run_graph`, `RunResult`, `NodeOutcome` (`outputs` by port) |
| `protocol` | messages in both directions | `Snapshot` (carries `theme` and `theme_changes`), `EditRequest`, `RunRequest`, `ClientMessage`, `Ack`, `Reject` (and `engine.RunResult`) |
| `session` | the live graph and the node types | `App(node_functions)`, `App.add_node`, `App.add_link`, `App.theme` (the getter returns the base theme's name; the setter takes a name or the path of a theme file, validates and pushes), `App.subscribe`, `App.save`, `App.load`, `App.apply` (silent), `App.execute`, `App.run` (nested: node id, then port), `App.snapshot`, `App.graph` |
| `theme` | the theme names Python offers, and theme files (the look is in the Rust core) | `THEME_NAMES`, `DEFAULT_THEME`, `check_theme`, `ThemeChanges`, `ThemeFile`, `resolve_theme` |
| `store` | save and load a graph as versioned JSON | `FORMAT_VERSION` (2; reads 1 and 2), `StoreError`, `save_graph`, `load_graph` |
| `server` | page, WASM files and `/ws` through starlette and uvicorn | `asgi_app(app, theme)`, `serve(app, port, open_browser, theme)`, `PageServer`, async `reply_to`, `Hub` (open pages, `broadcast`), `Connection` (one page's outbox, `send`, `pump`), `Sender`, `WEB_DIR`, `HOST` |

Import direction: `server` imports `session`, `protocol`, `edits`, `engine`;
`session` imports `nodes`, `protocol`, `edits`, `engine`, `store`, `model`; `store` imports `edits`, `model`; `engine`
imports `nodes`, `model`; `protocol` imports `edits`, `model`; `edits` and
`nodes` import `model`. Nothing imports
`server` except `__init__`. `web/` holds the built page and WASM (generated).

## Rust, `rust/`

| Crate | Purpose | Public interface |
| --- | --- | --- |
| `element-51-core` | records, messages, `address::websocket_url` (the page's websocket sits beside the page), edit rules, the browser's graph copy, box layout; native tests | `PortType`, `Port`, `NodeType`, `PortRef`, `Link`, `PlacedNode`, `Graph` (`next_id`, `is_linked`), `Snapshot` (`theme`, `theme_changes`), `Edit`, `EditRequest`, `Ack`, `Reject`, `RunRequest`, `RunResult` (`outcome_for`, `error_summary`), `NodeOutcome` (`outputs`, `output_text`), `Status`, `Message` (`from_json`, `to_json`), `can_link`, `apply_edit`, `value_fits`, `EditError` (`kind`), `fields::{parse_input, format_value}`, `menu::{Entry, Match, build_menu, search, CardAction, card_actions, clamp_into, flyout_at}` (`CardAction::label`, `edit`), `history::History` (`record`, `undo`, `redo`, `can_undo`, `can_redo`), `Replica` (`new`, `reset` (keeps pending edits that still apply), `graph`, `node_types`, `submit`, `ack`, `reject`, `pending_len`, `next_node_id`), `layout::{node_rect, icon_rect, icon_at, input_anchor, output_anchor, output_row, anchor_of, port_at, node_at, link_at, wire_controls, PortHit, Side, Rect}` (every one takes the theme's `Metrics`; a card's rows are its inputs, then its outputs), `theme::{Theme, Colours, Ports, PortSides, PortStyle, PortShape, Metrics, Page, Fonts, THEME_NAMES}` (`builtin`, `with_changes`, `port_style`, `css_variables`; built-in themes are JSON in `element-51-core/themes/`), `view::{View, MIN_ZOOM, MAX_ZOOM}` (`pan_by`, `zoom_at`, `to_screen`, `to_graph`) |
| `element-51-web` | WASM layer: websocket, canvas drawing (`draw`, one `Frame` per frame, which carries the `Theme`; `columns` and `field_box` place the labels and value boxes; the dot grid, cards, wires and ports are drawn in that order), `apply_theme` (CSS custom properties from the theme), the HTML input boxes over the canvas (`sync_fields`), the add-node menu (`fill_menu`, `on_menu_search`, `on_menu_hover` places the flyouts), the card menu on a card's header icon (`open_card_menu`, `on_card_menu_click`), an opaque canvas context at device-pixel size (`fit_canvas`, `json_parse`), undo and redo keys (`step_history`), mouse, wheel, key, menu and Run button handling (a `Gesture` is a link drag, a node move or a pan), error banner | `start()` (runs on load) |

Import direction: `element-51-web` imports `element-51-core`. The reverse never
happens. `element-51-web/static/index.html` is copied into `src/element_51/web/`.

## Shared fixtures, `tests/fixtures/`

Each is read by a Python test and a Rust test, so the two sides cannot drift:
`snapshot.json` (the snapshot message), `messages.json` (edit request, ack,
reject, run, run result), `link_cases.json` (a graph with a multi-output node, candidate links and the expected verdicts), `edit_cases.json` (move, delete and set-value edits with the expected graphs),
`theme_names.json` (the theme names both sides offer). `snapshot.json` carries a `theme_changes` object, so both sides parse theme files' changes.

## Tests

Python, `tests/`, one file per module: `test_model.py`, `test_edits.py`,
`test_nodes.py`, `test_engine.py`, `test_protocol.py`, `test_session.py`,
`test_store.py`, `test_theme.py`, `test_server.py` (a real uvicorn server and the `websockets` client; it also runs the FastAPI example's host with a fake web folder),
`test_examples.py` (loads every example in `examples/` except the template `a1_hello_world.py` and runs its graph).
`sample_nodes.py` holds node functions shared by several test files.

Rust, `rust/element-51-core/tests/`: `address.rs`, `snapshot.rs`, `messages.rs`, `rules.rs`
(the shared fixtures), `replica.rs`, `layout.rs`, `run.rs`, `view.rs`, `fields.rs`,
`history.rs`, `menu.rs`, `theme.rs` (includes the no-hex-colour scan of the web
crate, the example theme file and the key tables in `docs/themes.md`); unit tests for
`can_link` sit in `src/lib.rs`. `element-51-web` has no automated tests. `theme.rs` and `address.rs` read files outside `rust/` with `include_str!` (`examples/e_look/my_theme.json`, `docs/themes.md`, `element-51-web/static/index.html`).
