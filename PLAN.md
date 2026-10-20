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
    4. `uv run examples/a_basics/a2_two_nodes.py` starts a local server, opens the
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
    `serve(theme=...)`. In progress: tasks 11 to 15 and 18 are done (the user accepted the look, so
    criteria 1 to 5 are met); the goal waits only for the user's look at task 18.
- Added: 2026-10-19

### restructure: done (2026-10-20)

Commits `71f466e..HEAD` on branch `restructure` (measure `08438ca`, web split
`dd7584d`, test split `2651239`, ports `4009aa0`, and the closing notes). Files
over 400 lines 7 to 5, modules over five imports 2 to 2 (the web `lib.rs` went
from 7 to 6, `session.py` stays at 7 by design), clones 0 to 0; left alone on
purpose, with reasons, in DESIGN.md. Tests unchanged: 220 Python, 122 Rust.

### speed-up

- Goal: the editor and the Python side are faster on three workloads, with
    behaviour unchanged.
- Kind: improvement (behaviour unchanged).
- Success measure (the user chose all three metrics, 2026-10-20): (a) redraw time
    on a 200-node graph, `tools/bench_page.js` run against `tools/bench_page.py`;
    (b) page load time, the same script; (c) `App` build, run and snapshot time
    for a 2000-node chain, `uv run python tools/bench_python.py`. A gain counts
    only if it is larger than the spread under Baseline. Budgets are not set; none
    was asked for.
