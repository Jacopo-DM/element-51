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

### ui-design-pass

- Goal: the page parts of the editor (Run button, add-node menu, card menu,
    error banner, value boxes) and the theme colours meet WCAG 2.2 AA and look
    polished and consistent, without changing what the editor does.
- Kind: improvement (behaviour unchanged).
- Success measure: axe-core violations on the page parts from 5 rules to 0 in both
    themes (Baseline); every theme text and graphic colour pair at its AA ratio
    (4.5:1 text, 3:1 graphics; Baseline lists the 4 that fail); every interactive
    page part with hover, focus-visible and active states; no screen slower or
    more prone to layout shift (the page has no shift: the canvas and boxes are
    positioned absolutely; bundle size is the WASM 706 KB plus a 3 KB page, to be
    re-measured after each task).
- Status: in progress (Phase 0 and 1 done 2026-10-20; direction C chosen by the
    user 2026-10-20, recorded in DESIGN.md; tasks 1 to 4 approved in words
    2026-10-20).
- Added: 2026-10-20

### palette-try

- Goal: the user can paste a Coolors palette link into the page and see the whole
    editor take its colours at once, to try palettes, without restarting.
- Kind: new behaviour.
- Success measure (each is a test, unless marked by hand):
    1. A hamburger button beside Run opens a small menu with one entry, "Try
        palette", which opens a box for a link (by hand, and a native test that the
        page has the button, the menu and the box).
    2. A Coolors link such as `https://coolors.co/acadbc-9b9ece-6665dd-473bf0-000500`
        gives its colours (2 to 10, any case, with or without `https://`, `www.` or
        `/palette/`); anything else gives an error in the banner that names what is
        wrong (native tests on the parser: good links and five bad ones).
    3. The colours map to theme colours by lightness and colour, with no input
        order needed: darkest to the canvas, the next to cards and menus, mid tones
        to headers and borders, the lightest to the text, the most saturated to the
        accent (a native test pins the result for the example link and for a
        two-colour link).
    4. Every text and graphic pair of the resulting theme meets WCAG AA (4.5 for
        text, 3 for graphics): text and hint colours are nudged lighter or darker
        until they do (a native test over the example, an all-light palette, an
        all-dark one and a one-hue one).
    5. The page applies it at once; a reload shows the theme Python holds, and
        Python is not told (no wire change; a native test that the palette changes
        only the page's `Theme`, a by-hand reload check).
    6. No network request is made: the colours come from the link text (by hand,
        in the network log).
- Status: approved 2026-10-20, to follow `ui-design-pass` tasks 1 to 4. Specified from
    the user's answers of 2026-10-20 (by lightness and colour; a hamburger menu
    beside Run; kept until reload; text adjusted to stay readable).
- Order: after goal `ui-design-pass` tasks 1 to 4, because it reuses their contrast
    test, the accent and focus colours, and the page keys, and edits the same
    files (`index.html`, `theme.rs`, the theme files, `docs/themes.md`).
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

### Goal `ui-design-pass`: audit findings (waiting for the design direction)

Each with the screen and state, the evidence, and who it affects.

1. Bool inputs have no accessible name: `#field-n6-loud` fails axe `label`
    (critical). Default state; screenshot `dark-1280-default`. Every user with a
    bool input and a screen reader.
2. The page has no landmarks or heading (`landmark-one-main`,
    `page-has-heading-one`, `region`): the Run button, boxes and menus sit outside
    any landmark. Every state; every screen-reader user.
3. "No matching node" is 4.01:1 on the menu (axe `color-contrast`, serious).
    Search with no match; `light-1280-search-none-and-banner`. Anyone who searches.
4. Theme colours below AA (see Baseline): hints in value boxes 1.64:1 (the most
    visible: every unlinked input shows one; `dark-1280-default`), skipped nodes'
    title 2.88:1 (`dark-1280-run-and-banner`), the Run button's hover text 3.34:1,
    and the light theme's selection outline 1.14:1 against its canvas.
5. Focus: only `#run` has a `:focus-visible` style. The menu buttons, the card
    menu buttons, the search box, Dismiss and the checkbox use the browser's
    default ring, which the dark canvas hides in places. All keyboard users.
6. States: hover only; no `:active`, no `:disabled`, no transitions, no
    `prefers-reduced-motion` (no motion exists to reduce). The Run button has no
    "running" state; it does not know when a run finishes.
7. The error banner has no `role="alert"`, so a failure is not announced; it
    stays until Dismiss (correct) and joins several messages with "; ". Screenshot
    `dark-1280-run-and-banner`. Every run with an error.
