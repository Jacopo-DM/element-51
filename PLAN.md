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
- Status: approved 2026-10-19; all six criteria are met, pending the user's
  own try-out. Criteria 1, 2, 3 and 5 by tasks 1 to 4 (tests, and by hand in
  the browser). Criterion 6 by task 6 (round-trip tests; the open page getting
  the loaded graph is a websocket test, not yet looked at by eye). Criterion 4
  by task 10 (a real process: the page URL is opened, Ctrl+C ends it with exit
  0; the browser opener was stubbed in my check, so the user's own browser
  opening is not yet seen).
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
- Status: approved 2026-10-19. Spec approved 2026-10-19 (DESIGN.md, under
  "antimony-theme spec"), with these choices: rows stacked like Antimony, the
  header icon opens a small menu (task 18), the theme is chosen only by
  `serve(theme=...)`. In progress: next is task 11.
- Added: 2026-10-19

## Current state

Phase: executing the task list. Done: walking skeleton, rename to `element-51`,
tasks 1 to 10 (Tier 1 is complete, the graph runs, nodes can have several
outputs and be written as classes, and a reload keeps the graph).
Next step: task 11 (`Theme`), goal `antimony-theme`. The user's own try-out of
goal `skeleton` is still pending.
Nothing is uncommitted except `.vscode/settings.json`, where an editor
extension added a setting; the user chose to leave it. There is no remote, so
nothing is pushed.

## Baseline

Measured 2026-10-19 after task 10, Apple silicon, Python 3.13:

- ruff format, ruff check, mypy, ty: pass.
- pytest: 167 passed, 100% coverage, about 4 s (includes every example).
- cargo fmt, clippy (`-D warnings`), test: pass, 65 tests.
- `tools/build_wasm.py`: builds, about 7 s from cold.
- By hand in the browser pane: the two-output example shows `3 q` and `2 r` on
  the node's rows; the class-form example runs and its menu lists the class
  names; a long text value is trimmed with an ellipsis to fit its row; a node
  added in the browser is still there after a reload, with the run values
  cleared; with `a10_live_push.py` an open page shows each node the script
  adds, with no reload, and Run then gives 2.0 to 6.0; with `a12_big_graph.py`
  a node drags and its move survives a reload, the wheel zooms and empty
  canvas pans, Delete removes the selected node (its links go) or the selected
  wire, and Run then shows `has no value` downstream; with `a13_values.py` typed
  numbers change the Run result, `abc` in a number box is refused with a message
  and the box restored, the boxes follow pan, zoom and drag, and the values and
  a ticked box survive a reload; with `a14_undo_redo.py` Ctrl+Z undid a drag and
  a node delete (the node came back with its links) and Ctrl+Shift+Z redid a
  move, and the result survived a reload.

## Tasks

Tasks 1 to 15 approved 2026-10-19; tasks 1 to 10 are done (removed from the
list below, their detail is in DESIGN.md and git). A task added later needs its
own approval in words. Sizes are estimates by analogy with the skeleton (about
80 lines of Python, 60 of Rust); there is no measured data yet. Every area below is new code, so no existing test pins it; each task
writes its tests first. `element-51-web` is checked by hand in the in-app
browser, and its logic goes into `element-51-core` where `cargo test` covers it.

### Goal `skeleton`, Tier 1: walking skeleton

Done: tasks 1 and 2 (node types reach the browser; edits synced with ack and
rollback).

### Goal `skeleton`, Tier 2 (ordered by value over risk plus cost)