- Status: in progress (measured, profiled and proposed 2026-10-20; the task list
    below waits for the user's approval in words).
- Added: 2026-10-20

## Current state

Phase: executing the task list. Done: walking skeleton, rename to `element-51`,
the restructure pass (2026-10-20), tasks 1 to 15 and 18 to 26 plus 6a (push), 16 (rename) and 17 (rounded look); the skeleton
goal is complete and the antimony-theme goal has criteria 1 to 5 met (the graph
runs, saves, undoes and has the Antimony look, a light theme, a nested menu and
the README credit).
Next step: the user's own try-out (below), then publishing, which the user does.
The README and docs drafts were reviewed and merged on 2026-10-20.
Waiting on the user: their own try-out of goal
`skeleton` and a look at the menus (examples `d4_menu` and `d5_card_menu`); they
have looked at `a2_two_nodes` and said it looks good.
Nothing is uncommitted in this repo (checked 2026-10-20 after task 26). All
merged branches are deleted (`notes-before-compact-4` too, 2026-10-20); the only
other branch is `main`.
In `~/Functions`, the VS Code profile settings and the project template
settings were committed, the `.bak` files deleted and `*.bak` ignored, and its
merged branches deleted, all at the user's say-so. There is no remote, so nothing
is pushed. The user approves each task start and each branch deletion in one
reply ("Yes, start task N, and delete the merged branch ...").

## Baseline

Measured 2026-10-20 after task 26, Apple silicon, Python 3.13:

- Structure, measured 2026-10-20 after the restructure pass
    (`tools/measure_structure.py`): 80 tracked Python, Rust and HTML files,
    12,688 lines; 5 files over 400 lines (`rust/element-51-web/src/draw.rs` 497,
    `rust/element-51-core/tests/theme.rs` 472, `tools/compress_images.py` 470,
    `rust/element-51-web/src/lib.rs` 455, `rust/element-51-core/src/theme.rs`
    427; before the pass 7, with `lib.rs` at 1147); 2 modules importing more than
    five others (`session.py` 7, web `lib.rs` 6; before 7 and 7); jscpd 5.4.2
    (`--min-lines 8 --min-tokens 70`): 0 clones, 0.00% duplicated, before and
    after.
- Speed baseline, measured 2026-10-20, Apple silicon (arm64), macOS 27.0,
    Python 3.13 under uv, WASM built with the release profile, the page in the
    built-in browser pane:
    - Python, 2000-node chain, 11 runs each (`tools/bench_python.py`): build
        (`add_node` and `add_link`) median 2059 ms, range 1942 to 2825; `run`
        median 490 ms, range 481 to 585; snapshot JSON median 1.8 ms, range 1.8
        to 2.9. Build and run grow four times per doubling of the node count
        (500 nodes: 137 and 36 ms; 4000: 9255 and 2249 ms), so both are quadratic.
    - Page, 200 nodes (10 rows of 20), 11 batches of 100 wheel events
        (`tools/bench_page.js`): redraw median 3.3 to 3.5 ms per event over four
        sets, range 3.26 to 4.06 (the recording work, not the browser's paint).
        With the browser painting between batches the median was 14 ms (range 3
        to 33), which is the figure to watch against a 16.7 ms frame.
    - Page load of a fresh copy of the page to the `ready` class, warm cache, 11
        loads: median 22 ms, range 21 to 47 (an earlier set gave 14.8, range 14
        to 40). Local server; a first, cold load was not measured.
- ruff format, ruff check, mypy, ty: pass.
- pytest: 220 passed, 100% coverage, about 4 s (includes every example).
- cargo fmt, clippy (`-D warnings`), test: pass, 122 tests.
- `tools/build_wasm.py`: builds, about 7 s from cold.
- By hand in the browser pane: the two-output example shows `3 q` and `2 r` on
    the node's rows; the class-form example runs and its menu lists the class
    names; a long text value is trimmed with an ellipsis to fit its row; a node
    added in the browser is still there after a reload, with the run values
    cleared; with `c2_live_push.py` an open page shows each node the script
    adds, with no reload, and Run then gives 2.0 to 6.0; with `d1_big_graph.py`
    a node drags and its move survives a reload, the wheel zooms and empty
    canvas pans, Delete removes the selected node (its links go) or the selected
    wire, and Run then shows `has no value` downstream; with `d2_values.py` typed
    numbers change the Run result, `abc` in a number box is refused with a message
    and the box restored, the boxes follow pan, zoom and drag, and the values and
    a ticked box survive a reload; with `d3_undo_redo.py` Ctrl+Z undid a drag and
    a node delete (the node came back with its links) and Ctrl+Shift+Z redid a
    move, and the result survived a reload; with `e1_light_theme.py` the canvas is
    light grey with a dot grid, the cards stay dark and a selected wire is dark;
    with `d4_menu.py` the right-click menu has Math > Trig > sine flyouts, the
    search box lists matches with their paths, Enter adds the first and Escape
    closes, and the flyouts flip left near the right edge; with `d5_card_menu.py`
    the icon opens a Delete node popup, the click removes the node and its links,
    Cmd+Z restores them, and Escape or a click elsewhere closes it.

## Tasks

Tasks 1 to 15 approved 2026-10-19; tasks 1 to 15 are done (removed from the
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
(value editing with HTML inputs), 9 (undo and redo), 10
(`a2_two_nodes.py`, `serve(open_browser=True)`, clean Ctrl+C). Extra task 17,
added at the user's request and approved in words 2026-10-19: rounded node cards
and wire ends (done; the radius is `card_radius` in the Theme's metrics). Rounded
port markers were tried at the user's request and reverted because they did not
like them. At the user's request the wires were also thickened and then copied
from the Antimony screenshots (colour by source port, thickness, curve), and the
wire curve was changed to stop it hugging a card's edge; details in DESIGN.md.

### Goal `antimony-theme`: done

Tasks 11 (`Theme`), 12 (the Antimony look; the user accepted it), 13
(`antimony-light`, `serve(theme=...)`) and 14 (categories and the nested menu
with search) are done, and 15 (README credit); detail in DESIGN.md. Task 18 (the card menu) is also done.


### Extra task, added by the user 2026-10-20

26. **`App` is thread-safe** (approved by the user 2026-10-20). Done: one
    re-entrant lock around every change and snapshot. Detail in DESIGN.md.
25. **No dark flash on the light theme** (approved by the user 2026-10-20). Done:
    the page stays hidden until the first snapshot has set the theme. Detail in
    DESIGN.md.
24. **The add-node menu scrolls** (approved by the user 2026-10-20). Done: flyouts
    are placed against the window, so the tree can scroll. Detail in DESIGN.md.
23. **CI for the Rust crates and the WASM build** (approved by the user
    2026-10-20). Done: `.github/workflows/rust.yml`. Detail in DESIGN.md.
22. **Page theme keys** (approved by the user 2026-10-20 in housekeeping). Done:
    the `page` section, six colours, docs. Detail in DESIGN.md.
21. **FastAPI and NiceGUI examples** (asked by the user 2026-10-20, with the
    page fix and the `examples` group approved in words). Done: `f1_fastapi_host`, `f2_nicegui_host`,
    the websocket address from the page's path. Detail in DESIGN.md.
20. **Port styles** (from the user's sketch, with their answers). Done: the
    `ports` theme section by type and side, with colour and shape. Detail in
    DESIGN.md.
19. **Theme files** (the user chose the option and asked for the docs). Done:
    `serve(theme="my_theme.json")`, `docs/themes.md`, example `e2_theme_file`. Detail in
    DESIGN.md.

### Goal `speed-up`: profile and proposed tasks (waiting for approval)

Profiles (how to reproduce):

- Python: `uv run python tools/bench_python.py --profile` (cProfile, 2000-node
    chain, build then run then snapshot, 6.6 s under the profiler). Build: 76% in
    `edits._add_link`, of which 3.6 s of 6.6 is pydantic's `__eq__` called 2
    million times by `any(link.target == edit.target ...)`; then `Graph.next_id`
    (0.9 s, parses every node id) and `_add_node`'s id scan. Run: 100% in
    `engine.run_graph`, which scans every link for every node (`run_graph`,
    `_evaluate`, `_order`). The snapshot is linear and takes 1.8 ms.
- Page: no sampling profiler reaches the WASM from here, so I switched parts of
    `draw` off with a throwaway attribute (reverted): per redraw of 200 nodes the
    cards take 91% of the time, wires 12%, port markers 11%, the dot grid 8%. The
    default view shows about 12 of the 200 nodes, and `draw` draws all of them.
- Page load: 22 ms warm for a 706 KB WASM on localhost; no hotspot to chase.

Experiments (throwaway patches, reverted; the numbers are the estimates below):

- Engine, links indexed by target and source: `run` 490 ms to 6.4 ms.
- `_add_link`, targets compared by field: build 2059 ms to 823 ms.
- `draw`, skip cards and port markers whose box is off screen: redraw with
    painting 15.4 ms to 0.7 ms (the default view of the 200-node graph).

Proposed tasks. None is started.

Tier 1:

1. **Done 2026-10-20** (`run` 490 to 5.9 ms, 83 times faster; DESIGN.md Results).
    **Index the links in `engine.py`** (goal `speed-up`, workload (c) run; touches
    100% of `run`; 76 times faster, 490 to 6.4 ms, measured by experiment). Build
    one dictionary of links by target node and one by source node, and pass them
    to `_order`, `run_graph` and `_evaluate`. About 15 lines, no new concept, no
    dependency, risk low. Verify: `tests/test_engine.py`, `test_session.py` and
    `test_examples.py` (65 tests passed with the experiment) and
    `bench_python.py`.
2. **Compare link targets by field in `edits._add_link`** (workload (c) build; 76%
    of build time; 2.5 times faster, 2059 to 823 ms, measured). Replace
    `link.target == edit.target` with a node and port comparison; 3 lines, risk
    low. Verify: `tests/test_edits.py` and `bench_python.py`. The same pydantic
    comparison is in `_delete_link` (`gone not in graph.links`); it runs once per
    delete, not per link, so it is left alone.

Tier 2:

3. **Draw only the nodes on screen** (workload (a); cards and ports are 100% of
    the nodes drawn; 15.4 to 0.7 ms measured when 12 of 200 are visible, no gain
    when all are; it falls to nothing for a graph that fits the window). Add
    `Rect::overlaps` to `element-51-core` with a native test, and skip a card and
    its ports when its rectangle is outside the view in `draw`; wires follow in a
    later step. About 20 lines, risk medium: the web crate has no automated test,
    and the rectangle must include the selection outline so an edge node does not
    flicker. Verify: the native test, `bench_page.js` before and after (same 11
    batches), and by hand in the browser (pan to a node at each edge, zoom out
    until everything is visible, select an edge node).

Tier 3, not executed (reason):

- Cull wires and the dot grid too: they are 12% and 8% of the redraw before task
    3, and I have not measured what remains after it. Revisit after task 3.
- `Graph.next_id` (0.9 s of the 2.2 s that build still takes after task 2): it
    scans every node id. A faster version needs a cached counter, and a counter
    changes which id is reused after the newest node is deleted, which is a
    behaviour change. Leave.
- `_add_node`'s id-in-use scan: the same order as `next_id` and about 14% of
    build; a set of ids needs invalidation rules for no gain a user feels.
- Snapshot JSON: linear, 1.8 ms for 2000 nodes.
- Page load: 22 ms warm. `wasm-opt` could shrink the 706 KB file, but no size
    budget exists and the cold load was not measured; revisit if the user wants
    one.
- The browser's own paint of a visible, large graph (14 ms): task 3 is the part of
    it the page controls.

### Tier 3: listed, not executed

- A release guard that fails `uv build` itself when `web/` is missing: CI now
    checks that the wheel built after `build_wasm.py` holds the WASM, but a local
    `uv build` without the build is still not stopped.
- Automated browser tests for `element-51-web`: large setup, and the logic is
    already in `element-51-core`.
- `wasm-opt` size pass on the 706 KB WASM: no size budget was set.
- Binary channel for large data: a non-goal for now.
- Export the graph as Python code: not in the goals.
- Docs prose and README body: not unasked.

## Open questions

Oldest first.

1. (2026-10-20) Text is still blurry on the user's screen (their answer in
    housekeeping 2026-10-20; zoom speed was not reported as a problem). Three
    fixes were already made (device-pixel canvas, opaque context, Verdana and
    Monaco). Options not yet tried: a larger card font, or snapping text
    positions to whole device pixels. Needs a task approved in words, and a
    screenshot from the user showing what looks blurry (which text, which zoom).

## Known problems

- The README says `uv add element-51`, which is true only after the user
    publishes the package.
- Without a build, `asgi_app()` fails at start (the `web/` folder is missing);
    the tests avoid this with a fake folder. `.github/workflows/rust.yml` has not
    run on GitHub yet, since there is no remote; its commands were run by hand.
- `element-51-web` has no automated test; it is checked by hand in the browser.
- A browser edit and a script edit that propose the same node id still collide
    (the browser picks ids from its own copy); Python refuses the later one and the
    browser may show the reason, or may not, if the snapshot that makes the edit
    moot arrives first.
- One `App` per process: several server workers would each hold their own graph.
- An empty input box means "use the default", so a text input cannot be set to
    the empty string. There is no script call for setting a value (`app.apply`
    with `SetValue` works but does not push to open pages).
- A long output value is trimmed with an ellipsis on the node. The full value is
    only available from `app.run()`; there is no hover or tooltip yet.
- A node that returns NaN or infinity as a float is sent as JSON `null` (pydantic's
    default), so the browser would show `null` for it. Checked 2026-10-19 with
    `NodeOutcome(node="n1", status="ok", outputs={"out": float("nan")})`. Not handled.
- The WASM build works around a broken `rust-lld` in the rustup toolchain
    (see CLAUDE.md, things that bite). Not reported upstream.
- `uv build` bundles `src/element_51/web/` only if `tools/build_wasm.py` ran
    first (checked 2026-10-19: the wheel held the WASM). Nothing stops a build
    without it, which would ship a package whose `asgi_app()` fails.
- A bool input is the browser's own checkbox. A theme can tint it when ticked
    (`accent`), but the unticked box stays white.
- Greyed hints in value boxes use Antimony's sampled `#505050` on `#303030`,
    which is faint. The user may want it lighter (a theme colour, `placeholder`).
- Text is still blurry on the user's screen (see Open questions); the next steps
    would be a larger card font or integer-pixel text positions.
- `a1_hello_world.py` is the project template, not an example of the library: it
    imports numpy and creates `__data__`, and the example tests skip it by name.
- The NiceGUI host example (`f2_nicegui_host.py`) has no automated test, because
    `ui.run_with` changes NiceGUI's global state; the FastAPI one is tested.
- The page keeps a few fixed sizes (1 px borders, the banner and search-box
    paddings, the Run triangle, letter spacing); see DESIGN.md, task 22.
- The undo history is per page: a reload empties it, and edits made by other
    pages or scripts are not in it (see DESIGN.md, undo and redo).
