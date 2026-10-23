# Subtask 05: Split-DB Lock Contention Timeout Escalator
Status: COMPLETED
Sequence: 200
Owner: Worker-3
Target Module: `src-tauri/src/modules/db_orm/lock_timeout_escalator.rs`
Test Suite: `src-tauri/src/modules/db_orm/tests_lock_timeout_escalator.rs`

## Evidence
- Implemented `is_busy_error_code` (detecting SQLITE_BUSY 5, SQLITE_LOCKED 6 and extended codes), `LockEscalationPolicy`, `LockEscalationStep`, `LockEscalationDecision`, and `LockTimeoutEscalator`.
- Escalates SQLite lock busy timeouts (250ms -> 500ms -> 1000ms -> 2000ms -> 4000ms clamped to 5000ms) with deterministic LCG jitter.
- Converts unrecoverable deadlocks into structured `AppError::Config` / `AppError::DatabaseLocked` without swallowing errors.
- 7 unit tests covering step progression, max attempts deadlock return, SQLite busy error code detection, reset behavior, jitter bounds, and Serde roundtrip.
