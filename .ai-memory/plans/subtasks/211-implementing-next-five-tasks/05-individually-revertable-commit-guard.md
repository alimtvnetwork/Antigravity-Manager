# Subtask 05: Individually Revertable Commit Guard (`Imp-684`)
Status: COMPLETED
Parent Plan: 211-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md PR Scope & Grouping:
"Self-Contained & Individually Revertable: A PR may contain multiple commits, but each commit must represent an independent, self-contained functional unit that is individually revertable, avoiding messy or tangled changesets."

Implemented `src-tauri/src/modules/release_gate/individually_revertable_commit_guard.rs` and unit tests in `src-tauri/src/modules/release_gate/tests_individually_revertable_commit_guard.rs`.
Registered in `src-tauri/src/modules/release_gate/mod.rs`.

## Verification Evidence
- Defined `VALID_COMMIT_PREFIXES`, `NOISE_KEYWORDS`, and `CommitQualityViolation`.
- Implemented `evaluate_commit` and `audit_commit_series`.
- Enforced message length, prefix convention (`Feature:`, `Fix:`), trial-and-error noise detection, and file scope boundaries.
- Added comprehensive unit tests covering compliant commits, vague messages, missing prefixes, and noise keywords.
- Deferred E2E command documented in `02-spec/21-app/211-implementing-next-five-tasks/e2e_master_plan.md`.
