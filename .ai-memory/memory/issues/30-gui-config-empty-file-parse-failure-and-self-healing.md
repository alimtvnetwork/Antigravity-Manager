# Issue 30: GUI Config Empty File Parse Failure (E9001) and Self-Healing Architecture

## Why It Happened
During rapid navigation to `/accounts` or simultaneous application bootstrap, multiple concurrent Tauri IPC invocations of `load_config` failed with:
`E9001 — failed_to_parse_config_file: expected value at line 1 column 1`
This exact error (`expected value at line 1 column 1`) is produced by `serde_json::from_str` when the input string is completely empty (`""`) or consists solely of whitespace. When `gui_config.json` was truncated to 0 bytes or read during an un-synchronized file transition, `load_app_config()` encountered an empty string and returned an error instead of self-healing or restoring from backup.

## How It Happened
1. Multiple frontend routes (such as `/accounts`, `/proxy`, and navbar status monitors) call `invoke('load_config')` concurrently during initial load.
2. Simultaneously, background threads (such as `check_and_recover_crashed_instance`, `save_config`, or token managers) access and update `gui_config.json`.
3. Because `src-tauri/src/modules/config.rs` lacked an in-process synchronization lock (`RwLock`), concurrent reads and writes competed for the file.
4. Furthermore, if `gui_config.json` became 0 bytes (due to an interrupted write, abrupt process termination, or system reboot), `config_path.exists()` evaluated to true.
5. As a result, `load_app_config()` bypassed the `!config_path.exists()` check, read the empty file, and passed `""` to `serde_json::from_str(&content)`.
6. `serde_json` returned `Err("expected value at line 1 column 1")`. Because there was no self-healing fallback or backup restore mechanism, every subsequent `load_config` call failed repeatedly with `E9001`.

## Root Cause
1. **Zero-Byte File Fragility**: `load_app_config()` assumed that any file for which `.exists()` was true was valid JSON. It had no guard for `content.trim().is_empty()` or invalid JSON structures.
2. **Missing Backup & Self-Healing Mechanism**: There was no `gui_config.json.bak` snapshot creation during saves, and no automatic recovery to fallback defaults when corruption or truncation occurred.
3. **Lack of Concurrency Synchronization**: No `RwLock` or mutual exclusion protected concurrent reads and atomic writes of `gui_config.json`.

## Code Fix
1. **Process-Level `RwLock` Synchronization**:
   Introduced `CONFIG_LOCK: OnceLock<RwLock<()>>` in `src-tauri/src/modules/config.rs` to synchronize all reads and writes across background tasks and IPC commands.
2. **Transient Read Retry on Windows**:
   Added retry logic in `load_app_config()` to handle transient sharing/lock contention.
3. **Automatic Backup on Save**:
   In `save_app_config()`, before atomically replacing `gui_config.json`, valid existing files are copied to `gui_config.json.bak`.
4. **Resilient Self-Healing**:
   In `load_app_config()`, if `content.trim().is_empty()` or if `serde_json::from_str` fails:
   - Attempt recovery from `gui_config.json.bak`.
   - If backup is unavailable or invalid, safely archive the corrupted file to `gui_config.json.corrupt.<timestamp>` and self-heal by generating and persisting `AppConfig::new()`.
   - Return `Ok(config)` so the application never crashes or locks the user out with `E9001`.
