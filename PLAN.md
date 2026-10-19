# Plan

## Goals

### skeleton

- Goal: a Python author defines two node types, edits a graph of them in the
  browser, saves and loads it, and Python sees the result, all from one
  command.
- Kind: new behaviour.
- Success measure (each is a test):
    1. `@node` on a typed function and a pydantic `Node` subclass each appear
       as a node type in the browser palette.
    2. In the browser, two nodes can be added and linked; a link between
       incompatible port types (float to str) is refused by the Rust engine.
    3. After editing, `app.graph` in Python holds the same nodes and links,
       and `app.run()` returns the computed value.
    4. `uv run examples/a1_two_nodes.py` starts a local server, opens the
       page, and stops cleanly on Ctrl+C.
    5. A function returning a pydantic model (e.g. `-> DivMod` with fields `q`
       and `r`) shows one output port per field; a link from either port
       works and `app.run()` passes the right value on.
    6. `app.save("g.json")` writes nodes, links and positions with a format
       version field, and `app.load("g.json")` restores the same graph in
       Python and in the browser.
- Status: approved 2026-10-19
- Added: 2026-10-19

### antimony-theme

- Goal: the editor looks and feels like Antimony (Matt Keeter's CAD tool) by
  default, with themes switchable and the credit in the README. Depends on
  `skeleton` (needs a canvas to draw on).
- Kind: new behaviour.
- Success measure (each is a test, except 1, which you judge):
    1. Side by side with the reference screenshots (kept outside the repo),
       you accept the default dark theme: charcoal node cards with a grey
       header bar (id left, type name right, menu icon), right-aligned
       labels with dark value boxes, a greyed placeholder on a linked input,
       type-coloured port squares (orange numbers, olive green for others),
       smooth S-curve wires, a dark dot-grid canvas, a light outline on the
       selected node.
    2. All colours, sizes and fonts come from one `Theme` definition in
       `patchbay-core`; no colour literal appears in the drawing code.
    3. Two themes ship: `antimony` (dark, default) and `antimony-light`
       (light grey canvas). `bay.serve(theme="antimony-light")` shows the
       light one; an unknown name raises an error naming the valid ones.
    4. Right-click opens a nested category menu in the Antimony style; typing
       filters across all categories. `@node(category="Math")` and the
       pydantic class form both set the category.
    5. `README.md` credits Antimony and Matt Keeter with a link, and says the
       look is inspired by it. No Antimony image or code is in the repo.
- Status: approved 2026-10-19
- Added: 2026-10-19

## Current state

Phase: skeleton built, task list not written.
Next step: write the tiered task list for goals `skeleton` and
`antimony-theme` (/zz-02-project-plan, step 6), then approve it.

## Baseline

Measured 2026-10-19 on the skeleton, Apple silicon, Python 3.13:

- ruff format, ruff check, mypy, ty: pass.
- pytest: 4 passed, 100% coverage, about 2 s.
- cargo fmt, clippy (`-D warnings`), test: pass, 4 tests.
- `tools/build_wasm.py`: builds, about 4 s; the page draws in the in-app
  browser with no console errors.

## Tasks

None yet.

## Open questions

None yet.

## Known problems

- `README.md`, `docs/index.md` and the `pyproject.toml` description are still
  template text. The README credit for Antimony is part of goal
  `antimony-theme`.
- `docs/reference.md` autosummary points at `.`, not at `patchbay`.
- CI (`.github/workflows/ci.yml`) runs Python checks only. It does not build
  the WASM or run the Rust checks. Without a build, `asgi_app()` fails at
  start (the `web/` folder is missing); the tests avoid this with a fake folder.
- `patchbay-web` has no automated test; it is checked by hand in the browser.
- The WASM build works around a broken `rust-lld` in the rustup toolchain
  (see CLAUDE.md, things that bite). Not reported upstream.
- `uv build` bundles `src/patchbay/web/` only if `tools/build_wasm.py` ran
  first (checked 2026-10-19: the wheel held the WASM). Nothing stops a build
  without it, which would ship a package whose `asgi_app()` fails.
