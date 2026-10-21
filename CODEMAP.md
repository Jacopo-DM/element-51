# Code map

Generated from the code on 2026-10-21. Generated and downloaded paths are in
the ignore list in CLAUDE.md.

## Entry points

- `element_51.serve(app, port=8765, *, open_browser=False, theme=None)`: run the
    editor for an `App` on 127.0.0.1 until interrupted; Ctrl+C ends it with exit 0.
    `theme` is a built-in name or the path of a theme file.
- `element_51.asgi_app(app, theme=None)`: an ASGI app to mount in an existing
    server.
- `uv run python tools/build_wasm.py`: build the browser layer.
- `tools/ui_states.py`, `tools/ui_states.js`, `tools/ui_axe.js` and `tools/ui_keys.js`: serve a fixed graph (`--graph display`: a picture, its negative, two series and a Plot) and put the page in
    each state, for screenshots and accessibility scans, and walk the hidden tree by keyboard (`uiKeys` on the standard graph, `uiKeysDisplay` on the display one; commands in CLAUDE.md).
- `tools/bench_python.py`, `tools/bench_page.py` (workloads `nodes`, `pictures`, `chart`) and `tools/bench_page.js`: the speed
    benchmarks (see the commands in CLAUDE.md).
- `uv run python tools/run_notebooks.py [file ...]`: run the functional notebooks in `tests/functional/` and print PASS, FAIL or SKIP for each (the shared copy from `~/Functions/_rsync/python/tools/`; commands in CLAUDE.md).
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
    in a FastAPI or NiceGUI host, with a side panel). `g_experiments`:
    `g1_picture_and_plot.py` (nodes that make and pass pictures and series; a picture shows on the card, and a `Plot` node draws the series linked to it, with a row for each link and an empty one after them). Each example has
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
| `__init__` | package surface | `App`, `Image`, `Model`, `Node`, `NodeFunction`, `PlotNode`, `Series`, `node`, `asgi_app`, `serve`, `StoreError` |
| `model` | immutable records for node types and graphs | `Model`, `PortType` (`float`, `int`, `str`, `bool`, `image`, `series`), `PortValue`, `value_fits`, `can_link`, `NodeId`, `Port`, `NodeType` (`defaults`, `category`, `display`: `"none"`, `"image"` or `"chart"`, `variadic`), `Variadic` (`prefix`, `kind`: rows `<prefix>_1`, `<prefix>_2`, ... that grow as they are linked), `variadic_index`, `find_input` (a listed input, or any row of a variadic type), `PortRef`, `Link`, `PlacedNode` (`values`), `Graph` (with `next_id()`) |
| `edits` | edits and the rules that allow or refuse them | `AddNode`, `AddLink`, `MoveNode`, `DeleteNode`, `DeleteLink`, `SetValue`, `AnyEdit`, `Edit`, `apply_edit`, `EditError`, `EditErrorKind` |
| `visuals` | what a node can carry besides the four plain kinds, and show on its card | `Image` (`width`, `height`, `channels` 1 or 3, `pixels` as bytes, at most 256 by 256, a JSON list of integers on the wire), `Series` (`values`, a tuple of floats, at most 100,000; NaN is `null` on the wire), `Shown` (`Image` or `Series`), `Carried` (`PortValue`, `Image` or `Series`), `carries`, `MAX_SIDE`, `MAX_POINTS` |
| `ports` | maps a type hint to a port type, and builds input and output ports, checking defaults (an `Image` or `Series` input has no default; `-> Image` or `-> Series` is one output `out`) | `PORT_TYPES`, `NO_DEFAULT`, `port_type`, `input_port`, `output_ports`, `display_of` |
| `nodes` | `@node` and the `Node` base class turn a typed function or a class into a node type, with its defaults and output model | `node` (bare, or `node(category=...)`), `Node` (`run`, `call`; `node_type`, `defaults`, `output_model` built per subclass), `NodeFunction` (`call`, `defaults`, `output_model`), `Registered` (protocol: `node_type`, `defaults`, `output_model`, `call`, `source_text`), `PlotNode` (a registered class, not a decorated function: type `Plot`, a variadic `series` input starting at `series_1`, no outputs, `display` chart; register the class itself) |
| `engine` | runs a graph in dependency order; an input takes its link, then the node's own value, then the default | `run_graph`, `RunResult`, `NodeOutcome` (`outputs` by port: the plain values; `shown` by port: the pictures a node produced, or the series linked to a chart node by input port; a chart node ignores an unlinked series input and takes every linked row of a variadic type, in row order, and a node without outputs runs and returns nothing) |
| `protocol` | messages in both directions | `Snapshot` (carries `theme` and `theme_changes`), `EditRequest`, `RunRequest`, `SourceRequest` (`node`), `SourceReply` (`node`, then `source` or `reason`; it goes to the asking page only), `ClientMessage`, `Ack`, `Reject` (and `engine.RunResult`) |
| `session` | the live graph and the node types | `App(node_functions)` (every change and snapshot takes one re-entrant lock), `App.add_node`, `App.add_link`, `App.theme` (the getter returns the base theme's name; the setter takes a name or the path of a theme file, validates and pushes), `App.subscribe`, `App.save`, `App.load`, `App.apply` (silent), `App.execute`, `App.run` (nested: node id, then port; a picture is an `Image`), `App.source` (the code of a node's type as a `SourceReply`, or the reason there is none), `App.snapshot`, `App.graph` |
| `theme` | the theme names Python offers, and theme files (the look is in the Rust core) | `THEME_NAMES`, `DEFAULT_THEME`, `check_theme`, `ThemeChanges`, `ThemeFile`, `resolve_theme` |
| `store` | save and load a graph as versioned JSON | `FORMAT_VERSION` (2; reads 1 and 2), `StoreError`, `save_graph`, `load_graph` |
| `server` | page, WASM files and `/ws` through starlette and uvicorn | `asgi_app(app, theme)`, `serve(app, port, open_browser, theme)`, `PageServer`, async `reply_to`, `Hub` (open pages, `broadcast`), `Connection` (one page's outbox, `send`, `pump`), `Sender`, `WEB_DIR`, `HOST` |

Sizes in lines: `__init__` 22, `edits` 312, `engine` 329, `model` 216, `nodes` 457, `ports` 124, `protocol` 79, `server` 233, `session` 284, `store` 126, `theme` 101, `visuals` 137 (2,420 in all).

Import direction: `server` imports `session`, `protocol`, `edits`, `engine`;
`session` imports `nodes`, `protocol`, `edits`, `engine`, `store`, `model`, `visuals`; `store` imports `edits`, `model`; `engine`
imports `nodes`, `model`, `visuals`; `protocol` imports `edits`, `model`; `edits` imports `model`; `visuals` imports `model`; `ports` imports `model` and `visuals`; `nodes` imports `model`, `ports` and `visuals`. Nothing imports
`server` except `__init__`. `web/` holds the built page and WASM (generated).

## Rust, `rust/`

| Crate | Purpose | Public interface |
| --- | --- | --- |
| `element-51-core` | records, messages, `address::websocket_url` (the page's websocket sits beside the page), edit rules, the browser's graph copy, box layout; native tests | `PortType` (with `Image` and `Series`), `Port`, `NodeType` (`display`: `CardDisplay`, none, image or chart), `Image` (`rgba`: the pixels as opaque RGBA for a canvas), `Series` (`values` with `None` for a gap), `Shown` (`Image` or `Series`; `as_image`, `as_series`), `PortRef`, `Link`, `PlacedNode`, `Graph` (`next_id`, `is_linked`), `Snapshot` (`theme`, `theme_changes`), `Edit`, `EditRequest`, `Ack`, `Reject`, `RunRequest`, `RunResult` (`outcome_for`, `error_summary`), `NodeOutcome` (`outputs`, `shown` by port name, `output_text`), `Status`, `Message` (`from_json`, `to_json`), `can_link`, `apply_edit`, `value_fits`, `EditError` (`kind`), `contrast::{parse_hex, luminance, ratio}`, `palette::{parse_link, with_palette}` (a Coolors link as the theme's colours, kept readable at AA), `Theme::contrast_failures`, `source::{SourcePanel, source_panel, source_request_text}` (the panel's title, code or reason, the code as coloured runs (`spans`), and the sentence a screen reader gets), `highlight::{Kind, Span, highlight}` (cuts Python source into runs of keyword, string, comment, number, name or plain text; `Kind::class` is the page's class for a run), `SourceRequest`, `SourceReply`, `menu::CardAction` (`ViewSource`, `SavePng`, `DeleteNode`; `edit` is None for a view or a save; `offered(display)` says whether a card has the entry; `png_file_name`), `fields::{parse_input, format_value, field_label, linked_text, LinkedText, share_room}` (`linked_text` is what a linked input shows: the source's value from the last run and the source name, bracketed after a value; `share_room` gives the source what the value leaves, so the source is cut first), `menu::{Entry, Match, build_menu, search, CardAction, card_actions, clamp_into, flyout_at, step_menu}` (`CardAction::label`, `edit`), `history::History` (`record`, `undo`, `redo`, `can_undo`, `can_redo`), `access::{Direction, NodeItem, PortItem, describe_graph, list_nodes, describe_ports, neighbour, pan_to_show, edit_text, undo_text, refusal_text, selected_text, cannot_delete_text, link_edit, link_started_text, link_cancelled_text, editing_text, linked_input_text, picture_input_text, with_displays, run_text, moved_to, pan_delta, pan_text, zoom_text, ZOOM_STEP}` (`moved_to`, `pan_delta` and `ZOOM_STEP` are the keyboard's step sizes; `link_edit` is the edit both the mouse and the keyboard build; the others are status-line sentences) (what a screen reader is told about the graph, the nearest node in a direction, the pan that brings a box into view; the hidden tree uses `list_nodes` and `describe_ports`), `Replica` (`new`, `reset` (keeps pending edits that still apply), `graph`, `node_types`, `submit`, `ack`, `reject`, `pending_len`, `next_node_id`), `layout::{Rect (overlaps, grown), node_rect, icon_rect, icon_at, input_anchor, output_anchor, output_row, anchor_of, port_at, node_at, link_at, wire_controls, PortHit, Side}` (every one takes the theme's `Metrics`; a card's rows are its inputs, then its outputs), `rows::{variadic_index, find_input, effective_inputs, resolve_type, type_of}` (what a node with rows that grow has in a graph: the listed inputs that are not rows, each linked row in order of row number, and one empty row one past the highest; `resolve_type` is the node type with those as its inputs, borrowed without copying when the type does not grow, and every function that places or hit-tests a card works on it: `layout`, `access`, and in the web crate drawing, the boxes and the menus), `Variadic`, `display::{line_names, picture_side, chart_height, display_height, picture_rect, chart_rect, fit_picture, shown_summary, short_number, nice_ticks, thin, last_value, line_style, CHART_LINE_STYLES, chart_layout, Chart, Tick}` (the picture or chart area of a card with `display` image or chart: below the port rows, inset by the title padding, square for a picture and 0.6 of the card width high for a chart; `node_rect` adds `display_height`, which depends on the type only; `chart_layout` works out the plot rectangle, the ticks and each series as runs of canvas points, thinned to the plot width keeping the lowest and highest point of each part, one scale for every line; `with_displays` writes what a card showed into the hidden tree label), `theme::{Theme, Colours, Ports, PortSides, PortStyle, PortShape, Metrics, Page, Fonts, THEME_NAMES}` (`builtin`, `with_changes`, `port_style`, `css_variables`; built-in themes are JSON in `element-51-core/themes/`: `antimony`, `antimony-light`, `purple-synthwave`), `view::{View, MIN_ZOOM, MAX_ZOOM}` (`pan_by`, `zoom_at`, `to_screen`, `to_graph`) |
| `element-51-web` | WASM layer: websocket, canvas drawing (`draw`, one `Frame` per frame, which carries the `Theme`; `columns` and `field_box` place the labels and value boxes; the dot grid, cards, wires and ports are drawn in that order), `apply_theme` (CSS custom properties from the theme), the HTML input boxes over the canvas (`sync_fields`), the add-node menu (`fill_menu`, `on_menu_search`, `on_menu_hover` places the flyouts), `mark_ready` (reveals the page after the first snapshot), the card menu on a card's header icon (`open_card_menu`, `on_card_menu_click`), an opaque canvas context at device-pixel size (`fit_canvas`, `json_parse`), undo and redo keys (`step_history`), mouse, wheel, key, menu and Run button handling (a `Gesture` is a link drag, a node move or a pan), error banner | `start()` (runs on load) |

Sizes in lines. `element-51-core/src`: `access` 486, `address` 13, `contrast` 33, `display` 348, `edit` 268, `fields` 91, `highlight` 197, `history` 140, `layout` 285, `lib` 67, `menu` 202, `model` 411, `palette` 265, `replica` 107, `rows` 98, `source` 61, `theme` 561, `view` 70. `element-51-web/src`: `a11y_ui` 756, `draw` 804, `fields_ui` 182, `input` 271, `lib` 574, `menu_ui` 805, `pictures` 51, `save_png` 118, `source_ui` 116.

`element-51-web` modules: `lib` holds the shared `State`, `start()`, the websocket and the edit flow (`submit`, `step_history`, `on_message`) and drawing the canvas; `input` holds `Gesture` and the mouse, wheel and key handlers (and `dragged_graph`, the preview of a node being dragged); `menu_ui` holds the add-node menu, the card menu (`fill_card_menu` builds its entries each time it opens, Save as PNG only on a card with a picture or chart) and their placement, and their keyboard use (`on_menu_key` and `on_card_menu_key`: arrows, Home, End, Right and Left for a flyout, Escape; `open_add_menu_from_keyboard` and `open_card_menu_from_keyboard`, which remember the focused item in a `data-return` attribute so `return_focus` can go back to it); `fields_ui` holds the input boxes over the canvas; `a11y_ui` holds the hidden tree `#graph-tree` (one `treeitem` per node in reading order, a node's port items built the first time it is entered, one tab stop, arrow keys, Home, End, PageUp, PageDown, a rebuild whenever the replica changed, Enter or Space to select, Delete to delete a node or the link on an input port, `announce` for the status line, Enter on an output port starts a link and Enter on an input port finishes it, Escape cancels, `pending_preview` gives the canvas the wire to draw, Enter on an unlinked input moves the focus into its value box (`data-original` remembers the text for Escape, `return_from_field` goes back to the port), Shift+arrow moves a node, Alt+arrow pans, `+`, `-` and `0` zoom about the middle of the window, `N` opens the add-node menu, Shift+F10 or the Menu key opens a node's card menu) and the focus handling that pans the view and gives the canvas its ring (`Focus` in `draw.rs`); `draw` holds the canvas drawing (wires are drawn solid under the cards and again over them, clipped to the cards, at the theme's `wire_over_card` opacity; a card with a picture or chart area fills it with the `field` colour; after a run it copies its node's picture over the area with smoothing off, or draws the chart: gridlines and tick labels in `card_border` and `field_text`, one line per linked series in the port colours, and each line's source port name and last value at its end); `source_ui` holds the source panel (`request_source` sends a `SourceRequest` and remembers the element to return to, `show_source` fills the panel from the reply and moves the focus into the code, `close_source`, `on_source_key` for Escape); `save_png` holds Save as PNG (`save_png` renders node `id`'s picture, or draws its chart on an off-screen canvas at 4 times the card size with the same `chart` function the card uses, makes a data URL and clicks a download link named by `png_file_name`; a node with nothing to save gets the banner); `pictures` holds `Pictures`, one off-screen canvas per node that showed a picture, built once when a run result arrives (`from_result`, `of`). The three small modules reach `State`, `Shared`, `submit`, `redraw`, `show_error` and `listen` through `crate::`.

Import direction: `element-51-web` imports `element-51-core`. The reverse never
happens. `element-51-web/static/index.html` is copied into `src/element_51/web/`.

## Screens and states

One screen: the editor (`element-51-web/static/index.html` and the canvas). Page
parts, all styled by `var(--...)` from the `Theme`: `#run` (the Run button), `#graph-tree` (the hidden tree, between Run and the menu button in tab order; `hidden` while the graph is empty), `#status` (the hidden `role="status"` line beside it, written by `announce`),
`#menu` (the add-node menu with `#menu-search`, `#menu-tree`, `#menu-results`),
`#app-menu-button` (fixed at the top right) and `#app-menu` (with `#select-theme` and `#try-palette`), `#theme-menu` (the built-in themes, each a `data-theme` button), `#palette-box`, `#card-menu` (View source code, Delete node), `#source-panel` (a `role="dialog"` with `#source-title`, a Dismiss button `#source-close` and the read-only code in `#source-text`, which takes the focus; Escape or Dismiss closes it), `#error` (the banner with Dismiss), and the value boxes in
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
`snapshot.json` (the snapshot message, with a `picture` node type whose `display` is `image` and a `Plot` node type whose `display` is `chart`), `messages.json` (edit request, ack,
reject, run, run result, a source request and its two replies (code, and no source), a run result with a picture in `shown` and one with a chart, its series holding a `null`), `link_cases.json` (a graph with a multi-output node, two picture nodes and a curve with a Plot (rows that grow), candidate links and the expected verdicts), `edit_cases.json` (move, delete and set-value edits with the expected graphs),
`theme_names.json` (the theme names both sides offer). `snapshot.json` carries a `theme_changes` object, so both sides parse theme files' changes.

## Tests

Python, `tests/`, one file per module: `test_model.py`, `test_edits.py`,
`test_nodes.py`, `test_engine.py`, `test_protocol.py`, `test_session.py`,
`test_store.py`, `test_theme.py`, `test_visuals.py` (the `Image` and `Series` records, the `image` and `series` port kinds, `PlotNode` and the engine's `shown`), `test_server.py` (the page, `serve` and the process), `test_server_websocket.py` (a real uvicorn server and the `websockets` client), `test_server_hosts.py` (themes, and the FastAPI example's host with a fake web folder); `server_support.py` holds `running`, `fetch`, `post` and `free_port`, and `conftest.py` the `web_dir` fixture,
`test_examples.py` (loads every example in `examples/` and runs its graph), `test_functional_coverage.py` (every public module has a row in `tests/functional/README.md`).
`sample_nodes.py` holds node functions shared by several test files.

Functional notebooks, `tests/functional/`: `nodes_and_running.ipynb`, `save_and_load.ipynb`, `themes.ipynb`, `pictures_and_series.ipynb`, and `README.md` with the coverage table (a row per public module of `src/element_51/`: its notebook or "none yet"). They cover the Python side only.

Rust, `rust/element-51-core/tests/`: `address.rs`, `snapshot.rs`, `messages.rs`, `rules.rs`
(the shared fixtures), `replica.rs`, `layout.rs`, `run.rs`, `view.rs`, `fields.rs`,
`history.rs`, `menu.rs`, `source.rs` (the panel words and the shared source messages), `highlight.rs` (Python source cut into coloured runs for the panel), `access.rs`, `rows.rs` (the rows that grow, how a card grows with its links, hit-testing and anchors on a grown card, line names), `display.rs` (the picture and series records, the picture and chart geometry, the number format, ticks, thinning and chart layout, the rules for a picture or series port, the shared fixtures' picture and chart cases), `theme.rs` (includes the no-hex-colour scan of the web
crate, the example theme file and the key tables in `docs/themes.md`); unit tests for
`can_link` sit in `src/lib.rs`. `element-51-web` has no automated tests. `theme.rs` and `address.rs` read files outside `rust/` with `include_str!` (`examples/e_look/my_theme.json`, `docs/themes.md`, `element-51-web/static/index.html`).
