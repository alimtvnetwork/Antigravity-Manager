# Subtask 01: Coding Guidelines & Condition Simplification

**Parent Task:** `ui-and-functionality-enhancement-instructions`  
**Subtask Code:** `Task-01`  
**Status:** `READY`

## 1. Objectives & Scope
- Invert deep nested if-else ladders and compound negative conditions (`!isA && !isB`).
- Refactor `src-tauri/src/commands/ssh.rs` (`test_ssh_node_connection`, `export_system_data_csv`).
- Refactor `src-tauri/src/modules/ssh_manager/connections.rs` (`load_ssh_connections`, `sync_ssh_connections_locally`).
- Adopt affirmative booleans (`is_`, `has_`), early guard returns, and helper extraction.

## 2. Disjoint File Ownership
- `Owned Files:` `src-tauri/src/commands/ssh.rs`, `src-tauri/src/modules/ssh_manager/connections.rs`

## 3. Verification Evidence Contract
- Targeted `cargo clippy --all-targets --all-features` compiles cleanly.
- Code inspection proves zero negative boolean combinations in touched functions.
