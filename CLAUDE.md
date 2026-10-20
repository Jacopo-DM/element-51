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
| Rust tests (native) | `R cargo test --manifest-path rust/Cargo.toml` | about 1 s warm |
| Build the browser layer | `uv run python tools/build_wasm.py` | about 4 s; rewrites `src/element_51/web/` |
| Serve the editor | `uv run examples/a1_two_nodes.py` | needs the build first; opens the page in your browser; Ctrl+C stops it with exit 0 |
| Serve another example | `uv run python examples/a2_arithmetic.py` (a2 to a6, a8, a9 to a20 serve on http://127.0.0.1:8765; a7 prints and exits) | same port, so stop one before starting the next |

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
- A feature the user can try gets a runnable example in `examples/`, laid out like
  `examples/a0_hello_world.py`: a docstring with a `Notes` section that says what
  to try, the comment headers (Standard library, Third-party libraries, Local
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
- `typing.TypeGuard` narrows only the first argument of the guard function, so
  the value being checked goes first (`value_fits(value, kind)`).
- Every indent in every file you write is a multiple of 4: Python, docstrings, strings that hold HTML, CSS or JS, the page, and JSON (`json.dumps(..., indent=4)`). Check with `grep -rnP '^(    )* {1,3}[^ ]' --include='*.py' src tests examples`, which must print nothing, and look at new HTML and JSON by eye. A continuation line of a docstring bullet goes in by 4.
- Checking the editor in the browser pane: click and drag coordinates are
  screenshot pixels (the page is about 1.28 times larger in CSS pixels), so read
  positions off the screenshot. A menu opened by a right-click may not be painted
  yet in an immediate screenshot; wait one second first.
- BSD `sed` on macOS has no `\b`. Use Python `re` for whole-word replacements.
- `ls` and `mv` read a name starting with `-` as an option. Claude's history
  folders under `~/.claude/projects/` start with one; prefix `./`.
- `fastapi` and `nicegui` are in the `examples` group, which `[tool.uv] default-groups` installs with `dev`, so mypy can resolve the two host examples. They are not runtime dependencies of the library.
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
  instead (`BROWSER="echo OPENED %s" uv run examples/a1_two_nodes.py`).
- zsh stops on an unquoted `--include=*.rs` ("no matches found"); quote it or use
  `rg -g '*.rs'`.
- An override of `Node.run` takes `@override` (from `typing`), or ruff's
  `no-self-use` flags every `run` that does not use `self`.
