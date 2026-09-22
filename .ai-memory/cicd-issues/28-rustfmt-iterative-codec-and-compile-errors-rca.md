# CI/CD Issue 28: Rustfmt Drift in Iterative Codec and Compilation Errors in Watcher and Command Queue

- Job: Check Rust Code & Build Tauri App
- Type: FAIL
- Detected: 2026-09-22T00:22:15Z
- Status: resolved

## Error
1. `cargo fmt -- --check` failure across Linux, macOS, and Windows runners:
```text
Diff in src-tauri/src/modules/iterative_codec.rs:138:
-                    "auto_prune_secondary_mb" => auto_prune_secondary_mb = val.parse().unwrap_or(200),
-                    "heartbeat_interval_secs" => heartbeat_interval_secs = val.parse().unwrap_or(30),
+                    "auto_prune_secondary_mb" => {
+                        auto_prune_secondary_mb = val.parse().unwrap_or(200)
+                    }
+                    "heartbeat_interval_secs" => {
+                        heartbeat_interval_secs = val.parse().unwrap_or(30)
+                    }
```
2. `cargo check` / `cargo build` compilation failure:
```text
error[E0425]: cannot find function `get_email_settings` in module `email_vault_db`
  --> src/modules/email_watcher.rs:75:43
   |
75 |     if let Ok(settings) = email_vault_db::get_email_settings() {
   |                                           ^^^^^^^^^^^^^^^^^^ not found in `email_vault_db`

error[E0599]: no method named `output_hidden` found for mutable reference `&mut std::process::Command` in the current scope
  --> src/modules/supabase_command_queue.rs:96:10
```

## Root Cause
1. **Rustfmt match arm line length:** In `iterative_codec.rs`, the match arms exceeded standard Rustfmt line length constraints when parsing integers with default fallbacks.
2. **Function name discrepancy:** In `email_watcher.rs:75`, `email_vault_db::get_email_settings()` was invoked instead of `email_vault_db::get_notification_settings()`.
3. **Nonexistent helper method:** In `supabase_command_queue.rs`, `.output_hidden()` was invoked on `std::process::Command` instead of calling `cmd_proc.creation_flags_windows().output()` with `CommandExtWrapper`.

## Fix Applied
1. Formatted `iterative_codec.rs:138` with block-wrapped match arms.
2. Updated `email_watcher.rs:75` to call `email_vault_db::get_notification_settings()`.
3. Replaced `.output_hidden()` in `supabase_command_queue.rs` with `cmd_proc.creation_flags_windows().args(...).output()` for Windows and `Command::new("sh").args(...).output()` for Unix.

## Plan Task
Enqueued at `.ai-memory/plans/pending/12-cicd-rustfmt-and-compile-errors.md`
