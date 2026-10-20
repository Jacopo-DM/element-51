# element-51-core

The logic of the editor that has no browser in it: the records and messages
that mirror the Python side, the edit rules, the browser's copy of the graph
(`Replica`), box layout and hit testing, undo history, the menu tree and the
`Theme`. It builds natively, so `cargo test` covers all of it. The interface of
each module is in `CODEMAP.md`.

- Assumes the wire format in `protocol.py` and the files in `tests/fixtures/`;
    a change to a message changes both sides and the fixtures.
- Every layout function takes the theme's `Metrics`; nothing here has a fixed
    pixel size.
- Must never depend on `web-sys`, `wasm-bindgen` or `element-51-web`, or touch
    the DOM or the canvas.
- Must never hold a colour literal outside `themes/*.json`.