8. The add-node and card menus open only by right-click and the flyouts only by
    `:hover`: no keyboard or touch way to reach them. Behaviour change if fixed.
9. At 375 wide the page shows a clipped part of the graph with no fit-to-view,
    and touch input is not handled (mouse and wheel only);
    `dark-375-default`. An edge screen today.
10. Copy: the empty output placeholder is "output", the same word as the port
    name; "No matching node" is the only empty-state message.
12. Found while checking task 1: the add-node menu's scrolling lists (`#menu-tree`,
    `#menu-results`) fail axe `scrollable-region-focusable` (serious) when a search
    shows no match; to fix in task 3.
11. The canvas (nodes, wires, ports) has no keyboard access or accessible
    description at all. The user chose "AA including the canvas" on 2026-10-20.
    That adds behaviour (keyboard selection, moving and linking, names for
    screen readers), so it is a new goal for `/zz-02-project-plan`, not part of
    this pass; see Open questions.

### Goal `ui-design-pass`: tasks (direction C; approved 2026-10-20)

Tier 1, foundations:

1. **Done 2026-10-20** (DESIGN.md Results; `color-contrast` 1 to 0, all theme pairs at AA).
    **Contrast test and AA colours in both themes** (findings 3 and 4; axe
    `color-contrast` 1 to 0 and four failing pairs to 0). A native test in
    `rust/element-51-core/tests/theme.rs` computes the WCAG ratio for each text
    pair and graphic pair of both themes (text 4.5, graphics 3) and fails below;
    then change `placeholder`, `text_dim`, `run_hover_text`, the light theme's
    `selected`, and the menu colours of direction C. Files: the two theme JSON
    files, `docs/themes.md` if keys change. About 40 lines of test and 10 colour
    values, no dependency. Risk low to medium: hints and skipped titles get
    lighter, so the canvas screenshots change on purpose. Verify: the new test,
    the existing theme tests, axe on both themes, screenshots of default, run and
    search-none.
2. **Visible focus and press states on every page control** (finding 5, 6). One
    focus ring token (`focus` page key, `#7c86f0` dark) used by Run, the menu
    buttons, card menu buttons, search box, Dismiss and the value boxes; `:active`
    for buttons; 120 ms transitions off under `prefers-reduced-motion`. Files:
    `index.html`, `theme.rs` (the key), both themes, `docs/themes.md`. About 25
    lines. Risk low. Verify: native tests (key present, documented, no raw colour
    in the page), by hand Tab through the page in both themes, screenshots of
    focus on each control.
3. **Page structure and names** (findings 1, 2, 7): wrap the page parts in
    `<main>`, a visually hidden `<h1>`, an `aria-label` on the bool checkbox (the
    port name) and on the search box, `role="alert"` on the banner. Files:
    `index.html`, `fields_ui.rs` (the checkbox label). About 15 lines. Risk low;
    no behaviour change. Verify: axe 5 rules to 0 on every state, native test that
    the page has the landmark and the banner role, the existing 126 Rust tests.

Tier 2:

4. **The soft modern look for the page parts** (direction C, finding 6 states):
    rounder Run button in the accent, 8 px menus with the popover shadow, 5 px
    fields, the 8 px banner, the system UI font; new page keys in both themes and
    in `docs/themes.md`, applied through `var(--...)`. Files: `index.html`, both
    themes, `theme.rs`, docs. About 40 lines. Risk medium: the look of every page
    part changes, and a user's theme file that sets page keys keeps working because
    new keys have defaults. Verify: the page-key tests, side-by-side screenshots in
    every state in both themes at 1280 and 375, axe still at 0.

Tier 3, not executed (reason):

- Fit-to-view and touch handling at phone width (finding 9): new behaviour, not a
    style change.
- Keyboard and touch ways to open the menus and flyouts (finding 8): new
    behaviour; part of the canvas accessibility goal (open question 2).
- Custom illustration, decorative animation, a component library: nothing in the
    page needs one.
- A "running" state for the Run button (finding 6): the page does not learn when a
    run ends today, so it needs a protocol change.

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
2. (2026-10-20) Canvas accessibility (finding 11 under goal `ui-design-pass`): the
    user asked for WCAG 2.2 AA including the canvas. It is new behaviour and large
    (keyboard selection, move and link, screen-reader names and live updates).
    Needs `/zz-02-project-plan` to make it a goal with its own spec and approval.
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
