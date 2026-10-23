# Subtask 03: PR Unverified Paths & Cost Disclosure Linter (`Imp-652`)
Status: IN_PROGRESS
Parent Plan: 205-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Contributor Respect & Attribution & PR Scope:
"Disclose costs before merging: highlight affected existing behaviors and unverified paths alongside improvements."

Implement `src-tauri/src/modules/release_gate/pr_unverified_paths_linter.rs` and unit tests in `src-tauri/src/modules/release_gate/tests_pr_unverified_paths_linter.rs`.

## Requirements
- Parse PR descriptions and extract the "Unverified Paths" and "Affected Existing Behaviors" sections.
- Identify when high-risk subsystem paths (proxy handlers, database ORM, instance lifecycle) are modified.
- Reject boilerplate, empty, or generic "N/A" entries in unverified paths when high-risk subsystems are modified.
- Return structured `CostDisclosureAuditReport` with compliance status, high-risk detection, and disclosure details.
