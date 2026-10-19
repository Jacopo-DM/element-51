# Plan

## Goals

No goals yet: run /zz-02-project-plan.

## Current state

Phase: setup done, skeleton not started.
Next step: run /zz-02-project-plan.

## Baseline

No code yet.

## Tasks

None yet.

## Open questions

None yet.

## Known problems

Found by /zz-01-project-setup on the uv scaffolding (2026-10-19):

- `uv run --locked ruff check --no-fix` reports 2 errors in
  `src/patchbay/__init__.py`: `non-empty-init-module` (a function lives in
  `__init__`) and `undocumented-public-package` (no docstring).
- `uv run --locked pytest` fails: there is no `tests/` folder, so coverage is 0%
  against the 100% floor.
- `src/__init__.py` and `src/hello.py` sit beside the real package
  `src/patchbay/`. They look like leftover scaffolding ("Hello from .!").
- `README.md`, `docs/index.md` and the `pyproject.toml` description are still
  template text.
- `uv run --locked mypy .` passes (6 files).
