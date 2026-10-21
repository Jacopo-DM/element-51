# element-51

This file adds to `~/.claude/CLAUDE.md`; it does not repeat it.

## Opening

Read PLAN.md first, then DESIGN.md, CODEMAP.md and the README of the module you
will change. If a file and the code disagree, trust the code, fix the file, and
say what was wrong. If a file and my instructions disagree, stop and ask.

## Where facts live

Each fact lives in exactly one file.

- CLAUDE.md: rules and commands that apply to every session in this repo.
- PLAN.md: the state of the work, which goes stale as it progresses.
- CODEMAP.md: the structure of the code, derived from it and regenerable. Never
    plans, decisions, findings or measurements.
- DESIGN.md: reasoning and reference that stays true after a task is done.

To place a fact: if the code could regenerate it, CODEMAP.md. If it will be
stale once the work is done, PLAN.md. If it is a rule for every session in this
repo, CLAUDE.md. If it is a rule for every repo, `~/.claude/CLAUDE.md` (never
written unasked). Otherwise DESIGN.md.

## Commands

From the repo root. Python 3.13 (`.python-version`), uv 0.13.1. Rust is the
rustup `stable` toolchain, rustc 1.99.0. The Homebrew `rustc` on PATH (1.99.1)
has no WASM target and is not used. Every Rust command below is prefixed with
`RUST=$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin`, shown as
`R`, meaning `PATH="$RUST:$PATH"`.

