# Subtask 08: CLI Integration (`agm db ...`)

**Parent Plan:** `.ai-memory/plans/orm-and-database-migration-strategy.md`  
**Status:** PENDING  
**Target Files:** `src-tauri/src/bin/agm/db_cmds.rs`, `src-tauri/src/bin/agm/main.rs`, `help_text.rs`

## Objective
Add `agm db` CLI command family:
- `agm db backup [--file <path>]`: Export pre-migration snapshot.
- `agm db restore [--file <path>]`: Fresh initialize and re-import snapshot.
- `agm db status`: Print status of all split databases, schema versions, and row counts.
- `agm db verify`: Validate zero account loss and schema integrity.

## Acceptance Criteria
- Commands print human-readable tables and support `--json`.
- Integrates with `help_text.rs` and terminal diagnostics.
