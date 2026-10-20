# element-51-web

The WASM layer: it opens the websocket, draws the graph on the canvas and
turns pointer, key and menu events into edits for `element-51-core` to apply.
It is checked by hand in the browser, because it has no automated tests, so
logic that can be tested goes into `element-51-core` instead. The interface is
`start()`; the modules are described in `CODEMAP.md`.

- `lib.rs` holds the shared `State` and the edit flow; `input`, `menu_ui` and
    `fields_ui` each handle one kind of event and reach the shared parts through
    `crate::`; `draw` only draws.
- Assumes `static/index.html` has the elements it looks up by id (`canvas`,
    `menu`, `card-menu`, `fields`, `error`, `run`).
- Must never hold a hex colour: the style comes from the `Theme` and from CSS
    custom properties (a native test in `element-51-core` scans these files).
- Must never be imported by `element-51-core`.
