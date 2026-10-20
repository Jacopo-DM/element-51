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
- `tools/ui_states.py`, `tools/ui_states.js` and `tools/ui_axe.js`: serve a fixed graph and put the page in
    each state, for screenshots and accessibility scans (commands in CLAUDE.md).
- `tools/bench_python.py`, `tools/bench_page.py` and `tools/bench_page.js`: the speed
    benchmarks (see the commands in CLAUDE.md).
- `uv run python tools/check_wheel.py`: build the wheel in a temporary folder and
    fail if it lacks the browser layer.
- `uv run python tools/measure_structure.py [--jscpd <binary>]`: print the
    structure numbers (files, lines, files over 400 lines, modules over five
    imports, clones).
- `examples/template.py`: the project template for a script (not an example of the
    library; the example tests do not load it).
- `examples/`, in group folders; the file name starts with the folder's letter and
    a number. `a_basics`: `a1_hello_world.py` (one node, runs uvicorn itself on a `HOST` you can change, no browser opened), `a2_two_nodes.py` (opens the browser), `a3_arithmetic.py`,
    `a4_strings.py`. `b_node_types`: `b1_port_types.py`, `b2_multiple_outputs.py`,
    `b3_class_nodes.py`, `b4_diamond.py`, `b5_errors.py`. `c_python_api`:
    `c1_script_api.py` (the graph from plain Python, no browser), `c2_live_push.py`
    (a script edits the graph while the page is open), `c3_save_load.py`.
    `d_editor`: `d1_big_graph.py` (pan, zoom, move, select, delete),
    `d2_values.py`, `d3_undo_redo.py`, `d4_menu.py`, `d5_card_menu.py`. `e_look`:
    `e1_light_theme.py`, `e2_theme_file.py` with `my_theme.json`, `e3_palette.py`, `e4_purple_synthwave.py`. `f_embedding`:
    `f1_fastapi_host.py` and `f2_nicegui_host.py` (the editor mounted at `/editor`
    in a FastAPI or NiceGUI host, with a side panel). Each example has
    `build_app()`; `tests/test_examples.py` runs every one.
- `docs/images/`: the three README screenshots (`dark`, `light`, `palette`; how they
    were made is in DESIGN.md, Gotchas).
- `docs/themes.md`: the theme reference. A hand-written introduction, then one
    table per theme section (`colours`, `ports`, `page`, `metrics`, `fonts`) with
    every key, its meaning from the doc comment in `theme.rs` and the built-in
    values; the native tests in `theme.rs` fail if a key has no row.
- `.github/dependabot.yml`: weekly updates for github-actions, uv and cargo (`/rust`).
- `.github/workflows/`: `ci.yml` (Python checks, from the `rsync-sync` template),
    `rust.yml` (Rust checks and the WASM build and wheel check) and `docs.yml`.
- Browser: `element_51_web::start()` runs when the page loads, opens `/ws`, and
    wires the canvas, the right-click menu and the window.

## Python, `src/element_51/`

Module READMEs (what each assumes and must never depend on): `src/element_51/README.md`, `rust/element-51-core/README.md`, `rust/element-51-web/README.md`.

