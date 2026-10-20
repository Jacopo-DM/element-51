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

`a1_hello_world.py` is the project template and keeps its numpy and data setup;
`tests/test_examples.py` skips it by name.

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
item starts with a bare dash. There is no remote, so the workflow has not run on
GitHub; the same commands, the YAML parse and the wheel check were run locally.

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

## Results

| Date | Change | Result |
| --- | --- | --- |
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

## Gotchas

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
