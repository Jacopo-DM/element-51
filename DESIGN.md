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

### antimony-theme (goal approved 2026-10-19; spec not yet written)

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
- Renamed from `patchbay` to `element-51` on 2026-10-19: `patchbay` is taken on
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
  what the user just did. `App` itself is not thread-safe: a script that
  edits while the server runs should do so from one thread, and a browser
  edit racing a script edit with the same id is refused with a reason.
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
  a reach of half the horizontal gap, at least 20 units (was 40). The port
  markers take the same two colours. Port squares and input boxes were left as they were by the
  user's choice. The radius is a constant in `draw.rs` until the Theme exists.
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
- Reference screenshots of Antimony stay outside the repo; they are not ours to
  redistribute.

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
- starlette's `TestClient`: needs `httpx` as a new dev dependency; tests use a
  real uvicorn server and the `websockets` client instead (see Decisions).

## Results

| Date | Change | Result |
| --- | --- | --- |
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

- The 22 transitive Rust crates come from wasm-bindgen (`futures-*`, `slab`,
  `bumpalo`, `syn` 3.0.9 and friends) and serde (`serde_derive`).
