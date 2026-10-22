# Subtask 04: Steps 8 & 9 - Split-SQLite WAL Checkpoint Schedulers & Sandboxed Multi-Instance Environment Unit Tests

- **Parent Plan**: `.ai-memory/plans/pending/175-test-case-implementation-and-task-management.md`
- **Spec Reference**: `02-spec/21-app/175-test-case-implementation-and-task-management/spec.md#step-8-unit-test-suite-for-split-sqlite-wal-truncate--passive-checkpoint-schedulers`
- **Target Files**:
  - `src-tauri/src/modules/db_orm/tests_wal_schedulers.rs`
  - `src-tauri/src/modules/instance/tests_sandboxed_environment.rs`
- **Status**: [COMPLETED]

## Completed Work
1. Implemented `tests_wal_schedulers.rs` covering:
   - Max WAL page threshold evaluation (default 500 pages, custom 100 pages)
   - Truncate checkpoint execution on in-memory and file-backed WAL databases
   - Passive non-blocking checkpoint execution and `remaining_uncheckpointed` invariants
2. Implemented `tests_sandboxed_environment.rs` covering:
   - Case-insensitive environment token scrubber for 10 sensitive keys
   - Launch environment sanitizer removing `NODE_OPTIONS`, `ELECTRON_RUN_AS_NODE`, `VSCODE_*`
   - Sandboxed PATH normalizer with priority toolchain prepending and deduplication
   - Remote debugging port guard allocating within [9222..9299] with idempotency and mutual exclusivity
   - Directory permission pre-flight checks and probe cleanup
   - Keychain file bypass marker generation (`1\n`) and verification
   - Storage scrubber synthetic GUID generation and auth token stripping
3. Registered modules in `src-tauri/src/modules/db_orm/mod.rs` and `src-tauri/src/modules/instance.rs`.
