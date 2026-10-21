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

### speed-up: done (2026-10-20)

Commits `971fb48..29d5f57` (benchmarks `30634e3`, engine
`b49e16b`, edits `6fc7183`, drawing `e258928`, and the closing notes). Python
`run` on a 2000-node chain 490 to 5.9 ms, build 2059 to 795 ms; redraw of a
200-node page 3.4 to 0.21 ms; page load unchanged. The numbers, the decisions and
the Tier 3 items left alone on purpose are in DESIGN.md.

### ui-design-pass: done (2026-10-20)

Commits `0eeb4ee..cb6a819` on branch `ui-design-pass` (palette-try is `cb6a819`; baseline `710a82f`, direction `05d8ae7`, tasks 1 to 4 `f6146a2`, `3fcdd25`, `a5b971e`, `3ffabdd`). axe-core
violations 5 rules to 0 in both themes in every state checked; all theme colour
pairs at WCAG 2.2 AA, enforced by a native test; focus and press states on every
page control; the soft modern look in both themes. The numbers and screenshots
are in DESIGN.md (Results). Left alone on purpose, with reasons, in DESIGN.md.
The canvas itself has no keyboard or screen-reader access: open question 2.

### canvas-accessibility

- Goal: a person who cannot use a mouse, or who uses a screen reader, can do
    everything on the editor's canvas that a mouse user can: select, move, add,
    link, delete and edit nodes, pan and zoom, run, and undo, and hear what
    happened. Depends on `skeleton`.
- Kind: new behaviour.
- Success measure (each is a test or a recorded walk-through):
    1. Every canvas action has a keyboard route: select a node or link, move a
        node, add a node (the add-node menu opens by keyboard too), start and
        finish a link from port to port, delete a node or link, set a value in a
        node's box, pan and zoom, open a node's card menu, Run, undo and redo.
        Each is a test where the logic is native (Rust), and a walk-through by me
        in the browser pane for the DOM parts.
    2. The page holds a hidden but real HTML list that mirrors the graph (one
        item per node with its ports and its links, rebuilt on every change); a
        focused item outlines the matching node on the canvas; a test checks the
        list equals the snapshot.
    3. A live status line announces each result: linked, refused (with the
        reason), deleted, moved, added, and the end of a run (counts of ok, error
        and skipped). It never vanishes on a timer for refusals, which also show
        in the banner with Dismiss.
    4. Focus is always visible (a 2 px ring in the theme's `focus` colour, 3:1
        against its background, checked at AA for every theme and palette), and
        the tab order is Run, the node list, the menu button.
    5. axe-core 4.14.0 reports 0 violations in every state of `tools/ui_states.py`
        (a new state per new part), in all three themes; no new colour pair fails
        the native AA test.
    6. Nothing a mouse user does today changes: the Rust and Python suites pass
        unchanged apart from the new tests.
- Non-goals (proposed): touch input and fit-to-view (a separate goal, PLAN.md
    known problems), voice control, a tested run with a real screen reader (I
    cannot run one here; it stays listed as not verified until the user tries
    VoiceOver), keyboard shortcuts that can be rebound, and any change to the look.
- Status: approved 2026-10-21; spec approved 2026-10-21 (DESIGN.md, under
    "canvas-accessibility spec"); the task list (tasks 27 to 33) approved 2026-10-21; tasks 27 to 31 are done. The user's answers (2026-10-21): everything the
    mouse does by keyboard; a hidden list beside the canvas for the screen
    reader; a live status line; checked with axe plus a keyboard walk-through by
    me.
- Added: 2026-10-21

### Experiment goals (approved 2026-10-21)

