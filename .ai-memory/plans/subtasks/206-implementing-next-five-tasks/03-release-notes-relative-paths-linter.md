# Subtask 03: Release Notes & Changelog Relative Paths Linter (`Imp-657`)
Status: COMPLETED
Parent Plan: 206-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Strictly Relative Git Paths Invariant:
"Never output or record absolute filesystem paths (e.g. `D:\...`, `C:\...`, `/home/...`) in specifications, plans, commit messages, `changelog.md`, `changelog_en.md`, `README.md`, `README_EN.md`, or GitHub release notes. All file references MUST be repository-relative Git paths (e.g., `02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`)."

Implement `src-tauri/src/modules/release_gate/release_notes_relative_paths_linter.rs` and unit tests in `src-tauri/src/modules/release_gate/tests_release_notes_relative_paths_linter.rs`.

## Requirements
- [x] Scan text for absolute paths matching Windows (`C:\...`, `D:\...`, `\\...`), Unix (`/home/...`, `/Users/...`, `/etc/...`), or `file:///` URIs.
- [x] Flag absolute path violations with line number, snippet, and recommended relative path.
- [x] Emit structured `RelativePathAuditReport`.
- [x] Comprehensive unit tests implemented and registered in `src-tauri/src/modules/release_gate/mod.rs`.
