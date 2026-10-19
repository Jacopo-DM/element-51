# Code map

Generated from the code on 2026-10-19. Generated and downloaded paths are in
the ignore list in CLAUDE.md.

## Entry points

- `patchbay.serve(port=8765)`: run the editor on 127.0.0.1 until interrupted.
- `patchbay.asgi_app()`: an ASGI app to mount in an existing server.
- `uv run python tools/build_wasm.py`: build the browser layer.
- `examples/a1_serve.py`: calls `patchbay.serve()`.
- Browser: `patchbay_web::start()` runs when the page loads.

## Python, `src/patchbay/`

| Module | Purpose | Public interface |
| --- | --- | --- |
| `__init__` | package surface | `asgi_app`, `serve` |
| `server` | serves `web/` through starlette and uvicorn | `asgi_app()`, `serve(port)`, `WEB_DIR`, `HOST` |

Import direction: `__init__` imports `server`. Nothing imports `__init__`.
`web/` holds the built page and WASM (generated).

## Rust, `rust/`

| Crate | Purpose | Public interface |
| --- | --- | --- |
| `patchbay-core` | graph logic, native tests | `PortType`, `can_link(from, to)` |
| `patchbay-web` | WASM layer: canvas and page | `start()` (runs on load) |

Import direction: `patchbay-web` imports `patchbay-core`. The reverse never
happens. `patchbay-web/static/index.html` is copied into `src/patchbay/web/`.

## Not built yet

Graph model, node decorator, engine, protocol, session, store, theme. The
planned layout is in DESIGN.md under Specs.
