# patchbay

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
| Build the browser layer | `uv run python tools/build_wasm.py` | about 4 s; rewrites `src/patchbay/web/` |
| Serve the editor | `uv run python examples/a1_serve.py` | needs the build first; Ctrl+C stops it |

## Code rules

- Entry points and import direction are in CODEMAP.md.
- Python records are pydantic models; Rust message types derive serde and
  mirror them.
- Modules talk to each other only through their public interfaces.
- No compatibility shims.
- No abstraction before there are two real call sites.
- Clear names over clever one-liners.
- No options nobody asked for.

## Ignore list

Do not edit, search or commit these.

| Path | Generated or hand-written | What makes it |
| --- | --- | --- |
| `src/patchbay/web/` | generated, git-ignored | `tools/build_wasm.py` |
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
