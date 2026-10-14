# Task-01: Interaction audit recorder — backend

## Status
DONE — verified by lead (fmt clean, tsc clean, wired into mod.rs/run.rs/Layout/Navbar)

## Objective
Build the Rust backend for UI interaction audit recording in a new module
`src-tauri/src/modules/audit/`.

## Requirements
- Recorder service with `start`, `stop`, `clear`, `export` operations,
  exposed as Tauri commands.
- Each interaction appends an entry:
  `timestamp, actor (user|cli|telegram|system), method_name,
  ui_target (route + component + element id), trigger
  (click|keyboard|programmatic|navigation|input|scroll), params (JSON),
  result (ok|error + message)`.
- Persist entries in SQLite (follow the repo's existing storage
  conventions — discover via GitMap search; do not invent a new DB).
- Recording indicator state queryable from the frontend.
- "No Silent Errors" (AGENTS.md): every fallible op routes into
  `crate::error` (`AppError` + tracing, or `record_ignored` + required
  `// Justification:` comment for best-effort).
- `cargo fmt` clean; `cargo clippy` clean on touched files.

## Acceptance
- [ ] start/stop/clear/export commands work via Tauri IPC.
- [ ] Entries contain all 8 fields above.
- [ ] Errors surfaced, never swallowed.

## Constraints
- GitMap search only. Relative paths only. No secrets.
- Workers never run git.
