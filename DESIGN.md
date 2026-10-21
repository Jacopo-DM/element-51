# Design

## Specs

### skeleton (approved 2026-10-19)

Product: a Python library for browser dataflow / node-editor GUIs. A Rust
program compiled to WASM draws the canvas and owns the graph while you edit.
Python defines node types, serves the page, holds the authoritative copy, and
runs the nodes.

Users: Python authors who want a node-editor GUI for their own node types.

Acceptance criteria: the six tests listed under goal `skeleton` in PLAN.md.

Non-goals: multi-user live editing, static export without Python, hosted or
remote deployment (binds to 127.0.0.1 only), mobile and touch, a binary
channel for large data (JSON only; the message format carries a version and
leaves room for one), live re-run on every edit.

Behaviour:

- Graph owner: the browser edits first and sends numbered edits. Python replies
    `ack` or `reject` with the request id and a reason. On a rejection the browser rolls the edit
    back and shows a message with a Dismiss button. On connect or reload Python
    sends a full snapshot.
- Port types: float, int, str, bool. An int output may feed a float input. Any
    other pairing is refused by the Rust engine at drag time. A link that would
    make a cycle is refused.
- Ports: inputs come from the function parameters or class fields. A function
    returning a plain type has one output port named `out`. A function returning
    a pydantic model has one output port per field, in field order, named after
    the field.
- Python API: `@node` on a typed function, and a pydantic `Node` subclass.
    Both build the same node type. An `App([...])` holds the node functions and
    the authoritative graph; `app.add_node(fn, x=, y=)` places a node.
- Execution: on demand (`app.run()` or a Run button), in dependency order. A
    node that raises is marked and shows its message with a Dismiss button;
    nodes downstream of it are skipped; `app.run()` raises the same error.
- Undo and redo: every edit (add, delete, move, link, typed value) is one
    reversible step; a node drag is one step. Undo lives in the browser and
    reaches Python as ordinary edits.
- Canvas: pan and zoom, move, select, delete, edit values in nodes through real
    HTML inputs placed over the canvas.
- Palette: right-click menu.
- Server: starlette and uvicorn. `asgi_app(app)` and `serve(app)` are both
    public, so the editor can be mounted in an existing ASGI app. They are module
    functions that take the `App`, so `session` never imports `server`.
- Save and load: `app.save("g.json")` and `app.load("g.json")`, versioned JSON
    holding nodes, links, positions and values.

Architecture:

- Python package `src/element_51/`: `model` (Graph, PlacedNode, Port, Link,
    PortType; pydantic, frozen), `edits`, `nodes`, `engine`, `protocol`,
    `session`, `server`, `store`, and `web/` (built WASM,
    glue JS, index.html; git-ignored output). Import direction as built is in
    CODEMAP.md; the rule is that nothing imports `server` except the package
    `__init__`, and `session` never imports `server`.
- Rust workspace `rust/`: `element-51-core` (graph, port types, link rules,
    message types, undo log, layout, hit-testing, theme; plain `cargo test`) and
    `element-51-web` (wasm-bindgen layer: canvas drawing, DOM inputs, websocket).
    `element-51-web` imports `element-51-core`, never the reverse.
- Build: `tools/build_wasm.py` calls `rustup run stable cargo build --target
    wasm32-unknown-unknown --release`, then `wasm-bindgen`, and copies the output
    into `src/element_51/web/`. The wheel bundles the output; users need no Rust.
- Browser side: Rust draws everything except text boxes and dropdowns, which
    are real HTML elements Rust places and reads. A small loader in JS and no
    framework, no npm.

Testing: Python at 100% coverage. `element-51-core` under native `cargo test`.
`element-51-web` cannot reach 100% automatically; it is checked through the
in-app browser against the running example.

### antimony-theme spec (approved 2026-10-19)

Goal and criteria are in PLAN.md (approved). This is the architecture for them.
The user approved it as written, with three choices: (1) rows are stacked like
Antimony (the first option under "Layout change"); (2) the header icon opens a
small menu when clicked, which is new task 18, after task 12; (3) the theme is
chosen only by `serve(theme=...)`, with no switcher in the page.

**Measured from the user's Antimony screenshots** (sampled pixel colours; the
screenshots stay outside the repo): card body `#202020`, header bar `#505050`,
value box `#303030`, title text `#e0e0e0`, label text `#b0b0b0`, greyed
placeholder text `#505050`, number port and wire `#edc27b`, other ports and
wires (olive) `#94a95f`, selected outline light (`#e0e0e0`), website canvas
`#eeeeee`. Wire colour and thickness are already copied (extra task 17). The
dark app canvas and its dot grid were not in the screenshots; I pick
`#1a1a1a` with dots `#2a2a2a` and the user judges it by eye (criterion 1).

**Layout change the look needs.** In Antimony a card is a header plus one row
per input (port on the left, right-aligned label, dark value box) and then one
row per output (right-aligned label, value box showing `Output` greyed or the
value after a run, port on the right). Today we share rows: an input and an
output sit on the same row. The proposal is to stack them (inputs first, then
outputs), so a card is `header + (inputs + outputs) rows` tall. A linked input
shows its box greyed with the name of what feeds it. This changes
`layout::node_rect`, the anchors and the input-box placement, and their tests.

**Architecture.**

1. `Theme` in `element-51-core` (`theme.rs`, serde, native tests). Three parts:
    `colours` (canvas, dot, card body, header, header error, header skipped,
    value box, title text, label text, placeholder text, value text, selection,
    number port and wire, other port and wire, bool port, allowed, refused),
    `metrics` (card width, header height, row height, card radius, port size,
    wire width, selected wire width, field height, spacing) and `fonts` (title,
    label, value). Layout and hit-testing take the theme's `metrics` as an
    argument instead of reading constants, so a theme can change sizes and the
    tests still pass one in. Criterion 2: no colour literal in drawing code,
    checked by a test that greps `element-51-web/src` for `#` followed by hex.
2. Built-in themes are two JSON files in `element-51-core/themes/`
    (`antimony.json` dark and default, `antimony-light.json`), embedded with
    `include_str!`, parsed by `Theme::builtin(name)`. A native test parses
    both and checks every field is set.
3. The page's HTML parts (input boxes, right-click menu, Run button, error
    banner) read CSS custom properties (`--card`, `--header`, ...) that Rust
    sets from the theme when it starts. `index.html` holds no colour literals
    of its own. The canvas is drawn from the theme directly.
4. Python picks the theme by name: `serve(app, theme="antimony-light")` and
    `asgi_app(app, theme=...)`. Python holds the list of valid names; an unknown
    name raises `ValueError` listing them (criterion 3). The server puts the
    name in every snapshot (`Snapshot.theme`, protocol version 2). A shared
    fixture lists the theme names, run by a Python and a Rust test so the two
    lists cannot differ.
5. Categories: `NodeType.category` is a string, `""` for top level, with `/`
    between levels (`"Math/Trig"`). `@node(category="Math")` and a `category`
    class variable on `Node` set it. It travels in the snapshot and the shared
    fixtures. `node` becomes usable bare or with arguments.
6. Menu logic in `element-51-core` (`menu.rs`): build the tree from the node
    types and filter it for a search text (case-insensitive substring on the
    node name and its whole path; matches show as a flat list with their path).
    Native tests cover the tree, nesting, empty categories and filtering. The
    web crate only renders it: a light grey popup, submenus opening to the
    right on hover, a search box at the top that has focus on open, Enter picks
    the first match, Escape closes.
7. README credit: I draft, you review. It names Antimony and Matt Keeter with
    a link, says the look is inspired by it, and states that no Antimony image
    or code is in the repo.

**Alternatives considered.**

- A global "current theme" that drawing code reads: less plumbing, but hidden
    state, and native tests could not run two themes side by side. Rejected.
- Themes as CSS only (class on the page): the canvas is not styled by CSS, so
    the drawing still needs its own colours. Rejected for the canvas; used for
    the HTML parts through custom properties.
- Themes defined in Python and sent in the snapshot: lets users write their
    own theme, but then the colour check cannot be a native test and a bad theme
    fails in the browser. Not needed by the criteria; can be added later without
    changing the `Theme` type, because it already parses from JSON.
- Keeping shared rows and only restyling: much less work, but the cards would
    not look like Antimony's. Proposed above as the default; the user decides.
- Building the menu in Python and sending HTML: logic would be untestable
    natively and duplicated for search. Rejected.

**Tasks (revised from PLAN.md, same count and order).**

11. `Theme` and metrics in core, all drawing and layout through it, CSS
    variables for the HTML parts, the colour-literal test. About 400 lines
    Rust. Risk low.
12. The Antimony look: stacked rows and value boxes, header with id left and
    type right and the menu icon, dot grid, selection outline, input boxes
    moved onto the new rows. About 600 lines Rust. Risk medium (judged by eye
    against the screenshots).
13. `antimony-light`, `serve(theme=...)`, snapshot `theme`, protocol version 2,
    shared theme-name fixture. About 100 lines Python, 80 Rust. Risk low.
14. Categories and the nested menu with search (items 5 and 6). About 350
    lines Rust, 60 Python. Risk medium.
15. README credit (item 7). About 5 lines. Risk low.

**Verification.** Native tests for theme parsing, metrics-driven layout, menu
tree and filter. pytest for the theme name check, `category`, protocol
version 2. By eye, side by side with the screenshots, for criterion 1.

### antimony-theme reference material (goal approved 2026-10-19)

Reference material for the look and feel, not an approved spec. The user loves
the look of Antimony, the CAD tool by Matt Keeter, and wants it as the default
theme, with the credit in the README.

- Sources, all outside the repo and only on the user's machine: the project
    page https://www.mattkeeter.com/projects/antimony/3/ (saved as
    `~/Downloads/Antimony (19_10_2026 11：04：14).html`), a screencast at
    https://vimeo.com/125111378, and screenshots in `~/Downloads/antimony_pics/`
    (the app window with the right-click menu, a Cube minus Circle example, and a
    large graph on a near-black canvas).
- What the screenshots show, beyond the list under criterion 1 in PLAN.md: the
    app canvas is near-black with a faint dot grid, while the website figures use
    a light grey canvas, so both themes exist (task 13). Every input row has a
    right-aligned label and a dark rounded value box; a linked input shows a
    greyed placeholder such as `Shape [output]`; value ports are orange squares,
    shape ports olive green. The right-click menu is a light grey popup with
    nested categories (`3D`, then `cones`, then `Cone (center)`), with a
    separator before `Export` and `Script`.
- Antimony nodes are Python scripts (`input('x', float)`, `output('shape', ...)`),
    and it has a 3D viewport; neither is in scope here.
- Not to be copied: Antimony's code or images. Only the look is replicated.

### ui-design-pass spec (answers of 2026-10-20)

- Scope: "Also polish the look", not only fix gaps. The look is shown as mock
    renderings of each candidate direction before any code changes (the user's
    request).
- Accessibility target: WCAG 2.2 AA. The user chose to include the canvas; the
    canvas part is a separate goal because it adds behaviour (PLAN.md, open
    question 2). This pass covers the page parts and the theme colours.
- Checker: axe-core 4.14.0, downloaded with approval: `axe-core-4.14.0.tgz`
    (707 KB, sha1 `a9819d39...cfbcf31d` checked against the registry) from
    `registry.npmjs.org` with `curl`, because `npm` fails here on a certificate
    error; only `package/axe.min.js` (652 KB) was extracted, into the scratchpad,
    and served to the page by `python3 -m http.server 8769` for the scan. Nothing
    was added to the repo.
- Feel: the user wants renderings of Antimony, a Blender or Houdini feel, and a
    Figma or Linear feel side by side before choosing.
- Users and density: not stated; the product is a dense node editor for people
    who write Python (see the goal `antimony-theme`).

### canvas-accessibility spec (approved 2026-10-21)

- Users: a person who uses only a keyboard, and a person who uses a screen reader
    (VoiceOver on macOS is the one to try). Target: WCAG 2.2 AA for the canvas.
- Answers of 2026-10-21: everything the mouse does is available by keyboard; the
    screen reader reads a hidden list that mirrors the graph; a live status line
    announces results; checked with axe-core plus a keyboard walk-through by Claude
    (a real screen reader is not available here, so it stays "not verified" until the
    user tries VoiceOver).
- Acceptance criteria and non-goals: PLAN.md, goal `canvas-accessibility`.
- Integration points: all hit-testing and layout is in `element-51-core::layout`
    (native-tested); the page's key handling is `on_key_down` in
    `element-51-web/src/input.rs` (Delete, Backspace, Escape, Ctrl+Z, Ctrl+Y today,
    and it returns early when the target is an input box); the page receives
    `RunResult` in `on_message` in `lib.rs` (it only redraws today); the menus are
    `menu_ui.rs` (hover and right-click only); the theme already has a `focus`
    colour at AA and `selection_width`. Must not regress: every mouse gesture, the
    existing key shortcuts, redraw time (200-node baseline in Results).
- Architecture:
    1. `element-51-core/src/access.rs` (new, pure, native-tested): `describe_node`
        and `describe_port` (text such as "n1, hello. Output out, float, linked to n3
        value"), `neighbour` (the nearest node in a direction), `LinkMode` (idle, or
        "from port X"; `choose(port)` gives an `Edit` or the refusal reason; built as
        `link_edit` and a `pending_link` field, see the outcome below), the
        status texts (linked, refused with its reason, deleted, moved, added, run
        result), and the move step (the theme's `grid_step`, 24; four times that with
        Alt held).
    2. `element-51-web/src/a11y_ui.rs` (new): a visually hidden tree (`role="tree"`,
        a `treeitem` per node with its ports as children), rebuilt from the replica after
        every change; one tab stop (roving `tabindex`), arrows move inside it; a
        `role="status"` live region; the canvas draws a 2 px ring in the theme's
        `focus` colour on the focused node or port, and the view pans to keep it in
        view (WCAG 2.4.7 and 2.4.11).
    3. Menus: arrow-key navigation in `#menu`, its flyouts and `#card-menu` (Down and
        Up move, Right opens a flyout, Left closes it, Enter picks, Escape closes).
- Keys (inside the tree; Delete, Backspace, Escape, Ctrl+Z, Ctrl+Y keep their meaning):
    Arrows move to the nearest node in that direction; Right enters a node's ports
    (Up and Down move between ports, Left goes back to the node); Enter or Space
    selects a node, and on an output port starts a link, on an input port with a link
    pending finishes it, on an input port with none focuses its value box (the view
    pans to it first and the status says so); Shift+Arrows move the node by 24 (Alt+
    Shift+Arrows by 96); Delete deletes a node, or the link on a focused input port;
    Escape closes menus, clears the selection and cancels a pending link; Shift+F10
    or the Menu key opens the node's card menu; `N` opens the add-node menu at the
    canvas centre; `+`, `-` and `0` zoom in, out and reset; Alt+Arrows pan.
- Status text names ports as the cards do (`out`, `value`): the user accepted the
    Python names by approving the table.
- Measurements: native tests cover `access.rs` fully and check that the described
    list equals the snapshot; `tools/ui_keys.js` (new, like `ui_states.js`) dispatches
    key sequences in the browser pane and checks the live region and the graph; axe-core
    4.14.0 on every new state in all three themes; redraw median at 200 nodes
    (`tools/bench_page.js`) stays within its recorded spread; building the hidden tree
    takes 50 ms or less per edit on the 2000-node chain, otherwise the tree is limited
    to the focused node's neighbourhood.

