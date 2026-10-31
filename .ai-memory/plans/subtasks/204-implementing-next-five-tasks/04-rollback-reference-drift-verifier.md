# Subtask 04: Rollback Reference & Content Drift Verifier (`Imp-648`)
Status: COMPLETED
Parent Plan: 204-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Branch & History Hygiene:
"Inspect in-flight PRs (`gh pr list --base main`) before rewriting published tips, avoiding force-pushes across shared branches. Retain local rollback refs (`backup/*`) before history rewrites, confirming zero content drift via `git diff --stat <backup> HEAD`."

Implement `src-tauri/src/modules/release_gate/rollback_reference_drift_verifier.rs` and unit tests in `src-tauri/src/modules/release_gate/tests_rollback_reference_drift_verifier.rs`.

## Requirements
- Verify presence of `backup/*` local rollback ref before allowing any history rewrite or force push.
- Parse `git diff --stat <backup> HEAD` output to verify zero content drift. If files changed or insertions/deletions exist, flag content drift violation.
- Inspect in-flight PR count for target branch; flag violation if active in-flight PRs exist.
- Return structured `HistoryRewriteSafetyReport` with safety verdict, rollback ref status, and drift findings.
