# 4-Part Root Cause Analysis: `user_token_db` Cross-Thread Test Concurrency and Environment Pollution

## Metadata
- **Pipeline Run ID**: 35046504473
- **Repository**: alimtvnetwork/Antigravity-Manager
- **Commit**: `5cb55270`
- **Impacted Jobs**:
  - `Check Rust Code` (`macos-latest`)
- **Date**: 2026-09-16
- **Status**: Resolved

---

## 1. Symptoms

In GitHub Actions CI run [#35046504473](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35046504473), the job `Check Rust Code (macos-latest)` failed during the `Run Rust tests` step with 2 failed tests:

```text
---- modules::user_token_db::tests::test_create_and_query_token stdout ----
thread 'modules::user_token_db::tests::test_create_and_query_token' panicked at src/modules/user_token_db.rs:718:9:
assertion failed: token_res.is_ok()

---- modules::user_token_db::tests::test_never_expire_token_validation stdout ----
thread 'modules::user_token_db::tests::test_never_expire_token_validation' panicked at src/modules/user_token_db.rs:731:48:
called `Result::unwrap()` on an `Err` value: PoisonError { .. }

failures:
    modules::user_token_db::tests::test_create_and_query_token
    modules::user_token_db::tests::test_never_expire_token_validation

test result: FAILED. 610 passed; 2 failed; finished in 7.67s
```

---

## 2. Root Cause

1. **Process-Global Environment Mutation**:
   In `src-tauri/src/modules/account.rs`, tests `test_set_current_account_id_with_target` and `task_quota_refresh_keeps_unexpired_live_limit` modify process-global environment variable `ABV_DATA_DIR` (`std::env::set_var("ABV_DATA_DIR", dir.path())`) to point to a temporary test directory (`TestDataDir`). Upon test completion, `TestDataDir` is dropped and deletes that directory from disk.
2. **Unsynchronized Cross-Module Concurrency**:
   `account.rs` guarded its test execution with an internal mutex `TEST_MUTEX`, while `user_token_db.rs` guarded its tests with a separate internal mutex `TEST_TOKEN_DB_MUTEX`.
3. **Database Disconnection / Schema Drift**:
   When `cargo test` runs tests in parallel across worker threads, `user_token_db::tests::test_create_and_query_token` executes concurrently. Its calls to `get_db_path()` invoke `crate::modules::account::get_data_dir()`, which reads `ABV_DATA_DIR`. When `ABV_DATA_DIR` pointed to the temporary directory of `account.rs`, `init_db()` or `create_token()` attempted database operations inside the transient directory that was either uninitialized or concurrently deleted, resulting in `token_res` failure and mutex poisoning.

---

## 3. Resolution

1. **Unified Environment Synchronization Mutex**:
   Exported `pub static TEST_DATA_DIR_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());` in `src-tauri/src/modules/account.rs` to synchronize any test mutating or relying on `ABV_DATA_DIR`.
2. **Synchronized Account Tests**:
   Updated `test_set_current_account_id_with_target` and `task_quota_refresh_keeps_unexpired_live_limit` in `account.rs` to acquire `TEST_DATA_DIR_MUTEX.lock().unwrap_or_else(|e| e.into_inner())`.
3. **Hermetic Test Isolation in `user_token_db`**:
   Refactored `modules::user_token_db::tests` to:
   - Acquire `crate::modules::account::TEST_DATA_DIR_MUTEX`.
   - Use an RAII `EnvDataDirGuard` wrapping `tempfile::tempdir()` that sets and automatically cleans up `ABV_DATA_DIR` on drop.
   - Assert `init_db().is_ok()` and `create_token(...).is_ok()` with explicit error detail logging.

---

## 4. Verification & Prevention

1. **Verification**:
   - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` verified clean.
   - Push to `main` branch to trigger CI validation across all runners.
2. **Prevention**:
   - All tests modifying environment variables must synchronize on a shared mutex and use RAII guards (`Drop`) to guarantee variable cleanup.
   - Database tests should execute in hermetic, per-test temporary directories instead of the user home directory.
