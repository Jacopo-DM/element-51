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
The canvas's own keyboard and screen-reader access became the goal `canvas-accessibility` (done 2026-10-21).

### canvas-accessibility: done (2026-10-21)

Commits `12fc4fe..` (task 27) to the commit "Say when a run ends through the status line"
(task 33), branches `canvas-access-27` to `canvas-access-33`. Everything a mouse does on
the canvas now has a keyboard route (the keys are in DESIGN.md, under "canvas-accessibility
spec"), a hidden tree mirrors the graph for screen readers, and a status line says what
happened, including refusals and the end of a run. Measures, numbers and the changes from
the spec are in DESIGN.md (Results, and the spec's task outcomes). Left alone on purpose:
the Tier 3 items below. Not verified: a real screen reader (VoiceOver) and axe-core in the other
two themes for tasks 28 to 33 (no new colour). The redraw median at 200 nodes was measured
on 2026-10-21 (Known problems).

### Experiment goals (approved 2026-10-21)

The user wants to set up a simple experiment visually, for example MNIST
classification: a node that shows a sample image, a node that starts a training
run, and a node that plots the live results. Four goals, in this build order.
They start after `canvas-accessibility`, which is done (the user's
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

### visual-ports: done (2026-10-21)

Commits `527571e..HEAD` on `main` (tasks 34 to 38, tasks `box-summaries` and `code-colours` after
the user tried it). Pictures and series are port kinds, a card shows a picture or a chart, `PlotNode`
grows a row for each link, Save as PNG is in the card menu, the hidden tree describes each display,
and `docs/visual-ports.md` explains it. All seven criteria are met (spec and results in DESIGN.md);
axe-core reports 0 violations in the new states in all three themes. Left open, with reasons, in
Known problems: the 12-picture-card redraw budget (1.0 ms) is missed, a real download and a real
screen reader were not tried, and the narrow window and touch were not checked.

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

### functional-notebooks: done (2026-10-21)

Four notebooks in `tests/functional/` (`nodes_and_running`, `save_and_load`, `themes`,
`pictures_and_series`), the runner `tools/run_notebooks.py`, pre-commit hooks at commit
and at push, and a coverage table with a test (commits `0988e97` to `6b48b32`). Left out
on purpose (user's answer, 2026-10-21): `edits`, `protocol` and `server` (the live-push
and websocket part); `tests/test_server_websocket.py` covers it, and a notebook there
needs a local port. The table says "none yet" for them. Decisions are in DESIGN.md.

### card-polish: done (2026-10-21)

See-through wires over cards, the value beside a linked input's source, and View source
code in the card menu (tasks 39 to 41, commits `1efd344`, `ae687a2` and the task 41
commit). Decisions and numbers are in DESIGN.md (the card-polish section, and Results).
Left out on purpose: renaming a node, which needs a display-name field in the saved file
(useful when many nodes share a type; for after the experiment goals).

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
Next step: none is approved. `visual-ports` and `card-polish` are done, and so are two small follow-ups the user asked for on 2026-10-21 after trying it: `box-summaries` (a picture or series says `28 x 28 grey` or `last 0.05` in its box) and `code-colours` (the source panel is coloured by an own tokenizer, `highlight.rs`). The goals `job-nodes`, `auto-run` and `mnist-experiment` are approved but have no spec: the user runs `/zz-02-project-plan` for the next one (`job-nodes` first, because the Start button the user asked for belongs to it, and the two others build on it). The user's own try-out of the editor and the new features comes first.
Tasks 34 (a picture on a card), 35 (a series and a one-line chart), 36 (rows that grow with links), 37 (Save as PNG) and 38 (accessibility and docs) are done: see DESIGN.md, Results and the spec's task 34
outcome, and try it with `uv run python examples/g_experiments/g1_picture_and_plot.py`
after `uv run python tools/build_wasm.py` (it now also shows a `training` node feeding two
lines to a `Plot` node whose rows grow with its links). `canvas-accessibility` is done, so every new part of
the experiment goals is built to be reachable by keyboard from the start. The user's own try-out (below) is still pending, and so
is trying the keyboard use with VoiceOver (Cmd+F5 on this Mac: Tab to the graph, then the arrow
keys).
`examples/a_basics/a1_hello_world.py` is now a one-node example (the template is
`examples/template.py`); it serves with `uvicorn` on `HOST` and `PORT` set at its top, so a
script can choose the address (`element_51.serve` always uses `127.0.0.1`). On 2026-10-21 the user asked
Claude twice to stop their own example to free port 8765 (`a1_hello_world.py`, pid 2151, then
`g1_picture_and_plot.py`, pid 56292); each was a one-off leave, not a standing one. No example of
theirs was running at the end of that day. Never kill a process of theirs unless they ask, and use
another port for Claude's own checks (8766 to 8769).
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
`main` equals `origin/main` at `b74ce11` (pushed 2026-10-21 at the user's say-so, each time:
tasks 34 to 36 of `visual-ports` and `card-polish`). The GitHub workflows for `1488eb2` (CI,
Rust and WASM, Docs) all finished with success (checked 2026-10-21); the later pushes' runs
were not read.
No other local branch is left: every merged branch was deleted on 2026-10-21 at the
user's say-so.
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

### Goal `canvas-accessibility`: Tier 3, listed and not executed

- Limiting the tree to the focused node's neighbourhood: task 27 stayed under the 50 ms
    budget at 2000 nodes (36 to 42 ms), so it was not needed.
- A recorded VoiceOver session: not available here; steps for the user are in DESIGN.md
    (Results, task 33).
- Multi-select and moving several nodes: no multi-select exists, so it is new behaviour.
- Rebinding shortcuts: a non-goal.

### Goal `visual-ports`: done

Tasks 34 to 38 are done (DESIGN.md, Results and the spec's task outcomes).

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

## Known problems

- Nothing on the page works by touch: panning, zooming and the menus need a mouse
    and a wheel, and a 375 px wide window shows a clipped part of the graph with no
    fit-to-view (`docs/ui-baseline/dark-375-default.jpg`).
- The redraw time at 200 nodes (`tools/bench_page.js`) is 1.3 to 1.6 ms per wheel event (1.54 median on 2026-10-21 after the last visual-ports task) in
    the browser pane today (1024 by 768 CSS pixels at device ratio 2; measured 2026-10-21 on
    `main` before task 34, with it, and at `e90563a` before the accessibility work), not the
    0.21 to 0.22 ms recorded on 2026-10-20 (Baseline). The cause is not found; it is not
    task 34 or the accessibility work. With 12 picture cards in view it is 2.3 to 2.8 ms,
    and drawing the pictures adds nothing measurable, so the spec's budget of 1.0 ms for
    12 picture cards is missed: the cost is in drawing a card (about 0.15 ms each).
    Next step: profile one frame (`columns` measures text, every row fills and clips) and
    decide whether to cut it; needs the user's say-so as a new task. Task 35 (a chart on
    a card) added to the same frame: a 100,000-point chart costs about 1.6 ms a frame.
- A card shows only the first picture of its node (the one first in port name order).
- Not checked on a narrow window or by touch: the source panel, the growing Plot card, the
    chart. Not run in a real screen reader: the panel and the chart label.
- The Run button has no running state: nothing shows while a run is in progress (the
    end of a run is announced to screen readers; the goal `job-nodes` adds the state).
- A node's empty output box says `output`, the same word as the port name.
- The page parts and the theme colours are checked at AA by tests and by axe in
    the built-in browser pane; a scan with a real screen reader was not done.
- Without a build, `asgi_app()` fails at start (the `web/` folder is missing);
    the tests avoid this with a fake folder. The workflows ran on GitHub for the
    first time on 2026-10-21 (see the results in DESIGN.md, Results); every push
    from `527571e` to `3c67a10` passed CI, Docs and Rust and WASM (read 2026-10-21).
    The push `825a608` (the property tests, the first run that installs `hypothesis`): CI passed;
    Docs and Rust and WASM were still running when this was written, so read them next session
    (`gh run list --limit 6`).
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
