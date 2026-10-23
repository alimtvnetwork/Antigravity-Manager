# Subtask 03: Co-authored-by Trailer Alignment Guard (`Imp-667`)
Status: COMPLETED
Parent Plan: 208-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Contributor Respect & Attribution:
"Preserve authorship by preferring the contributor's own PR for squash commits, or attaching explicit `Co-authored-by:` trailers on merge commits and proxy PRs."

Implement `src-tauri/src/modules/release_gate/coauthor_trailer_alignment_guard.rs` and unit tests in `src-tauri/src/modules/release_gate/tests_coauthor_trailer_alignment_guard.rs`.

## Requirements
- Parse and validate `Co-authored-by:` git commit trailers (`Co-authored-by: Name <email>`).
- Enforce valid name and email syntax (`<user@domain.tld>`).
- Verify that proxy PRs and multi-author squash commits preserve contributor credit.
- Disallow malformed trailers, empty emails, or placeholder handles.
- Provide structured report tracking extracted trailers and validation findings.