The user wants to set up a simple experiment visually, for example MNIST
classification: a node that shows a sample image, a node that starts a training
run, and a node that plots the live results. Four goals, in this build order.
They start after `canvas-accessibility` tasks 27 to 33 are done (the user's
choice, 2026-10-21: every new part is then written to be reachable by keyboard
and screen reader from the start; they touch the same files: `draw.rs`,
`a11y_ui.rs`, `index.html`, `layout.rs`). Constraints for all four: no new
runtime dependency (numpy and anything else the example needs go in the `examples`
group, each package approved by name, source and size before it is installed);
the `examples` group is a default group, so it installs on every `uv sync` and in CI,
which keeps it light (numpy only); tests and CI never download; a user's own
data is never touched; every new page part is labelled, operable by keyboard, AA
in all themes, and described in the hidden tree.
Non-goals for all four: a GPU or a distributed run, resuming a run from a
checkpoint, zoom or pan inside a chart, editing images, a dataset catalogue in
the library (datasets live in examples), and running two jobs of one node at once.

### visual-ports

- Goal: nodes can pass pictures and time series, a card shows a picture, a Plot
    node draws live lines, and either can be saved as a PNG. Depends on `skeleton`
    and `antimony-theme`.
- Kind: new behaviour.
- Success measure (each is a test, a pure-function test where the code runs on
    the canvas, or a recorded walk-through in the browser pane):
    1. Two new port kinds, `image` and `series`, exist in Python (`element_51.Image`,
        `element_51.Series`, pydantic models) and in Rust, pinned by shared fixtures;
        a link joins only equal kinds; all three themes give each kind a colour and a
        shape, and the contrast tests pass for every theme and palette.
    2. After a run, a card with an `Image` output shows the picture below its ports,
        scaled up with crisp pixels; a native test covers the picture area's size and
        position.
    3. `element_51.PlotNode` shows each series it receives as a named line on one
        chart with a scale on both axes and the last value as text; it starts with
        one empty series input, linking it adds a fresh empty row below, and every
        linked row is one line named after the port it comes from; native tests
        cover the rows rule and the chart scaling.
    4. A node's series show up as output ports named by the node (`loss`,
        `accuracy`), as any output does.
    5. The card menu has `Save as PNG` on a node with a picture or a chart.
    6. The hidden tree describes each display (for example "Plot, 2 lines, loss
        last 0.31, accuracy last 0.92"); axe-core reports 0 violations in every
        new state in all three themes.
    7. Nothing that works today changes: both suites pass apart from the new tests,
        and redraw time at 200 nodes stays within its recorded spread.
- Status: approved 2026-10-21; spec approved 2026-10-21 (DESIGN.md, under
    "visual-ports spec"); the task list (tasks 34 to 38) approved 2026-10-21. Starts after `canvas-accessibility`
    task 33.
- Added: 2026-10-21

### job-nodes

- Goal: a node that you start and stop by hand, works in the background and
    reports as it goes, so a long task can drive live displays. The class is
    `element_51.JobNode` (the user's choice; names weighed: `ExecutorNode` clashes
    with `concurrent.futures`, `TaskNode` with this repo's tasks). Depends on
    `visual-ports`.
- Kind: new behaviour.
- Success measure (each is a test or a recorded walk-through):
    1. A `JobNode` subclass declares inputs and outputs like `Node` and does its
        work in a method that calls `report(...)` with partial outputs; its card has
        Start and Stop buttons and shows its state (idle, running, finished,
        stopped, failed).
    2. Run and `Run automatically` never start a `JobNode`; Start does, with the
        node's current input values; Start while it is running is refused with a
        reason.
    3. Each report reaches open pages at most 20 times a second; display nodes
        (Plot, Image) after the job redraw on every report; every other node after
        it runs once, when the job ends. Measured: a report reaches the page within
        100 ms on localhost.
    4. Stop ends the job within 1 s; a job that raises shows its error on the card
        and in the banner (with Dismiss) and leaves the editor working; a page
        reload shows the job's current state and data.
    5. The Run button has a running state while a plain run is in progress (this
        clears the known problem in this file).
    6. Python tests with a fake job (no sleeping longer than a few milliseconds);
        the existing 226 Python tests and all Rust tests pass unchanged.
- Status: approved 2026-10-21; no spec yet (it comes after `visual-ports`).
- Added: 2026-10-21

### auto-run

- Goal: a `Run automatically` checkbox next to Run, so results refresh as you edit.
    Independent of the other three goals.
- Kind: new behaviour.
- Success measure:
    1. The checkbox is unticked at every page load and is not saved.
    2. Ticked, a change that alters a result (add or delete a node, link or unlink,
        typing a value, undo or redo, a change pushed from Python) runs the graph
        once, 300 ms after the last change; moving a node does not.
    3. It never starts a `JobNode`.
    4. It is labelled, operable by keyboard, AA in all themes, and axe reports 0
        violations; a native test covers the debounce rule.
- Status: approved 2026-10-21; no spec yet. Independent: it can start any time
    after `canvas-accessibility`.
- Added: 2026-10-21

### mnist-experiment

- Goal: the example that proves the other three: sample an MNIST digit, train a
    small network live, watch the curves. Depends on `visual-ports`, `job-nodes`
    (and uses `auto-run` for the sample node).
- Kind: new behaviour (an example, plus its tests and docs page).
- Success measure:
    1. `examples/g_experiments/g2_mnist.py` builds three linked nodes: `MnistSample`
        (an `index` box; outputs the image and its label), `Train` (a `JobNode`;
        inputs for hidden size, learning rate and epochs; outputs the `loss` and
        `accuracy` series) and a `PlotNode`.
    2. numpy only; no new dependency. The data are fetched once into
        `__data__/mnist/` (git-ignored) from a source the user approves by name,
        with a sha256 check; tests use a tiny fake dataset and never download.
    3. Pressing Start fills the plot live, and the default settings reach at least
        95% test accuracy (measured here, with the time it takes, before the
        example is called done).
    4. A `dataset` choice also offers Fashion-MNIST, with its own checked download.
    5. The docstring lists what to try; a row in `tests/test_examples.py` (with the
        fake data); a docs page.
- Status: approved 2026-10-21; no spec yet (it comes last).
- Added: 2026-10-21

### palette-try: done (2026-10-20)

Pasting a Coolors link in the page (menu button at the top right, Load palette from Coolors link) applies
the palette to the page's colours at once; kept until reload; text readable at AA
whatever the palette. Tests, behaviour and the mapping rule are in DESIGN.md
(Results and Decisions); example `examples/e_look/e3_palette.py`.

## Current state

Phase: executing the task list. Done: walking skeleton, rename to `element-51`,
the restructure, speed-up and ui-design-pass passes and the `palette-try` goal (all 2026-10-20), tasks 1 to 15 and 18 to 26 plus 6a (push), 16 (rename) and 17 (rounded look); the skeleton
goal is complete and the antimony-theme goal has criteria 1 to 5 met (the graph
runs, saves, undoes and has the Antimony look, a light theme, a nested menu and
the README credit).
0.1.0 was published to PyPI by the user on 2026-10-20 (built from commit
`ca74873`, tagged `v0.1.0`). 0.1.1 followed on 2026-10-21 (tagged `v0.1.1`) only to
add the GitHub link to the PyPI page (`[project.urls]`); PyPI lists 0.0.1, 0.1.0
and 0.1.1. A version number cannot be reused, so the next release is 0.1.2 or
0.2.0 (bump `version` in `pyproject.toml`, run `uv run python
tools/check_wheel.py`, then the user publishes).
The third built-in theme `purple-synthwave` (the user's favourite Coolors palette) was
added on 2026-10-21; it ships with the next release (0.1.2 or 0.2.0), together with the
classifiers.
The menu was reworked on 2026-10-21 at the user's request (branch `menu-and-default-theme`):
the menu button sits at the top right, `Try palette` is `Load palette from Coolors link`,
a new `Select theme` entry lists the three built-in themes and switches at once, and
the default theme is now `purple-synthwave`. The user closed their unfinished second
point on 2026-10-21 (nothing more to add; the link box keeps its grey placeholder hint).
Next step: task 32 of `canvas-accessibility` (approved). The experiment goals are
approved (2026-10-21; `visual-ports` has its spec and tasks 34 to 38); they start after
task 33, as the user chose. Open question 4 is answered (the goals are the answer). The user's own try-out
(below) is still pending.
`examples/a_basics/a1_hello_world.py` is now a one-node example (the template is
`examples/template.py`); it serves with `uvicorn` on `HOST` and `PORT` set at its top, so a
script can choose the address (`element_51.serve` always uses `127.0.0.1`). The user has
`a1_hello_world.py` running in their own terminal (pid 2151 on 2026-10-21, started before
the default-theme change): it needs a restart to show purple-synthwave; never kill it.
To try the newest work: `uv run python tools/build_wasm.py`, then
`uv run python examples/e_look/e3_palette.py`, open http://127.0.0.1:8765, press
the menu button at the top right, choose Load palette from Coolors link and paste
`https://coolors.co/acadbc-9b9ece-6665dd-473bf0-000500` (stop any example already
on port 8765 first).
The README and docs drafts were reviewed and merged on 2026-10-20.
Waiting on the user: their own try-out of goal
`skeleton`, a look at the menus (examples `d4_menu` and `d5_card_menu`), the new
page look and the palette box; they have looked at `a2_two_nodes` and said it
looks good. The sharpness question below needs their screenshot.
Nothing is uncommitted in this repo (checked 2026-10-21 after the header change);
`main` equals `origin/main` (`ed1aed5`). No other local branch is left: every merged
branch was deleted on 2026-10-21 at the user's say-so.
In `~/Functions`, the VS Code profile settings and the project template
settings were committed, the `.bak` files deleted and `*.bak` ignored, and its
merged branches deleted, all at the user's say-so. The repo was pushed on
2026-10-21 to https://github.com/Jacopo-DM/element-51 (public; `main` and the tags
`v0.1.0` and `v0.1.1`); every later push needs the user's say-so. New commits in this repo use
GitHub's noreply address (`git config user.email`, set locally); the first 116 keep
the user's own address, which is already public in `ci-group/ariel`, so no rewrite
was done (user's choice). The user approves each task start and each branch deletion in one
reply ("Yes, start task N, and delete the merged branch ...").
On GitHub (2026-10-21, all at the user's say-so): GitHub Pages is on (build type
`workflow`) and the docs are live at https://jacopo-dm.github.io/element-51/; the
repo's About homepage is that address; Dependabot (`.github/dependabot.yml`: weekly
for github-actions, uv and cargo in `/rust`) is running; the workflows `CI`, `Rust
and WASM` and `Docs` ran for the first time and passed (the first Docs run failed
only because Pages was off, then passed after the user enabled it). The README has
seven badges, a header screenshot (the `purple-synthwave` theme) and a Looks section;
its Python badge is a fixed `python 3.13+` image until 0.1.2, because the live one
reads classifiers that the published releases do not have. The next release (0.1.2 or
0.2.0) carries the classifiers, the `purple-synthwave` theme and the README on the
PyPI page; PyPI's page for 0.1.1 shows the old README.

## Baseline

Measured 2026-10-20 after the palette goal, Apple silicon, Python 3.13:

- Structure, measured 2026-10-20 after the restructure pass
    (`tools/measure_structure.py`): 80 tracked Python, Rust and HTML files,
    12,688 lines; 5 files over 400 lines (`rust/element-51-web/src/draw.rs` 497,
    `rust/element-51-core/tests/theme.rs` 472, `tools/compress_images.py` 470,
    `rust/element-51-web/src/lib.rs` 455, `rust/element-51-core/src/theme.rs`
    427; before the pass 7, with `lib.rs` at 1147); 2 modules importing more than
    five others (`session.py` 7, web `lib.rs` 6; before 7 and 7); jscpd 5.4.2
    (`--min-lines 8 --min-tokens 70`): 0 clones, 0.00% duplicated, before and
    after.
    After the design and palette work (audit 2026-10-20): 88 files, 14,138 lines;
    6 files over 400 lines (`rust/element-51-core/tests/theme.rs` 652,
    `rust/element-51-core/src/theme.rs` 525, `rust/element-51-web/src/draw.rs` 517,
    `rust/element-51-web/src/lib.rs` 480, `tools/compress_images.py` 470,
    `rust/element-51-core/tests/layout.rs` 405); modules over five imports still 2.
- Speed, measured 2026-10-20 after the speed-up pass (Apple silicon arm64, macOS
    27.0, Python 3.13 under uv, WASM built with the release profile, the page in
    the built-in browser pane; before in brackets):
    - Python, 2000-node chain, 11 runs each (`tools/bench_python.py`): build
        median 795 ms, range 783 to 889 [2059, 1942 to 2825]; `run` median 5.7 ms,
        range 5.6 to 6.1 [490, 481 to 585]; snapshot JSON median 1.8 ms, range 1.8
        to 2.0 [1.8]. Build is still quadratic (the immutable graph copies its
        tuples on every edit), 4000 nodes was not re-measured.
    - Page, 200 nodes (`tools/bench_page.js`): redraw median 0.21 to 0.22 ms per
        wheel event over four sets, range 0.207 to 0.520 [3.3 to 3.5, 3.26 to
        4.06]; page load median 13.3 ms, range 10.6 to 21.1 [22.3, 20.6 to 46.8;
        14.8 in another set]: the load did not change, and the sets differ by more
        than their own ranges, so load numbers need a same-session before and
        after.
- UI baseline, measured 2026-10-20 (dark and light, `tools/ui_states.py`, axe-core
    4.14.0 with the wcag2a, wcag2aa, wcag21aa, wcag22aa and best-practice tags,
    in the built-in browser pane at 1280 wide). Screenshots in
    `docs/ui-baseline/`: `dark-1280-default`, `-menu`, `-search`, `-card-menu`,
    `-run-and-banner`, `dark-375-default`, `light-1280-search-none-and-banner`.
    - axe violations, rule (impact): nodes. Default state, both themes: `label`
        (critical) 1, `landmark-one-main` (moderate) 1, `page-has-heading-one`
        (moderate) 1, `region` (moderate) 2. After Run, `region` 3. With the menu
        showing "No matching node", `color-contrast` (serious) 1 (4.01:1) and
        `region` 5. Card menu, banner: nothing beyond these.
    - Theme colour pairs below AA (computed from the theme files, WCAG ratio):
        `placeholder` on `field` 1.64:1 (both themes); `text_dim` on
        `header_skipped` 2.88:1; `run_hover_text` on `run_hover` 3.34:1; in
        `antimony-light` the `selected` outline on the canvas 1.14:1. Passing:
        title 6.11, label 7.51, field text 10.00, Run 13.47, card icon 3.01
        (a graphic, needs 3).
    - Raw values in `static/index.html` (81 lines): 0 hex colours; 12 distinct
        pixel values (1, 2, 3, 4, 5, 6, 8, 10, 12, 14, 16, 90), 2 font families
        (both variables), 5 border radii, 2 box shadows, 2 z-indexes, 0
        transitions, 0 media queries. Everything else is `var(--...)` from the
        theme: 33 colours, 4 port types by 2 sides, 25 metrics, 21 page keys and
        6 fonts, in each of the two themes.
    - After the pass (2026-10-20): axe violations 0 in both themes in the default
        state, after Run, the menus and the palette box; theme pairs below AA 0
        (the native test `every_built_in_theme_meets_aa_wherever_it_draws_text_or_graphics`);
        page parts with a `:focus-visible` style: all 8 (was 1). Raw values in
        `index.html`: still 0 hex colours.
    - Components: 1 button kind (Run) plus the banner's Dismiss, 2 menus (add-node
        and card) of one button style, 1 search box, value boxes (text, checkbox).
        No near-duplicates.
    - The harness is semi-automatic: `uv run python tools/ui_states.py
        [--theme antimony-light]` serves a fixed graph on port 8768 and
        `tools/ui_states.js` sets a state; Claude then takes the screenshot in the
        browser pane. The pane scales a 1280 wide viewport to an 800 by 500 image,
        so fine detail is lost; 768 and 1920 wide and the card menu in the light
        theme were not captured (the matrix is cut because the page has one screen
        whose states differ only in the page parts).
- ruff format, ruff check, mypy, ty: pass.
- pytest: 221 passed, 100% coverage, about 8 s (includes every example).
- cargo fmt, clippy (`-D warnings`), test: pass, 153 tests.
- After the `purple-synthwave` theme (2026-10-21): pytest 225 passed at 100% (about 8 s
    alone; 14 to 22 s while the machine was busy), cargo fmt, clippy and 153 tests pass
    (the new theme runs through the existing loops over `THEME_NAMES`), ruff, mypy and ty
    pass, the docs build with `-W` has 0 warnings; axe was not re-run on the new theme.
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

### Goal `canvas-accessibility` (task list approved 2026-10-21)

No test covers the web crate (`element-51-web`), so each task puts its logic in
`element-51-core/src/access.rs` with native tests written first, and checks the
page by `tools/ui_keys.js` and axe-core. Existing behaviour near the change is pinned
by the core suites (`layout.rs`, `history.rs`, `replica.rs`, `menu.rs`); no characterisation
tests are needed first. Estimates are by analogy with the menu work (about 300 lines
of Rust and 60 of CSS and HTML per task of this size); nothing is measured yet.

#### Tier 1: walking skeleton (criteria 2, 4, 5)

Done: task 27 (2026-10-21), the hidden tree, the focus ring and the pan to keep
focus in view. Task 28 is done too (2026-10-21): the status line, Enter or Space to
select, Delete on a node or a linked input port, undo and redo announced. Task 29 is done too (2026-10-21): link by keyboard. Task 30 is done too (2026-10-21): move, pan and zoom by keyboard. Task 31 is done too (2026-10-21): the menus by keyboard. Detail, numbers and the changes from the spec are in DESIGN.md
(Results, and the spec's "Task 27 outcome").

#### Tier 2 (ordered by value over risk plus cost)

32. **Value boxes by keyboard.** Enter on an unlinked input port pans to its box and
    focuses it; Enter in the box returns focus to the port (the box already blurs on
    Enter). Files: `a11y_ui.rs`, `fields_ui.rs`. Criterion 1. Risk: low. Cost: about
    80 lines. Verify: `ui_keys.js`, axe.
33. **Run result announced.** The end of a run (ok, error and skipped counts, the
    first error) goes to the status line; the Run button keeps its look. Files:
    `a11y_ui.rs`, `lib.rs`, `access.rs`. Criterion 3. Risk: low. Cost: about 60 lines.
    Verify: native test of the text, `ui_keys.js`, axe after Run.

#### Tier 3: listed, not executed

- Limiting the tree to the focused node's neighbourhood: only if task 27 misses the
    50 ms budget at 2000 nodes.
- A recorded VoiceOver session: not available here; the user may try it (steps will
    be given when the goal is done).
- Multi-select and moving several nodes: no multi-select exists, so it is new behaviour.
- Rebinding shortcuts: a non-goal.

### Goal `visual-ports` (task list approved 2026-10-21)

All five tasks wait for `canvas-accessibility` task 33 (the user's order). The web
crate has no automated tests, so each task puts its logic in `element-51-core`
(`display.rs`, `layout.rs`, `edit.rs`, `access.rs`) with native tests written first,
and checks the page by hand in the browser pane and with axe-core. The shared
fixtures in `tests/fixtures/` change with every wire record, so both suites
read them. Estimates are by analogy with the menu work (about 300 lines of Rust
and 60 of Python per task of this size); nothing is measured yet. Existing
behaviour near the changes is pinned by `layout.rs`, `rules.rs`, `fields.rs`,
`theme.rs` and `palette.rs` in the core tests and by `test_engine.py`,
`test_nodes.py`, `test_edits.py` in Python; no characterisation tests are needed
first. The example for these tasks is `examples/g_experiments/g1_picture_and_plot.py`
(a new group `g_experiments`; the MNIST example is `g2_mnist.py`).

#### Tier 1: walking skeleton (criteria 1, 2, 4 for images, 6 in part)

34. **A picture on a card, end to end.** What changes: `Image` and `Carried` in
    `visuals.py`; the `image` port kind in Python and Rust (`PortType`, `can_link`,
    `value_fits`, `parse_input` refuses it, no input box); `-> Image` gives one output
    `out`; `NodeType.display` (none or image, set from an `Image` output);
    `NodeOutcome.shown` built by the engine; the card's picture area in `layout.rs`
    and `display.rs` (geometry only); the picture drawn crisp and cached in `draw.rs`;
    `image` colour and shape in all three theme files, `docs/themes.md`,
    `contrast_pairs` and the palette mapping; the hidden tree label "Shows a W by H
    picture"; fixtures; the example. Files: `src/element_51/{visuals,model,ports,
    nodes,engine,protocol}.py`, `rust/element-51-core/src/{model,layout,display,
    theme,fields,access,palette}.rs`, the theme JSONs, `rust/element-51-web/src/
    {draw,fields_ui}.rs`, `tests/fixtures/*.json`, `docs/themes.md`,
    `examples/g_experiments/g1_picture_and_plot.py`, `tests/test_examples.py`. Risk:
    medium (it touches every layer and every theme). Cost: about 600 lines, no new
    dependency. Verify: native and Python tests for each rule (the link rule, the
    size check, the geometry, the AA pairs for six kinds in all palettes, the
    fixtures in both suites), the example by hand in the pane, redraw median at 200
    nodes within its spread and at 12 picture cards in view at 1.0 ms or less.

#### Tier 2 (ordered by value over risk plus cost)

35. **A series and a one-line chart.** `Series` and the `series` kind (NaN as
    `null`, 100,000 points at most); `PlotNode` with one `series_1` input and a
    chart area; `display.rs` ticks, scale, thinning (min and max per bucket) and the
    "last value" text; the chart drawn in `draw.rs` on the card colour with the
    theme's port colours; `shown` carries a chart node's series inputs; the hidden
    tree says "Plot of 1 line, loss last 0.31". Files as for task 34 plus
    `nodes.py` (`PlotNode`). Criteria 1, 3 (one line), 4. Risk: medium. Cost: about
    500 lines. Verify: native tests for ticks, scale, thinning and the NaN gap,
    Python tests for the limits and the engine, the example by hand, a 100,000-point
    chart drawn in 4 ms or less.
36. **Rows that grow with links.** `NodeType.variadic`; the effective rows
    (`layout::effective_inputs`: linked `series_k` in order, then one empty row) used
    by `node_rect`, hit-testing, drawing, the hidden tree and `neighbour`; `edit.rs`
    and `edits.py` accept a link to a `series_k` port; line names after the source
    port (node id prefixed on duplicates), dashes from the sixth line, names at the
    line ends. Files: `layout.rs`, `edit.rs`, `access.rs`, `display.rs`, `draw.rs`,
    `edits.py`, `engine.py`, `nodes.py`, fixtures (`edit_cases.json`,
    `link_cases.json`). Criterion 3. Risk: high (card height now depends on the graph;
    every layout call takes the links). Cost: about 450 lines. Verify: native tests
    for the rows rule (grow, shrink, a gap in the middle, a deleted node), the same
    rules in Python against the shared fixtures, hit-tests on a grown card, the
    existing `layout.rs` tests unchanged.
37. **Save as PNG.** `CardAction::Save` (enabled for a node with a picture or
    chart); the picture or chart drawn at full size on an off-screen canvas and
    downloaded as `<node>-<port>.png`; a native test for the file name and the
    action's availability. Files: `menu.rs`, `menu_ui.rs`, `draw.rs`. Criterion 5.
    Risk: low. Cost: about 150 lines. Verify: native tests, by hand (the downloaded
    file opens and matches the card).
38. **Accessibility and docs.** New `ui_states.py` states (a picture, a chart with
    three lines, a grown Plot node), axe-core in all three themes, `ui_keys.js` rows
    for the new labels, a docs page `docs/visual-ports.md`. Files: `tools/ui_states.py`,
    `tools/ui_keys.js`, `docs/`. Criteria 6, 7. Risk: low. Cost: about 150 lines and
    one page. Verify: axe 0 violations, redraw baseline, `uv-docs build`.

### Tier 3: listed, not executed

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
2. (2026-10-20; now the draft goal `canvas-accessibility`, 2026-10-21) Canvas accessibility: the user asked for WCAG 2.2 AA including the
    canvas (nodes, wires, ports) when the design pass started. It is new behaviour and large
    (keyboard selection, move and link, screen-reader names and live updates).
    Needs `/zz-02-project-plan` to make it a goal with its own spec and approval.
    Priority (user, 2026-10-20): the next work (0.1.0 is published), ahead of the blur
    fix and touch support.

## Known problems

- `.pre-commit-config.yaml` lacks the template's notebook hooks (`run-notebooks`,
    `run-notebooks-all`, a pre-push stage). Left on purpose (2026-10-20): the repo has no
    `tests/functional/` and no `tools/run_notebooks.py`; the pre-push hook would fail
    without the runner. Adopt it together with the first notebooks.
- The canvas has keyboard and screen-reader access only for moving the focus between
    nodes and ports (task 27); select, delete, link, move, menus, value boxes and the run
    result are tasks 28 to 33 of the goal `canvas-accessibility`.
- Nothing on the page works by touch: panning, zooming and the menus need a mouse
    and a wheel, and a 375 px wide window shows a clipped part of the graph with no
    fit-to-view (`docs/ui-baseline/dark-375-default.jpg`).
- The add-node and card menus open only by right-click or a click on the card's
    icon, and the flyouts only by hover; the menu beside Run works by keyboard.
- The Run button has no running state: the page is not told when a run ends.
- A node's empty output box says `output`, the same word as the port name.
- The page parts and the theme colours are checked at AA by tests and by axe in
    the built-in browser pane; a scan with a real screen reader was not done.
- Without a build, `asgi_app()` fails at start (the `web/` folder is missing);
    the tests avoid this with a fake folder. The workflows ran on GitHub for the
    first time on 2026-10-21 (see the results in DESIGN.md, Results).
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
    first (checked 2026-10-19: the wheel held the WASM). `uv build` itself still
    succeeds without it; `tools/check_wheel.py` (CI runs it; run it by hand before
    publishing) fails when the wheel lacks the browser layer.
- A bool input is the browser's own checkbox. A theme can tint it when ticked
    (`accent`), but the unticked box stays white.
- Greyed hints in value boxes use Antimony's sampled `#505050` on `#303030`,
    which is faint. The user may want it lighter (a theme colour, `placeholder`).
- `examples/template.py` is the project template, not an example of the library: it
    imports numpy and creates `__data__`; it sits outside the group folders, so the
    example tests do not load it.
- The NiceGUI host example (`f2_nicegui_host.py`) has no automated test, because
    `ui.run_with` changes NiceGUI's global state; the FastAPI one is tested.
- The page keeps a few fixed sizes (1 px borders, the banner and search-box
    paddings, the Run triangle, letter spacing); see DESIGN.md, task 22.
- The undo history is per page: a reload empties it, and edits made by other
    pages or scripts are not in it (see DESIGN.md, undo and redo).
