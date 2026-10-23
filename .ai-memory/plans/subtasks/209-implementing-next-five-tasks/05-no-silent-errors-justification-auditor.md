# Subtask 05: No Silent Errors Justification Auditor (`Imp-674`)
Status: IN_PROGRESS
Parent Plan: 209-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md No Silent Errors Invariant:
"Every fallible operation must route its failure into the error module. Swallowing an error — discarding it where no one can ever see it — is forbidden.
- Rust (`src-tauri/src/`): errors flow into `AppError` (`src-tauri/src/error.rs`) and are logged via `tracing` (file + console + Tauri log bridge). Never leave a bare `let _ = <fallible>`, `.ok()`, or `.unwrap_or_default()` on a `Result` without routing the `Err`.
- For genuinely best-effort ops (cleanup, best-effort notifies): `crate::error::record_ignored(std::fs::remove_file(&tmp), "remove temp file")` plus a one-line `// Justification:` comment explaining why it is safe to continue. Both the tracking call and the comment are required — neither alone suffices."

Implement `src-tauri/src/modules/release_gate/no_silent_errors_justification_auditor.rs` and unit tests in `src-tauri/src/modules/release_gate/tests_no_silent_errors_justification_auditor.rs`.

## Requirements
- Scan source lines for `record_ignored(...)` calls.
- Verify that every `record_ignored` call has an immediately adjacent or preceding line matching `// Justification:`.
- Detect and flag bare `record_ignored` calls missing justification comments.
- Detect bare ignored fallible patterns (`let _ =` or `.ok()` on fallible results without context).
- Provide structured report tracking audited sites, valid justifications, and violations.
