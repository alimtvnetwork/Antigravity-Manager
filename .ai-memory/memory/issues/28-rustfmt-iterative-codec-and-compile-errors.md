# 4-Part Root Cause Analysis: Rustfmt Drift in Iterative Codec and Compilation Errors in Watcher and Command Queue

> **Version:** 1.0.0
> **Date:** 2026-09-22
> **Failed Run:** [#35671509636](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35671509636) (CI)
> **Trigger Commit:** `833162de`
> **Fixed In:** `HEAD`

---

## 4-Part Root Cause Analysis (RCA)

### 1. Symptom (Why it happened)
In GitHub Actions CI workflow run [#35671509636](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35671509636), all runner matrix platforms (`windows-2025`, `ubuntu-latest`, and `macos-latest`) failed at two distinct pipeline stages:
1. `Check Rust formatting` failed with a formatting diff in `src-tauri/src/modules/iterative_codec.rs` at line 138.
2. `Build Tauri App (debug)` failed with two compilation errors:
   - `error[E0425]: cannot find function get_email_settings in module email_vault_db` in `src-tauri/src/modules/email_watcher.rs:75:43`.
   - `error[E0599]: no method named output_hidden found for mutable reference &mut std::process::Command` in `src-tauri/src/modules/supabase_command_queue.rs:96:10` and `101:10`.

### 2. How it happened & Root Cause
#### How it happened:
- In commit `258ace00`, the iterative codec was enhanced to support both YAML and JSON exports with iterative Base64 encoding. The match arms for parsing `auto_prune_secondary_mb` and `heartbeat_interval_secs` were written on single lines, exceeding Rustfmt's line-budget limit in deep indentation blocks.
- In commit `d043c279`, the Supabase and Telegram inbound systems were implemented. In `email_watcher.rs`, `detect_machine_name` was written calling `email_vault_db::get_email_settings()` instead of the actual public function `email_vault_db::get_notification_settings()`.
- In `supabase_command_queue.rs`, process execution was written using a hypothetical helper `.output_hidden()`, but `std::process::Command` does not define `output_hidden()`. The project standard is `cmd_proc.creation_flags_windows().args(...).output()` via the in-tree trait `CommandExtWrapper`.

#### Root Cause:
1. **Rustfmt Formatting Budget:** Match expressions nested inside loop conditionals exceeded 100 characters, requiring block braces `{ ... }` under standard Rustfmt rules.
2. **Identifier Typo in Module Call:** `email_vault_db` exports `get_notification_settings()`, while `commands::email` defines the async IPC command `get_email_settings()`. Calling `email_vault_db::get_email_settings()` caused compiler error `E0425`.
3. **Missing Trait / Method Mismatch:** `.output_hidden()` does not exist on `std::process::Command`. The correct pattern is using `CommandExtWrapper::creation_flags_windows` on Windows and standard `.output()` on both platforms.

### 3. Resolution
1. **`src-tauri/src/modules/iterative_codec.rs`:**
   Wrapped the two match arms in braces:
   ```rust
   "auto_prune_secondary_mb" => {
       auto_prune_secondary_mb = val.parse().unwrap_or(200)
   }
   "heartbeat_interval_secs" => {
       heartbeat_interval_secs = val.parse().unwrap_or(30)
   }
   ```
2. **`src-tauri/src/modules/email_watcher.rs`:**
   Changed `email_vault_db::get_email_settings()` to `email_vault_db::get_notification_settings()`.
3. **`src-tauri/src/modules/supabase_command_queue.rs`:**
   Updated process execution to use `cmd_proc.creation_flags_windows().args(...).output()` on Windows and `Command::new("sh").args(...).output()` on Unix.

### 4. Prevention & Learnings
1. **Check Module Export Names:** Before calling helper functions in backend modules, verify their exact identifier in the declaring file (`get_notification_settings` vs `get_email_settings`).
2. **Use Established Process Execution Idioms:** Always use `CommandExtWrapper::creation_flags_windows` + `.output()` for hidden subprocess execution on Windows.
3. **Block Wrapping for Long Match Arms:** When writing match arms with method chains or parsing fallbacks, wrap them in `{ ... }` to prevent Rustfmt line-length failures.
