# Task-04: CLI-UI bridge — scroll / click / help

## Status
PENDING

## Objective
Complete `src-tauri/src/bin/agm/ui_cmds.rs` with scroll, click, and the
help text.

## Requirements
- `agm ui scroll <element-id|selector>` — scroll the current view until
  the element is visible, then focus it. Reports what it did
  ("scrolled to #settings-supabase, focused").
- `agm ui click <element-id>` — activate an element through the same
  code path as a user click (so the audit recorder logs it with
  `actor=cli`, and behavior matches the GUI exactly).
- `agm ui --help` (and per-subcommand help) — help text that tells the
  user WHAT TO DO: each subcommand gets a one-line purpose, a concrete
  example, and the expected output. No bare flag lists.
- Same constraints as Task-03: CLI/GUI parity, audit `actor=cli`,
  real errors to stderr, `cargo fmt` + `clippy` clean.

## Acceptance
- [ ] `scroll` visibly scrolls + focuses; `click` triggers the real handler.
- [ ] `--help` output is actionable (examples for every subcommand).
- [ ] `cargo fmt` + `clippy` clean.

## Constraints
- GitMap search only. Relative paths only.
- Coordinate with Task-03 on `ui_cmds.rs` sections (lead arbitrates).
