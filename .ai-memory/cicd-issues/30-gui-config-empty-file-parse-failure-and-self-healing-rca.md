# CI/CD Issue: GUI Config Empty File Parse Failure (E9001) and Self-Healing Architecture

- Job: `INVOKE load_config` / Error Manager E9001
- Type: FAIL
- Detected: 2026-09-22T10:52:27.312Z
- Status: resolved

## Error
```text
E9001 — failed_to_parse_config_file: expected value at line 1 column 1
Endpoint: INVOKE load_config
Route / Page: /accounts
```

## Root Cause
- Zero-byte or whitespace-only `gui_config.json` file passed to `serde_json::from_str(&content)`, resulting in `expected value at line 1 column 1`.
- Lack of concurrency locks (`RwLock`) between concurrent `load_config` readers and background `save_app_config` writers.
- Missing automatic backup creation (`gui_config.json.bak`) and absence of self-healing default fallbacks upon detecting corrupted or empty configuration files.

## Fix Applied
- Implemented `OnceLock<RwLock<()>>` in `src-tauri/src/modules/config.rs` for full thread-safe read/write concurrency.
- Added automatic creation of `gui_config.json.bak` on every successful save.
- Added self-healing recovery in `load_app_config()`: if the file is empty or corrupted, it automatically restores from `.bak`, or archives corrupted state and re-initializes with `AppConfig::new()`, completely eliminating `E9001`.
- Added transient file read retry on Windows for lock contention mitigation.

## Plan Task
Documented at `.ai-memory/plans/completed/05-gui-config-empty-file-parse-failure-and-self-healing.md`
