# Subtask 03: PR Unverified Paths & Cost Disclosure Linter (`Imp-652`)
Status: COMPLETED
Parent Plan: 205-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Contributor Respect & Attribution:
"Disclose costs before merging: highlight affected existing behaviors and unverified paths alongside improvements. Own tooling and documentation gaps directly rather than attributing downstream frictions to contributors."

Implemented `src-tauri/src/modules/release_gate/pr_unverified_paths_linter.rs` and unit tests in `src-tauri/src/modules/release_gate/tests_pr_unverified_paths_linter.rs`.

## Requirements
- Audit pull request description bodies against the PR template.
- Verify presence of explicit "Unverified Paths" and "Affected Existing Behaviors" sections.
- Reject generic placeholders ("None", "N/A", "TODO", "TBD").
- Return structured linter verdict.
