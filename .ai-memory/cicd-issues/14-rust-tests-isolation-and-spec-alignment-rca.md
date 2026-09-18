# Root Cause Analysis (RCA): Rust Test Isolation & Spec Alignment (#14)

**File Path:** `.ai-memory/cicd-issues/14-rust-tests-isolation-and-spec-alignment-rca.md`
**Run ID:** [35250711335](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35250711335)
**Branch:** `main` / `release/v4.16.0`
**Date:** 2026-09-18

---

## 1. Symptoms & Incident Summary

During CI run `#35250711335` on Ubuntu runners following the initial compilation fixes, frontend build passed (`✓ Build Frontend`), clippy passed (`✓ Run Clippy`), formatting passed (`✓ Check Rust formatting`), and syntax check passed (`✓ Check Rust compilation`).

However, the step `Run Rust tests` failed with exit code 101 (`689 passed; 20 failed`), reporting failures in:
- `modules::account::tests::test_migrate_data_dir_rename_and_copy`
- `modules::account::tests::test_missing_index_with_existing_accounts`
- `modules::account::tests::test_save_account_index_roundtrip`
- `modules::account::tests::test_set_current_account_id_with_target`
- `proxy::common::variant_mapping::tests::test_resolve_37_flash_variants`
- `proxy::mappers::claude::request::tests::test_claude_flash_thinking_budget_capping`
- `proxy::handlers::openai::stream_peek_tests::responses_tool_output_image_is_sent_as_inline_data`
- Cascading file-not-found / database / token manager failures across concurrent threads.

---

## 2. Root Cause Analysis (4-Part RCA)

### A. Millisecond Collision in `TestDataDir::new()` & Mutex Poisoning
- **Mechanism:** In `src-tauri/src/modules/account.rs`, `TestDataDir::new()` constructed temporary directory paths using `.as_millis()`. In high-speed CI execution on GitHub Actions runners, consecutive invocations within `test_migrate_data_dir_rename_and_copy` (`let src = TestDataDir::new(); let dest_parent = TestDataDir::new();`) completed within microseconds, generating identical paths.
- **Consequence:** `dest = dest_parent.path().join("moved_data")` was evaluated as a child directory of `src`, triggering the cycle detection guard `if is_nested_data_dir(&new_dir, &old_dir)` which returned `Err("Cannot migrate data directory into itself")`.
- **Cascading Failure:** The `.unwrap()` inside `test_migrate_data_dir_rename_and_copy` panicked while holding `TEST_MUTEX`, leaving it poisoned. Every subsequent account test crashed with `PoisonError`. Additionally, `TEST_DATA_DIR_MUTEX` was not held during `test_migrate_data_dir_rename_and_copy`, allowing parallel SQLite tests to access directories during migration.

### B. Obsolete `GEMINI_FAMILIES` Entry vs Universal Dynamic Resolution
- **Mechanism:** Upstream PR #2 introduced universal dynamic resolution (Step 3) for all Gemini >= 3.0 models (`gemini-3.7-flash`, `gemini-3.8-flash`, etc.) to generate calibrated `RealModelSpec` instances where `id` matches the canonical query.
- **Conflict:** A legacy `CanonicalFamily` for `gemini-3.7-flash` remained in `GEMINI_FAMILIES` with static specs mapped to `id: "gemini-3.7-flash-medium"`.
- **Consequence:** `resolve("gemini-3.7-flash", None)` matched the static family before reaching Step 3, returning `"gemini-3.7-flash-medium"` instead of `"gemini-3.7-flash"`, violating the assertion in `test_resolve_37_flash_variants`.

### C. Pro Model Thinking Budget Cap Spec Alignment
- **Mechanism:** In `src-tauri/src/proxy/mappers/claude/request.rs`, `test_claude_flash_thinking_budget_capping` verified thinking budget clamping. For `req_pro` (`gemini-2.0-pro-thinking-exp`) with budget 32,000, the test asserted `24576`.
- **Mismatch:** Under `model_specs::get_thinking_budget`, Pro models have an upper ceiling of 49,152 tokens. Hence, a requested budget of 32,000 is preserved without truncation. The test assertion had not been updated from the legacy 24k cap to match the modern spec.

### D. Image Defense Base64 Validation Minimum Length
- **Mechanism:** In `src-tauri/src/proxy/handlers/openai.rs`, test `responses_tool_output_image_is_sent_as_inline_data` mocked image data with 1-byte base64 `"AQ=="`.
- **Security Check:** Commit `8c4ad9a8` introduced `validate_and_sanitize_inline_data` in `src-tauri/src/proxy/mappers/common_utils.rs` requiring at least 8 base64 characters and 5 decoded bytes to protect upstream Gemini APIs against corrupt payloads. The 1-byte mock string was correctly rejected as invalid and omitted, causing the test to fail.

---

## 3. Grounded Remediation

1. **Nanosecond & Atomic Test Isolation (`src-tauri/src/modules/account.rs`):**
   - Added `static TEST_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);`
   - Switched `TestDataDir::new()` path formatting to use `.as_nanos()` and atomic increment counter, eliminating path collision risks entirely.
   - Updated `test_migrate_data_dir_rename_and_copy` to acquire both `TEST_DATA_DIR_MUTEX` and `TEST_MUTEX`.
   - Converted `.lock().unwrap()` across all account tests to `.lock().unwrap_or_else(|e| e.into_inner())` to immunize tests against mutex poisoning.

2. **Removed Obsolete Registry Entry (`src-tauri/src/proxy/common/variant_mapping.rs`):**
   - Removed the obsolete static `CanonicalFamily` block for `gemini-3.7-flash` and associated constants (`SPEC_37_FLASH_*`).
   - All `gemini >= 3` models now cleanly flow through the Step 3 universal dynamic resolver, correctly producing canonical IDs and calibrated budgets.

3. **Aligned Pro Thinking Test Assertion (`src-tauri/src/proxy/mappers/claude/request.rs`):**
   - Updated `test_claude_flash_thinking_budget_capping` to assert `32000` for `gemini-2.0-pro-thinking-exp`, matching `model_specs` Pro 49,152 headroom.

4. **Updated Mock Image Base64 (`src-tauri/src/proxy/handlers/openai.rs`):**
   - Replaced mock base64 `"AQ=="` in `responses_tool_output_image_is_sent_as_inline_data` with valid PNG header base64 `"iVBORw0KGgo="`, satisfying image-defense validation rules.

---

## 4. Verification & Prevention

- All 4 modified files strictly adhere to:
  - Implicit boolean checks (no `== true`).
  - Single polarity conditions (no `is_a && !is_b`).
  - Strictly relative git paths.
  - Zero local test runner execution during turn.
