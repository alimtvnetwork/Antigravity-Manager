# Subtask 03 - E0061 fix (244)

**Date**: 01-Nov-26
**Status**: TODO at plan time; completed by the fix commit (see report)

- File: `src-tauri/src/modules/repo_db/queue.rs`
- Change: `invalidate_prompt_tree_cache();` becomes
  `invalidate_prompt_tree_cache(instance_id);` with a brief WHY comment
  above the call (scoped clear invalidates only that instance's cache;
  all-instances clear invalidates everything).
- Precedent: same-file `queue.rs:363` passes `target_instance` through;
  same pattern at `import.rs:509`, `resend.rs:321`, `sqlite_export.rs:558`.
- Verification: remote CI (`gitmap pe -t`) on the fix commit; no local
  full build (no cargo toolchain on PATH here; prior full cargo check
  OOM-killed on this VM).
