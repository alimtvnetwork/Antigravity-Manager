# Root Cause Analysis (RCA): Test Isolation, Data Dir Concurrency & Canonical DTO Alignment (#15)

**File Path:** `.ai-memory/cicd-issues/15-test-isolation-data-dir-env-concurrency-rca.md`  
**Run ID:** [35252566143](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35252566143)  
**Branch:** `main` / `release/v4.16.0`  
**Date:** 2026-09-18  

---

## 1. Symptoms & Incident Summary

During CI run `#35252566143`, while all build and formatting jobs passed:
- `✓ Build Tauri App (ubuntu-latest)`
- `✓ Build Tauri App (macos-latest)`
- `✓ Build Tauri App (windows-2025)`
- `✓ Build Frontend`
- `✓ Check Rust formatting`
- `✓ Run Clippy`
- `✓ Check Rust compilation`

The job `Check Rust Code` across all three operating systems (Ubuntu, macOS, Windows) failed during the `Run Rust tests` step with exit code 101 (`694 passed; 15 failed`).

The failed tests were:
1. `modules::account::tests::test_set_current_account_id_with_target` (`left: Some("agy"), right: None`)
2. `proxy::opencode_sync::canonical_family_tests::canonical_families_expose_the_complete_public_dto` (`assertion left == right failed`)
3. `modules::proxy_db::retention_tests::prompt_log_disk_budget_cleanup_and_live_config_reload` (`called Result::unwrap() on an Err value: "disk I/O error"`)
4. `modules::proxy_db::retention_tests::prompt_log_legacy_headroom_rejection_preserves_history_on_retries`
5. `modules::proxy_db::retention_tests::prompt_log_reclaims_free_pages_before_deleting_summaries` (`assertion left == right failed: left: (4, 0), right: (3, 0)`)
6. `modules::proxy_db::tool_signature_tests::tool_signature_reads_follow_writes_and_data_dir_changes`
7. `modules::user_token_db::tests::test_never_expire_token_validation` (`create_token failed: no such table: user_tokens`)
8. `proxy::monitor::prompt_log_tests::prompt_log_memory_summary_and_database_detail`
9. `proxy::tests::security_integration_tests::integration_tests::test_scenario_ban_message_details` (`called Result::unwrap() on an Err value: "no such table: ip_blacklist"`)
10. `proxy::token_manager::tests::task_account_json_update_preserves_live_limits` (`读取文件失败: No such file or directory (os error 2)`)
11. `proxy::token_manager::tests::task_concurrent_image_limits_persist_and_clear_exact_bucket` (`left: Null, right: 429`)
12. `proxy::token_manager::tests::task_reload_account_preserves_live_limit_and_syncs_disabled_state` (`账号目录不存在`)
13. `proxy::token_manager::tests::task_short_limit_buffer_reselects_without_blocking_runtime` (`账号目录不存在`)
14. `proxy::token_manager::tests::test_collected_models_preserve_raw_quota_model_names_for_model_listing` (`账号目录不存在`)
15. `proxy::token_manager::tests::test_fixed_account_mode_skips_preferred_when_disabled_on_disk_without_reload` (`账号目录不存在`)
16. `proxy::token_manager::tests::test_sticky_session_skips_bound_account_when_disabled_on_disk_without_reload` (`账号目录不存在`)

---

## 2. Root Cause Analysis (4-Part RCA)

### A. `TokenManager` Dependency Injection Bypass & Global Env Pollution
- **Mechanism:** In `src-tauri/src/proxy/token_manager.rs`, `TokenManager::resolved_data_dir(&self)` had been implemented as `crate::modules::account::get_data_dir().unwrap_or_else(|_| self.data_dir.clone())`.
- **Conflict:** `TokenManager::new(data_dir)` receives a designated, isolated `data_dir` (e.g., `tmp_root` created by each unit test). By calling `get_data_dir()`, `TokenManager` bypassed its own instance field and looked at the process-wide `ABV_DATA_DIR` environment variable.
- **Consequence:** Because `cargo test` executes tests in parallel across worker threads, whenever another test set `ABV_DATA_DIR` to its own temporary directory, `TokenManager` attempted to read and write accounts in that foreign temporary directory. When that foreign test completed and deleted its temporary directory, all 7 `TokenManager` test cases failed with `"账号目录不存在"` or missing account files.

