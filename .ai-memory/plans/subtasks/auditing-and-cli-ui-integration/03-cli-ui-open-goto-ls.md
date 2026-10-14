# Task-03: CLI-UI bridge — open / goto / ls

## Status
DONE — verified by lead (file exists, fmt clean, main.rs wired)

## Objective
New CLI module `src-tauri/src/bin/agm/ui_cmds.rs` (following the existing
`*_cmds.rs` pattern) with the first three subcommands.

## Requirements
- `agm ui open` — open the AGM UI (launch if closed, bring to foreground
  if running). Cross-platform (respect the repo's process-spawn
  conventions in AGENTS.md).
- `agm ui goto <route>` — navigate the running UI to any page / settings
  page / settings subpage (e.g. `settings`, `settings/supabase`,
  `instances`, `accounts`). Discover the route registry via GitMap search.
  Unknown routes must list valid routes (never fail silently).
- `agm ui ls [selector]` — list the UI element tree of the current view:
  route, components, element ids, focusable elements. This is the "ls on
  any UI".
- Every CLI-driven action must flow through the same underlying service
  routes as the GUI (AGENTS.md "Headless & CLI Parity") so behavior is
  identical.
- Audit integration: CLI-driven actions are recorded with `actor=cli`
  (coordinate with Task-01's command names; do not duplicate the module).
- "No Silent Errors": CLI errors print the real message to stderr with a
  non-zero exit; never silent success.

## Acceptance
- [ ] `open`, `goto`, `ls` work against a running UI.
- [ ] Unknown route lists valid routes.
- [ ] `cargo fmt` + `clippy` clean.

## Constraints
- GitMap search only. Relative paths only.
- Owns `src-tauri/src/bin/agm/ui_cmds.rs` jointly with Task-04 — coordinate
  via the lead to avoid edit collisions (split: Task-03 writes open/goto/ls
  sections, Task-04 writes scroll/click/help sections).
