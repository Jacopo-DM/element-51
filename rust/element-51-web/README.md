# element-51-web

The WASM layer: it opens the websocket, draws the graph on the canvas and
turns pointer, key and menu events into edits for `element-51-core` to apply.
It is checked by hand in the browser, because it has no automated tests, so
logic that can be tested goes into `element-51-core` instead. The interface is
`start()`; the modules are described in `CODEMAP.md`.

- `lib.rs` holds the shared `State` and `start()`; `edit_ui`, `messages`,
    `run_ui` and `canvas_ui` hold the edit flow, what Python sends, the run and
    the jobs, and the canvas element; `input`, `menu_ui`, `menu_keys`,
    `theme_ui`, `fields_ui`, `a11y_ui` and `tree_keys` each handle one kind of
    event; all reach the shared parts through `crate::`. `draw` and the
    `draw_*` modules only draw.
- Assumes `static/index.html` has the elements it looks up by id (`canvas`,
    `menu`, `card-menu`, `fields`, `error`, `run`).
- Must never hold a hex colour: the style comes from the `Theme` and from CSS
    custom properties (a native test in `element-51-core` scans these files).
- Must never be imported by `element-51-core`.
