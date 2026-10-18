# Subtask 05: Caller IDE Auto-Detection & Transient Test Sandbox CLI

## Scope
- Files: `src-tauri/src/bin/agm/instance_cmds.rs`, `src-tauri/src/bin/agm/instance_handlers.rs`, `src-tauri/src/modules/instance/caller.rs`

## Implementation Steps
1. In `src-tauri/src/modules/instance/caller.rs`:
   - Implement `detect_caller_instance_id() -> Option<String>` based on `AGM_INSTANCE_ID`, process hierarchy, and cwd.
2. In `src-tauri/src/bin/agm/instance_cmds.rs` & `instance_handlers.rs`:
   - Add caller protection to `stop`, `rm`, `switch` subcommands (fail with exit code 2 unless `--force`).
   - Add `agm instances test-create` to spin up isolated sandbox mock instances.
   - Add `agm instances test-clean` to purge all test sandbox instances cleanly.

## Verification
- CLI test commands exit 0 and protect active caller instance.
