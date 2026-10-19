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

## Code rules

- Entry points and import direction are recorded in CODEMAP.md once they exist.
- Modules talk to each other only through their public interfaces.
- No compatibility shims.
- No abstraction before there are two real call sites.
- Clear names over clever one-liners.
- No options nobody asked for.
