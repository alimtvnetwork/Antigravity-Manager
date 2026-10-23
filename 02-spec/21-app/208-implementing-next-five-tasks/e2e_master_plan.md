# End-to-End Master Execution Plan: Batch 208 (Final Milestone: Steps 251–255 of 255)

## 1. Batch Objective
Execute the grand final five steps of the 255-step improvement backlog across proxy handlers, instance management, release gate, and SQLite Split-DB ORM.

## 2. Work Breakdown & Disjoint Subagent Allocation
- **Worker 01 (Task 1 & Task 2)**:
  - `src-tauri/src/modules/instance/settings_deep_merge_validator.rs` & `tests_settings_deep_merge_validator.rs`
  - `src-tauri/src/proxy/handlers/common/thinking_block_backfill_pipeline.rs` & `tests_thinking_block_backfill_pipeline.rs`
- **Worker 02 (Task 3 & Task 5)**:
  - `src-tauri/src/modules/release_gate/coauthor_trailer_alignment_guard.rs` & `tests_coauthor_trailer_alignment_guard.rs`
  - `src-tauri/src/modules/release_gate/unverified_paths_disclosure_guard.rs` & `tests_unverified_paths_disclosure_guard.rs`
- **Worker 03 (Task 4)**:
  - `src-tauri/src/modules/db_orm/fifo_restore_transaction_coordinator.rs` & `tests_fifo_restore_transaction_coordinator.rs`

## 3. Disjoint Mod Registration Responsibility
- All `mod.rs` wiring (`instance.rs`, `common.rs`, `release_gate/mod.rs`, `db_orm/mod.rs`) will be conducted by the lead orchestrator to avoid write conflicts among workers.

## 4. Verification & Commit Protocol
- Zero builds and zero tests during implementation.
- Secrets pattern scan verification.
- Relative paths audit across documentation and commit messages.
- Commit via `gitmap cpf` with standard hyphenated prefix format.
