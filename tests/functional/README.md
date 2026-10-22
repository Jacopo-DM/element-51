# Functional notebooks

Each notebook shows one feature area working end to end, with output you can
judge at a glance. It is a tutorial, a visual check and a smoke test in one. The
unit tests in `tests/` pin exact behaviour; these say whether it looks right.

They cover the Python side only. The Rust and WASM canvas and the browser page
are checked by the Rust tests and by hand in the browser (see `CLAUDE.md`).

## Run them

```bash
uv run python tools/run_notebooks.py
```

One line per notebook: its name, PASS, FAIL or SKIP, and the seconds it took. Name
files to run only those. `--include-slow` adds notebooks marked slow, and
`--save` writes the new outputs back into a notebook that passed.

The pre-commit config runs the notebooks in a commit at commit time, and all of
them at push time. `uv run pre-commit install` sets up both hooks. `git commit
--no-verify` skips them. A notebook that needs a display or a service the
machine lacks shows as SKIP, not as a pass.

## Coverage

Every public module of `src/element_51/` has a row. Adding a module means adding
a row (a test fails without it).

| Module | Notebook |
| --- | --- |
| `nodes` | `nodes_and_running.ipynb` |
| `ports` | `nodes_and_running.ipynb` |
| `model` | `nodes_and_running.ipynb` |
| `engine` | `nodes_and_running.ipynb` |
| `job_node` | `jobs.ipynb` |
| `jobs` | `jobs.ipynb` |
| `job_exec` | `jobs.ipynb` |
| `isolated` | `jobs.ipynb` |
| `session` | `nodes_and_running.ipynb` |
| `store` | `save_and_load.ipynb` |
| `theme` | `themes.ipynb` |
| `visuals` | `pictures_and_series.ipynb` |
| `edits` | `editing.ipynb` |
| `protocol` | none yet |
| `server` | none yet |
