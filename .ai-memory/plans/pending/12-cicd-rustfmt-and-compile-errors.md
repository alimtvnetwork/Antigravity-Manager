# CI/CD Task: Rustfmt Drift in Iterative Codec & Rust Compilation Errors

## Source
- Runner job: Check Rust Code & Build Tauri App
- Error type: FAIL
- Detected at: 2026-09-22T00:22:15Z

## Error Summary
1. `src-tauri/src/modules/iterative_codec.rs:138`: Line wrapping formatting diff in match arm (`auto_prune_secondary_mb` & `heartbeat_interval_secs`).
2. `src-tauri/src/modules/email_watcher.rs:75`: E0425 `cannot find function get_email_settings in module email_vault_db`.
3. `src-tauri/src/modules/supabase_command_queue.rs:96,101`: E0599 `no method named output_hidden found for mutable reference &mut std::process::Command`.

## Required Fix
Format match arms in `iterative_codec.rs` with multi-line blocks, update `email_watcher.rs` to call `email_vault_db::get_notification_settings()`, and replace nonexistent `.output_hidden()` in `supabase_command_queue.rs` with `creation_flags_windows()` + `.output()`.

## Acceptance Criteria
- [ ] `iterative_codec.rs` formatting matches rustfmt specification
- [ ] `email_watcher.rs` calls correct `get_notification_settings()` function
- [ ] `supabase_command_queue.rs` uses standard `output()` with `creation_flags_windows()`
- [ ] Local quality gates pass (exit code 0)
- [ ] No regression in any other job

## Status
- [x] resolved
