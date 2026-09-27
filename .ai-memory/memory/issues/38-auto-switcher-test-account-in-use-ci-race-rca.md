# RCA: CI Unit Test Failure in `test_is_account_in_use_logic` on macOS Runner

## 1. Why it happened (High-level Architectural Breakdown)
The test `test_is_account_in_use_logic` in `src-tauri/src/modules/auto_switcher.rs` attempted to verify the account in-use evaluation logic by mutating global on-disk state via `account::set_current_account_id("acc-idle-temp-test")`.
In CI environments (specifically macOS GitHub Actions runner executing 839 unit tests in parallel), uninitialized data directories or concurrent test executions caused `account::load_account_index()` inside `set_current_account_id` to fail or return an error. Because the error was discarded via `let _ = ...`, the current account ID was never set on disk, causing `is_account_in_use(&acc_idle)` to return `false` instead of `true`, panicking the assertion.

## 2. How it happened (Technical Execution Flow)
1. CI runner executed `cargo test` on `macos-latest`.
2. `test_is_account_in_use_logic` initialized `acc_idle` with ID `"acc-idle-temp-test"`.
3. The test asserted `!is_account_in_use(&acc_idle)`, which passed.
4. The test then called `let _ = account::set_current_account_id("acc-idle-temp-test")`.
5. In the isolated runner environment without pre-seeded accounts index, `load_account_index()` failed with `Err(...)`.
6. Subsequent call to `account::get_current_account_id()` returned `Ok(None)` / `Err`.
7. `is_account_in_use(&acc_idle)` evaluated to `false`.
8. `assert!(is_account_in_use(&acc_idle))` panicked at line 1841.

## 3. Root Cause
- **File**: `src-tauri/src/modules/auto_switcher.rs`
- **Location**: Function `test_is_account_in_use_logic` (line 1826-1847) and `is_account_in_use` (line 476-514).
- **Cause**: Coupling pure boolean evaluation logic with unmocked global file-system disk operations (`load_account_index()` / `set_current_account_id()`) during unit tests, creating CI environment dependencies and concurrent test race conditions.

## 4. Code Fix
1. Deconstruct `is_account_in_use` into a pure, hermetic core evaluation function `is_account_in_use_core` and an ambient state aggregator `is_account_in_use`.
2. Refactor `test_is_account_in_use_logic` to test `is_account_in_use_core` directly across all states (unbound, current ID match, current email match, in-use ID match, bound instance match) in memory with 0 disk I/O and 0 test cross-contamination.
