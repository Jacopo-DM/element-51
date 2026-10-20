# element_51 (Python package)

Defines nodes (`@node` functions and `Node` classes), holds the one live graph,
runs it, saves and loads it, and serves the page and the websocket. The
interface of each module is in `CODEMAP.md`; this file says what the package
assumes and must never do.

- Assumes the browser layer was built (`tools/build_wasm.py`): `web/` is
    generated and git-ignored, and `asgi_app()` fails at start without it.
- Assumes one `App` per process: `session.App` holds the authoritative graph
    behind one re-entrant lock.
- Other modules import a module only for its public names (listed in
    `CODEMAP.md`). Nothing imports `server` except `__init__`.
- Must never depend on the Rust crates at run time: the two sides share only
    the JSON messages in `protocol.py` and the files in `tests/fixtures/`.
- Must never draw or decide how the editor looks: colours, sizes and fonts live
    in the Rust core's themes. `theme.py` only validates theme files.
