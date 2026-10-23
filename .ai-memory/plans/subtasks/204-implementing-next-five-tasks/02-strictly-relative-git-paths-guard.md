# Subtask 02: Strictly Relative Git Paths & Anti-Absolute-Path Guard (`Imp-646`)
Status: COMPLETED
Parent Plan: 204-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Release Workflow & Strictly Relative Git Paths Invariant:
"Never output or record absolute filesystem paths in specifications, plans, commit messages, `changelog.md`, `changelog_en.md`, `README.md`, `README_EN.md`, or GitHub release notes. All file references MUST be repository-relative Git paths (e.g., `02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`)."

Implement `src-tauri/src/modules/release_gate/strictly_relative_git_paths_guard.rs` and unit tests in `src-tauri/src/modules/release_gate/tests_strictly_relative_git_paths_guard.rs`.

## Requirements
- Scan text content (commit messages, changelogs, specs, release notes).
- Detect and flag Windows drive absolute path patterns.
- Detect and flag POSIX home or system absolute path patterns.
- Detect and flag file URI protocols.
- Verify repository-relative path structure (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- Return structured `PathHygieneReport` with detected violations, line numbers, and snippets.
