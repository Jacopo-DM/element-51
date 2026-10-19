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
       `element-51-core`; no colour literal appears in the drawing code.
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

Phase: executing the task list. Done: walking skeleton, rename to `element-51`,
tasks 1 and 2 (the walking skeleton, Tier 1, is complete).
Next step: task 3, run the graph.

## Baseline

Measured 2026-10-19 after task 2, Apple silicon, Python 3.13:

- ruff format, ruff check, mypy, ty: pass.
- pytest: 55 passed, 100% coverage, about 2 s.
- cargo fmt, clippy (`-D warnings`), test: pass, 36 tests.
- `tools/build_wasm.py`: builds, about 7 s from cold.
- By hand in the browser pane: right-click menu adds nodes; a link between
  float ports is accepted and Python keeps it (a reload shows it); a float to
  str drag is refused in the browser with a Dismiss banner; a conflicting node
  id is rolled back with Python's reason.

## Tasks

Tasks 1 to 15 approved 2026-10-19; tasks 1 and 2 are done. Sizes are estimates by analogy with the
skeleton (about 80 lines of Python, 60 of Rust); there is no measured data
yet. Every area below is new code, so no existing test pins it; each task
writes its tests first. `element-51-web` is checked by hand in the in-app
browser, and its logic goes into `element-51-core` where `cargo test` covers it.

### Goal `skeleton`, Tier 1: walking skeleton

### Goal `skeleton`, Tier 2 (ordered by value over risk plus cost)

3. **Run the graph.** Engine in dependency order, per-node error capture,
   `app.run()`, Run button, red node with a Dismiss message, downstream nodes
   skipped. Criterion 3 and the error behaviour. Risk medium. Cost about 250
   lines Python. Verify: pytest on order, error and skip cases.
4. **Pydantic `Node` class and multiple outputs.** One output port per model
   field, in field order, named after the field; a plain return type gives
   `out`. Criteria 1 and 5. Risk low. Cost about 150 lines Python. Verify:
   pytest, including a wrong-type field.
5. **Reload resends the graph.** Python sends a snapshot on every connect.
   Risk low. Cost about 60 lines. Verify: pytest with two connections in turn.
6. **Save and load.** `store.py`, versioned JSON, `app.save` and `app.load`
   pushing a snapshot. Criterion 6. Risk low. Cost about 120 lines. Verify:
   round-trip pytest; an old version number is rejected with a clear error.
7. **Pan, zoom, move, select, delete.** Hit-testing and view maths in
   `element-51-core`, input handling in `element-51-web`. Risk low. Cost about 400
   lines Rust. Verify: native tests for hit-testing and zoom maths; by hand.
8. **Value editing with HTML inputs.** Real inputs placed over the canvas for
   unlinked inputs. Risk medium: overlay positioning under pan and zoom. Cost
   about 200 lines Rust, 60 JS. Verify: by hand, including copy-paste.
9. **Undo and redo.** Reversible command log in `element-51-core`; one step per
   edit, a drag is one step. Risk medium. Depends on 2, 7 and 8. Cost about
   300 lines Rust. Verify: native tests for every command and its inverse.
10. **`examples/a1_two_nodes.py`.** Replaces `a1_serve.py`; Ctrl+C stops the
    server cleanly. Criterion 4. Risk low. Cost about 40 lines. Verify: run it
    by hand and check the exit.

### Goal `antimony-theme` (depends on tasks 1, 2 and 7)

11. **`Theme` in `element-51-core`, all drawing through it.** Colours, sizes and
    fonts in one definition; no colour literal in drawing code. Criterion 2.
    Risk low. Cost about 300 lines Rust. Verify: native test that the theme
    parses; grep for colour literals in `element-51-web`.
12. **The Antimony look.** Card with grey header bar (id left, type right, menu
    icon), right-aligned labels, dark value boxes, greyed placeholder on a
    linked input, type-coloured port squares, S-curve wires, dot grid,
    selection outline. Criterion 1. Risk medium: it is a matter of judgement.
    Cost about 500 lines Rust. Verify: you accept it side by side with the
    reference screenshots.
13. **`antimony-light` and `serve(theme=...)`.** Criterion 3. Risk low. Cost
    about 100 lines Python, 80 Rust. Verify: pytest; unknown name lists valid
    names.
14. **Nested category menu with search.** `@node(category=...)` and the class
    form set the category. Criterion 4. Risk medium. Cost about 350 lines Rust,
    40 Python. Verify: native tests for the menu tree and filtering.
15. **README credit.** I draft the Antimony and Matt Keeter credit for your
    review; no Antimony asset in the repo. Criterion 5. Risk low. Cost about 5
    lines.

### Tier 3: listed, not executed

- CI builds the WASM and runs the Rust checks: outside the acceptance
  criteria, though it closes a known problem; I suggest doing it soon.
- A release guard that fails `uv build` when `web/` is missing: same.
- Automated browser tests for `element-51-web`: large setup, and the logic is
  already in `element-51-core`.
- `wasm-opt` size pass on the 706 KB WASM: no size budget was set.
- Binary channel for large data: a non-goal for now.
- Export the graph as Python code: not in the goals.
- Docs prose and README body: not unasked.

## Open questions

None yet.

## Known problems

- `README.md`, `docs/index.md` and the `pyproject.toml` description are still
  template text. The README credit for Antimony is part of goal
  `antimony-theme`.
- `docs/reference.md` autosummary points at `.`, not at `element_51`.
- CI (`.github/workflows/ci.yml`) runs Python checks only. It does not build
  the WASM or run the Rust checks. Without a build, `asgi_app()` fails at
  start (the `web/` folder is missing); the tests avoid this with a fake folder.
- `element-51-web` has no automated test; it is checked by hand in the browser.
- Python does not push edits to a connected browser. A script's `app.add_node`,
  or a second tab's edits, reach a browser only on reload. Two clients can
  therefore propose the same node id; Python rejects the second and the first
  browser shows the reason, and a reload shows the other node. Not in the task
  list; proposed as a follow-up.
- Links cannot yet be removed, and a node cannot be moved or deleted (task 7).
- The WASM build works around a broken `rust-lld` in the rustup toolchain
  (see CLAUDE.md, things that bite). Not reported upstream.
- `uv build` bundles `src/element_51/web/` only if `tools/build_wasm.py` ran
  first (checked 2026-10-19: the wheel held the WASM). Nothing stops a build
  without it, which would ship a package whose `asgi_app()` fails.
