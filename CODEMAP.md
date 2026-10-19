# Code map

Generated from the code on 2026-10-19. Generated and downloaded paths are in
the ignore list in CLAUDE.md.

## Entry points

- `element_51.serve(port=8765)`: run the editor on 127.0.0.1 until interrupted.
- `element_51.asgi_app()`: an ASGI app to mount in an existing server.
- `uv run python tools/build_wasm.py`: build the browser layer.
- `examples/a1_serve.py`: calls `element_51.serve()`.
- Browser: `element_51_web::start()` runs when the page loads.

## Python, `src/element_51/`

| Module | Purpose | Public interface |
| --- | --- | --- |
| `__init__` | package surface | `asgi_app`, `serve` |
| `server` | serves `web/` through starlette and uvicorn | `asgi_app()`, `serve(port)`, `WEB_DIR`, `HOST` |

Import direction: `__init__` imports `server`. Nothing imports `__init__`.
`web/` holds the built page and WASM (generated).

## Rust, `rust/`

| Crate | Purpose | Public interface |
| --- | --- | --- |
| `element-51-core` | graph logic, native tests | `PortType`, `can_link(from, to)` |
| `element-51-web` | WASM layer: canvas and page | `start()` (runs on load) |

Import direction: `element-51-web` imports `element-51-core`. The reverse never
happens. `element-51-web/static/index.html` is copied into `src/element_51/web/`.

## Not built yet

Graph model, node decorator, engine, protocol, session, store, theme. The
planned layout is in DESIGN.md under Specs.
