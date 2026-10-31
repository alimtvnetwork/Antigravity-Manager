# Subtask 04: Auto-Switcher Score Variance & SQLite Mmap Tuner Tests

- **Task**: Test Case Implementation and Task Management (Sequence 181)
- **Slug**: `181-test-case-implementation-and-task-management`
- **Subtask**: 04 (Steps 8 & 9)
- **Status**: [COMPLETED]

## Scope
1. **Step 8 (Imp-559)**: Author unit tests for score variance guard in `src-tauri/src/modules/auto_switcher/tests_score_variance_guard.rs`.
   - Test variance calculation across candidate scores.
   - Test low-variance scenario detection (near-identical account scores).
   - Test outlier damping and selection stability.
   - Register module in `src-tauri/src/modules/auto_switcher/mod.rs`.
2. **Step 9 (Imp-560)**: Author unit tests for split-SQLite mmap tuner in `src-tauri/src/modules/db_orm/tests_mmap_tuner.rs`.
   - Test PRAGMA mmap_size syntax execution.
   - Test platform byte clamping [0 .. 1_073_741_824] (1GB max).
   - Test in-memory SQLite connection verification.
   - Register module in `src-tauri/src/modules/db_orm/mod.rs`.