| Module | Purpose | Public interface |
| --- | --- | --- |
| `__init__` | package surface | `App`, `Model`, `Node`, `NodeFunction`, `node`, `asgi_app`, `serve`, `StoreError` |
| `model` | immutable records for node types and graphs | `Model`, `PortType`, `PortValue`, `value_fits`, `can_link`, `NodeId`, `Port`, `NodeType` (`defaults`, `category`), `PortRef`, `Link`, `PlacedNode` (`values`), `Graph` (with `next_id()`) |
| `edits` | edits and the rules that allow or refuse them | `AddNode`, `AddLink`, `MoveNode`, `DeleteNode`, `DeleteLink`, `SetValue`, `AnyEdit`, `Edit`, `apply_edit`, `EditError`, `EditErrorKind` |
| `ports` | maps a type hint to a port type, and builds input and output ports, checking defaults | `PORT_TYPES`, `NO_DEFAULT`, `port_type`, `input_port`, `output_ports` |
| `nodes` | `@node` and the `Node` base class turn a typed function or a class into a node type, with its defaults and output model | `node` (bare, or `node(category=...)`), `Node` (`run`, `call`; `node_type`, `defaults`, `output_model` built per subclass), `NodeFunction` (`call`, `defaults`, `output_model`), `Registered` (protocol) |
| `engine` | runs a graph in dependency order; an input takes its link, then the node's own value, then the default | `run_graph`, `RunResult`, `NodeOutcome` (`outputs` by port) |
| `protocol` | messages in both directions | `Snapshot` (carries `theme` and `theme_changes`), `EditRequest`, `RunRequest`, `ClientMessage`, `Ack`, `Reject` (and `engine.RunResult`) |
| `session` | the live graph and the node types | `App(node_functions)` (every change and snapshot takes one re-entrant lock), `App.add_node`, `App.add_link`, `App.theme` (the getter returns the base theme's name; the setter takes a name or the path of a theme file, validates and pushes), `App.subscribe`, `App.save`, `App.load`, `App.apply` (silent), `App.execute`, `App.run` (nested: node id, then port), `App.snapshot`, `App.graph` |
| `theme` | the theme names Python offers, and theme files (the look is in the Rust core) | `THEME_NAMES`, `DEFAULT_THEME`, `check_theme`, `ThemeChanges`, `ThemeFile`, `resolve_theme` |
| `store` | save and load a graph as versioned JSON | `FORMAT_VERSION` (2; reads 1 and 2), `StoreError`, `save_graph`, `load_graph` |
| `server` | page, WASM files and `/ws` through starlette and uvicorn | `asgi_app(app, theme)`, `serve(app, port, open_browser, theme)`, `PageServer`, async `reply_to`, `Hub` (open pages, `broadcast`), `Connection` (one page's outbox, `send`, `pump`), `Sender`, `WEB_DIR`, `HOST` |

Sizes in lines: `__init__` 18, `edits` 303, `engine` 237, `model` 150, `nodes` 355, `ports` 95, `protocol` 54, `server` 222, `session` 250, `store` 126, `theme` 101 (1,911 in all).

Import direction: `server` imports `session`, `protocol`, `edits`, `engine`;
`session` imports `nodes`, `protocol`, `edits`, `engine`, `store`, `model`; `store` imports `edits`, `model`; `engine`
imports `nodes`, `model`; `protocol` imports `edits`, `model`; `edits` and
`ports` import `model`; `nodes` imports `model` and `ports`. Nothing imports
`server` except `__init__`. `web/` holds the built page and WASM (generated).

## Rust, `rust/`

| Crate | Purpose | Public interface |
| --- | --- | --- |
| `element-51-core` | records, messages, `address::websocket_url` (the page's websocket sits beside the page), edit rules, the browser's graph copy, box layout; native tests | `PortType`, `Port`, `NodeType`, `PortRef`, `Link`, `PlacedNode`, `Graph` (`next_id`, `is_linked`), `Snapshot` (`theme`, `theme_changes`), `Edit`, `EditRequest`, `Ack`, `Reject`, `RunRequest`, `RunResult` (`outcome_for`, `error_summary`), `NodeOutcome` (`outputs`, `output_text`), `Status`, `Message` (`from_json`, `to_json`), `can_link`, `apply_edit`, `value_fits`, `EditError` (`kind`), `contrast::{parse_hex, luminance, ratio}`, `palette::{parse_link, with_palette}` (a Coolors link as the theme's colours, kept readable at AA), `Theme::contrast_failures`, `fields::{parse_input, format_value, field_label}`, `menu::{Entry, Match, build_menu, search, CardAction, card_actions, clamp_into, flyout_at}` (`CardAction::label`, `edit`), `history::History` (`record`, `undo`, `redo`, `can_undo`, `can_redo`), `Replica` (`new`, `reset` (keeps pending edits that still apply), `graph`, `node_types`, `submit`, `ack`, `reject`, `pending_len`, `next_node_id`), `layout::{Rect (overlaps, grown), node_rect, icon_rect, icon_at, input_anchor, output_anchor, output_row, anchor_of, port_at, node_at, link_at, wire_controls, PortHit, Side}` (every one takes the theme's `Metrics`; a card's rows are its inputs, then its outputs), `theme::{Theme, Colours, Ports, PortSides, PortStyle, PortShape, Metrics, Page, Fonts, THEME_NAMES}` (`builtin`, `with_changes`, `port_style`, `css_variables`; built-in themes are JSON in `element-51-core/themes/`: `antimony`, `antimony-light`, `purple-synthwave`), `view::{View, MIN_ZOOM, MAX_ZOOM}` (`pan_by`, `zoom_at`, `to_screen`, `to_graph`) |
| `element-51-web` | WASM layer: websocket, canvas drawing (`draw`, one `Frame` per frame, which carries the `Theme`; `columns` and `field_box` place the labels and value boxes; the dot grid, cards, wires and ports are drawn in that order), `apply_theme` (CSS custom properties from the theme), the HTML input boxes over the canvas (`sync_fields`), the add-node menu (`fill_menu`, `on_menu_search`, `on_menu_hover` places the flyouts), `mark_ready` (reveals the page after the first snapshot), the card menu on a card's header icon (`open_card_menu`, `on_card_menu_click`), an opaque canvas context at device-pixel size (`fit_canvas`, `json_parse`), undo and redo keys (`step_history`), mouse, wheel, key, menu and Run button handling (a `Gesture` is a link drag, a node move or a pan), error banner | `start()` (runs on load) |

Sizes in lines. `element-51-core/src`: `address` 13, `edit` 266, `fields` 43, `history` 140, `layout` 265, `lib` 59, `menu` 150, `model` 287, `replica` 107, `theme` 427, `view` 70. `element-51-web/src`: `lib` 455, `draw` 497, `menu_ui` 293, `input` 258, `fields_ui` 182.

`element-51-web` modules: `lib` holds the shared `State`, `start()`, the websocket and the edit flow (`submit`, `step_history`, `on_message`) and drawing the canvas; `input` holds `Gesture` and the mouse, wheel and key handlers (and `dragged_graph`, the preview of a node being dragged); `menu_ui` holds the add-node menu, the card menu and their placement; `fields_ui` holds the input boxes over the canvas; `draw` holds the canvas drawing. The three small modules reach `State`, `Shared`, `submit`, `redraw`, `show_error` and `listen` through `crate::`.

Import direction: `element-51-web` imports `element-51-core`. The reverse never
happens. `element-51-web/static/index.html` is copied into `src/element_51/web/`.

## Screens and states

One screen: the editor (`element-51-web/static/index.html` and the canvas). Page
parts, all styled by `var(--...)` from the `Theme`: `#run` (the Run button),
`#menu` (the add-node menu with `#menu-search`, `#menu-tree`, `#menu-results`),
`#app-menu-button` (fixed at the top right) and `#app-menu` (with `#select-theme` and `#try-palette`), `#theme-menu` (the built-in themes, each a `data-theme` button), `#palette-box`, `#card-menu`, `#error` (the banner with Dismiss), and the value boxes in
`#fields`. States: nothing open, the add-node menu (tree, a flyout, search with
results, search with none), the card menu, a run with ok, error and skipped
nodes, the banner. Styles and tokens: the `<style>` block in `index.html`, the
`page` and `colours` sections of `rust/element-51-core/themes/*.json`, and
`Theme::css_variables` in `rust/element-51-core/src/theme.rs`.

## Boundaries between languages

- Python and the browser: JSON messages over the websocket `/ws`. Python side:
    pydantic models in `protocol.py` (`Snapshot`, `EditRequest`, `Ack`, `Reject`,
    `RunRequest`, `RunResult`); browser side: the serde types in
    `element-51-core` (`Message::from_json`, `to_json`). `tests/fixtures/` pins
    both.
- Rust and the page: `element-51-web` is compiled to WASM and bound with
    `wasm-bindgen` and `web_sys` (canvas, DOM, websocket). `element_51_web::start()`
    is the one exported entry. `tools/build_wasm.py` copies the output into
    `src/element_51/web/`.
- Python and Rust share no process: `element-51-core` is also built natively for
    its tests, and Python never loads it.

## Shared fixtures, `tests/fixtures/`

Each is read by a Python test and a Rust test, so the two sides cannot drift:
`snapshot.json` (the snapshot message), `messages.json` (edit request, ack,
reject, run, run result), `link_cases.json` (a graph with a multi-output node, candidate links and the expected verdicts), `edit_cases.json` (move, delete and set-value edits with the expected graphs),
`theme_names.json` (the theme names both sides offer). `snapshot.json` carries a `theme_changes` object, so both sides parse theme files' changes.

## Tests

Python, `tests/`, one file per module: `test_model.py`, `test_edits.py`,
`test_nodes.py`, `test_engine.py`, `test_protocol.py`, `test_session.py`,
`test_store.py`, `test_theme.py`, `test_server.py` (the page, `serve` and the process), `test_server_websocket.py` (a real uvicorn server and the `websockets` client), `test_server_hosts.py` (themes, and the FastAPI example's host with a fake web folder); `server_support.py` holds `running`, `fetch`, `post` and `free_port`, and `conftest.py` the `web_dir` fixture,
`test_examples.py` (loads every example in `examples/` and runs its graph).
`sample_nodes.py` holds node functions shared by several test files.

Rust, `rust/element-51-core/tests/`: `address.rs`, `snapshot.rs`, `messages.rs`, `rules.rs`
(the shared fixtures), `replica.rs`, `layout.rs`, `run.rs`, `view.rs`, `fields.rs`,
`history.rs`, `menu.rs`, `theme.rs` (includes the no-hex-colour scan of the web
crate, the example theme file and the key tables in `docs/themes.md`); unit tests for
`can_link` sit in `src/lib.rs`. `element-51-web` has no automated tests. `theme.rs` and `address.rs` read files outside `rust/` with `include_str!` (`examples/e_look/my_theme.json`, `docs/themes.md`, `element-51-web/static/index.html`).