Done: tasks 3 (run the graph), 4 (several outputs, class-form nodes), 5
(reload keeps the graph; no product code was needed) and 6a (Python pushes to
open browsers; added 2026-10-19 at the user's choice, detail in DESIGN.md), 6
(save and load as versioned JSON), 7 (pan, zoom, move, select, delete), 8
(value editing with HTML inputs) 9 (undo and redo) and 10
(`a1_two_nodes.py`, `serve(open_browser=True)`, clean Ctrl+C). Extra task 17, added at the user's request and
approved in words 2026-10-19: rounded node cards and wire ends (done; the radius
is a constant that task 11 moves into the Theme).


### Goal `antimony-theme` (depends on tasks 1, 2 and 7)

11. **`Theme` in `element-51-core`, all drawing and layout through it.**
    Colours, metrics and fonts in one `Theme`; layout and hit-testing take the
    metrics as an argument; CSS custom properties for the HTML parts; a test
    that no colour literal is left in `element-51-web/src`. Criterion 2. Risk
    low. Cost about 400 lines Rust. Verify: native tests (theme parses, layout
    follows the metrics, no colour literals).
12. **The Antimony look.** Stacked rows (inputs, then outputs), right-aligned
    labels, dark value boxes, greyed placeholder on a linked input, header with
    id left, type right and the three-bar icon, dot grid, selection outline,
    input boxes moved onto the new rows. Criterion 1. Risk medium: judged by
    eye. Cost about 600 lines Rust. Verify: you accept it side by side with
    the reference screenshots.
13. **`antimony-light`, `serve(theme=...)`, snapshot `theme`.** Protocol
    version 2; unknown name lists the valid ones; shared theme-name fixture.
    Criterion 3. Risk low. Cost about 100 lines Python, 80 Rust. Verify:
    pytest and native tests.
14. **Categories and the nested menu with search.** `@node(category=...)` and
    the class form set the category (`/` between levels); menu tree and search
    in `element-51-core`, rendered by the web crate. Criterion 4. Risk medium.
    Cost about 350 lines Rust, 60 Python. Verify: native tests for the tree
    and filtering; pytest for `category`.
15. **README credit.** I draft the Antimony and Matt Keeter credit for your
    review; no Antimony asset in the repo. Criterion 5. Risk low. Cost about 5
    lines.
18. **The header icon opens a small menu** (added 2026-10-19 by the user's
    choice while approving the spec). Clicking the three-bar icon on a card
    opens a small popup with one entry, Delete node, in the theme's menu
    style. The user gave "for example Delete node"; I will confirm the entries
    before starting. After task 12. Risk low. Cost about 120 lines Rust.
    Verify: native test for the entry list; by hand.

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

Oldest first.

None.

## Known problems

- `README.md`, `docs/index.md` and the `pyproject.toml` description are still
  template text. The README credit for Antimony is part of goal
  `antimony-theme`.
- `docs/reference.md` autosummary points at `.`, not at `element_51`.
- CI (`.github/workflows/ci.yml`) runs Python checks only. It does not build
  the WASM or run the Rust checks. Without a build, `asgi_app()` fails at
  start (the `web/` folder is missing); the tests avoid this with a fake folder.
- `element-51-web` has no automated test; it is checked by hand in the browser.
- `App` is not thread-safe. A script that edits while the server runs should do
  so from one thread. A browser edit and a script edit that propose the same
  node id race; Python refuses the later one and the browser may show the
  reason, or may not, if the snapshot that makes the edit moot arrives first.
- An empty input box means "use the default", so a text input cannot be set to
  the empty string. There is no script call for setting a value (`app.apply`
  with `SetValue` works but does not push to open pages).
- A long output value is trimmed with an ellipsis on the node. The full value is
  only available from `app.run()`; there is no hover or tooltip yet.
- Port colours are placeholders; the nodes still look plain until tasks 11 and 12.
- A node that returns NaN or infinity as a float is sent as JSON `null` (pydantic's
  default), so the browser would show `null` for it. Checked 2026-10-19 with
  `NodeOutcome(node="n1", status="ok", outputs={"out": float("nan")})`. Not handled.
- The WASM build works around a broken `rust-lld` in the rustup toolchain
  (see CLAUDE.md, things that bite). Not reported upstream.
- `uv build` bundles `src/element_51/web/` only if `tools/build_wasm.py` ran
  first (checked 2026-10-19: the wheel held the WASM). Nothing stops a build
  without it, which would ship a package whose `asgi_app()` fails.