| Purpose | Command | Time, notes |
| --- | --- | --- |
| Python format check | `uv run --locked ruff format --check` | under 1 s |
| Python lint (read only) | `uv run --locked ruff check --no-fix` | under 1 s; plain `ruff check` rewrites files |
| Python types | `uv run --locked mypy .` | about 1 s |
| Python types, advisory | `uv run --locked ty check` | under 1 s |
| Python tests, 100% coverage | `uv run --locked pytest` | about 2 s |
| Rust format check | `R cargo fmt --all --check --manifest-path rust/Cargo.toml` | under 1 s; `cargo fmt --all` rewrites |
| Rust lint | `R cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings` | about 1 s warm |
| Rust tests (native) | `R cargo test --manifest-path rust/Cargo.toml --no-fail-fast` | about 1 s warm; without the flag the files after a failing one do not run |
| Build the browser layer | `uv run python tools/build_wasm.py` | about 4 s; rewrites `src/element_51/web/` |
| Structure numbers | `uv run python tools/measure_structure.py [--jscpd <binary>]` | under 1 s; files and lines, files over 400 lines, modules over 5 imports; jscpd is not a dependency (downloaded by hand, see DESIGN.md) |
| Time the Python side | `uv run python tools/bench_python.py` | about 30 s; build, run and snapshot of a 2000-node chain, median and range of 11 runs; `--profile` prints a cProfile |
| Time the page | `uv run python tools/bench_page.py`, open http://127.0.0.1:8767, then run `tools/bench_page.js` in the page (Claude: the browser pane's javascript tool) | redraw ms per wheel event and page load ms; Ctrl+C stops the server; the pane may need its tab fronted |
| Serve every page state | `uv run python tools/ui_states.py [--theme antimony-light]`, open http://127.0.0.1:8768, then `await uiState("menu")` from `tools/ui_states.js` in the page | port 8768; states `default`, `menu`, `flyout`, `search`, `search-none`, `card-menu`, `run`; Claude takes the screenshot in the browser pane |
| Keyboard walk-through | paste the function from `tools/ui_keys.js` into the page of `tools/ui_states.py` (browser pane javascript tool), then `await uiKeys()` | returns pass and fail counts and the failing rows (a script tag works too: serve `tools/` with `python3 -m http.server 8769 --bind 127.0.0.1` and add `http://127.0.0.1:8769/ui_keys.js`; restart `ui_states.py` and reload the page first: the rows delete, move and link things and a failed run leaves them changed); the page's window needs the focus first (take a screenshot, then click the empty canvas), or the browser sends no focus events and the ring never moves |
| Accessibility scan | download `axe.min.js` (DESIGN.md, ui-design-pass spec), serve it on port 8769 (`python3 -m http.server 8769 --bind 127.0.0.1` in its folder), then run `tools/ui_axe.js` in the page of `tools/ui_states.py` (paste its body into the javascript tool as `await (async () => { ... })()`: loaded through a `<script>` tag its returned result is lost) | not a dependency; the numbers are in PLAN.md, Baseline |
| Build the docs | `uv-docs build` (or `uv run --locked --group docs sphinx-build -W --keep-going -b html docs docs/_build/html`) | about 5 s; warnings are errors, as in CI (`docs.yml`); needs `uv sync --group docs` first; writes only `docs/_build/` and `docs/api/`, both ignored |
| Preview the docs | `uv-docs` | live preview in the browser; rebuilds when `docs/` or `src/` change; Ctrl+C stops it |
| Check the wheel before publishing | `uv run python tools/check_wheel.py` | about 3 s; builds a wheel in a temp folder and exits 1 if `src/element_51/web/` was not built first (run `tools/build_wasm.py`) |
| Publish a release (the user runs it, never Claude) | `read -rs UV_PUBLISH_TOKEN && export UV_PUBLISH_TOKEN && uv run python tools/check_wheel.py && uv publish; unset UV_PUBLISH_TOKEN` | after `uv run python tools/build_wasm.py` and `uv build`, and a version bump in `pyproject.toml` and `uv.lock`; paste a PyPI token scoped to the project at the silent prompt, never at a username prompt, never in chat; then tag `v<version>` on the release commit |
| Serve the editor | `uv run examples/a_basics/a2_two_nodes.py` | needs the build first; opens the page in your browser; Ctrl+C stops it with exit 0 |
| Serve another example | `uv run python examples/a_basics/a3_arithmetic.py` (every example except `c1_script_api.py`, which prints and exits, serves on http://127.0.0.1:8765) | same port, so stop one before starting the next |

## Code rules

- Entry points and import direction are in CODEMAP.md.
- Python records are pydantic models; Rust message types derive serde and
    mirror them.
- Modules talk to each other only through their public interfaces.
- No compatibility shims.
- No abstraction before there are two real call sites.
- Clear names over clever one-liners.
- No options nobody asked for.
- Colours, sizes and fonts come from the `Theme` (`rust/element-51-core/themes/*.json`);
    the web crate and `index.html` hold no hex colour (a native test in
    `rust/element-51-core/tests/theme.rs` fails if one appears). The page's own
    parts use CSS custom properties such as `var(--menu-bg)`.
- A change to a wire record (snapshot, edit, node type) changes the shared files in
    `tests/fixtures/` too; both test suites read them.
- A new theme colour or a new colour pair needs: its key in `Colours` and in both
    theme JSON files, a row in `docs/themes.md` (a native test fails without it),
    a `--name` in `Theme::css_variables` if the page uses it, and a pair in
    `Theme::contrast_pairs` so the native AA test covers it (4.5:1 for text, 3:1
    for graphics); `tests/palette.rs` then checks it for every palette too.
- A feature the user can try gets a runnable example in `examples/`, in the group
    folder it belongs to (`a_basics`, `b_node_types`, `c_python_api`, `d_editor`,
    `e_look`, `f_embedding`, `g_experiments`; the file is named with the folder's letter and the next
    number), laid out like
    `examples/template.py` (the project template; copy it, it is not an example
    of the library): a docstring with a `Notes` section that
    says what to try, the comment headers (Standard library, Third-party libraries, Local
    libraries, Globals, Warnings, Typing, Terminal output setup), the rich traceback
    and console, `main()` and the guard. Leave out the random generator and the
    data folder unless the example needs them. Add a `build_app()` and a row in
    `tests/test_examples.py`.
    When a task is done, tell the user when they can try it, with the exact
    commands.

## Git and approvals

Differences from the global file only.

- Never publish to PyPI or crates.io (no `uv publish`, no `cargo publish`). The
    user publishes. The name `element-51` is already registered on both by the
    user.
- A task in PLAN.md is approved only where PLAN.md says so. A task added later
    needs its own approval in words.
- The repo is public on GitHub (`origin`, https://github.com/Jacopo-DM/element-51):
    nothing private goes into a commit, and every push (and tag push) needs the
    user's say-so, each time. Commits here use GitHub's noreply address
    (`30981226+Jacopo-DM@users.noreply.github.com`, set with `git config user.email`
    in this clone); the first 116 commits carry the user's own address on purpose.

## Ignore list

Do not edit, search or commit these.

| Path | Generated or hand-written | What makes it |
| --- | --- | --- |
| `src/element_51/web/` | generated, git-ignored | `tools/build_wasm.py` |
| `rust/target/` | generated, git-ignored, about 440 MB | cargo |
| `tools/bin/wasm-bindgen` | downloaded, git-ignored | prebuilt 0.2.129 release, sha256 checked |
| `rust/Cargo.lock`, `uv.lock` | generated, committed | cargo, uv |
| `.venv/`, `.ruff_cache/`, `docs/_build/`, `__data__/` | generated, git-ignored | uv, ruff, sphinx, scripts |

No tool ignore or deny file exists in this repo, so none was changed.

## Things that bite

- `rustup run stable cargo` still uses the Homebrew `rustc`, because the
    Homebrew bin folder comes first on PATH. Put the rustup toolchain `bin`
    folder first (the `R` prefix above) instead.
- The rustup `rust-lld` cannot find its `libLLVM.dylib`, so a WASM link aborts
    with signal 6 and a long list of "tried:" paths. `tools/build_wasm.py` sets
    `DYLD_FALLBACK_LIBRARY_PATH` to the toolchain `lib` folder. Run raw
    `cargo build --target wasm32-unknown-unknown` with that variable set, or use
    the script.
- The `wasm-bindgen` CLI and the `wasm-bindgen` crate must be the same version
    (`0.2.129`, pinned with `=`). A mismatch fails at build time.
- zsh does not split an unquoted `$cmd`: a loop over command strings needs
    `eval "$cmd"`, or every command exits 127.
- `cargo fmt` at a workspace root needs `--all`.
- mypy counts `Callable[..., X]` as an explicit `Any`. Type a callable with a
    `ParamSpec` generic (`Callable[P, R]`), and use a `Protocol` where a container
    holds callables of different signatures.
- Test dummies whose body never runs (functions built only to read their hints)
    take `# pragma: no cover  # reason` on the `def` line; everything else is
    called, so coverage stays at 100%.
- The browser pane's page is a background page while the Claude window is not in
    view: `requestAnimationFrame` never fires, timers slow to about one a second,
    and `javascript_tool` calls with many `await`s time out after 45 s. Measure
    with synchronous loops, call `tabs_select`, and check `document.hidden`;
    `mcp__ccd_view__show_pane` only helps while the window is on screen.
- `npm` and `npx` fail here with `UNABLE_TO_GET_ISSUER_CERT_LOCALLY`; `curl` works.
    Fetch a package tarball from `registry.npmjs.org` with `curl`, check its sha1
    against the registry's `dist.shasum`, and unpack it into the scratchpad (only
    after the user approves the download by name, source and size).
- `cargo test` can pass while `cargo clippy ... -D warnings` fails (a redundant
    closure in new code); run both before committing. A `sed '/x/,+2d'` over a
    multi-line Rust struct literal left a dangling `(`: edit such lists with Python
    and re-run the tests.
- `typing.TypeGuard` narrows only the first argument of the guard function, so
    the value being checked goes first (`value_fits(value, kind)`).
- Every indent in every file you write is a multiple of 4: Python, docstrings, strings that hold HTML, CSS or JS, the page, JSON (`json.dumps(..., indent=4)`) and Markdown outside code blocks. The YAML
    files that `rsync-sync` copies from the template keep their own style. Check with `grep -rnP '^(    )* {1,3}[^ ]' --include='*.py' src tests examples`, which must print nothing, and look at new HTML and JSON by eye. A continuation line of a docstring bullet goes in by 4.
- Checking the editor in the browser pane: click and drag coordinates are
    screenshot pixels (the page is about 1.28 times larger in CSS pixels), so read
    positions off the screenshot. A menu opened by a right-click may not be painted
    yet in an immediate screenshot; wait one second first.
- BSD `sed` on macOS has no `\b`. Use Python `re` for whole-word replacements.
- `ls` and `mv` read a name starting with `-` as an option. Claude's history
    folders under `~/.claude/projects/` start with one; prefix `./`.
- `fastapi`, `nicegui`, `numpy` and `rich` are in the `examples` group, which `[tool.uv] default-groups` installs with `dev`, so mypy can resolve the examples' imports. They are not runtime dependencies of the library.
- `uv sync` with no flags uninstalls the `docs` group. Use `uv sync --group docs`
    before building the docs.
- After moving the repo folder, run `uv sync --reinstall` and `uv run
    pre-commit install`: the virtual environment and the installed git hook both
    hold the old absolute path.
- `cargo fmt` rewraps struct literals, so a scripted text replacement in a Rust
    file that ran after formatting silently matched nothing twice. Check that a
    replacement applied, or use a regex.
- The browser pane may refuse `navigate` to `http://127.0.0.1:8765` the first
    time; `preview_start` with that `url` opens it, after which `navigate` works.
    Rebuild with `tools/build_wasm.py` and reload the page after any Rust change;
    restart the example only after a Python change.
- To run an example that opens a browser without opening the user's own browser,
    set `BROWSER="echo OPENED %s"`: Python's `webbrowser` then prints the URL
    instead (`BROWSER="echo OPENED %s" uv run examples/a_basics/a2_two_nodes.py`).
- Run both gates after moving or renaming a file. The Rust tests read files
    outside `rust/` with `include_str!` (`examples/e_look/my_theme.json`,
    `docs/themes.md`, `index.html`), so a move can break `cargo test` while pytest
    stays green; this happened when the examples went into folders.
- An example server can outlive a `pkill` of its `uv run` parent and keep port
    8765, so the next example fails with "address already in use" while the page
    still shows the old one. Check with `ps aux | rg examples/` and kill the python
    process before starting another.
- `uv add` has no `--dry-run`, and the hook blocks `uv pip`. `uv sync --group docs
    --offline` works from the cache. `uv venv --clear` then `uv sync` recreates the
    environment (its `pyvenv.cfg` prompt takes the project name).
- The user may be running an example in their own terminal (they tried
    `a_basics/a2_two_nodes.py` while I worked), which holds port 8765. Check
    `ps aux | rg examples/` first and never kill their process; for my own checks
    use a throwaway script in the scratchpad that calls `serve(app, port=8766)`.
- A browser-pane tab can disappear between calls ("Preview not found"), and a
    resized pane asks for a fresh screenshot before a click by coordinate; call
    `preview_start` with the URL again, then take a screenshot.
- When a script edits several state files, apply the edits only if every
    replacement matched, then check that `git diff --stat` lists every file before
    committing. The task 24 commit once carried only PLAN.md because the other
    edits failed and I committed anyway.
- zsh stops on an unquoted `--include=*.rs` ("no matches found"); quote it or use
    `rg -g '*.rs'`.
- When another session shares this folder and has a branch checked out, do not
    switch branches. Fast-forward `main` without a checkout: `git fetch . <branch>:main`
    (it refuses anything that is not a fast-forward, and also refuses while `main` is checked
    out in the folder; then commit from a `git worktree add <scratchpad>/wt -b <branch> main`
    and, once `main` is checked out and clean, `git merge --ff-only <branch>` there).
- A pane screenshot taken right after a script action can show the previous frame (a ring or
    menu that is not there yet); take a second screenshot before deciding something is wrong.
- An override of `Node.run` takes `@override` (from `typing`), or ruff's
    `no-self-use` flags every `run` that does not use `self`.