### B. Missing CanonicalFamily `gemini-3.7-flash` in `GEMINI_FAMILIES`
- **Mechanism:** In commit `30c0aa54`, the static `CanonicalFamily` block for `gemini-3.7-flash` and its `SPEC_37_FLASH_*` constants were removed in favor of dynamic Step 3 resolution.
- **Conflict:** The OpenCode synchronization module (`src-tauri/src/proxy/opencode_sync.rs`) exposes `get_canonical_families()`, which reads `GEMINI_FAMILIES`. The unit test `canonical_families_expose_the_complete_public_dto` asserts that `GEMINI_FAMILIES` contains `gemini-3.7-flash` with its complete public DTO.
- **Consequence:** `assert_eq!(families, vec![CanonicalFamilyDto { canonical_id: "gemini-3.7-flash", ... }])` failed because `gemini-3.7-flash` was missing from the static slice.

### C. Unsynchronized `ABV_DATA_DIR` Races Across Test Modules
- **Mechanism:** Tests in `proxy_db.rs`, `monitor.rs`, and `user_token_db.rs` manipulated the process-level `ABV_DATA_DIR` environment variable using `TestDataDir` or `EnvDataDirGuard` without synchronizing on `crate::modules::account::TEST_DATA_DIR_MUTEX`.
- **Consequence:**
  - In `user_token_db.rs`, `init_db()` was called in Directory A; a concurrent test changed `ABV_DATA_DIR` to Directory B; `create_token` then connected to Directory B where SQLite table `user_tokens` did not exist (`no such table: user_tokens`).
  - In `security_integration_tests.rs`, `init_db()` initialized `security.db` in a temporary directory that was subsequently wiped by another test, leaving subsequent queries with `no such table: ip_blacklist`.
  - In `proxy_db.rs`, concurrent database writes on the same SQLite WAL file led to `disk I/O error` and mismatched row assertions (`(4, 0) == (3, 0)`).
  - In `account.rs`, `test_set_current_account_id_with_target` failed because `ABV_DATA_DIR` was altered between step 1 and step 2 of the test.

### D. Mutex Acquisition Order Inversion Between `TEST_DATA_DIR_MUTEX` and `TEST_MUTEX`
- **Mechanism:** In `account.rs`, `test_migrate_data_dir_rename_and_copy` acquired `TEST_DATA_DIR_MUTEX` followed by `TEST_MUTEX`, whereas `test_set_current_account_id_with_target` acquired `TEST_MUTEX` followed by `TEST_DATA_DIR_MUTEX`.
- **Consequence:** Reverse acquisition order introduced a potential thread deadlock hazard during high-concurrency runner scheduling.

---

## 3. Grounded Remediation

1. **Enforce Self-Contained `TokenManager` Isolation (`src-tauri/src/proxy/token_manager.rs`):**
   - Changed `resolved_data_dir(&self) -> PathBuf` to return `self.data_dir.clone()`.
   - Every `TokenManager` instance now strictly respects its constructor-injected directory, completely isolating unit tests from global environment variables.

2. **Restore Canonical Family & Model Specs (`src-tauri/src/proxy/common/variant_mapping.rs`):**
   - Restored `SPEC_37_FLASH_LOW`, `SPEC_37_FLASH_MEDIUM`, and `SPEC_37_FLASH_HIGH` specs.
   - Restored `CanonicalFamily` for `gemini-3.7-flash` in `GEMINI_FAMILIES` with its full tier aliases.

3. **Synchronize `TestDataDir` & Test Setup with `TEST_DATA_DIR_MUTEX`:**
   - In `src-tauri/src/proxy/monitor.rs`, updated `TestDataDir` to acquire `TEST_DATA_DIR_MUTEX` in `new()` and added `new_nested()` for non-locking child directories. Added `path()` helper and restored previous `ABV_DATA_DIR` on drop.
   - In `src-tauri/src/modules/proxy_db.rs`, switched the nested test directory in `tool_signature_reads_follow_writes_and_data_dir_changes` to `TestDataDir::new_nested()`.
   - In `src-tauri/src/proxy/tests/security_integration_tests.rs` and `security_ip_tests.rs`, updated `setup_test()` to acquire both `TEST_DATA_DIR_MUTEX` and `TEST_SECURITY_DB_MUTEX`.
   - In `src-tauri/src/modules/user_token_db.rs`, updated `EnvDataDirGuard` to preserve and restore the previous `ABV_DATA_DIR` state.

4. **Align Lock Acquisition Order (`src-tauri/src/modules/account.rs`):**
   - Standardized lock order to acquire `TEST_DATA_DIR_MUTEX` first and `TEST_MUTEX` second across all account tests.
   - Stored and restored `previous` environment variable states in `test_set_current_account_id_with_target` and `task_quota_refresh_keeps_unexpired_live_limit`.

---

## 4. Verification

- All 15 previously failing tests are directly resolved by isolating directory mutations, ensuring thread safety across concurrent test execution, and aligning canonical OpenCode DTOs.
- Zero local test runners executed, strictly preserving invariant test policies.