- Task 27 outcome (2026-10-21), changes from the spec above: (a) Right on a node
    enters its ports, as the key table says, so the node to the right is reached with
    PageDown (reading order) and Home, End and PageUp go with it; Up, Down and Left on a
    node are spatial. (b) The focus is announced by the browser itself, because the
    tree items take real DOM focus and carry their `aria-label`; a status line that
    also said what is focused would be read twice, so the `role="status"` line moved to
    task 28, where it carries results. (c) The focus ring is two-tone (the canvas colour
    under the theme's `focus` colour, 2 px wide in screen pixels at any zoom), so it
    reads on a card as well as on the canvas and needed no new colour pair; the
    existing "focus on canvas" pair covers it for every theme and palette. (d) A node's
    port items are built the first time the keyboard enters it, not in the rebuild: at
    2000 nodes the full tree (5,900 items) made an edit take 70 to 77 ms, the lazy one
    36 to 42 ms. (e) The tree is rebuilt only when the replica changed (a flag set on
    snapshot, submit and reject), not compared on every redraw: comparing the graph on
    each redraw cost about 0.3 ms of a 1.4 ms redraw at 200 nodes. (f) The tree is
    `hidden` while the graph is empty, because axe flags a `role="tree"` with no items.
    (g) Focus events only fire while the page's window has focus, so the browser-pane
    walk-through needs a click on the page first (CLAUDE.md, Commands).
- Tasks 28 to 33 outcome (2026-10-21), changes from the spec above: (a) There is no
    `LinkMode` type: a link in progress is `State.pending_link` (the output port it
    started from), and `access::link_edit` builds the `AddLink` edit for both the mouse
    and the keyboard, so both meet the same rules and refusal text (a native test runs
    every case of the shared `link_cases.json` through it). (b) Enter on an input port
    with no link pending opens its value box, so the "No link started" sentence was
    dropped; Enter on a linked input says it has no box. (c) Delete acts on the focused
    item, not on the mouse selection, and moves the focus to a neighbour. (d) The status
    line says what an edit, an undo, a redo, a refusal, a pan, a zoom, a menu opening
    and a run did; focus changes are not said (the focused item speaks for itself); undo
    and redo say only "Undid the last change" or "Redid the last change". (e) Menus:
    lists stop at their ends (no wrap), Up from the first add-node item goes to the
    search box, Tab closes a menu, `N` and Shift+F10 or the Menu key work only inside the
    hidden tree, and a keyboard-opened menu remembers the item to return to in a
    `data-return` attribute (a mouse-opened one leaves none, so it never moves focus).
    (f) Shift+Arrow moves a node one grid step (24) and Alt+Shift+Arrow four, each press
    one undo step; Alt+Arrow pans 48 px toward the key; `+` or `=` zoom in by 1.25, `-`
    or `_` out, `0` back to 100 percent, about the middle of the window. (g) A value box
    opened by the keyboard remembers its text in `data-original`, so Escape puts it back
    and Enter or Escape returns the focus to the port; a box the mouse focused behaves as
    before. (h) The run sentence counts ok, error and skipped nodes and names the first
    error only (the banner still lists them all); nothing is said when a run starts.
    (i) `tools/ui_states.py` was not given new states: `tools/ui_keys.js` reaches the open
    menus, the focused value box and the pending link, and axe was run on them. (j) axe-core
    was run in all three themes only for task 27's states; for tasks 28 to 33 it ran in the
    default theme, because no colour or element type changed.

### visual-ports spec (approved 2026-10-21)

Part of the experiment goals (PLAN.md): the user wants to set up an experiment such
as MNIST classification visually, with a sample image, a training run and a live
plot. This spec covers `visual-ports` only; `job-nodes`, `auto-run` and
`mnist-experiment` get their own specs when their turn comes.

- Users: Python authors who write nodes that produce pictures or time series and
    want to see them on the card. Non-goals: PLAN.md, "Experiment goals".
- Existing behaviour that must not change: the four port kinds, the int to float
    rule, typed value boxes, the saved graph format (`FORMAT_VERSION` stays 2, the
    snapshot version stays 2, so old files load unchanged), redraw time at 200 nodes
    when no card shows a picture (median 0.21 to 0.22 ms), and every current test
    (only fixtures gain new fields that have defaults).
- Data model:
    * `Image` (pydantic, `visuals.py`): `width`, `height`, `channels` (1 or 3) and
        `pixels` (bytes), checked as `width * height * channels`, at most 256 by 256.
        On the wire the pixels are a JSON list of integers (no base64 dependency or
        hand-written decoder; about 3 kB for 28 by 28).
    * `Series` (pydantic): `values`, a tuple of floats, the x axis is the index, at
        most 100,000 points (a node that returns more fails with a clear error). NaN
        goes out as `null` and is drawn as a gap, so a diverging loss does not crash a
        node.
    * `PortType` gains `image` and `series` in Python and Rust; `can_link` joins only
        equal kinds for them. `-> Image` and `-> Series` give one output `out` (the
        "model with fields" rule is skipped for these two types). Inputs of these
        kinds have no default and no typed value, so they get no box and can only be
        linked; `parse_input` refuses them. A new alias `Carried` is `PortValue`
        plus `Image` and `Series`; `PortValue` stays the four plain kinds.
    * `NodeType` gains `display` (none, image or chart; a function returning `Image`
        gets `image` by itself) and `variadic` (a prefix and a kind; `PlotNode` has
        `series` and `series`, so its inputs are `series_1`, `series_2`, ...).
    * `NodeOutcome` gains `shown`: the values the card displays, by port name (an
        image node's image outputs, a chart node's series inputs). The page reads only
        `shown`.
- Behaviour:
    * A `PlotNode` shows one row per linked `series_k`, in order of `k`, plus one
        empty row named with the next unused `k`. Removing a middle link removes its
        row and keeps the other port names. `edit.rs` and `edits.py` both accept a
        link to such a port.
    * A card's height depends on its node type, not on its data, so it does not
        jump when results arrive: a square picture area (node width minus the inset)
        for `image`, a chart area of 0.85 of the node width for `chart` (0.6 until 2026-10-21).
    * The picture is drawn with smoothing off (crisp pixels) and cached per result.
    * The chart: one line per linked row on the card colour, a scale on both axes,
        a legend row above the plot (a swatch of the line's colour and dash and its
        name, one equal slot a line, the name cut to fit; the last value is in the
        input's box, not on the chart); a line is named after its source port, and when two
        share a name each gets its node id (`n3 loss`); line colours reuse the
        theme's port colours (no new theme colour key), the sixth and later lines
        cycle with a dash, and the legend carries the dash, so colour is never the
        only cue; the grid is the card border colour at 0.4 alpha and the two axes
        are drawn at full strength, with no new theme colour (decided 2026-10-21, the
        user asked for a more professional plot: axes and grid, lines and legend, a
        bigger plot; a hover readout was offered and not wanted); a long series is thinned to about twice the chart's width in
        pixels with min and max per bucket.
    * `Save as PNG` is a card-menu entry on a node with a picture or chart. It draws
        at full size on an off-screen canvas and downloads through the browser as
        `<node>-<port>.png` (the file lands in the Downloads folder; Python is not
        involved).
        Task 37 outcome: the entry is built each time the card menu opens, so a card
        with no picture or chart does not have it (`CardAction::offered`). A picture
        is saved at its own size (28 by 28 stays 28 by 28); a chart is drawn by the
        same `chart` function as the card, at 4 times the card's size, on the `field`
        colour. The file is made as a data URL and a download link is clicked; the
        browser decides where it goes and whether to ask. File names are
        `n1-out.png` (node and port) and `n5-chart.png`, with anything but letters,
        digits, `_` and `-` turned to `_`. Before a run (or after the node failed) the
        entry says so in the banner, with Dismiss, rather than saving an empty
        picture.
    * The hidden tree labels a display ("Shows a 28 by 28 picture", "Plot of 2
        lines, loss last 0.31, accuracy last 0.92") and lists the effective rows.
    * The `ports` section of the three theme files gets `image` and `series` colours
        and shapes; `docs/themes.md`, `Theme::contrast_pairs`, the palette mapping and
        its tests extend to six kinds.
- Modules: Python, new `visuals.py`; changed `model.py`, `ports.py`, `nodes.py` (a
    registered `PlotNode` object), `engine.py` (collects the variadic inputs and
    builds `shown`), `edits.py`. Rust core, new `display.rs` (ticks, thinning, line
    names, picture geometry); changed `model.rs`, `edit.rs`, `layout.rs` (effective rows,
    display height), `theme.rs` and the three JSONs, `menu.rs` (a Save action),
    `fields.rs`, `access.rs`. Web: `draw.rs` (picture and chart, cache), `menu_ui.rs`
    (Save), `fields_ui.rs` (no box for these kinds).
- Measurements at build time: the redraw median with 12 picture cards in view
    (budget 1.0 ms or less) and the draw time of a 100,000-point chart (budget 4 ms
    or less), both next to the 200-node baseline.
- Open points with their defaults: grey and RGB only (no alpha); a node returning
    more than 100,000 points fails (live growth belongs to `job-nodes`, which will
    send partial updates instead of whole series); images up to 256 by 256.

- Task 34 outcome (2026-10-21), changes from the spec above: (a) Only `image` was added
    to `PortType`; `series`, `Series` and the `series` theme colour come with task 35,
    so the AA pairs cover five kinds now and six after task 35. (b) `NodeOutcome.outputs`
    keeps the plain values and `shown` holds the pictures, so a picture is sent once
    and the output box beside a picture port shows the placeholder `output`;
    `App.run()` merges both and returns `Carried` values, and `element_51.Image` is
    exported. `carries(value, kind)` in `visuals.py` replaces `value_fits` wherever an
    `Image` can occur; `value_fits` stays the four plain kinds (it cannot import
    `visuals`, which imports `model`). (c) `Image.pixels` is `bytes` in Python (a
    before-validator reads the JSON list, a JSON serializer writes it) and `Vec<u8>` in
    Rust. (d) The picture area is inset by `title_pad`, so no new metric exists; a card
    is `rows + side + title_pad` tall (`display::display_height`, by type only); the
    area is filled with the `field` colour, so no new colour exists, and the picture
    is fitted into it keeping its shape (`fit_picture`). A card shows the first picture
    of its node, in port name order. (e) Pictures are drawn once onto off-screen
    canvases when a run result arrives (`pictures.rs`) and dropped with the result;
    a card draws its picture only while its node's outcome is ok. (f) The hidden
    tree says "Shows a 28 by 28 picture." only after a run (`access::with_pictures`,
    and the tree is rebuilt when a result arrives); Enter on an unlinked picture input
    says it can only be linked (`picture_input_text`). (g) The picture colour is
    `#d98cc4` and the shape a diamond, in all three themes (the cards are dark in all
    three, so one colour passes 3:1 on each, and the shape is a second cue). (h) The
    shared fixtures: `snapshot.json` has a `picture` node type, `link_cases.json` a
    `picture` and an `invert` node (so the next node id there is `n8`) and three
    cases, `messages.json` a run result with a picture. Node types carry `display` and
    outcomes `shown` on the wire, both with defaults on the Rust side, so a message
    without them still reads. (i) The two measurement budgets are not met or not
    reproducible in this pane: see Results, 2026-10-21, task 34.

- Task 35 outcome (2026-10-21), changes from the spec above: (a) `Series` and the
    `series` kind are in Python and Rust; its theme colour is `#5fc9c0` (teal) with a
    circle in all three themes (the picture is pink with a diamond), so the AA pairs
    now cover six kinds. (b) `PlotNode` is a registered class, not a decorated
    function: `App([..., element_51.PlotNode])`. It has type name `Plot`, the one
    input `series_1`, no outputs and `display` chart; it is a plain class with
    class attributes (not a `Node` subclass, because a `Node` needs a return type
    and an output). The engine lets a node with no outputs run, and a chart node
    ignores a series input that has no link (an empty row is a place to link, not a
    missing value). (c) `NodeOutcome.shown` is `Image | Series` (`Shown`); a series
    output is carried to the next node and is also in the producing node's `shown`
    by output port name (added by `box-summaries`, so the page can write `last 0.05` in
    the box; only a chart draws it), a chart node's `shown` holds its linked series
    by input port name. On the Rust side `Shown` is an
    untagged enum and both records refuse unknown fields, so a picture and a series
    cannot be mistaken for each other. (d) The chart area is filled with the `field`
    colour like a picture area (the spec said the card colour); the plot has room on
    the left for the y labels (1.8 row heights) and below for the x labels (one row
    height), so no new theme number was added. Gridlines use `card_border`, tick
    text `field_text`. (e) The y range is the data's range plus 5 percent each side,
    or plus and minus 0.5 for a flat series; ticks are 1, 2 or 5 times a power of
    ten, about four per axis; the x axis is the index from 0 to the longest series
    less one. A series is thinned only when it has more than twice as many points as
    the plot has pixels in width: then each of `width` equal parts keeps its lowest
    and highest point, and a part with no number breaks the line (a gap inside a part
    is bridged). (f) A line's name is its source port's name; each line ends with
    `name last-value` in the line's colour, on the side of the end away from the
    line. Line colours take the port colours in turn (float, int, str, bool, image)
    and dash from the sixth (`line_style`), written now and used by task 36. Two lines
    with the same name are not yet told apart by node id (task 36). (g) The hidden
    tree label is "Shows a plot of 2 lines, loss last 0.31 and accuracy last 0.92."
    (`with_displays`, which replaces `with_pictures` and takes the graph to find the
    source port names); a line with no number says "loss has no value yet".
    (h) The example `g1_picture_and_plot.py` now has `loss_curve` (with `steps` and
    `blow_up`) and a `Plot` node.

### card-polish (approved 2026-10-21, done 2026-10-21)

Three small changes from the user's feedback on the picture example. Decisions, with
the reason:

- Wires are drawn twice: solid under the cards, so they show on the canvas, and again
    over the cards at `wire_over_card` (0.5), clipped to the cards. Chosen over one
    faint pass (the user's choice: a wire off the cards should stay easy to follow).
    The second pass strokes every wire, not only those that cross a card; culling was
    not done (about 0.15 ms at 200 nodes).
- A linked input shows `21.0 [n1.out]` after a run: the value in the box text colour,
    the source in the dim colour, the source cut first (`share_room`). Chosen over
    source first and over a value on hover (the user's choice). A picture or series
    source has no text value, so only its name shows (until `box-summaries`, below).
- `box-summaries` (2026-10-21, the user's choice after seeing `output` and `n1.out`
    where a picture or series is): after a run the box of a picture output says
    `28 x 28 grey` (`colour` for 3 channels) and of a series output `last 0.0842`
    (the last finite number, as the chart's end label); a linked input shows the same
    text then `[n4.loss]`. The text is worked out on the page from the data (`display::shown_summary`,
    used by `NodeOutcome::output_text`), nothing in Python says what to write. A
    series with no number says `2 gaps`. First drafted as `100 values, last 0.05`;
    the box holds about a dozen characters, so it cut to `100 values, la`; the last
    value comes first and the count is gone (the chart's x axis shows it). Cost:
    Python now sends a series once more for the node that makes it (up to about 2 MB
    at 100,000 points, with every Plot it feeds repeating it as before).
- `code-colours` (2026-10-21, the user's choice: an own tokenizer, not highlight.js, after
    both were laid out): `highlight.rs` cuts the code into runs (keyword, string with
    its prefix and triple quotes, comment, number, decorator or the name after `def` or
    `class`, plain), and the page makes one `<span class="tok-...">` per run, so the
    text is the code byte for byte. Chosen over highlight.js (a vendored file with its
    licence, its own class names to map by hand, no native test for the result) because
    the tokenizer has native tests, the five colours go through the theme with the
    contrast test, and nothing is downloaded. It colours what a reader needs and
    leaves the rest plain: soft keywords (`match`, `type`), f-string fields and a call's
    name are plain; text that is not Python is plain and whole. Five new theme colours
    (`code_keyword`, `code_string`, `code_comment`, `code_number`, `code_name`), each
    4.5:1 on the code box (`menu_field`) in every built-in theme and every palette. The
    long docstring line that looked cut off in the panel was not lost: `#source-text`
    has `overflow: auto` and macOS hides the scrollbar until you scroll, so the line
    scrolls sideways (667 px of code in a 614 px box).
- `View source code` is always available (the user's choice). The reply carries the
    code of the node type's function or class read with `inspect.getsource`, moved to
    the left margin; a function made by `exec`, or a missing file, gives a reason and no
    code. Anyone who can open the page can read the code of every node type, which is
    why the user was asked; an opt-in switch for embedding hosts was offered and not
    chosen. The panel is a non-modal `role="dialog"`: the code takes the focus, Tab
    reaches Close, and Close, Escape (wherever the focus is) or a mouse press outside the
    panel closes it (changed 2026-10-21 at the user's request; the button said Dismiss
    before), and the focus goes back to where it
    was when the request was made (a keyboard-opened menu remembers it). The method on
    nodes is `source_text`, not `source`, because a `Node` field may be called `source`.
- Rejected: renaming a node. The id (`n1`) is what links and saved files refer to, so
    a rename would be a separate optional display name in the saved file and both test
    suites; judged useful (24 nodes of one type are hard to tell apart) and left for
    after the experiment goals.

- Task 36 outcome (2026-10-21), changes from the spec above: (a) `NodeType.variadic` is
    `{prefix, kind}` (null by default, so old messages still read); `Variadic` and
    `PlotNode.node_type.variadic` are `series` and the `series` kind. (b) The empty row
    is named one past the highest row in use (the node id rule), not the lowest unused
    number: with rows 1 and 3 linked the card shows 1, 3 and 4. A new link then lands
    below the others and no row moves when it is made. Removing a middle link removes its
    row and keeps the others' names. An unlinked row that is listed in `inputs` (the
    first, `series_1`) is not shown once it has no link and another row has one. (c) A
    row number is the prefix, an underscore and a positive whole number with no sign and
    no leading zero; `series_0`, `series_01` and `series_x` are unknown ports on both
    sides (three cases in the shared `link_cases.json`). (d) The layout code was not given
    the links: every function that needs a node's inputs now takes the node type as
    `rows::resolve_type`/`type_of` gives it, which is the same type with the rows that grew
    as its inputs and is borrowed, not copied, when the type does not grow. That kept
    `node_rect`, `port_at`, `anchor_of` and the rest with their signatures, and the
    existing `layout.rs` tests unchanged. (e) `edit.rs`' `port` returns an owned `Port`
    because a row that is not listed is made on the spot. (f) The Python engine gives a
    chart node every linked row, in order of row number, in `shown`. (g) Lines take the
    colours of the float, text, bool, image and series ports in turn (the first five are
    different in all three themes; int shares float's colour by default, so it was
    dropped from the cycle) and dash from the sixth. A line is named after its source
    port, and when two lines share a name each gets its node id (`n4 accuracy`), in the
    chart and in the hidden tree label. (h) The example `g1_picture_and_plot.py` now has
    `training` (two outputs, `loss` and `accuracy`) feeding rows 1 and 2 of its Plot.

### speed-up-frame spec (2026-10-21)

Metric chosen by the user: redraw time per frame, the median over batches of 100 wheel
events from `tools/bench_page.js`, in the built-in browser pane (1024 by 768 CSS pixels,
device ratio 2, tab fronted). Budgets per workload (`tools/bench_page.py --workload`):
`nodes` (200 chained nodes, 12 in view): lower than the 1.51 to 1.55 ms of 2026-10-21;
`pictures` (12 picture cards with a value box each, Run pressed): 1.0 ms or less, as the
visual-ports spec says (1.16 to 1.22 ms before); `chart` (a 100,000-point series on a
Plot): 4 ms or less, as the visual-ports spec says (2.36 to 2.55 ms after Run), and no
slower than before. The benchmark times the work the page does to record a frame, not the
browser's painting of it, so a change that only cuts painting work may not show. The
profile and the tasks are in PLAN.md.

### job-nodes spec (approved 2026-10-21)

Part of the experiment goals (PLAN.md). The success measure is in PLAN.md; this is the
design behind it. The separate-process variant is the goal `job-isolation` (spec below).

- Users: Python authors who write a long task (a training run, a simulation) and want
    to start it from a card, watch its curves or pictures fill in, and stop it. Non-goals:
    PLAN.md, "Experiment goals" (no resume from a checkpoint, no two jobs of one node at
    once), and no global `report()`.
- Existing behaviour that must not change: `Node`, `@node`, Run, the saved graph format
    (`FORMAT_VERSION` stays 2; a job's state and results are not saved, so old files load
    unchanged), the snapshot version (stays 2; `jobs` has a default), the redraw time
    budgets of `speed-up-frame`, and every current test (fixtures only gain fields with
    defaults).
- Python surface (`element_51.JobNode`, a subclass of `Node`, so fields are inputs):
    * A step job defines `steps(self) -> int`, `begin(self) -> State` and
        `step(self, state: State, i: int) -> Out`. A job that cannot be split defines
        `work(self, report: Report) -> Out` instead. Defining both, or neither, is a
        `TypeError` at class creation. The runner (`jobs.py`, task 2) drives a step job
        itself: `begin`, then `step` for each `i`, checking Stop between steps and
        publishing each return value; a `work` job it calls once. The same driving
        function serves the child process of `job-isolation`.
    * `Out` is the return hint of `step` (or `work`), read like `Node.run`'s: a plain
        type gives one output `out`, a pydantic model one output per field. The class
        lives in `job_node.py` (`nodes.py` was over 400 lines); the engine only reads
        `NodeType.job`.
    * An update (a step's return value, a `report(...)` call, or `work`'s return value)
        sets a series output to its old points plus the new ones, and replaces any other
        output. `report(**fields)` takes a subset of the output fields; an unknown name
        raises `ValueError` naming the outputs. `Report` is a `Protocol`:
        `__call__(**fields: Carried)` and `stopped: bool`.
    * `Stopped` derives from `BaseException`, so a job's own `except Exception` does not
        swallow it. `report` raises it on the first call after Stop; a job may catch it to
        clean up and must then end. `report.stopped` is the same fact as a flag.
    * `State` is typed by a PEP 695 generic, `JobNode[State]` (spike done in task 1:
        pydantic 2.14.1 builds `JobNode[Net]` as an intermediate class and
        `get_type_hints` on a subclass's `step` gives the right hints; mypy strict passes
        with no `Any`). The intermediate class and the unparametrised base build no node
        type: `Node`'s hook returns early when `__pydantic_generic_metadata__` shows
        parameters or an origin.
- Engine (`engine.py`): `NodeType` gains `job: bool` (default false; on the wire and in
    Rust). `run_graph` gains `only: frozenset[str] | None` (run just these nodes) and
    `fixed: Mapping[str, Mapping[str, Carried]]` (nodes that are not run; their outputs
    are taken from here). A plain Run passes every job node's stored results as `fixed`,
    so Run never starts a job and a node after a job sees its latest values; a job node
    without results is skipped, with the reason `no output yet: press Start`.
    `ancestors(graph, node)` and `closure(graph, nodes)` (the nodes and everything they
    need) pick the subsets below. A skipped job node carries its reason in the
    existing `NodeOutcome.error` (`NO_OUTPUT`); the Rust side reads `error` only for an
    `error` status, so nothing there changes and no `reason` field is added.
- Runner (`jobs.py`, new): `JobState` (pydantic: `node`, `state` of idle, running,
    stopping, finished, stopped or failed, `step`, `steps`, `outputs`, `error`) and
    `JobRunner`, which owns one thread per running job. `start(graph, node)` refuses with a
    reason when the node is not a job, its job is running or stopping, or an ancestor
    fails; otherwise it runs `ancestors(node)` once, copies the inputs (an edit made while
    the job runs does not reach it), and starts a daemon thread (Ctrl+C must end the
    process). `stop(node)` sets the flag and the state to stopping. Every update is merged
    into the results under the lock and published at most every 50 ms (a timer sends the
    last one if updates stop coming; finished, stopped and failed are sent at once). The
    clock is a constructor argument so tests need no sleep. Deleting a job node stops its
    job and drops its state; loading a graph stops every job and clears all state.
- Messages (`protocol.py`, mirrored in `element-51-core`): client to Python `StartRequest`
    and `StopRequest` (`node`); Python to client `JobUpdate` (`JobView` plus `op`: `node`, `state`, `step`,
    `steps`, `error`, `outputs`, `shown`, and `linked`: the series now reaching each chart
    node input linked directly from this job, as (node, input port, series)), `JobRefusal`
    (`node`, `reason`, to the asking page only; the page shows the banner with Dismiss and
    announces it) and a partial `RunResult` (`partial: true`) that the page merges into
    its last run by node id instead of replacing it. `Snapshot` gains `jobs` (default
    empty; a tuple of `JobView`, the same fields as `JobUpdate` without `op`, because the
    Rust side tags a message by `op` and a snapshot's job list is untagged) so a page
    opened mid-run shows state and data. A start or stop that is accepted gets no
    reply (the broadcast `JobUpdate` is the answer), which `reply_to` says with
    `NoReply.NONE`; `None` still means a malformed message. `JobUpdate` goes to every open
    page through `Hub.broadcast` from the job's thread (`Connection.send` is thread-safe).
- End of a job: on finished or stopped, the server runs `closure(descendants(node))` once
    with the job's results as `fixed` and broadcasts the outcomes as a partial
    `RunResult`. On failed it does not (the results may be incomplete).
- Page (`element-51-web`, `element-51-core`; built in task 4): a node type with `job` adds two rows to the
    card below its ports (the buttons and the state word; the bar and `step 3 of 5`), with a Start and a Stop button, the state word and, for a step
    job, a thin bar and `step 3 of 5` (`node_rect` grows by a height that depends on the
    type only, as `display_height` does); the buttons and bar use existing theme colours
    (a new colour needs a contrast pair, CLAUDE.md). A click on Start or Stop sends the
    request; Stop is shown only while the job runs. `State` keeps a job map by node id,
    applied from `Snapshot.jobs` and `JobUpdate`; a chart node's lines come from the
    patch in `linked`. The hidden tree gives a job node two button items (after its ports; Start and Stop are
    `aria-disabled` and say why while they do nothing) and a sentence about its job in
    its label (`Job running.`, `Job failed: boom.`), and announces state changes on the
    status line (not every report: the tree is rebuilt on a state change only, so a chart's
    label shows the line as it stood at the last change). The run after a job is spoken
    only if something in it failed, so it does not replace the job's own announcement. Start and
    Stop are reachable by Tab/arrows and Enter like the other items.
- Interpretation recorded: "display nodes" in criterion 5 means chart nodes (`Plot`)
    linked directly to a job output. A picture or series on the job's own card follows
    its own reports. A node between the job and a Plot makes the Plot update when the job
    ends.
- Run button state: `Run` gets `aria-busy` and `aria-disabled` and the label `Running` between
    sending a run and receiving its (non-partial) result; it stays focusable and a press
    while busy does nothing. Built in task 6: axe 0 violations while busy.
- Verification: Python tests with a fake clock and fake jobs (state machine, merge rule,
    rate limit, refusal reasons, stop, failure, delete and load); a websocket test that a
    report reaches a connected client within 100 ms (the measure of criterion 5); shared
    fixtures for the new messages, read by both suites; native tests for layout, merge and
    hidden-tree text; `ui_states.py --graph job` states, `uiKeys` rows and an axe scan in
    three themes; a recorded walk-through in the browser pane.

### job-isolation spec (approved 2026-10-21)

- A class with `isolated = True` runs its job in a child process made with
    `multiprocessing` (spawn, because macOS). The child imports the node's class by module
    and name, builds the node from the pickled inputs, and runs `work` with a `report`
    that puts updates on a queue; the parent's thread reads the queue and does what
    `JobRunner` does for a thread job. A class defined where the child cannot import it
    (inside a function, or a `__main__` without the usual guard) is refused at Start with
    a reason.
- Stop sets the stop event, which makes `report` raise `Stopped` in the child at its
    next call. If no update arrives within 0.5 s the parent kills the process and the
    state becomes stopped. A child that exits without a final message becomes failed, with
    its exit code in the reason.
- The step form runs inside the child, so its state never crosses the boundary; only the
    inputs, each update and the result are pickled.

Built 2026-10-21 (task 8). Where the build differs from, or settles, the spec:

- `job_exec.py` holds the one function that runs a job's code (`execute`: `begin`, then `step`
    for each `i` with a Stop check before each, or `work` with a `JobReport`). It reports to a
    `Sink` (`begin`, `update`, `advance`). The thread runner's sink writes into the runner's
    state; the child's sink puts pickled messages (`Began`, `Updated`, `Advanced`, `Ended`,
    `Failed`, pydantic records in `isolated.py`) on a `multiprocessing` queue, and the parent
    feeds them to the same thread-runner sink. So a thread job and an isolated one take the same
    path through publishing, rate limiting and the end-of-job run.
- Stop sets a `multiprocessing` event the child's `report` and step loop read (`StopFlag` is a
    protocol, so a thread's or a process's event fits). The parent kills the process
    `KILL_AFTER` (0.5 s) after Stop if the child has not ended. The spec said "if no update
    arrives within 0.5 s"; a job that reports ends at its next report anyway, so the rule is
    simply 0.5 s after Stop. A killed job counts as stopped, and gets no chance to clean up.
- A child that ends with no `Ended` or `Failed` message (killed from outside, `os._exit`, a
    native crash) becomes `failed`: "The job's process ended without a result (exit code N)".
    The parent notices on the first empty poll (`POLL` 0.05 s) after the process is gone, so
    messages still in the pipe are read first.
- Refused at Start (`check_importable`, before the job is registered): a class defined inside a
    function (`<locals>` in its qualified name), a class whose top-level module a fresh
    interpreter cannot find (`PathFinder.find_spec` over `sys.path`, which ignores
    `sys.modules`: a module loaded by file path, as the tests load examples, fails it), and a
    class in `__main__` when `__main__` has no `__file__` (a notebook or a prompt). A class in
    a script's `__main__` is accepted: the spawned child re-imports the script and finds it
    there, which needs the `if __name__ == "__main__":` guard.
- Coverage (the user's open question, answered 2026-10-21 by taking the recommended option; the
    user said "Lets go"): the child's entry point `child_main` is an ordinary function that the
    tests also call in the parent with a list as the outbox and a thread event as the stop flag, so
    its lines count and no `# pragma: no cover` and no coverage setting is needed. The real-process
    tests (`tests/test_isolated.py`, about 2 s together) prove spawn, the kill, a crash and the
    refusals. The only pragmas are on test jobs whose bodies run in the child (`Sleeper`, `Dies`,
    one line of `Chatty`), as CLAUDE.md allows.


### auto-run spec (approved 2026-10-21)

Part of the experiment goals (PLAN.md). Criteria 1 to 8 are in PLAN.md and were
approved in words on 2026-10-21.

- Users: anyone editing a graph in the browser who wants results to follow their
    edits (a number box, a link, a deleted node) without pressing Run each time.
    Non-goals: starting a `JobNode` (jobs keep Start and Stop), saving the
    checkbox between page loads, a different delay than 300 ms, running while a
    node is only being moved or dragged.
- Integration points: `submit_edit` in `edit_ui.rs` (every user edit, undo and redo
    pass through it), the `Message::Snapshot` arm of `on_message` in `messages.rs`
    (a change pushed from Python arrives as a snapshot), `on_run` and `set_running`
    in `run_ui.rs`, `draw_card` in `draw_card.rs`, the Run button and its row in the
    page. Python does not change: `Message::Run` already runs every node except job
    nodes (`engine.py`, "A job node is never run here").
- Architecture:
    * `element-51-core`, new module `autorun`: a pure rule, no clock and no DOM.
        `AutoRun` holds `ticked`, the time of the last change and whether a run is in
        progress. `change(now_ms)` says what to do (wait for 300 ms, or warn that a run
        is going and queue one more); `due(now_ms)` says whether the time has come;
        `run_started()` and `run_finished()` update it; `finished` says whether a queued
        run must start. `DEBOUNCE_MS = 300`. Native tests in `tests/autorun.rs` cover the
        rule: three changes inside 300 ms give one run, a move gives none, a change during
        a run queues exactly one run, unticking drops anything pending.
    * `element-51-web`, new module `autorun_ui`: the checkbox, one `setTimeout`, and
        the glue. It calls `on_run` when the rule says so. `State` gets one `AutoRun`.
        `submit_edit` reports every edit except `MoveNode`; the snapshot arm reports a
        pushed change while the box is ticked; `RunResult` (not partial) reports the end
        of a run.
    * Page: `Run automatically` as a real `<input type="checkbox">` with a `<label>`
        next to Run, styled with the page's custom properties (no hex colour). Ticking
        sets `aria-disabled="true"` on Run, the same way `Running` does (it stays focusable;
        a click does nothing); unticking restores it. The warning goes through the existing
        `announce` status line, not the banner.
    * Red border: a new theme colour `card_error_border` (key in `Colours` and both
        theme JSON files, a row in `docs/themes.md`, a pair in `Theme::contrast_pairs`
        at 3:1 against the canvas, `ensure` in `palette.rs` so every palette passes).
        `draw_card` strokes it, in the selection width, for `Status::Error`. The title
        bar already turns `header_error` red today; the border is a second cue, and the
        banner and the node's own error text stay.
- Ticking the box runs the graph once at once (the user's request, 2026-10-21), so the
    screen is current without an edit; if a run is going, one more is queued (`tick_run`).
- Existing behaviour that must not regress: Run still works with the box unticked; moving a
    node keeps the shown values; job rows and Start and Stop are untouched; every
    other keyboard row of the walk-through (90, 16 and 21) still passes.
- Rejected alternatives: debounce in Python (a round trip per keystroke, and the timer
    state would live in two places); saving the checkbox in `localStorage` (criterion 1
    says unticked at every load); the `disabled` attribute on Run (it drops keyboard focus,
    which `Running` deliberately keeps); a banner for the "still running" warning (it is
    not an error and would need a Dismiss on every slow-run edit); a timer-free "run on
    every edit" (a slow node would start many runs).
- Risk noted: today an edit clears the shown run (`clear_run`), so with auto-run the
    values and red borders blank for about 300 ms and come back with the result. This
    keeps the existing rule; softening it is a separate decision.

### mnist-experiment spec (approved 2026-10-21)

Part of the experiment goals (PLAN.md). The goal and its five criteria were approved on
2026-10-21; this spec fills in what the criteria leave open.

- Users: a Python author who wants to see the editor do real work: pick a picture, train a
    small network on it while watching the curves, change a setting and train again.
    Non-goals: a better model (no convolutions, no Adam, no augmentation), saving a trained
    network, a confusion matrix or a prediction node, a GPU, a new dependency.
- Integration points: `JobNode` (`steps`, `begin`, `step`, as `Training` in `g2_job_nodes.py`;
    a job's inputs come from the nodes it needs, which Start runs first), `element_51.Image` (grey,
    28 by 28, one channel) and `element_51.Series`, `PlotNode`, auto-run (the sample node refreshes
    when `index` changes), `tests/test_examples.py` (a row per example, no network), and the
    `examples` group, which already holds `numpy` (so mypy and the example both see it).
- Data: four gzip files of the IDX format per dataset. MNIST from
    `https://storage.googleapis.com/cvdf-datasets/mnist/` (11.6 MB: train images 9,912,422,
    train labels 28,881, test images 1,648,877, test labels 4,542 bytes) and Fashion-MNIST from
    `https://github.com/zalandoresearch/fashion-mnist/raw/master/data/fashion/` (30.9 MB: 26,421,880,
    29,515, 4,422,102 and 5,148 bytes), both read-only checked on 2026-10-21. The original MNIST site
    (`yann.lecun.com`) answered 404 that day; PyTorch's `ossci-datasets` S3 bucket serves the same
    four MNIST files and is the cross-check for the pinned hashes. Files go in
    `__data__/mnist/` and `__data__/fashion/` (git-ignored). The sha256 of each file is pinned in
    the example after one approved download (trust on first use, plus the MNIST cross-check against
    the second host); a download goes to `<name>.part`, is hashed, and is renamed only if it matches,
    so a file on disk is always a verified one and a re-run does not hash again.
- Architecture, all in `examples/g_experiments/g3_mnist.py` so the example reads top to bottom
    (about 300 lines, the pieces named in this order):
    * `Idx` (a pydantic `Model`: magic, count, rows, columns, validated with `model_validate` from the
        header) and `load_images`, `load_labels`: parse a gzip file into a numpy array.
    * `DATASETS`: a table of name, folder, base address, and the four files with size and sha256;
        `fetch(name, folder)` downloads what is missing (standard library `urllib`, no new dependency).
        `main()` calls it for both datasets before serving, printing name, source and size, so Run never
        downloads and a user sees what is fetched. Tests give a folder of fake files and never fetch.
    * The network: `784 -> hidden -> 10`, ReLU, softmax with cross-entropy, plain minibatch SGD
        (batch 64, the learning rate as given), He-style initial weights from a fixed seed, written as
        plain numpy functions (`init`, `forward`, `backward`, `accuracy`).
    * `MnistSample` (plain node): inputs `index: int = 0` and `dataset: str = "mnist"` (a text box
        holding `mnist` or `fashion`; anything else is an error naming both words); outputs an
        `Image`, the `label: int` and the `dataset` word, so `Train`'s own `dataset` box can be linked to it.
    * `Train` (`JobNode`): fields `hidden: int = 128`, `learning_rate: float = 0.1`, `epochs: int = 30`,
        `dataset: str = "mnist"`; one step is 100 batches (938 batches make an epoch, so 282 steps
        for 30 epochs); each step returns one new point of `loss` (mean over the step) and of
        `accuracy` (on the first 1,000 test pictures, and on all 10,000 in the last step); the job's
        outputs are the two series.
    * `PlotNode` after `Train`.
- Acceptance (the criteria in PLAN.md, made testable): a tiny fake dataset (200 pictures written as real
    gzip IDX files into a temporary folder) drives every test; a gradient check against finite
    differences pins `backward`; the same seed gives the same curves twice; a separable fake set
    trains to 100% in the test; Start on `Train` fills a `Plot` with `loss` falling and `accuracy`
    rising on the fake set; `dataset = "fashion"` selects the other folder; a bad word is a clear
    error. By hand, with the real data after an approved download: the default settings reach at
    least 95% test accuracy on MNIST, in at most two minutes on this Mac (measured: 7 to 8 s and
    97.7%, see Results); no accuracy target is set for Fashion-MNIST (87.8% measured).
- Changed after measuring (2026-10-21): the spec's first defaults, hidden 64 and 10 epochs, ended in
    1.8 s at 97.2%, so the plot filled before anyone could watch it. Hidden 128 and 30 epochs take
    about 7 s (282 points) and reach 97.7%, still far inside the two-minute budget; the example is
    meant to be watched.
- Existing behaviour that must not regress: every other example still builds and runs; `g2_job_nodes.py`
    is untouched; the library gets no new dependency and no new public name.
- Open points decided by the user on 2026-10-21: the sources above; a text box `dataset` (not a tick
    box); one plot point per 100 batches; a two-minute budget for the default run.
- Decided by me: both datasets download in `main()` before the page opens (42.5 MB once) instead of on
    first use in a node, because a Run or Start that waits on the network with no sign on the card is a
    worse surprise than a printed line at startup; the data helper lives in the example, not in the
    library, because it is example code (the library stays free of numpy and of any dataset).
- Rejected alternatives: `torchvision` or `scikit-learn` loaders (a large new dependency for 40 lines of
    parsing); a library module for datasets (the library would grow a public API for one example); one
    point per epoch (10 points, the plot jumps); a tick box for the dataset (the goal said a `dataset`
    choice and the user chose the text box); a convolutional network (slow in plain numpy and beside
    the point of the example); Adam (more code, no better demo than a visibly falling SGD loss).

## Decisions

- Product shape and Rust/WASM role (renderer and engine): user choice,
    2026-10-19.
- Python API offers both the decorator and the pydantic class: user choice.
- Python serves, browser edits: user choice.
- JSON for the wire format, binary where it matters later: user choice.
- Rust owns the graph while editing, synced to Python: user choice.
- On-demand execution: user choice.
- Fixed small port type set: user choice.
- Two Rust crates in a workspace: user choice.
- Multiple outputs in the skeleton, as pydantic model fields: user choice.
- starlette and uvicorn rather than websockets alone: the user wants the editor
    mountable in an existing ASGI app, which websockets alone cannot do.
- Hybrid canvas: Rust draws, real HTML inputs for text, because a canvas text
    box has no copy-paste, input-method support or screen-reader access.
- One wheel with the WASM bundled, so users need no Rust toolchain.
- Rust built with `rustup run stable cargo`: the Homebrew `rustc` on PATH has no
    WASM target.
- wasm-bindgen CLI is the prebuilt release, pinned to the crate version
    (0.2.129): the two must match exactly.
- Servers tested with a real uvicorn server and the websockets client, to avoid
    adding httpx as a dev dependency.
- Renamed to `element-51` on 2026-10-19: the first working name was taken on
    PyPI and crates.io, and `element-51` is free on both. PyPI name `element-51`,
    import `element_51`, crates `element-51-core` and `element-51-web`, licence
    Apache-2.0 (copyright 2026 jmdm). The repo folder was renamed to `element-51`
    later the same day, together with Claude's history folder for the project.
    Earlier entries in this file were corrected to the new names, not left as
    history.
- Approved downloads and dependencies (2026-10-19, by name, source and size):
    Python, PyPI, runtime: starlette 1.9.0 (0.08 MB), uvicorn 0.54.0 (0.09 MB),
    websockets 17.2 (0.22 MB), and their dependencies anyio (0.13 MB), h11
    (0.04 MB), click (0.13 MB), idna (0.07 MB). Rust target
    `wasm32-unknown-unknown` for the rustup stable toolchain, static.rust-lang.org,
    23.2 MB. wasm-bindgen CLI 0.2.129, prebuilt `aarch64-apple-darwin` tarball from
    the GitHub release, 7.9 MB, sha256 `81d4a23d...1cd1` checked. Rust crates,
    crates.io: wasm-bindgen =0.2.129, js-sys 0.3.106, web-sys 0.3.106, serde
    1.0.229, serde_json 1.0.154. Only some of these are used today (js-sys is not).
    Any further dependency needs approval again.
- The Python server dependencies use `>=` lower bounds, not `==`: a library
    must not pin exact versions for the people who install it. `uv.lock` pins.
- The author email is left out of `pyproject.toml` (the name is `jmdm`), to match
    the placeholder packages the user published without one, because the file's
    metadata is published with every wheel.
- The `LICENSE` file is the standard Apache-2.0 text, taken from the
    `LICENSE-APACHE` inside the wasm-bindgen release tarball already downloaded,
    with the appendix line filled in as `Copyright 2026 jmdm`.
- The user registered `element-51` on PyPI and on crates.io with 0.0.1
    placeholder packages (author jmdm, Apache-2.0), through another session. The
    repo never publishes: no `uv publish`, no `cargo publish`; the user does it.
    Per that session, the 0.0.1 packages carry no repository link, so a 0.0.2
    could add one; that is the user's call.
- Every feature the user can try ships with a runnable example in `examples/`
    (user's request, 2026-10-19: "plenty of examples"). Each example sets up a
    graph in `build_app()`, says in its docstring what to try, and has a row in
    `tests/test_examples.py`, so it cannot silently break. `a0_hello_world.py` is
    the project template and is not an example of the library.
- JSON field names (task 1): ports carry `kind`, nodes carry `node_type`,
    messages carry `op` and `version`. Avoids `type`, which shadows a Python
    builtin, and keeps `op` for message kinds. Links are `source` and `target`
    port references.
- Node ids are `n<number>`, one above the highest in use, so a deleted id is
    never reused.
- Both sides refuse unknown fields (pydantic `extra="forbid"`, serde
    `deny_unknown_fields`) and a protocol version they do not read.
- The browser shows errors in a banner that stays until the user presses
    Dismiss; the same pattern serves the rollback and run-error messages later.
- `@node` accepts only plain functions: a `functools.partial` has no name to
    give the node type.
- Edit protocol (task 2): the browser proposes the id of a node it adds
    (`n<number>`, one above the highest it knows), so it can apply the edit before
    Python answers. Python refuses an id already in use. Chosen over letting
    Python allocate ids, which would need temporary ids and a rename on ack.
- Requests are numbered by the browser; acks and rejects carry the number.
    The browser keeps the last confirmed graph plus pending edits (`Replica`). A
    reject removes that edit and replays the rest on the confirmed graph,
    dropping any that no longer apply, so a node and the links that depended on
    it roll back together. A reject for an id already dropped is ignored.
- The link rules (types agree, int feeds float, one link per input, no cycles)
    exist twice, in `edits.py` and `edit.rs`. The shared `link_cases.json` is run
    by both test suites, so they cannot drift apart unnoticed. Check order:
    nodes, ports, types, input taken, cycle.
- A malformed message from the browser closes the websocket with code 1007 and
    the reason "malformed message", not a guess at what was meant.
- The right-click menu is a real HTML element, not canvas drawing, with one
    delegated click handler. The canvas bitmap is sized to the window, so one
    pixel is one pixel.
- Port colours and wire style are placeholders until the theme tasks (11, 12).
- The browser's graph copy is called `Replica`: `Sync` collides with the
    standard `Sync` trait.
- Values for unlinked inputs (task 3): the Python default of the function
    parameter. An input with no link and no default makes that node fail with
    "input 'x' has no value". This needs nothing in the graph format; task 8 adds
    per-node values the browser can edit.
- Two ways to run: `app.run()` returns each node's value by id and raises the
    failing node's own exception with a note naming the node (3.11 `add_note`);
    `app.execute()` never raises and returns an outcome per node, which is what
    the browser's Run button gets back as `run_result`.
- A node that raises is `error`; every node downstream of it is `skipped` and
    never runs; independent nodes still run. A node that returns a value its
    output port cannot carry (a bool for an int, a str for a float) is an
    `error`: the port types are stricter than Python's, except that an int may
    stand in for a float.
- Running happens in a worker thread (`run_in_threadpool`), so a slow node does
    not block the websocket. The engine reads the graph once at the start; it is
    immutable, so edits during a run are safe.
- The browser clears run results on any edit, snapshot or rejection, so what it
    shows is always from the last run of the graph it is showing.
- `@node` accepts defaults (checked against the port type) and refuses
    positional-only parameters, because the engine calls by name.
- The right-click menu is moved back inside the window and scrolls if the list
    is longer than the window (found by trying it near the bottom edge).
- Class-form nodes (task 4, user's choice of three shapes): fields are the
    inputs, with their pydantic defaults; a `run` method returns the outputs. A
    plain return hint gives one output `out`; a pydantic model gives one output
    per field, in declaration order. The class name is the node type name.
    Rejected: outputs declared as `Output[T]` fields, because a frozen model
    cannot be assigned to, so `run` would need a special helper.
- A node type is built the same way for both forms, from a name, the inputs
    with their hints and defaults, and a return hint, so the two forms cannot
    differ. The class form builds it in `Node.__pydantic_init_subclass__`.
- Both forms refuse unsupported hints, defaults of the wrong type, and (for
    classes) `default_factory`, with an error that names the node and the field.
- `app.run()` returns `{node id: {port: value}}` for every node, a single
    shape whether a node has one output or several (user's choice). It replaces
    the flat `{node id: value}` of task 3, which had no way to hold two outputs.
- On the wire, a node outcome carries `outputs` (a value per output port)
    instead of one `value`.
- After Run the browser draws each output's value on that output's row, just
    left of the port name, and trims it with an ellipsis to the room left by the
    input label on the same row (user's choice). It replaces the single value in
    the header.
- The engine checks every output of a model-returning node, not only the
    model: pydantic can build a model without validation (`model_construct`),
    and a wrong type must not pass as a port value.
- The internal graph node record is `PlacedNode`, in Python and in Rust, so the
    user-facing base class can be called `Node`.
- A reload shows the graph Python holds and nothing else: pending edits that
    were not yet acknowledged are dropped with the old page, and run results are
    not kept in Python, so values disappear until the next Run (task 5).
- Python pushes to open pages (task 6a). `App.add_node` and `App.add_link`
    call each subscribed listener with a fresh snapshot; `App.apply`, the
    browser's route, is silent. The server keeps a `Hub` of open pages, each
    with an outbox queue and one sender task, so a script on another thread can
    push safely (`call_soon_threadsafe`). After a browser edit is acknowledged
    the server sends a snapshot to every other page and not to the sender, which
    already has the edit. Whole snapshots, not edit deltas: simple, and the
    graph is small. The browser's `Replica::reset` keeps pending edits that
    still apply to the new snapshot and drops the rest, so a push never wipes
    what the user just did. `App` was not thread-safe at this point (it is since task 26, below); a
    browser edit racing a script edit with the same id is refused with a reason.
- Save and load (task 6): the file is `{"format_version": 1, "nodes": [...],
    "links": [...]}`, pretty-printed JSON, positions included. Loading reads the
    version first and refuses any other with a message naming both versions,
    then replays every node and link through `apply_edit`, so a file whose
    node type is missing, or whose links break the rules, is refused and the
    app keeps its old graph. A missing file raises `FileNotFoundError` as is.
    Task 8 adds per-node values; it must raise `FORMAT_VERSION` and decide
    whether to read version 1 files.
- Pan, zoom, move, select, delete (task 7). Three new edits cross the wire:
    `move_node`, `delete_node` (takes the node's links with it) and
    `delete_link`, with a new refusal `unknown_link`. A shared fixture
    (`edit_cases.json`) holds the expected graphs, run by both sides. The view
    (pan and zoom) lives only in the browser and is not saved or sent. A node
    drag shows the node at the pointer without touching the replica and sends
    one `move_node` when the button is released, so one drag is one edit (and
    later one undo step). Moving a node keeps the shown run values; any other
    edit clears them. Pressing a node anywhere except an output port selects it
    and starts a drag; pressing a wire selects it; pressing empty canvas clears
    the selection and starts a pan. Leaving the canvas cancels the gesture.
    Delete and Backspace remove the selection, Escape drops it. The zoom range
    is 0.25 to 4. The page's `web-sys` dependency gained the `WheelEvent` and
    `KeyboardEvent` features (no new crate).
- Value editing (task 8). A node holds `values`, the numbers and text the user
    typed for unlinked inputs. An input takes its link, then the node's own
    value, then the Python default, else the node errors "has no value". The
    node type sends its defaults (`NodeType.defaults`), so the page can show them
    in grey. One edit, `set_value` with a null value to clear, so an empty box
    means "use the default" (a text input cannot be the empty string). The rule
    checks the node, that the port is an input, and that the value fits the
    port (an int fits a float, a bool is not a number); setting a value on a
    linked input is allowed and ignored until the link goes. The file format is
    version 2 and still reads version 1. Loading replays values through the same
    rule, so a hand-edited bad value is refused. The boxes are real HTML inputs
    in a `#fields` layer over the canvas, one per unlinked input, placed from
    the same label-width maths the canvas uses and rescaled with the zoom; a
    bool is a checkbox. An edit commits on change (Enter or leaving the box);
    unreadable text shows a message and the box returns to the stored value.
    While a box has focus the page does not overwrite it, and keys typed in a box
    never reach the Delete handler. Nodes are 220 wide, up from 160, to leave
    room for a box beside the label and a value on the output side.
- Rounded look (extra task 17, the user's request): node cards have corners
    of radius 8 (graph units), the header takes them by clipping to the card
    path, the selection outline follows the rounded path, and wires have round
    caps and joins and were then copied from the Antimony screenshots at the user's
    request: a wire takes the colour of the port it leaves (numbers `#edc27b`,
    text `#94a95f`, sampled from the screenshots; bool stays blue), is 3.5 units
    thick (5 when selected; measured as about 7 screenshot pixels on a card 450
    pixels wide against our 220 units), and the curve keeps horizontal ends with
    a reach of half the horizontal gap, at least 20 units (was 40), and when
    the ends are nearly in one column a sideways swing of a third of the height
    between them (at most 70), so a wire to a port above or below does not run
    along the card's edge (the user spotted that overlap). The port
    markers take the same two colours. Port squares and input boxes were left as
    they were by the user's choice. The card radius is `card_radius` in the
    theme's metrics (moved there in task 11).
- Undo and redo (task 9). The history lives in `element-51-core`
    (`History`). For each edit the user makes, it records the edit and the edits
    that undo it, worked out from the graph before the edit: a move goes back to
    the old position, a value to the old value (or none), a delete restores the
    node, its values and its links (several edits, one step), an add is undone
    by the matching delete. Undo and redo send their edits to Python like any
    other edit and are not recorded again, so Python stays the one true copy
    and other pages see the result. A new edit clears the redo stack. A drag is
    already one `move_node`, so it is one step. A restored node goes to the end
    of the node list, so it is drawn on top. The history is the page's own: it
    is not saved, a reload empties it, and edits made by other pages or scripts
    are not in it; an undo that no longer applies shows Python's or the rules'
    reason and stops that step. Keys: Cmd or Ctrl with Z, Shift added for redo,
    Ctrl+Y also redoes; no buttons, because none were asked for. Keys typed in
    an input box belong to the box.
- Serving (task 10). `serve` builds a `PageServer`, a uvicorn server that
    opens the page with `webbrowser.open` once it is listening, when
    `open_browser=True` (off by default, so tests and mounted use never open a
    browser). uvicorn stops gracefully on Ctrl+C and then raises
    `KeyboardInterrupt` again; `serve` swallows that so a script ends with exit
    status 0. A test starts a real process, sends SIGINT and checks the status
    and the absence of a traceback. `a1_two_nodes.py` replaced `a1_serve.py`; it
    shows one node written as a function and one as a class, linked.
- Theme (task 11). `Theme` is plain data in `element-51-core::theme`, read from
    JSON (`themes/antimony.json`, embedded). It has `colours`, `metrics` and
    `fonts`. Layout and hit-testing take `&Metrics` as their last argument
    (the old constants `NODE_WIDTH`, `HEADER_HEIGHT`, `ROW_HEIGHT` and
    `PORT_RADIUS` are gone), so a theme can change sizes and the tests pass a
    value in. Drawing reads colours and sizes from the theme in the `Frame`. The
    page's own parts (input boxes, right-click menu, error banner, Run button)
    read CSS custom properties that `apply_theme` sets from
    `Theme::css_variables()` on the root element; `index.html` has no hex
    colours, and uses the CSS keyword `Canvas` before the script runs. A native
    test scans `draw.rs`, `lib.rs` and `index.html` for hex colours, which is how
    criterion 2 is kept true as the code changes. The first theme repeats the
    current look exactly, so the refactor changed nothing visible (checked in
    the browser pane, including the right-click menu); the Antimony values come
    in task 12. Interaction constants that are not part of a look (menu margin,
    wheel zoom rate, link pick distance) stay as constants in the web crate.
- The Antimony look (task 12). Cards are stacked rows: header, one row per
    input, one per output (`layout::output_row`). A row is a right-aligned name
    and a dark rounded value box; ports sit inside the card (`port_inset`) and
    are drawn over the wires, which are drawn over the cards, as in the
    screenshots. A linked input's box shows its source (`n2.out`) greyed; an
    output's box shows `output` greyed, then the value after a run; the page's
    input boxes sit over unlinked inputs' boxes and a bool is a small checkbox
    at the start of its box. The title bar has the id bold on the left, the type
    name bold on the right and a three-bar icon (clicking it opens the card menu,
    task 18). Cards have a 2-unit border in the header grey and a light
    selection outline. The canvas has a dot grid (every 24 units, left out when
    the dots would be under 8 pixels apart). Colours are the sampled ones where
    we have them (card `#202020`, header and border `#505050`, value box
    `#303030`, title `#e0e0e0`, names `#b0b0b0`, hints `#505050`) and judgement
    where we do not (canvas `#1a1a1a`, dots `#2c2c2c`, typed text `#e0e0e0`).
    The hint colour is faithful and low in contrast; the user may want it
    lighter. The theme lost `text`, `value`, `field_width`, `field_gap`,
    `label_pad` and `value_gap`, and gained `title`, `label`, `dot`,
    `card_border`, `icon`, `border_width`, `port_inset`, `port_gap`, `box_gap`,
    `box_pad`, `box_radius`, `icon_*`, `grid_step` and `dot_radius`.
- Themes by name (task 13). `antimony-light` is the same file as the dark theme
    with a light canvas (`#eeeeee`), lighter dots and a dark selected wire; the
    cards, boxes, sizes and fonts are shared, as in the screenshots of the
    Antimony website. A new colour, `selected_wire`, exists because one
    selection colour cannot show on both canvases (a pale wire vanishes on a
    light canvas). Python holds only the names (`element_51.theme`); a shared
    file `tests/fixtures/theme_names.json` is read by a Python and a Rust test,
    so the lists cannot differ. `App.theme` is a property: the setter checks the
    name (a `ValueError` listing the valid ones) and pushes a snapshot, so a
    running page switches at once; `serve(theme=)` and `asgi_app(theme=)` set
    it. The snapshot carries `theme` and the protocol version is 2; the page
    looks the name up in its built-in themes, applies it, and shows an error for
    a name it does not know. The page starts in the default theme until the
    first snapshot arrives, so a light page can flash dark for a moment. There
    is no switcher in the page, by the user's choice.
- Categories and the menu (task 14). A node type has `category`, a string with
    `/` between levels (`Math/Trig`), empty for the top level. `@node` works bare
    or as `@node(category="Math")`; a `Node` subclass sets a `category` class
    variable. A level with an empty name or spaces at its ends is a `ValueError`
    at definition time. The menu logic is in `element-51-core::menu`: a level
    lists its categories first (in order of first appearance), then its nodes in
    registration order, and `search` matches case-insensitively on the node name
    or its whole path, in registration order, showing each match with its path;
    empty text matches nothing. The web crate only renders: nested flyout panels
    opened by hovering (CSS), a search box that takes focus when the menu opens,
    Enter adding the first match, Escape closing, and the flyouts flipping to the
    left when the menu is near the right edge. Flyouts rule out scrolling the
    tree, so a menu with more top-level entries than fit in the window would
    run off the bottom; search results do scroll. The search box colour is a new
    theme colour, `menu_field`.
- Reference screenshots of Antimony stay outside the repo; they are not ours to
    redistribute.

- Restructure pass (2026-10-20, approved by the user). Thresholds were the
    defaults, 400 lines and five imports. The first measure found 7 files over
    400 lines and 2 modules over five imports, and jscpd found 0 clones, so the
    pass was about moving code by concern, not removing duplicates. Three moves:
    the web crate's `lib.rs` (1147 lines) split into `input`, `menu_ui` and
    `fields_ui` beside `draw`; `tests/test_server.py` (649) split into pages,
    websocket and hosts files with `server_support.py` and `conftest.py`; port
    inference moved from `nodes.py` into `ports.py` (the helpers lost their
    underscore, since `nodes` imports them). Each was a pure move, checked by the
    same tests, and the web split by hand in the browser. The hex-colour scan in
    `rust/element-51-core/tests/theme.rs` reads the new web files too.
- Second restructure pass (2026-10-21, approved by the user: all six tasks). Thresholds
    as before, 400 lines and five imports. The web crate and the core had grown
    since the first pass, so the first pass's decision to leave `draw.rs` whole (it
    was 497 lines) was reversed at 932: it held the chart legend, the job row and
    the focus ring beside the frame. Splits, all pure moves: web `menu_ui` into
    `menu_ui`, `menu_keys` and `theme_ui`; `a11y_ui` into `a11y_ui` and `tree_keys`
    (with the field return going to `fields_ui` and the link preview and the
    keyboard zoom and pan to `input`); `draw` into `draw`, `draw_card`,
    `draw_chart`, `draw_focus` and `draw_wires`; `lib` into `lib`, `edit_ui`,
    `messages`, `run_ui` and `canvas_ui`; core `access` into `access` and
    `announce` (the status-line sentences). jscpd found 2 clones (the arrow, Home
    and End step and the key-target lookup, in `on_menu_key` and `on_card_menu_key`);
    they change together, so each became one function (`focus_step`,
    `button_target`). Cost: modules over five imports went from 11 to 17, since a
    piece that moved out still calls its old neighbours; the line count fell and
    the number of things a reader must hold at once per file fell with it, which
    is what the pass was for. Two commits left a Rust test failing because the
    test read a file by name (`menu_ui.rs`, `lib.rs`); the next commit fixed each.
- A README per module (restructure pass): the Python package is one flat set of
    modules, so it has one README (`src/element_51/README.md`); each Rust crate
    has its own. They say what each assumes and must never depend on, and point
    to CODEMAP.md for the interface.

- Drawing only what is in view (speed-up task 3, 2026-10-20): `draw` skips a
    card, and its port markers, whose box does not overlap the visible part of the
    canvas. The box is grown by the selection width plus the border width first, so
    the outline and the markers at an edge are not cut off. Wires, the dot grid and
    the page's input boxes (`sync_fields`) are still drawn for every node: wires
    were 12% and the grid 8% of the old redraw and have not been measured since;
    a wire can cross the view between two nodes that are both outside it, so
    skipping wires needs a test of its own. `Rect::overlaps` and `Rect::grown` are
    in the core so a native test covers them.
- Task 2 (speed-up): `_delete_link` still compares pydantic records
    (`gone not in graph.links`); it runs once per delete, not per link added.

- Design direction for the page parts (ui-design-pass, chosen by the user
    2026-10-20 after three mock renderings): "soft modern" (direction C). Dark
    values, all text pairs at or above WCAG AA (ratio in brackets): menu `#1c1c1f`
    with text `#e6e6e8` (13.64), menu border `#2e2e32`, hover row `#2c2c31`
    (13.89), muted text and "No matching node" `#9b9ba3` (6.16), search box
    `#26262a` with border `#3a3a40`, Run button `#5e6ad2` with white text (4.70),
    its hover `#4f5bbf` (5.86), focus ring `#7c86f0` (5.41 on the canvas, drawn as a
    2 px gap and a 2 px ring), error banner `#4a2328` with text `#fbdada` (10.33),
    value-box hints `#9a9a9a` on `#303030` (4.69), skipped-node title lighter than
    the old `#808080`. Radii: Run button 6 px, fields 5 px, menus and banner 8 px.
    Popover shadow `0 8px 24px` at 60% black. The page font becomes the system UI
    font; the canvas keeps Verdana, which the user chose for sharpness. Motion:
    hover and press changes of 120 ms, none under `prefers-reduced-motion`. The
    light theme gets the matching values, each checked by a native contrast test
    (task 1). Reason: the user's choice; the menus sit apart from the canvas
    cards, which keep the Antimony card look.

- Trying a palette (goal palette-try, 2026-10-20): `element-51-core::palette`
    reads a Coolors link (the colours are in the link, so nothing is fetched) and
    returns the current theme with new colours; the page's `Theme` is replaced and
    the CSS custom properties are set again, and the wire protocol is untouched, so
    a reload shows the theme Python holds. Mapping, by the user's choice of "by
    lightness and colour": the colours are sorted by lightness; the darkest is the
    canvas, the lightest the text, the most saturated of the rest the accent; the
    next dark one, toned down until its luminance is at most 0.05, the cards and
    menus; the third (or a mix of the card and the text when there are fewer than
    four colours), toned down until the text reads on it, the title bars and
    borders. The other colours are mixes of those. Readability ("adjust text to
    stay readable", the user's choice): `ensure` moves a colour toward white or
    black, whichever needs the smaller move, in 4% steps until it has the contrast
    against every background it sits on; the one background that moves is the red
    bar of a failed node, so the title keeps the palette's text colour. Port
    colours are nudged to 3:1 on the card too; shapes, sizes and fonts are not
    touched. One list of pairs (`Theme::contrast_pairs`) serves the built-in
    theme tests and the palette tests, so a new colour pair is checked in both.
    Rejected: sending the palette to Python (it would change the graph owner's
    state from a click, and the user chose "until reload"); fetching the palette
    from coolors.co (needs the network and an API for no gain).
- Release guard (2026-10-20): `src/element_51/web/` is generated and git-ignored, so
    `uv build` succeeds without it and ships a wheel with no editor. `uv_build` has
    no build hook, so the guard is a script, `tools/check_wheel.py`, that builds the
    wheel in a temporary folder and exits 1 naming the missing files. CI runs it
    after `build_wasm.py`; before publishing, run it by hand (CLAUDE.md). Rejected:
    switching to a build backend with hooks, for one check.
- Third built-in theme `purple-synthwave` (user's request, 2026-10-21): the colours
    are what `with_palette` gives `antimony` for
    `https://coolors.co/acadbc-9b9ece-6665dd-473bf0-000500`, written out once as
    `rust/element-51-core/themes/purple-synthwave.json` (a temporary native test dumped
    the theme with `serde_json`; the file is then indented by 4 and named
    `purple-synthwave`), so the shipped values do not depend on the mapping rule
    changing later. Shapes, sizes, fonts and port shapes are `antimony`'s. It joins
    `THEME_NAMES` (Rust and Python, `tests/fixtures/theme_names.json`), so the native
    AA test, the no-hex test and the palette test run over it too; `docs/themes.md`
    has a third column in the colours table; example `e_look/e4_purple_synthwave.py`.
    Rejected: a pink and cyan neon palette, since the user asked for this one.
- Publishing and the public repo (2026-10-21, the user's choices): 0.1.0 went to PyPI on
    2026-10-20 and 0.1.1 on 2026-10-21, only to add `[project.urls]` (a published
    release's metadata cannot be edited). The repo went public on GitHub with the full
    history and the user's own commit email, with no rewrite: the same address is
    already public in `ci-group/ariel` (228 commits found with a read-only API search),
    so hiding it here would protect nothing, and a rewrite would change 116 hashes, the
    tag, and about 22 hashes quoted in the notes. New commits use the GitHub noreply
    address (`git config user.email`, local to this clone). A release is a version bump
    in `pyproject.toml` and `uv.lock`, `tools/check_wheel.py`, the user's upload, then an
    annotated tag `v<version>` on the release commit; the tags are pushed too.

- Default theme and the in-page theme list (2026-10-21, the user asked for both): the
    default is `purple-synthwave` (`DEFAULT_THEME` in `theme.py`, `DEFAULT_THEME` in
    `theme.rs`), no longer the first name in `THEME_NAMES`, so the list and the docs
    table keep their order. The menu button's `Select theme` opens `#theme-menu`; a click
    swaps in `Theme::builtin(name)` at once and tells nobody, like the palette box
    (a reload brings back the theme Python holds), and the choice is not stored in the
    browser. Reason: the user asked for the list and gave no wish to keep it, and
    storing it would be a new setting nobody asked for. The user did not answer
    the question about remembering it, so the smaller choice was made; say so if
    they want it kept. Tests that need the old dark look use `Theme::builtin("antimony")`
    (the palette test cannot use the default as its base: the default is the
    palette of its example link).

- `element_51.serve` binds `127.0.0.1` only and has no `host` argument (2026-10-21,
    the user asked whether a script can change the address): the server has no login,
    so exposing it to the network would let anyone edit and run the graph. A script that
    wants another address runs `uvicorn.run(element_51.asgi_app(app), host=..., port=...)`;
    `examples/a_basics/a1_hello_world.py` does this, with `HOST` and `PORT` at its top and
    a note about the missing password, and `docs/embedding.md` shows the same for FastAPI.
    Not done: a `host` argument on `serve()` (new behaviour, not asked for; if added, keep
    `127.0.0.1` the default and warn when it changes).

### Functional notebooks (2026-10-21)

Four notebooks, one per feature area a newcomer meets first (nodes and running, save and
load, themes, pictures and series), not one per module. `tests/functional/README.md` has a
row per public module of `src/element_51/` and `tests/test_functional_coverage.py` fails
when a module has none or a row names a missing file. Reason: the success measure is "every
public area has a notebook or a row saying none yet", and a test keeps it true.

- Dev dependencies added with the user's yes (2026-10-21): `nbclient`, `nbformat`,
    `ipykernel` and `matplotlib` (the user chose matplotlib for real line plots). Pictures
    are drawn with Pillow, which was already a dev dependency. Installed size of the
    whole `.venv` is 345 MB, of which matplotlib is 23 MB.
- The live-push and server area is left out: it needs a local port and is already
    covered by `tests/test_server_websocket.py`.
- Notebook output is a PNG through `IPython.display.Image` from bytes, not `plt.show()`:
    the runner sets `MPLBACKEND=agg`, under which `plt.show()` emits nothing.
- Ruff checks the notebooks with every rule (only `assert` and `print` are allowed
    there). So a notebook compares floats with `math.isclose`, puts expected values in a
    constants cell, and checks that an error was raised with a flag
    (`raised = False`, `try`, `raised = True`, `assert raised`), because `raise
    AssertionError("text")` breaks the exception-message rules.
- `ty check` reads the notebooks too. `app.run()` returns the `Carried` union, so a
    notebook narrows each value with `isinstance` (and `assert`) before it uses it, and
    uses `PilImage.Resampling.NEAREST`, not `PilImage.NEAREST`.
- A refused link raises `element_51.edits.EditError`, which is not exported from
    `element_51`, so the notebook imports it from `element_51.edits`.

### Job nodes (2026-10-21, planning talk with the user)

- Stop is cooperative in a thread (the next report or step raises or ends), with the
    job allowed to catch `Stopped` to clean up; a separate process killed after 0.5 s
    is built as the goal `job-isolation`, not left for later (the user's choice: both).
    Reason: Python cannot kill a thread safely, and a hard kill is the only way to
    keep the 1 s promise for a job that never reports.
- The shape is steps (`begin`, `step`) on top of `work(report)` (the user's choice).
    Reason: the framework then owns the loop, so Stop is exact between steps, the
    progress bar and the report rate limit are free, a test calls `step` directly, and
    a separate process keeps the state inside the child. `work` stays for a job that
    is one long call.
- A series output grows by what an update returns; others are replaced (the user's
    choice). Reason: a training loop returns its new epoch, and the job keeps no
    history.
- A node after a job sees the latest reported values, and results stay until the next
    Start (the user's choices). Reason: a live plot needs the values mid-run, and a
    stray keystroke in a box must not lose a long run.
- Start runs the nodes the job needs, once (the user's choice). Reason: a job started
    from a stale or never-run upstream value would be wrong with no sign.
- The progress bar and `step 3 of 5` are shown for step jobs (the user's choice,
    beyond the first goal text). It costs one more labelled part to check at AA.
- `fixed` and `only` on `run_graph` (not a second engine function): a plain Run, Start
    and the end-of-job run are the same loop over different subsets.

## Rejected approaches

- Python owns the graph and the browser asks first: every drag waits for a
    round trip, and a stuck Python script freezes the editor.
- A single Rust crate: link-rule tests would need a browser test runner.
- A JS framework around the canvas: adds npm and a second language.
- Rust runs the node code in the browser: makes Python only a description
    language, against the goal.
- websockets alone as the server: smallest, but cannot be mounted in an ASGI
    app.
- aiohttp: heaviest option, no benefit here.
- Committing the built WASM: stale builds and large binaries in every commit.
- Building the wasm-bindgen CLI from source with `cargo install`: it compiles for
    minutes, downloads more than the 7.9 MB prebuilt tarball, and installs into
    `~/.cargo`. The prebuilt tarball is kept in git-ignored `tools/bin/`.
- `wasm-pack`, `trunk` and `console_error_panic_hook`: not needed for the
    skeleton, so not added.
- Rounded port markers (corner radius 3, so small pills): tried at the user's
    request after the cards and wires were rounded; the user did not like them and
    they were reverted (commit `abc27f7`). Ports stay plain squares.
- starlette's `TestClient`: needs `httpx` as a new dev dependency; tests use a
    real uvicorn server and the `websockets` client instead (see Decisions).

Task 18, the card menu (2026-10-20): clicking the three-bar icon selects the
card and opens a small popup at the pointer, in the theme's menu colours
(`#card-menu`, the same CSS variables as the add-node menu). `layout::icon_rect`
gives the icon's box, which `draw.rs` also uses, and `layout::icon_at` finds
the node whose icon is under a point: only the topmost card there counts, and
the target is the icon plus half the title padding each side, over the header's
full height. `menu::CardAction` lists what the popup can do and maps each action
to an edit, so the entry list and the edit are native-tested; the page builds
its buttons from `card_actions()`. There is one entry, Delete node, as the user
chose. It goes through the normal edit channel, so undo brings the node back.
Escape, a click elsewhere and a right-click close the popup.

The page's own parts follow the theme (2026-10-20, at the user's requests): the
canvas is drawn at one pixel per device pixel (the first version stretched a
CSS-pixel canvas, which blurred on a Retina screen), and the Run button takes the
menus' colours, a 6-pixel radius, a shadow and a play triangle, with the menu
hover colour for hover and focus. No new theme colour was added.

The themes use Verdana at 12 px (2026-10-20, the user's choice after the generic
sans-serif still looked soft). Verdana is wide and drawn for small sizes, and it
ships with macOS and Windows; `sans-serif` is the fallback. The font strings are
in the theme files, and the card layout measures text, so nothing else changed.

Values use Monaco (2026-10-20, the user's request): `Fonts.value` is the canvas
font for the text in value boxes (numbers, strings, run outputs, link hints) and
`Fonts.value_family` is the family for the input boxes laid over them, shown to
the page as `--value-font`. Menlo and `monospace` are the fallbacks. Port names,
titles, menus and buttons stay in Verdana.

The canvas context is created with `alpha: false` (2026-10-20): an opaque canvas
lets Chrome smooth text with sub-pixel colour, which looks sharper on dark
backgrounds than the grey smoothing of a transparent one. The options object is
made with `JSON.parse` through a small `wasm_bindgen` import, so no new crate
is needed. The user still saw soft text after the device-pixel fix, so this is a
further attempt; whether it is enough depends on their screen and browser.

Zoom speed (2026-10-20, the user found it slow): a mouse wheel pixel scales by
`exp(0.002)`, so one notch (100 pixels) is about 22%; a trackpad pinch, which
Chrome and Safari send as a wheel event with Ctrl held and steps of a few
pixels, uses `exp(0.01)` per pixel. The time to draw was not the cause: 60
wheel events took 89 ms in the browser pane, about 1.5 ms each, so redraws are
not batched.

Theme files, task 19 (2026-10-20, the user chose the file option): `serve(theme=...)`
and `App.theme` take a built-in name or the path of a JSON file that lists only
the values to change, in `colours`, `metrics` and `fonts`, on top of a `base`
built-in theme. Python reads and shapes the file (`ThemeFile`, pydantic) and
the snapshot carries the base name plus the changes (`theme_changes`, optional
on read, so the protocol version stays 2). The browser applies them with
`Theme::with_changes`, which owns the list of valid keys, because Python does
not know them; a wrong key or a value of the wrong kind shows the error banner
with the section, the key and the valid keys, and the page keeps the previous
theme. Rejected: a Python `Theme` record (the key list would live in two
places) and merging in Python (it would need the base theme's data). The
reference in `docs/themes.md` lists every key with its doc comment from
`theme.rs`; a native test fails if a key has no row, and another that the
example file uses only real keys. Checking a key in Python at `serve` time
would be possible by exporting the key list into a shared fixture; not done.

Port styles, task 20 (2026-10-20, from the user's sketch): the theme has a
`ports` section, by type (`int`, `float`, `text`, `bool`), then side (`in`,
`out`), then `colour` and `shape` (`box`, `circle`, `diamond`). The user chose
separate int and float entries, wires that follow the output port's colour (no
wire key), those three shapes, and no old keys: `port_number`, `port_text` and
`port_bool` were removed from `colours`, with no shim. `Theme::with_changes`
now merges nested objects, naming the path of a wrong key (`ports.float.in`),
and an unknown shape is refused with serde's list of the valid ones. The
default look is unchanged: all boxes, one colour per type for both sides. A
dragged link's hover colour (`allowed`, `refused`) is drawn in the hovered
input's own shape. A circle's diameter is `port_size`; a diamond reaches
0.7 of it from the centre, about the area of the box. Hit-testing still uses a
round target for every shape.

Embedding examples, task 21 (2026-10-20, the user asked for a small test of how
simple it is to build a GUI around the editor): `a19_fastapi_host.py` mounts
`asgi_app(app)` at `/editor` in a FastAPI app with a side panel of plain HTML
and two endpoints (`/api/run`, `/api/add-node`); `a20_nicegui_host.py` does the
same with NiceGUI's `ui.run_with(host)` on the same FastAPI app and a splitter
page (editor in an iframe, NiceGUI widgets beside it). Findings: both are about
60 lines and worked on the first run once the page found its websocket. The
page used to connect to `/ws` at the host root, so a mounted editor loaded but
could not connect; it now takes the address from its own path
(`address::websocket_url`, so `/editor/` gives `/editor/ws`). A node added by
the panel reaches the open editor at once, because the editor listens to the
`App`. Starlette strips the mount prefix for a Starlette child app, so no
change was needed on the Python side. Limits: one `App` per process, so one
worker; the host and the editor must share the process. `fastapi` and `nicegui`
(about 31 packages, 8 MB for NiceGUI) are in a new `examples` dependency group
that is installed by default with `dev`, so the checks can type-check the
examples; the library does not depend on them. Only the FastAPI example is
tested automatically (a real server, the websocket and both endpoints); the
NiceGUI one is checked by hand, because `ui.run_with` changes NiceGUI's global
state and a second call in one test process is not safe.

Example layout (2026-10-20, the user: every example follows
`a0_hello_world.py`): each example has a docstring with the description, a
`Notes` section holding what to try, and `References` only where a docs page
exists; the comment headers; the rich traceback and console; `main()` and the
guard. The random generator, the data folder and the `Todo` section are left
out, because no example needs them. `a7` prints through the rich console.
`a0` itself stays as the template and is not run by the tests.

Example folders (2026-10-20, the user's grouping): the examples sit in group
folders named letter and topic, and each file takes the folder's letter and a
number that restarts in the group. Old names, which older Results rows and
Decisions still use, map like this:

| Old | New |
| --- | --- |
| `a0_hello_world` | `a_basics/a1_hello_world` |
| `a1_two_nodes` | `a_basics/a2_two_nodes` |
| `a2_arithmetic` | `a_basics/a3_arithmetic` |
| `a3_strings` | `a_basics/a4_strings` |
| `a6_port_types` | `b_node_types/b1_port_types` |
| `a8_multiple_outputs` | `b_node_types/b2_multiple_outputs` |
| `a9_class_nodes` | `b_node_types/b3_class_nodes` |
| `a5_diamond` | `b_node_types/b4_diamond` |
| `a4_errors` | `b_node_types/b5_errors` |
| `a7_script_api` | `c_python_api/c1_script_api` |
| `a10_live_push` | `c_python_api/c2_live_push` |
| `a11_save_load` | `c_python_api/c3_save_load` (writes `c3_graph.json`) |
| `a12_big_graph` | `d_editor/d1_big_graph` |
| `a13_values` | `d_editor/d2_values` |
| `a14_undo_redo` | `d_editor/d3_undo_redo` |
| `a16_menu` | `d_editor/d4_menu` |
| `a17_card_menu` | `d_editor/d5_card_menu` |
| `a15_light_theme` | `e_look/e1_light_theme` |
| `a18_theme_file` | `e_look/e2_theme_file` (with `my_theme.json`) |
| `a19_fastapi_host` | `f_embedding/f1_fastapi_host` |
| `a20_nicegui_host` | `f_embedding/f2_nicegui_host` |

`examples/template.py` is the project template and keeps its numpy and data setup;
it sits outside the group folders, so `tests/test_examples.py` does not load it.
`a_basics/a1_hello_world.py` is a one-node example (added 2026-10-21; before that
the file was the template, which looked empty).

Page theme keys, task 22 (2026-10-20, the user approved starting it): a new
`page` section holds the sizes of the page's own parts in CSS pixels (21 keys:
font sizes, menu radius, padding and width, shadows, the Run button's position,
radius, padding and gap, input and search radius, error banner radius and
margin), and six new colours (`run_bg`, `run_text`, `run_border`, `run_hover`,
`run_hover_text`, `accent`). `Theme::css_variables` turns them into CSS custom
properties and `index.html` reads only those. Defaults equal the old literals,
checked in the browser pane (Run button at 16 px, 6 px radius, 13 px text,
`#ececec` background). Two native tests guard it: every `var(--x)` in the page
is one the theme sets, and no font size or radius is written in pixels in the
page. Left unthemed on purpose: 1 px borders, a few fixed paddings (banner,
search box margin), the Run triangle's size, and the letter spacing. The bool
checkbox is the browser's; `accent` only tints it when ticked, and the
unticked box stays white. Run's hover and focus colours now have their own
keys instead of borrowing the menu's.

Embedding research (2026-10-20, from a second LLM's answers, checked where
possible): verified here, Starlette 1.9.0 gives a Starlette child app a path
without the mount prefix, and a plain ASGI callable the full path in
`scope["path"]` with `scope["root_path"]` set to the prefix. Run for real:
FastAPI and NiceGUI hosts (tasks above). Claimed by that LLM and not tried:
Gradio can be mounted onto FastAPI with `gr.mount_gradio_app` (the only direction
that matters here); Streamlit has an experimental `streamlit.starlette.App`
(1.53); Dash 4.2 can use a FastAPI backend; Reflex cannot mount a sub-app below
`/` (open issue); Flet mounts through `flet_fastapi.app`; Mesop is WSGI only. A
claim that websockets are broken in pywebview's GTK backend rests on a 2021
issue of unknown state and is not relied on. Untried idea: PySide6's
`QWebEngineView` pointing at a local `serve()` for a native window. Its version
and date claims could not be checked; `fastapi` 0.156.0 and `nicegui` 3.18.0 are
what `uv add` resolved.

CI for Rust and WASM, task 23 (2026-10-20, the user approved it): a new workflow
`.github/workflows/rust.yml` with two jobs, `rust` (rustfmt, clippy with
`-D warnings`, native tests) and `wasm` (installs the `wasm-bindgen` CLI 0.2.129
with `cargo install --locked`, runs `tools/build_wasm.py`, checks the three
output files, builds the wheel and asserts it holds `element_51/web/element_51_web_bg.wasm`).
It is a separate file because `ci.yml` is pulled in by `rsync-sync pull python`
and a template update would overwrite edits. Rejected: downloading the prebuilt
`wasm-bindgen` release as local development does, because I could not check its
sha256 for Linux offline, and `cargo install --locked` needs none (it costs a few
minutes per run; a cache could be added). The toolchain is `stable`, which is
what `build_wasm.py` asks rustup for, so a new stable release can add clippy
lints and fail CI; fix the code then. Indents are multiples of 4, so each list
item starts with a bare dash. The workflow first ran on GitHub on 2026-10-21 (Results); before
that the same commands, the YAML parse and the wheel check were run locally.

Scrolling add-node menu, task 24 (2026-10-20, the user approved it): the menu's
tree now has `max-height: calc(100vh - 90px)` and `overflow-y: auto`, as the
search results already did. A scrolling box clips an absolutely positioned child,
so the flyouts are `position: fixed` and a `mouseover` handler on `#menu`
(`on_menu_hover`) places the flyout of the entry under the pointer against the
window: `menu::flyout_at` (native-tested) puts it against the entry's right edge,
level with it and raised by the menu padding, flips it to the left edge when the
right has no room, and keeps it inside the window; `menu::clamp_into` does the
same clamping for the menu itself and for the card menu, which had two copies of
that code. The flyout touches its entry, so the hover is not lost on the way. This
replaces the `flip` class and `MENU_FLYOUT_WIDTH`, a guess at the flyout's width.
A flyout taller than the window scrolls inside itself. The tree's scroll position
is reset each time the menu opens. Checked in the browser pane with 30 top-level
categories (the tree scrolls, a flyout at the bottom stays level with its entry,
a menu opened at the right edge flips its flyouts left) and with three levels of
nesting.

No dark flash, task 25 (2026-10-20, the user approved it): `index.html` has
`html:not(.ready) { visibility: hidden; }`, and the web layer's `mark_ready` sets
the class `ready` on the root element after the first snapshot has applied the
theme, and in `show_error`, so a failure still shows the banner. Before that the
page keeps the default theme's variables as a fallback but is not painted, so a
light-theme page no longer shows the dark one first. The browser's own canvas
colour (`Canvas`) can still show for the moment before the page is ready; it is
the system colour, not a theme. Checked in the browser pane with the light
example: the class is `ready` after load, and removing it hides the page. The
flash itself is too short to see in a screenshot, so it was not observed before or
after; a native test checks that the rule and `mark_ready` exist and that
`show_error` calls it.

Thread-safe `App`, task 26 (2026-10-20, the user approved it): `App` has one
`threading.RLock`, held by `add_node` (picking the id and applying the edit are one
step), `apply`, `add_link`, `load` (the file is read and checked before the lock),
the theme setter, `subscribe`, `snapshot` and `_changed`. Listeners are called
inside the lock, so two threads' changes reach every listener in the order they
were made (a later snapshot never overtakes an earlier one); the lock is
re-entrant, so a listener may call back into the `App`, and listeners already
had to be quick (the server's only listener posts to the event loop). Running the
graph (`execute`, `run`) takes no lock: a `Graph` is immutable, a run reads it
once, and node functions run unlocked, so a slow node never blocks editing.
`save` reads the graph once and needs none. Three tests use 8 threads of 60 nodes
with the interpreter's switch interval at one microsecond: distinct ids and no lost
node through `add_node`, the same through `apply`, and listener snapshots in
non-decreasing size; all three failed on the old code and pass 5 runs out of 5
with the lock. Rejected: a lock for `run` (it would block edits for as long as the
slowest node), and an `asyncio` lock (scripts run in plain threads). Still true:
one `App` per process, so several server workers would each hold their own graph.

README and docs drafts (2026-10-20, the user asked for them; reviewed and merged
by the user's word the same day): the README says what the library is, lists what it
gives, installs with `uv add element-51` and from a clone (needs Rust and the
`wasm32-unknown-unknown` target), shows the two-node example (the same code as
`a_basics/a2_two_nodes.py`, run to confirm it gives 21.0 and 42.0), and links
the examples, the docs and the Antimony credit. `docs/index.md` repeats the
description and example and points to the themes, embedding and reference pages;
`docs/embedding.md` is new and holds the FastAPI and NiceGUI recipes and the
limits (one process and one worker, a lock in `App`, the editor works under any
prefix). Two assumptions to check at review: the install line is true only after
the user publishes the package, and the drafts contain no images (any image would
have to go through `tools/compress_images.py`).

- Restructure pass, left alone on purpose (2026-10-20, the user confirmed):
    - Splitting `session.py` (250 lines, 7 imports): it is the facade that joins
        edits, engine, store, theme and protocol; each import is used for one call
        it must make, so a split would add a layer with one call site.
    - Splitting `rust/element-51-web/src/draw.rs` (497), the core `theme.rs`
        (427, one struct per theme section plus the merge code) and
        `rust/element-51-core/tests/theme.rs` (462, key tables): one concern each;
        a split adds files without making a feature easier to hold.
    - `tools/compress_images.py` (470): a copy of the template that a project
        does not edit.
    - Extracting shared code between clones: none to extract, jscpd found 0
        clones (5,363 lines, `--min-lines 8 --min-tokens 70`).

- Second restructure pass, left alone on purpose (2026-10-21, the user approved the list):
    - Core `theme.rs` (561) and `model.rs` (539): one concern each (a struct per theme
        section plus the merge code; the wire records).
    - `session.py` (435, 9 imports), `engine.py` (502), `nodes.py` (473), `jobs.py` (438):
        one concern each, a little over the line limit; a split adds a layer with one
        call site.
    - `tools/run_notebooks.py` and `tools/compress_images.py`: copies of the user's
        templates that a project does not edit.
    - The 11 test files over 400 lines: each mirrors one module. `tests/access.rs` (916)
        still tests both `access` and `announce`; it was not split.
    - `messages.rs` (11 imports): it is the one handler for every message from Python,
        so it touches every part of the page by design.

- Speed-up pass, left alone on purpose (2026-10-20, the user confirmed):
    - `Graph.next_id` (about 0.9 s of the 2.2 s the build took after task 2): it
        scans every node id. A faster version needs a cached counter, and a counter
        changes which id is reused after the newest node is deleted, which is a
        behaviour change.
    - A set of node ids for `_add_node`'s id-in-use scan: about 14% of build, and
        a set needs invalidation rules for a gain nobody feels.
    - Snapshot JSON: linear, 1.8 ms for 2000 nodes.
    - Page load: 22 ms warm on localhost for the 706 KB WASM. `wasm-opt` could
        shrink the file, but no size budget exists and the cold load was not
        measured.
    - Skipping wires and the dot grid in `draw`: 12% and 8% of the old redraw;
        revisit only if a measured redraw after task 3 is still too slow.

- UI directions A and B (ui-design-pass, 2026-10-20, the user chose C): A,
    Antimony with the gaps fixed (same look, hover blue `#2f6db5`, 5.28:1), was my
    recommendation because the canvas matches it; B, a flat Blender-style node
    editor (square corners, `#4772b3` hover, 4.85:1), was the runner-up. The user
    preferred the softer, rounder look.

- UI design pass, left alone on purpose (2026-10-20, the user confirmed): fit-to-view
    and touch handling at phone width (new behaviour, not style); keyboard and
    touch ways to open the menus and flyouts (new behaviour, part of the canvas
    accessibility goal); custom illustration, decorative animation or a component
    library (nothing in the page needs one); a "running" state for the Run button
    (the page does not learn when a run ends, so it needs a protocol change).

- Rewriting the 116 commits to the GitHub noreply address before the first push
    (offered 2026-10-21, with a backup bundle first): the user chose to push as is once
    it was clear that the address is already public in `ci-group/ariel`; the cost would
    have been new hashes everywhere (tag `v0.1.0`, about 22 hashes in PLAN.md and
    DESIGN.md).
- For `canvas-accessibility` (2026-10-21), rejected: fallback content inside
    `<canvas>` (poor VoiceOver support, no real focus handling); drawing every node as
    DOM (a rewrite, and it loses the speed measured at 2000 nodes); a live region
    only (a blind user could not learn which nodes exist); a link mode on its own
    key (the Enter rule on ports needs fewer keys and no modes to remember).
- For `visual-ports` (2026-10-21), rejected: base64 or PNG-encoded pixels (a new
    dependency or a hand-written decoder for no gain at 28 by 28); a separate display
    panel (the user chose cards); a Plot node with a fixed number of inputs (the user
    asked for rows that appear as lines are linked); new theme colour keys for chart
    lines (more keys in three themes and the palette code for little benefit); sending
    pictures as binary frames (the binary channel is a non-goal for now).
- Speed-up-frame, left alone on purpose (2026-10-21, the user confirmed):
    - Caching text widths and skipping repeated `set_font` calls in `card` and `columns`
        (task 4): about 150 calls a frame at 0.4 us each is 0.06 ms, 4% of a frame, and it
        would need a cache invalidated when the theme's fonts change.
    - Drawing a card's title bar without the clip (task 5): switching the clip off gave
        +0.03 ms with the grid on and -0.18 ms with it off, inside the 0.1 ms spread, so the
        clip is not the cost.
    - Updating the page's value boxes only when they moved (task 6): `sync_fields` is 0.22
        ms of the 12-picture frame, but zoom and pan move every box every frame, so the
        wheel benchmark could not improve.

- For `job-nodes` (2026-10-21, rejected or left out, with the user): injecting an
    exception into the thread with `PyThreadState_SetAsyncExc` (it cannot interrupt a
    numpy call and can fire inside the job's own cleanup, leaving half-written state); a
    generator job (`yield`; a helper or library callback cannot report without `yield
    from`); a global `element_51.report()` (hidden state, and the stepper removes most of
    the need; it can be added later as a thin wrapper); async jobs (a numpy loop blocks
    the loop, so a thread is needed anyway); a series that a report replaces (the job would
    keep its whole history and the page would receive the whole curve each time; no job
    needs it yet); downstream nodes that see only a finished job's outputs (a live plot
    would stay empty); clearing results when an input changes (one keystroke loses a long
    run); Start with the last Run's values (can be stale without a sign); a status text
    line on the card (not asked; `step 3 of 5` covers progress).

- Coverage of the child through `pytest-cov`'s subprocess support (rejected 2026-10-21): it
    needs a change to the shared coverage settings (the user's say-so), slows every test, and a
    child killed on purpose never writes its data, so the kill test's lines would not count
    anyway. A thin wrapper marked `# pragma: no cover` was also not needed: passing `child_main`
    itself as the process target leaves no wrapper.
- `fork` as the start method (rejected): unsafe on macOS in a process that has threads (the
    server and the job threads), and the user's editor runs on macOS. `spawn` costs a fraction of
    a second per Start, which a hand-started job can pay.
- `concurrent.futures.ProcessPoolExecutor` (rejected): it cannot kill one running job without
    breaking the pool.
- Killing as soon as Stop is pressed (rejected): a job that reports often should get the chance
    to end with `Stopped` and keep its last values, so the kill waits 0.5 s.
- A cheaper grid for `speed-up-frame` task 3 (rejected by the user, 2026-10-21): a tile-pattern fill
    or a cached bitmap would save at most 0.5 ms of a 1.4 to 1.5 ms frame (the grid is 0.6 to
    0.7 ms), probably less because the benchmark measures recording time and not painting, and
    both change how the dots look (a tile is scaled by the zoom, so the dots get soft at some
    zooms; a cached bitmap helps only frames where the view did not move). The page already
    redraws well inside its 4 ms budget, so the crisp dots stay.

## Results

| Date | Change | Result |
| --- | --- | --- |
| 2026-10-21 | goal restructure-2: split the large web modules and core `access`, remove the last 2 clones | source files over 400 lines 13 to 8 (the 8 left alone on purpose), jscpd clones 2 to 0 (0.15% to 0.00%), `draw.rs` 932 to 371, `menu_ui.rs` 805 to 375, `a11y_ui.rs` 826 to 369, `lib.rs` 703 to 342, `access.rs` 583 to 361; modules over five imports 11 to 17; redraw 1.36 to 1.65 ms medians on 4 sets (before 1.3 to 1.6); all Rust tests and clippy pass, `uiKeys` 90, `uiKeysDisplay` 16, `uiKeysJob` 21 rows pass |
| 2026-10-21 | first push to GitHub, Pages, Dependabot | `CI` (ruff, mypy, pytest through uv with `--locked`) passed; `Rust and WASM` (fmt, clippy, tests, wasm build, `tools/check_wheel.py`) passed on `a22a269`; `Docs` built with `-W` and 0 warnings but its deploy step failed with "Get Pages site failed" until the user enabled Pages (build type `workflow`), then passed; Dependabot ran at once for github-actions, uv and cargo and opened pull request #1 (setup-uv 9.0.0 to 10.3.0); the site answers 200 at https://jacopo-dm.github.io/element-51/; 225 Python tests at 100% and 153 Rust tests locally |
| 2026-10-20 | goal palette-try: paste a Coolors link, the editor takes its colours | native tests (9 in `tests/palette.rs`, 3 more for the page, 3 for `contrast_failures`): the link parser (the example, five other written forms, eight bad links each with its reason), the mapping for the example (canvas `#000500`, text `#acadbc`, accent `#473bf0`, cards and headers dark, menus take the card colour), two colours are enough, the order in the link does not matter, six palettes (the example, all light, all dark, one hue, near-grey, ten colours) leave both base themes with 0 AA failures, a palette changes colours only (not metrics, page, fonts or port shapes), a bad link leaves the theme as it was, the page has the menu button after Run, the `Try palette` entry and the box, focus and press styles for them, and the page code never sends a palette to Python; by hand in the browser pane: the menu button opens a menu with Try palette, the box takes the link, Enter applies it at once (near-black canvas, blue-violet cards, violet Run), `hello` shows "That is not a Coolors link" in the banner and leaves the box open, a reload brings back `#1a1a1a`, the page made no request to coolors.co, and axe-core reported 0 violations with the menu closed, the menu open, the box open and after applying; screenshots `docs/ui-baseline/palette-1-menu-beside-run.jpg` to `palette-4-bad-link-banner.jpg`; example `examples/e_look/e3_palette.py`; 153 Rust tests, 221 Python tests at 100% |
| 2026-10-20 | ui-design-pass task 4: the soft modern look for the page parts | native tests: the page font is the system UI font, menu and banner radii 8, new `menu_border` colour and `menu_item_radius` page key reach the page and are used by it, the focus ring is the theme's focus colour in menus too, every colour pair still at AA (including the focus ring on the menu and its search box); the old test pinning the previous page values now pins only what did not change; by hand in both themes: violet Run button, dark rounded menu with an inset hover row and flyout, 8 px banner with a styled Dismiss, violet checkbox tick; axe-core 0 violations in both themes in the default state, after Run, and the search with and without a match; before `docs/ui-baseline/dark-1280-default.jpg`, `-menu.jpg`, `-run-and-banner.jpg`, after `docs/ui-baseline/task4-dark-1280-default.jpg`, `task4-dark-1280-menu-flyout.jpg`, `task4-dark-1280-run-and-banner.jpg`, `task4-light-1280-menu-flyout.jpg`, `task4-light-1280-run-and-banner.jpg`; 140 Rust tests, 220 Python tests at 100% |
| 2026-10-20 | ui-design-pass task 3: page structure and names | native tests: the page has `<main>`, a visually hidden `<h1>`, `role="alert"` on the banner and an `aria-label` on the search box; `field_label` names each value box "port of node id" (the checkbox included); axe-core in both themes, in the default state, after Run (with the banner), the search with matches, with no match and the tree: all five rules gone, 0 violations (before: `label` 1, `landmark-one-main` 1, `page-has-heading-one` 1, `region` 2 to 5, and after task 1 a `scrollable-region-focusable` that did not come back once the lists sat inside `<main>`); the page looks the same (the heading is hidden); 138 Rust tests, 220 Python tests at 100% |
| 2026-10-20 | ui-design-pass task 2: visible focus and press states | native tests: every page control (Run, menu and card-menu buttons, search box, Dismiss, value boxes) has a `:focus-visible` rule that uses `var(--focus)` (inside the light menus `--menu-hover-bg`), the three buttons have `:active`, transitions are 120 ms and `prefers-reduced-motion` sets them to none, the new `focus` colour reaches the page as `--focus` and meets 3:1 on both canvases (`#7c86f0` dark, `#4f46e5` light); by hand: Tab shows a ring on Run and on a menu entry; before `docs/ui-baseline/dark-1280-menu.jpg`, after `docs/ui-baseline/task2-dark-1280-menu-focus.jpg` and `task2-dark-run-focus.png`; axe unchanged (task 3 fixes the rest); 136 Rust tests, 220 Python tests at 100% |
| 2026-10-20 | ui-design-pass task 1: contrast test and AA colours | native tests: `contrast` (black on white 21:1, symmetric, same colour 1:1, a known pair, bad hex refused), and every text pair (13) and graphic pair (5 plus the 8 port styles) of both themes meets AA; axe `color-contrast` 1 to 0 on the no-match menu, both themes (the other rules unchanged, to be fixed by tasks 2 and 3); changed: `placeholder` `#505050` to `#9a9a9a` (1.64 to 4.69), `text_dim` `#808080` to `#aaaaaa` (2.88 to 4.9), `menu_hover` and `run_hover` `#4a90d9` to `#2f6db5` (3.34 to 5.28), new `menu_muted` `#595959` (replaces `opacity: 0.6`, 4.01 to 5.93), light theme `selected` `#e0e0e0` to `#202020` (1.14 to 15), `allowed` and `refused` darker in the light theme; before `docs/ui-baseline/dark-1280-run-and-banner.jpg`, after `docs/ui-baseline/task1-dark-1280-run-and-banner.jpg`; 133 Rust tests, 220 Python tests at 100% |
| 2026-10-20 | speed-up task 3: draw only the cards and ports in view | `tools/bench_page.js`, 200 nodes, 12 in view, 4 sets of 11 batches of 100 wheel events: redraw median 3.3 to 3.5 ms (range 3.26 to 4.06) before, 0.21 to 0.22 ms (range 0.207 to 0.520) after, 16 times faster; the experiment had given 15.4 to 0.7 ms with the browser painting between batches; native tests for `Rect::overlaps` (inside, partly outside, apart on each axis, touching) and `Rect::grown`; by hand in the browser pane: panning shows the partly visible nodes at each edge, zooming out shows every node, and a selected edge node keeps its outline; 126 Rust tests, 220 Python tests at 100% |
| 2026-10-20 | speed-up task 2: link targets compared by field in `edits._add_link` | `tools/bench_python.py`, 2000-node chain, 11 runs: build median 1968.9 ms (range 1944.7 to 2091.2; 2059.1 at the first baseline) before, 795.3 ms (782.8 to 889.4) after, 2.5 times faster; run and snapshot unchanged within the spread; 220 Python tests at 100%, unchanged. What is left of the build is `Graph.next_id` and the id-in-use scan, left alone (see the rejected approaches) |
| 2026-10-20 | speed-up task 1: links indexed in `engine.py` | `tools/bench_python.py`, 2000-node chain, 11 runs: `run` median 489.8 ms (range 480.6 to 585.2) before, 5.9 ms (5.7 to 6.7) after, 83 times faster; build 1969 ms (1945 to 2091) and snapshot 1.9 ms unchanged within the spread; 220 Python tests at 100%, unchanged |
| 2026-10-20 | task 26: thread-safe `App` | pytest: 8 threads adding 60 nodes each through `add_node` (distinct ids, none lost), through `apply` (none lost), and listener snapshots arriving in order; the three failed without the lock (stashed the change to check) and pass 5 of 5 runs with it; 220 Python tests at 100%, 122 Rust tests |
| 2026-10-20 | task 25: no dark flash on the light theme | native test: the page has the hiding rule, `mark_ready` exists and `show_error` calls it; by hand: with the light example the root has class `ready` and a `#eeeeee` body, and without the class its visibility is hidden; the flash itself was not observed; 217 Python tests at 100%, 122 Rust tests |
| 2026-10-20 | task 24: scrolling add-node menu | native tests: `clamp_into` (inside, hanging out, too tall) and `flyout_at` (right, flipped left, lifted near the bottom, kept below the top); by hand with 30 categories and with three nesting levels: the tree scrolls and resets on open, flyouts sit level with their entry, flip at the right edge and stay open on the way from the entry; 217 Python tests at 100%, 121 Rust tests |
| 2026-10-20 | task 23: CI for the Rust crates and the WASM build | not run on GitHub (no remote); locally: the workflow parses as YAML with the two jobs and 5 and 8 steps, `cargo fmt`, `clippy` and `cargo test` pass, `build_wasm.py` builds, and a wheel built with `uv build --wheel` holds `element_51/web/element_51_web_bg.wasm` |
| 2026-10-20 | task 22: page theme keys | native tests: the page reads only variables the theme sets, no pixel font sizes or radii in the page, page keys come out as pixel values, a wrong or mistyped page key lists the valid ones, defaults equal the old literals, docs list every key; pytest: a `page` section in a theme file and a bad value; by hand: `e2_theme_file` shows a round blue Run button and rounder menus, and the default theme reads back 16 px, 6 px, 13 px and `#ececec` on the Run button; 217 Python tests at 100%, 115 Rust tests |
| 2026-10-20 | task 21: FastAPI and NiceGUI examples, editor under a prefix | native tests: websocket address at the root, under prefixes, with a file name, https; pytest: the FastAPI host serves the panel and the editor, the editor's websocket works at `/editor/ws`, a node added through the API reaches the open page, Run through the API; by hand: both examples show the editor, Add node adds a node live, Run shows `{'n1': {'out': 3.0}, 'n2': {'out': 3.0}}`; 216 Python tests at 100%, 110 Rust tests |
| 2026-10-20 | task 20: port styles by type and side | native tests: every type and side has a style, defaults are boxes with equal in and out colours, a change reaches one side of one type only, wrong names and shapes name the path and the valid values, docs list every type; pytest: a ports file and bad shapes; by hand: `a18` shows diamonds for number inputs and circles for number outputs, `a6` keeps the box look; 213 Python tests at 100%, 105 Rust tests |
| 2026-10-20 | task 19: theme files | native tests: no changes, partial changes, base, unknown base, key, section, wrong kind, non-objects; the example file and every key in the docs; pytest: file shapes, name and path, errors, session and snapshot, `serve` with a file, example `a18`; by hand: `a18_theme_file.py` shows the blue theme, a file with `canvass` shows the banner naming the key and listing the keys; 210 Python tests at 100%, 100 Rust tests |
| 2026-10-20 | task 18: the header icon opens a Delete node menu | native tests: icon box position and size, icon hit-test (inside, header height, title text and body excluded, under another card refused), the entry list and its edit; by hand with `a17_card_menu.py`: the click opens the popup and selects the card, Delete node removes the node and its links, Cmd+Z restores them, Escape and a click on empty canvas close the popup; 194 Python tests at 100%, 90 Rust tests |
| 2026-10-19 | task 14: categories and the nested menu with search | native tests: tree nesting and order, search by name, path and case, empty results; pytest: bare and called decorator, class variable, badly formed categories refused, fixtures carry `category`; by hand with `a16_menu.py`: Math > Trig > sine adds a node at the click, `math` lists three matches with paths, Enter adds the first, Escape closes, flyouts flip left near the right edge; 193 Python tests at 100%, 86 Rust tests |
| 2026-10-19 | task 13: `antimony-light`, `serve(theme=...)` | pytest: theme names equal on both sides, unknown name lists the valid ones and starts no server, setting the theme pushes a snapshot, every snapshot names the theme; native: both themes parse, share sizes, differ in canvas and selected wire; by eye with `a15_light_theme.py`: light canvas and grid, dark cards, dark selected wire; 181 Python tests at 100%, 79 Rust tests |
| 2026-10-19 | task 12: the Antimony look | native tests rewritten for stacked rows and inset ports (anchors, hit-testing, link picking, metrics); by eye in the browser pane next to the screenshots: dark cards with a grey title bar, id left, type right, icon, right-aligned names, dark value boxes, greyed hints, ports over wires, dot grid; typed value 5 gives 15.0 on Run and a float to int link is refused with the right hit-testing; 77 Rust tests, 167 Python tests at 100%; the user's acceptance (criterion 1) is pending |
| 2026-10-19 | task 11: `Theme` | native tests: both themes parse (one so far), css variables, layout follows given metrics, no hex colour in `draw.rs`, `lib.rs` or `index.html`; the page looks the same as before in the browser pane (cards, values, input boxes, menu); 76 Rust tests, 167 Python tests at 100% |
| 2026-10-19 | task 10: `a1_two_nodes.py`, clean Ctrl+C | `uv run examples/a1_two_nodes.py` run for real with the browser opener stubbed: it logged the page URL as opened, Ctrl+C gave a graceful shutdown and exit 0; pytest runs a real process the same way; 167 Python tests at 100% |
| 2026-10-19 | task 9: undo and redo | native tests: every accepted edit in the shared cases is undone and redone to the same graph, add node and link, delete with links and values, stack order and redo cleared by a new edit; by hand with `a14_undo_redo.py`: drag, delete and their undo, redo, and a reload keeps the result; 165 Python tests at 100%, 65 Rust tests |
| 2026-10-19 | task 8: value editing with HTML inputs | by hand with `a13_values.py`: typing 5 gives 15.0 on Run, `abc` is refused with a message and the box restored, a ticked box turns the text to capitals, boxes follow zoom, pan and drag, values survive a reload; set_value, bad values and clearing give identical graphs in Python and Rust on 9 more shared cases; 164 Python tests at 100%, 60 Rust tests |
| 2026-10-19 | task 7: pan, zoom, move, select, delete | by hand with `a12_big_graph.py`: a node dragged to a new place and deleted, wheel zoom, empty-canvas pan, a wire selected and deleted, Run showing `has no value` after; both survive a reload; the move, delete node and delete link edits give identical graphs in Python and Rust on 7 shared cases; 145 Python tests at 100%, 55 Rust tests |
| 2026-10-19 | task 6: save and load | round trip, version refusal, five kinds of damaged file, rule-breaking file, and a real websocket test that an open page receives the loaded graph; `a11_save_load.py` saves on Ctrl+C and loads on the next start (run for real); 133 Python tests at 100% |
| 2026-10-19 | task 6a: Python pushes to open pages | `a10_live_push.py` adds nodes every two seconds and an already-open page shows each at once, no reload; Run then gives 2.0 to 6.0; 119 Python tests at 100%, 45 Rust tests; two-client pytest (script edit reaches an open page; a browser edit reaches the other page only) |
| 2026-10-19 | task 5: reload keeps the graph | no product code needed: every connection already gets a snapshot (task 1); added tests for edits made through the socket surviving a reconnect and for two open pages; by hand: a node added in the browser survives a reload; 109 Python tests at 100% |
| 2026-10-19 | task 4: several outputs, class-form nodes | `div_mod` shows `q` and `r` in the browser; class nodes run and list by class name; 107 Python tests at 100%, 43 Rust tests; 3 new link cases for a multi-output node run on both sides |
| 2026-10-19 | task 3: run the graph | engine runs in dependency order; failed nodes red, downstream skipped, independent branches still run; 86 Python tests at 100% (including all examples), 42 Rust tests; by hand: errors and arithmetic examples in the browser |
| 2026-10-19 | task 2: edits synced with ack and rollback | add node, drag link, float to str refused in the browser, id conflict rolled back with Python's reason, all by hand in the browser pane; 55 Python tests at 100%, 36 Rust tests; 13 link cases and the message fixture run on both sides |
| 2026-10-19 | task 1: node types reach the browser | `@node` hints become ports; snapshot over `/ws`; Rust parses it and draws both nodes; 27 Python tests at 100%, 13 Rust tests; one JSON fixture parsed by both sides |
| 2026-10-19 | skeleton: Python serves, WASM draws, core logic runs | page shows `float to str link allowed: false`; 4 Python tests at 100%, 4 Rust tests; WASM 706 KB unoptimised |
| 2026-10-21 | canvas-accessibility task 27: the hidden tree, focus ring and pan-to-focus | native tests: 21 in `rust/element-51-core/tests/access.rs` (reading order, labels, port lists with their links, the cheap list equals the full one, nearest node in each direction with ties by id, the pan to bring a box into view at any zoom and pan); by hand in the browser pane on `tools/ui_states.py`: `tools/ui_keys.js` 18 of 18 rows pass (tree shown, ports built only when entered, one tab stop, Right enters ports, Down and Left move, PageDown, End and Home, leaving a node closes its ports), the ring seen on a node and on a port in the dark and light themes; axe-core 4.14.0 (the copy from the ui-design-pass download): 0 violations in default, node focused, port focused and after Run, in all three themes (one transient `color-contrast` hit in the light theme 300 ms after a theme switch did not come back at 800 ms; the page has a 120 ms colour transition); timing at 2000 nodes (`tools/bench_page.py` with 100 rows): an edit that rebuilds the tree took 36 to 42 ms in the page (a wheel redraw 14 to 18 ms), under the 50 ms budget; at 200 nodes a wheel redraw is 1.37 ms median (range 1.02 to 1.69) with the tree and 1.34 without it, the two builds timed side by side because today's pane gave 1.3 ms where the recorded baseline says 0.21 to 0.22 ms; 226 Python tests at 100% |
| 2026-10-21 | canvas-accessibility task 28: select, delete and undo by keyboard, with a status line | native tests: 5 more in `tests/access.rs` (26 in all): a sentence for each of the six edit kinds (value set with a number and with text, value cleared), undo and redo done and not possible, a refusal, a selection, and why Delete does nothing on an output port or an unlinked input; by hand in the browser pane on `tools/ui_states.py` with the page focused: `tools/ui_keys.js` 30 of 30 rows pass (Enter and Space select and say "Selected nX", Delete on a linked input port removes its link and the port then says "not linked", Ctrl+Z puts it back and says "Undid the last change", Delete on an output port says why nothing happened, Delete on a node removes it and focus moves to a neighbour, Ctrl+Z brings it back); axe-core 4.14.0 (the copy from the ui-design-pass download): 0 violations in the default theme in the default state, node focused, port focused, after Enter and after Run (the other two themes were not rescanned: the new line is hidden and has no colour); the Rust and Python suites unchanged apart from the new tests |
| 2026-10-21 | canvas-accessibility task 29: link by keyboard | native tests: 2 more in `tests/access.rs` (28 in all): every case in the shared `link_cases.json` gets the same verdict through `link_edit` (the edit the mouse and the keyboard now both build) as `rules.rs` gives, and the three link sentences; by hand in the browser pane on `tools/ui_states.py` with the page focused: `tools/ui_keys.js` 42 of 42 rows pass (Enter on an output port starts a link and says where from, Escape cancels, Enter on an input with none started says so, a link the rules refuse says "Refused: cannot link str to float" in the status line and in the banner and ends the pending link, a fitting link says "Linked n1 out to n3 a", two Ctrl+Z put the graph back); the pane shows the wire from the started output to the focused input, with the input marked red when the rules would refuse it (an input that already has a link) and green when they would allow it; axe-core was not run again (no new element and no new colour); Rust and Python suites unchanged apart from the new tests |
| 2026-10-21 | canvas-accessibility task 30: move, pan and zoom by keyboard | native tests: 4 more in `tests/access.rs` (32 in all): a node moves one grid step (24) or four in each direction, the view pans 48 px toward the key (so the graph moves the other way), repeated zoom steps of 1.25 stop at the largest and smallest zoom, the zoom and pan sentences (whole percentages); by hand in the browser pane on `tools/ui_states.py` with the page focused: `tools/ui_keys.js` 54 of 54 rows pass (Shift+Arrow and Alt+Shift+Arrow move a node and say "Moved n1", the node keeps the focus, two Ctrl+Z undo the two moves and a third says "Nothing to undo", so each press was one undo step; Alt+Arrow says "Panned left" and "Panned up"; `+` gives 125 and 156 percent, `-` 125, `0` 100); a screenshot after moving n5 down and left by four steps each, panning and zooming shows the card, the wires and the value boxes in their new places; axe-core was not run again (no new element and no new colour); suites unchanged apart from the new tests. Spec detail: Alt+Arrow pans toward the key; the zoom keys act about the middle of the window |
| 2026-10-21 | canvas-accessibility task 31: the add-node and card menus by keyboard | native tests: 4 more in `tests/menu.rs` (18 in all there): `step_menu` steps down and up and stops at the ends, with nothing focused down goes to the first and up to the last, an empty list has none, a stale position is brought back in; by hand in the browser pane on `tools/ui_states.py` with the page focused: `tools/ui_keys.js` 71 of 71 rows pass (N opens the add-node menu with the cursor in the search box and says so, Down goes to the first item, Right opens a category's flyout and moves into it, Left closes it, Escape closes the flyout first and then the menu and the focus returns to the tree item it came from, picking an item adds a node and says "Added node n7, ten" and the focus comes back, Ctrl+Z removes it, Shift+F10 and the Menu key open the card menu beside the node's icon and say "Actions for n1", Escape closes it and returns to the node); a screenshot shows the menu with Math open, its flyout beside it and the ring on the focused item; axe-core 4.14.0: 0 violations in the default theme with the add-node menu open at the search box, on an item and with a flyout open, and with the card menu open (the other two themes were not rescanned: no new colour); suites unchanged apart from the new tests. Decisions: a menu closes on Tab (the focus goes back first); menus open by mouse leave no `data-return`, so they never move the focus; the list stops at its ends (no wrap), and Up from the first item goes to the search box; the states graph has a card menu with one action (Delete node), so the walk-through checks Down only stays inside it |
| 2026-10-21 | canvas-accessibility task 32: value boxes by keyboard | native tests: the two new sentences in `tests/access.rs` (33 in all; the "no link started" sentence went, because Enter on an input with no link pending now edits it); by hand in the browser pane on `tools/ui_states.py` with the page focused: `tools/ui_keys.js` 80 of 80 rows pass (Enter on an unlinked input puts the cursor in its box, which is named "text of node n6", Escape puts the old text back and returns to the port, Enter after a value sends it and says "Set text of n6 to 7" and returns to the port, Ctrl+Z takes it away, Enter on a linked input says it has no box and stays); with real typing from the browser pane's keyboard ("real typing" then Return) the value was sent, the status line said "Set text of n6 to real typing" and the focus was on the input port again; axe-core 4.14.0: 0 violations with a value box focused and back on the port, default theme; suites unchanged apart from the new tests. Decisions: the box remembers its text in `data-original` only when the keyboard opened it, so a box the mouse focused behaves as before (Enter blurs, Escape does nothing); the walk-through fires `change` by hand because the browser does not when a script sets a value |
| 2026-10-21 | canvas-accessibility task 33: the end of a run is said through the status line | native tests: 4 more in `tests/access.rs` (37 in all): a clean run ("Run finished: 2 ok"), a mixed run with counts, plural errors, skipped and the first error ("Run finished: 2 ok, 2 errors, 1 skipped. First error: n3: ZeroDivisionError: float division by zero"), one error without a message ("failed") and an empty graph; by hand in the browser pane on `tools/ui_states.py` with the page focused: `tools/ui_keys.js` 82 of 82 rows pass, and a run said "Run finished: 3 ok, 2 errors, 1 skipped. First error: n5: returned str, expected float" while the banner listed both errors with Dismiss; axe-core 4.14.0: 0 violations after Run in the default theme. The goal is done: see PLAN.md and the outcome notes in the spec. To try it with VoiceOver (not verified here): turn it on with Cmd+F5, open an example (for example `uv run examples/d_editor/d2_values.py`), press Tab until VoiceOver says "Graph", use the arrow keys, Enter on an output port then on an input port to link, Delete to remove, `N` for the add-node menu |
| 2026-10-21 | visual-ports task 34: a picture on a card, end to end | tests: 22 in `tests/test_visuals.py` (the `Image` record: size and pixel count checks, 256 by 256 at most, pixels as a JSON list both ways, frozen; the `image` kind links only to itself; `-> Image` gives one output `out` and `display` image; a picture input takes no default; a run puts the picture in `shown` and not in `outputs`; a picture reaches the next node; an unlinked picture input and a bad picture are errors), 17 in `rust/element-51-core/tests/display.rs` (the geometry, `rgba` for grey and colour and for pixels that do not fill the picture, the wire shape, the link rule, no box and no typed value, the shared fixtures, messages without the new fields still read), 4 more in `tests/access.rs` (the "Shows a 28 by 28 picture" label, two pictures, a list with none, the picture input sentence), the AA test now covers the image colour in every theme and palette, three new cases in `link_cases.json` on both sides, `messages.json` and `snapshot.json` carry a picture; 262 Python tests at 100%, 215 Rust tests; by hand in the browser pane with `examples/g_experiments/g1_picture_and_plot.py`: the cards have an empty picture area before a run, Run draws a left-to-right ramp on `n1` and its negative on `n2` with crisp pixels and `n3` shows 784, dragging a picture output to an int input is refused with "cannot link image to int" and Dismiss, the hidden tree says "Shows a 28 by 28 picture." on `n1` and `n2` and not on `n3`, the console had no errors; the keyboard walk-through (`tools/ui_keys.js`) passes all 82 rows. Redraw, same script (`tools/bench_page.js` batches: 11 sets of 100 wheel events) in this pane (1024 by 768 CSS pixels at device ratio 2): the 200-node page gives 1.33 to 1.40 ms (range 1.17 to 1.92) with task 34, 1.40 (1.17 to 1.63) on `main` before it and 1.46 (1.31 to 1.76) at `e90563a`, before the accessibility work; so the 0.21 ms recorded on 2026-10-20 does not reproduce at any of them (cause not found) and task 34 changes nothing within the spread. 12 picture cards in view (a test server, three sets): 2.6 to 2.85 ms with value boxes under them; with no value boxes, 2.35 to 2.45 before Run (empty areas) and 2.25 to 2.40 after Run (pictures drawn), so drawing a picture costs nothing measurable and the card itself is what costs (about 0.15 ms each, not isolated); the budget of 1.0 ms is not met. axe-core was not run: task 38 scans the new states. To try it: `uv run python tools/build_wasm.py`, then `uv run python examples/g_experiments/g1_picture_and_plot.py` and press Run |
| 2026-10-21 | visual-ports task 35: a series and a one-line chart | tests: 14 more test functions in `tests/test_visuals.py` (36 in all there: the `Series` record, NaN as `null` and back, 100,000 points at most with a clear error, text or a number instead of a list refused, the `series` kind links only to itself, `-> Series` gives one output and no display, a series input takes no default, `PlotNode` has one input and a chart display, a series reaches the next node, a plot shows the linked series by input name, an unlinked plot is ok and shows nothing, a gap reaches the page as `null`, `shown` reads back as the right record), 1 in `test_protocol.py` and 1 in `test_examples.py` for the chart; 21 more in `rust/element-51-core/tests/display.rs` (the `Series` and `Shown` records, the chart card's height and area, `short_number`, `nice_ticks`, `thin` (short series, gaps, min and max kept, an empty part breaks the line), `last_value`, `line_style`, `chart_layout` (plot inside the area, a falling line runs top left to bottom right, two lines share a scale, a flat series, one point, a gap, nothing to draw, 100,000 points thinned to about twice the width)) and 2 in `tests/access.rs` (the plot label with one and two lines, no value yet, an empty chart); 280 Python tests at 100%, 238 Rust tests, the keyboard walk-through 82 of 82; by hand in the browser pane with `examples/g_experiments/g1_picture_and_plot.py`: Run draws the falling ripple on `n5` with a scale on both axes and "out 0.0842" at its end, `steps` 300 makes the line longer and the x axis reach 300, `blow_up` stops the line at step 40 with a gap and no error, the hidden tree says "Shows a plot of 1 line, out last 0.0842.". Draw time, same script and pane as the task 34 row (`tools/bench_page.js` batches on a test server with one `big` node of 100,000 points linked to a Plot): the redraw median is 0.53 ms before Run (the chart area empty) and 2.1 ms after (3 sets: 2.10, 2.13, 2.15), so the 100,000-point chart costs about 1.6 ms a frame against a budget of 4 ms. axe-core was not run: task 38 scans the new states. To try it: `uv run python tools/build_wasm.py`, then `uv run python examples/g_experiments/g1_picture_and_plot.py` and press Run |
| 2026-10-21 | card-polish task 39: wires see-through over cards | native test: every built-in theme has `wire_over_card` strictly between 0 and 1 (0.5) and a theme file can change it (the docs test already needs its row in `docs/themes.md`); 239 Rust tests; by hand in the browser pane with the picture example after Run: the wire from `n1` to `n3` is solid on the canvas and, where it crosses `n2`, the `source` and `out` boxes show through it. Redraw, the 200-node page, same script and pane as the task 34 row, three sets: medians 1.59, 1.53 and 1.57 ms (ranges 1.40 to 2.07, 1.49 to 1.57, 1.23 to 1.85) against 1.33 to 1.46 before: about 0.15 ms more, because every wire is now stroked twice; the second pass is clipped to the cards but not culled. Not verified: the other two themes by eye (the key is the same in all three) |
| 2026-10-21 | card-polish task 40: a linked input shows its value | native tests: 8 more in `tests/fields.rs` (14 there): before a run only `n1.out`, after a run `21.0` then `[n1.out]` (a string keeps its quotes), a source that failed or was skipped, a picture or series source and a port the outcome lacks show only the source, `share_room` keeps both widths when they fit, cuts the source first, and leaves it nothing when the value fills the box; 247 Rust tests; by hand in the browser pane with `examples/a_basics/a3_arithmetic.py`: before Run the boxes say `n1.out`, after Run `3.0 [n1.out]`, `7.0 [n3.out]`, `21.0 [n4.out]` with the value in the box text colour and the source dimmer. Redraw, the 200-node page, same script and pane: median 1.52 ms before Run and 1.61 to 1.64 after (values drawn in every linked box in view), about 0.1 ms more |
| 2026-10-21 | card-polish task 41: View source code in the card menu | tests: 8 in Python (`test_session.py`: the source of a function node with its decorator, of a class node, of `PlotNode`, dedented when defined inside a function, an unreadable one gives a reason and no code, a node that is not there; `test_server_websocket.py`: the reply goes to the asking page only; `test_protocol.py`: the request and both replies round trip through the shared `messages.json`, and the client union takes a source request), 7 in Rust (`tests/source.rs`: the shared fixture on both sides, the request text, the title, line count and sentence for code, a reason, and a reply with neither; `tests/menu.rs`: the card offers View source code then Delete node and a view is not an edit); 288 Python tests at 100%, 254 Rust tests; the keyboard walk-through 89 of 89 (7 new rows: the first entry is View source code, the panel opens with the code focused, its title and sentence, the code is shown, Escape closes it and returns to the node, "Source closed" is said, Dismiss closes it); by hand in the browser pane on `tools/ui_states.py`: Shift+F10, Enter on View source code opens "Source of n2 (zero)" with the decorated function in a read-only box with Dismiss and the focus ring on the code; axe-core 4.14.0: 0 violations with the panel open and closed, and with it open in all three themes. Not verified: a real screen reader, and the panel on a narrow window. The goal is done: wires, linked values and source (tasks 39 to 41); renaming was left out on purpose |
| 2026-10-21 | visual-ports task 36: rows that grow with links | tests: 14 in `rust/element-51-core/tests/rows.rs` (the row number rule, `find_input` for a row that is not listed, a type that does not grow is borrowed, a plot with nothing linked has one empty row, each link adds a row, rows follow their numbers and the empty row is one past the highest, removing a middle link keeps the other names, deleting the feeding nodes shrinks the card, inputs that are not rows stay first, the resolved type, the card is one row taller per link, the empty row can be hit and a link lands on it, a link ends on the row it names after the rows above it change, line names with and without a shared name), 3 in `tests/access.rs` (the grown plot's ports and its "2 inputs" label, two lines with one name get node ids, row 10 follows row 9), 7 shared cases in `link_cases.json` run on both sides (rows 1 and 3 accepted, `series_0`, `series_01`, `other_1` and `series_x` refused as unknown ports, float to a row refused), 11 test functions in `tests/test_visuals.py` (the row rule, `find_input`, a plot shows every linked row, row 2 alone); the existing `layout.rs` tests pass unchanged; 310 Python tests at 100%, 271 Rust tests, the keyboard walk-through 89 of 89; by hand in the browser pane with `examples/g_experiments/g1_picture_and_plot.py`: the Plot card shows `series_1`, `series_2` and an empty `series_3` with two named lines (`loss`, `accuracy`), dragging `accuracy` onto the empty row grows the card to `series_4` and Run draws a third line with the two equal names told apart (`n4 accuracy`), Delete on the `series_2` port in the hidden tree removes the link and its row, the card shrinks and the rows read `series_1`, `series_3`, `series_4`. Redraw, 200-node page, same script and pane: 1.51 to 1.54 ms (three sets, 1.44 to 1.59), the same as before the change (1.52 to 1.64 with values shown), so looking a type up through `type_of` costs nothing measurable. axe-core was not run: task 38 scans the new states. Not verified: undo and redo of a link to a row by eye (the edits are the same ones undo already handles) |
| 2026-10-21 | box-summaries: a picture or series says what it holds | tests: 3 in `rust/element-51-core/tests/display.rs` and `run.rs` (a grey and a colour picture, a series gives its last number, skips a trailing gap or NaN, all gaps, and `output_text` for a picture, a series and a plain value), 1 Python test changed (`test_a_series_reaches_the_node_it_is_linked_to`: the producer's `shown` now holds the series); 310 Python tests at 100%, Rust tests pass and clippy is clean; by hand in the browser pane with `g1_picture_and_plot.py` on port 8766 after Run: `ramp` out `28 x 28 grey`, `invert` source `28 x 28 grey [n...`, `training` loss `last 0.0842` and accuracy `last 0.906`, the Plot inputs `last 0.0842 [n4...`. Not verified: redraw time (no new drawing, one string per linked box), a narrow window |
| 2026-10-21 | code-colours: the source panel is coloured | tests: 11 in `rust/element-51-core/tests/highlight.rs` (the runs put back together are the code; keywords, names, numbers and comments; a decorator with its dots; a word that only contains a keyword stays plain; numbers with exponents, underscores, hex; string prefixes, escapes and triple quotes; a word before a quote that is not a prefix; half-typed code; a comment and a string hide each other; text that is not Python; the class names), 2 existing tests in `tests/source.rs` extended (the spans are the code, a reason is one plain run); the contrast and palette tests cover the five new colours in all three themes and every palette (4.5:1 on `menu_field`), `docs/themes.md` has the five rows; 11 new Rust test functions, Python tests unchanged at 310; the keyboard walk-through 90 of 90 (one new row: the code has keyword and name runs); by hand in the browser pane with `g1_picture_and_plot.py` (dark theme): `@element_51.node`, `def`, `training`, strings, numbers and `if`/`else`/`for` each in their colour. Not verified: axe-core (no copy at hand; the new colours are covered by the native contrast test, no new element or role), the light and synthwave themes by eye, a real screen reader, a narrow window |
| 2026-10-21 | visual-ports task 37: Save as PNG | tests: 4 in `rust/element-51-core/tests/menu.rs` (the card offers View source code, Save as PNG, then Delete node; neither of the first two is an edit; Save as PNG is offered only for a picture or a chart; the file name for a picture and a chart, and a port name with a slash, dot, space or non-ASCII letter cannot reach a path); Rust and Python suites otherwise unchanged (310 Python at 100%); the keyboard walk-through 90 of 90; by hand in the browser pane with `g1_picture_and_plot.py` on port 8766 (the download click was caught by a script, so no file reached the Downloads folder): before Run, Save as PNG on `ramp` shows "n1 has nothing to save yet: press Run first." with Dismiss; after Run it makes `n1-out.png` (28 by 28) and, on the Plot, `n5-chart.png` (800 by 528), which looks like the card with its scale, both lines and their end labels; a card without a display (`training`) has no Save entry. Not verified: a real download into the Downloads folder in the user's browser, the keyboard path to the entry (task 38 adds the walk-through rows and the axe scan), redraw time (no change to drawing a frame) |
| 2026-10-21 | visual-ports task 38: accessibility and docs | `tools/ui_states.py --graph display` serves a picture, its negative, a node making two series and a Plot with two rows linked and one empty; `uiKeysDisplay` in `tools/ui_keys.js`, 16 rows, 16 of 16 (a picture card offers View source code, Save as PNG, Delete node; Down reaches Save as PNG; Escape returns to the node; a plain card has no Save entry; Save before a run says there is nothing to save, with Dismiss; after Run the labels say `Shows a 28 by 28 picture.` and `Shows a plot of 2 lines, loss last ... and accuracy last ...`; the Plot lists `series_1`, `series_2` (linked) and an empty `series_3`; Save on the picture makes `n1-out.png` and on the Plot `n4-chart.png`, with a PNG data URL, and says "Saving n1-out.png"; the focus returns to the node; the chart file is 800 by 528); the standard walk-through 90 of 90 (unchanged); axe-core 4.14.0 on the display graph: 0 violations in 7 states (default, a tree node, a tree port, after Run, the card menu, the source panel, search with no match) in each of `antimony`, `antimony-light` and `purple-synthwave`; `docs/visual-ports.md` and a link from the index, `uv-docs build` with 0 warnings. A real bug was found by the new walk-through and fixed in its own commit: after `box-summaries` the hidden tree said a node that only makes series (`training`) "Shows a plot of 2 lines", because `with_displays` read every series in `shown` as a chart; a series now counts only where it is linked to an input (`access.rs`, one new native test). Redraw, 200-node page, same script and pane (11 sets of 100 wheel events, after tasks 37 and 38, box-summaries and code-colours): median 1.54 ms, range 1.36 to 1.66, inside the 1.3 to 1.64 ms recorded since task 34 (this graph has no run, so the new box text is not drawn; with values shown, task 40 added about 0.1 ms). Not verified: a real screen reader, the picture and chart on a narrow window or by touch, a real file landing in the Downloads folder |
| 2026-10-21 | speed-up-frame task 1: draw only the wires that can reach the view (`wire_bounds` in `layout.rs`, used by `draw_links`) | `tools/bench_page.py` (200 nodes, 12 in view), `tools/bench_page.js` batches (11 of 100 wheel events), a throwaway build with a `data-nocull` switch on the canvas and four interleaved rounds in one page session: 1.53, 1.56, 1.56, 1.62 ms before (culling off), 1.37, 1.31, 1.30, 1.31 after, about 0.24 ms or 15% faster, above the 0.06 spread inside a session. Pixels: the canvas of the new and the old build hashed the same (SHA-256 of `toDataURL`) in 7 states (first frame, zoom out, drag pan, zoom in at a corner, pan far left, zoom far out, pan down right). Tests: 4 new in `rust/element-51-core/tests/layout.rs` (the box holds the control points and half the thickest line, a wire inside, a wire crossing with both ends outside, a wire wholly outside on each side); 292 Rust tests. Runs of the same build on different page loads gave 0.10, 7.0 and 1.5 ms, so only interleaved comparisons count (see Gotchas) |

| 2026-10-21 | speed-up-frame task 2: the grid's dots as one path with one fill (`grid` in `draw.rs`) | 12 picture cards, `tools/bench_page.js` batches, a throwaway build with a `data-oldgrid` switch, four interleaved rounds in one page session: 1.42, 1.31, 1.34, 1.40 ms before, 1.00, 0.94, 0.96, 0.92 after, about 0.4 ms or 30% faster; the 200-node page did not change (0.26 to 0.32 ms both ways in that session). Not pixel-identical, and the user accepted it (2026-10-21): 3,029 of 3,145,728 pixels differ, at most 27 of 255 per channel, on the anti-aliased edge of the dots (one fill of many arcs is rasterised slightly differently from many fills). No new test: the web crate has none and the grid has no native logic. Whole set after tasks 1 and 2, clean build, 4 sets of 11 batches, medians: 200 nodes 1.32, 1.40, 1.42, 1.43 ms (before 1.55, 1.55, 1.51, 1.52); 12 pictures 0.96, 0.97, 1.04, 1.07 (before 1.22, 1.22, 1.16, 1.16), at the 1.0 ms budget but not clearly under it; chart 0.46 to 0.53 before Run (0.52 to 0.62) and 2.11 to 2.19 after Run (2.36 to 2.55), so no workload got slower |
| 2026-10-21 | `job-nodes` task 5: keyboard and screen reader for job nodes | `uiKeysJob` on `ui_states.py --graph job`: 21 of 21 rows pass; axe-core 0 violations in the default state, with a job button focused, with a job running, stopped and failed (banner shown) in `antimony`, `antimony-light` and `purple-synthwave`; no new theme colour (the job row uses `field`, `field_text`, `field_border`, `placeholder` and `label`, whose pairs the native AA test already covers) |
| 2026-10-21 | `job-isolation` task 8: a job in a child process (`isolated = True`) | tests: 15 in `tests/test_isolated.py` (the child entry called in the parent: a work job, a step job, Stop, a raise, a class that cannot be imported; real child processes: a work job and a step job finish with their outputs, Stop kills a job that sleeps 30 s and never reports within 1 s after Stop (it took about 0.7 s with a 0.2 s settle), a job that reports ends at its next report, `os._exit(3)` gives `failed` with `exit code 3`, a raise in the child gives `failed` with its reason, and no child is left after each test; refusals for a class in a function, a module off the import path and a notebook `__main__`; acceptance of an importable class and of a script's `__main__`), one in `tests/test_examples.py` (Stop on `Stubborn` in the job example, with its folder on the import path), a section in `tests/functional/jobs.ipynb` (the notebook refusal, then a module written to a scratch folder and stopped in under a second); 397 Python tests at 100% branch coverage (13 to 25 s over several runs on a busy machine), no coverage setting changed; by hand in the browser pane on port 8766 (`examples/g_experiments/g2_job_nodes.py` through a throwaway script): Start on `Stubborn` shows `running` with Stop, Stop shows `stopped` within a second, no python child process left. To try it: `uv run python tools/build_wasm.py`, then `uv run python examples/g_experiments/g2_job_nodes.py`, press Start and then Stop on `Stubborn` |
| 2026-10-21 | `plot-polish`: legend row, axes and grid, thicker lines, taller chart | 3 new native tests in `tests/display.rs` (the legend row sits above the plot inside the area, `legend_slots` splits a row equally, a 0.05 to 0.95 series gets at least four y labels), the chart height test now 0.85; 397 Python tests and all Rust tests pass, clippy and fmt clean; in the browser pane with `examples/g_experiments/g2_job_nodes.py`: before, one y label (0.5), no visible grid and labels over the lines; after, labels 0.2 to 0.8, a legend with swatches, the plot about twice as tall. Chart workload (`tools/bench_page.py --workload chart`, Run pressed, one set of 11 batches): 2.11 ms median (2.00 to 2.25) after, 2.11 to 2.19 before, so no change and under the 4 ms budget. To try it: `uv run python tools/build_wasm.py`, then `uv run python examples/g_experiments/g2_job_nodes.py`, press Start on `Training` |
| 2026-10-21 | `auto-run` task 2: the `Run automatically` box | native: the rule's 8 tests (task 1); by hand in the browser pane: `uiKeys` 90 of 90, `uiKeysAuto` 19 of 19 (labelled box, unticked at load, tab order Run then box then tree, Run `aria-disabled` while ticked and a press does nothing, one run 302 ms after an edit, three edits 60 ms apart give one run, an edit during a held run says `Still running, will run again` and starts exactly one more run when it ends, unticking restores Run and stops runs); light theme looks right; the sentence also shows on the Run button (`Running, will run again`) so sighted users see it; axe, a screen reader and touch not run |
| 2026-10-21 | `auto-run` task 3: a red border on a node whose last run failed | native theme tests (key in all three themes, docs row, `card_error_border on canvas` at 3:1, every palette at 0 failures); by hand in the browser pane (dark): `n3` and `n5` get the border beside the red title bar; the banner replaced its text on a new failing run (`n4: input 'angle' has no value`) and hid after a passing run (`Run finished: 3 ok`); redraw at 200 nodes 1.52, 1.53 and 1.67 ms, inside the known 1.3 to 1.6 ms plus its spread; the light theme and palettes not looked at by eye |
| 2026-10-21 | `mnist-experiment`: `examples/g_experiments/g3_mnist.py` | 31 tests in `tests/test_mnist_example.py` on fake data (reader errors, `.part` and sha256 refusal, finite-difference gradients, seed, learning a separable set, sample, settings, a Start that fills the Plot) and a row in `tests/test_examples.py`; real data downloaded after the user's yes (MNIST 11.6 MB from Google's cvdf-datasets, identical sha256 to PyTorch's S3 copy; Fashion-MNIST 30.9 MB from Zalando's GitHub, md5 equal to the dataset README's), sha256 pinned in the example; measured on this Mac (numpy 2.5.4): MNIST at 64 hidden, 10 epochs 1.8 s 97.21%; Fashion the same 1.7 s 86.72%; defaults (128 hidden, 30 epochs, 282 points) MNIST 6.8 s 97.73% (first point 85.8%, loss 0.887 to 0.0106), Fashion 7.1 s 87.77%; hidden 16 3.2 s 95.36%; learning rate 0.01 6.8 s 96.20%; 1.0 6.9 s 97.52%; by hand in the browser pane: Run automatically shows the digit, Start fills the Plot live (step 77 of 282 seen), `fashion` in the sample swaps to a boot and its linked `dataset` follows, `epochs` 0 fails with its reason |

## Gotchas

- Property tests (Hypothesis, since 2026-10-21) cover the six places Python reads outside data
    or has a dump and load pair: `ClientMessage` (`test_protocol.py`), `Image`, `Series` and
    `RunResult` (`test_visuals.py`), `load_graph` and `save_graph` (`test_store.py`), and
    `resolve_theme` (`test_theme.py`). A test that needs a file makes its own folder with
    `tempfile.TemporaryDirectory()` inside the test body: the `tmp_path` fixture lasts for the
    whole test, so Hypothesis refuses it (`function_scoped_fixture` health check) and the
    standard forbids suppressing that. A series round trip must allow NaN (a gap) and compare
    with `same_values`, because NaN is not equal to NaN, and must exclude infinity, which
    pydantic writes as `null` and reads back as NaN. The Rust side has its own cargo tests.
- `cargo test` stops at the first test file that fails, so the files after it never run
    and the total looks smaller: a count of 166 instead of 215 once hid `theme.rs`.
    Run `cargo test --no-fail-fast` to see every failure.
- A change to the shared `link_cases.json` graph changes `next_id` there (now `n10`), which
    `rules.rs` and `test_edits.py` pin, and the "unknown node" cases name `n99` because any
    lower id may exist; add nodes there only with a reason.
- A synthetic `KeyboardEvent` for Enter does not press a button, so a walk-through row
    that "presses Enter" on a menu entry must `click()` the focused entry (a real Enter
    does the same). The hidden tree and the menus read keydown events, which synthetic
    ones do reach.
- `uv publish` asks for the username first. Type `__token__`, then paste the token at
    the password prompt (nothing is echoed). A token pasted at the username prompt is
    echoed on screen and refused (happened 2026-10-21; that token had to be revoked).
    The safe form is `read -rs UV_PUBLISH_TOKEN && export UV_PUBLISH_TOKEN && uv run
    python tools/check_wheel.py && uv publish; unset UV_PUBLISH_TOKEN`, which asks for no
    username. A token scoped to the project can be made once a first version (the 0.0.1
    placeholder) exists. The PyPI JSON listing can lag the upload by seconds.
- PyPI shows what a release carried: the README as it was when built, its
    `[project.urls]` and its classifiers. A shields.io `pypi/pyversions` badge reads
    classifiers, so it said "python missing" until the package had some (added after
    0.1.1). README images need absolute `raw.githubusercontent.com` addresses to show on
    PyPI. None of this can be fixed on a published version, only by a new one.
- The Docs workflow (`.github/workflows/docs.yml`) fails at "Set up Pages" until GitHub
    Pages is on for the repo: `gh api -X POST repos/Jacopo-DM/element-51/pages -f
    build_type=workflow`, or Settings, Pages, Source "GitHub Actions". The user did it
    on 2026-10-21.
- `uv_build` has no build hooks, and `src/element_51/web/` is generated and
    git-ignored, so `uv build` in a fresh clone makes a wheel without the editor;
    `tools/check_wheel.py` is the guard (a clean clone fails it, as it should).
- The README images (`docs/images/dark.jpg`, `light.jpg`, `palette.jpg`) are the
    editor's canvas exported with `canvas.toDataURL` (2560 by 1440 at 2x, cropped to
    2400 by 1200 and resized to 1600 by 800), not browser-pane screenshots: the pane's
    screenshots come back to Claude at 800 px and are not saved to disk, and a
    macOS `screencapture` would take the user's whole screen. The page's value boxes
    and buttons are HTML on top of the canvas, so an export leaves unlinked inputs
    blank; the demo graph therefore links every input. To redo them: serve a graph on
    port 8766 with an extra POST route that writes the body to a file, press Run, and
    `fetch` the data URL to it from the page; the palette shot is the Try palette
    box with `https://coolors.co/acadbc-9b9ece-6665dd-473bf0-000500`; then
    `tools/compress_images.py docs`.
- The user's VS Code can keep a stale unsaved buffer of a file that changed on
    disk (seen on `f1_fastapi_host.py` after a rewrite); saving it overwrites the
    new text. Say so when a file the user has open is rewritten.
- The 22 transitive Rust crates come from wasm-bindgen (`futures-*`, `slab`,
    `bumpalo`, `syn` 3.0.9 and friends) and serde (`serde_derive`).
- A page measurement is only comparable with another taken in the same state of
    the browser pane (visible or hidden): the same 200-node redraw measured 3.4 ms
    in a hidden pane, 15 ms visible with painting, and 14 ms when each batch
    yielded to the browser. `tools/bench_page.js` therefore runs its batches back
    to back, without yielding, and times the recording work only (3.3 to 3.5 ms
    before the culling, 0.21 ms after).
- `axe-core` reported a `scrollable-region-focusable` (serious) once, on the
    add-node menu's results list with no match; it did not return after the page
    was wrapped in `<main>` and the cause was not found.
- jscpd (the clone detector for the restructure pass) is not a dependency and
    `npx` cannot fetch it here: node fails with `UNABLE_TO_GET_ISSUER_CERT_LOCALLY`
    while `curl` reaches the registry. The user approved the download on
    2026-10-20. I fetched `jscpd-5.4.2.tgz` (5 KB, a launcher) and
    `jscpd-darwin-arm64-5.4.2.tgz` (8.8 MB packed, 43 MB unpacked) with `curl`
    from `registry.npmjs.org`, checked both against the registry's sha1, unpacked
    them into the scratchpad and ran `package/bin/jscpd` directly. Nothing was
    installed and nothing was added to the repo. Redo it the same way; do not
    turn off certificate checks.
- Removing a focused tree item makes the browser fire `focusout`, which cleared `State.focus`
    in the middle of `sync_tree`, so a rebuild under a focused port (a deleted link) lost the focus
    and the port list. `sync_tree` now marks the tree as built first (so the `focusout` handler's
    redraw does not rebuild again), saves `State.focus` before clearing the tree and puts it back
    after (found while building task 28, 2026-10-21).
- The browser pane's redraw time is not repeatable between page loads (2026-10-21, one
    build, the 200-node page: 0.10, 7.0 and 1.5 ms medians on three loads), most likely
    because canvas drawing is queued and its cost lands wherever the flush happens. Compare
    two versions only inside one page session, with a switch (a `data-` attribute on the
    canvas read by a throwaway build) and interleaved rounds. Timing the parts of a frame with
    `performance.now()` is unreliable for the same reason: the cost of a section moved to
    the next one that flushed. An ablation (switch a part off, time the whole frame) works.
- A page's `JobUpdate` must be applied to its last run result, not replace it: replacing it lost the
    run after another job (and a plain Run) at every report. Found by running
    `examples/g_experiments/g2_job_nodes.py` with both jobs going; `uiKeysJob` does not run two jobs at
    once, so it does not catch it.
- A pydantic generic such as `JobNode[int]` looks up the module of the class that uses it in
    `sys.modules`. A module loaded with `importlib.util.module_from_spec` and `exec_module` and
    not registered there fails with `KeyError: '<module name>'`: `tests/test_examples.py` registers
    each example before running it, and a job class made by `exec` has the same problem (docs/jobs.md).
- A spawned child gets the parent's `sys.path` and re-imports the main script as `__mp_main__`
    (and aliases it as `__main__`), so a script that starts the editor without the
    `if __name__ == "__main__":` guard starts it again in every child. `tests/test_isolated.py`
    relies on `tests/` being on `sys.path` (pytest puts it there) so that the child can import
    `sample_jobs`; `tests/test_examples.py` adds the examples folder with `monkeypatch.syspath_prepend`.
- `importlib.util.find_spec` finds a module that is only in `sys.modules`, which a fresh child
    would not. `check_importable` uses `importlib.machinery.PathFinder.find_spec(top, sys.path)`
    for that reason.
- A test that reads a source file by name (`include_str!`) breaks when the code moves out of
    that file. After moving web or core code, run the Rust tests before committing: the
    no-hex-colour scan lists its files by name (add each new web file that holds drawing or
    menu code), and the palette, page-hidden and similar tests read one file each
    (found while splitting `menu_ui` and `lib`, 2026-10-21).
