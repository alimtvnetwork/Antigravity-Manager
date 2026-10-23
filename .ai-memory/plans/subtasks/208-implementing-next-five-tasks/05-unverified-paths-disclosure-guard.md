# Subtask 05: Unverified Paths & Cost Disclosure Guard (`Imp-669`)
Status: IN_PROGRESS
Parent Plan: 208-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Contributor Respect & Attribution:
"Disclose costs before merging: highlight affected existing behaviors and unverified paths alongside improvements."

Implement `src-tauri/src/modules/release_gate/unverified_paths_disclosure_guard.rs` and unit tests in `src-tauri/src/modules/release_gate/tests_unverified_paths_disclosure_guard.rs`.

## Requirements
- Scan PR markdown templates and release notes for required disclosure sections: Affected Existing Behaviors and Unverified Paths.
- Verify that non-trivial changesets (modifying core logic or migrations) explicitly enumerate paths that could not be verified locally.
- Reject empty, boilerplate, or "N/A" disclosures when high-risk subsystems (schema, release machinery, process spawning) are modified.
- Provide structured audit report tracking compliance status, disclosed risks, and missing sections.
