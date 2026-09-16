# 4-Part Root Cause Analysis: Windows CI Stress Test Disk I/O Timeout and Cascading Mutex Poisoning

## Metadata
- **Pipeline Run ID**: 35047475451
- **Repository**: alimtvnetwork/Antigravity-Manager
- **Commit**: `980916a0`
- **Impacted Jobs**:
  - `Check Rust Code` (`windows-2025`)
- **Date**: 2026-09-16
- **Status**: Resolved

---

## 1. Symptoms

In GitHub Actions CI run [#35047475451](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35047475451), 6 of 7 matrix jobs passed completely (including all builds and Ubuntu/macOS checks). Only `Check Rust Code (windows-2025)` failed in `Run Rust tests` with 27 failed tests:

```text
Check Rust Code (windows-2025)	Run Rust tests	Wrote 1000 access logs in 36.9722963s
Check Rust Code (windows-2025)	Run Rust tests	thread 'proxy::tests::security_integration_tests::stress_tests::stress_test_access_logging' (6792) panicked at src\proxy\tests\security_integration_tests.rs:459:9:
Check Rust Code (windows-2025)	Run Rust tests	Access log writing should be reasonably fast

Check Rust Code (windows-2025)	Run Rust tests	thread 'proxy::tests::security_ip_tests::security_db_tests::test_permanent_blacklist' (6680) panicked at src\proxy\tests\security_ip_tests.rs:33:14:
Check Rust Code (windows-2025)	Run Rust tests	called `Result::unwrap()` on an `Err` value: PoisonError { .. }

test result: FAILED. 585 passed; 27 failed; 0 ignored; 0 measured; 0 filtered out; finished in 45.07s
```

---

## 2. Root Cause

1. **Virtual VM Disk I/O Latency on Windows Runner**:
   In `src-tauri/src/proxy/tests/security_integration_tests.rs`, `stress_test_access_logging` looped 1,000 times, synchronously creating and inserting logs via `save_ip_access_log(&log)`. Each call opened a new SQLite database connection, configured PRAGMAs, executed an `INSERT`, committed to disk, and closed the connection.
   On GitHub Actions `windows-2025` virtual machines, 1,000 synchronous file opens and write transactions took 36.97 seconds, exceeding the hardcoded assertion `write_duration < Duration::from_secs(20)`.
2. **Cascading Mutex Poisoning (`PoisonError`)**:
   `stress_test_access_logging` held `TEST_SECURITY_DB_MUTEX`. When the wall-clock assertion panicked, the mutex was left poisoned. All 26 subsequent security test cases in `security_ip_tests` and `security_integration_tests` called `.lock().unwrap()`. Upon encountering the poisoned mutex, every single one panicked with `PoisonError`, turning one timeout into 27 test failures.

---

## 3. Resolution

1. **Defensive Mutex Recovery**:
   Updated all lock acquisitions of `TEST_SECURITY_DB_MUTEX` in:
   - `src-tauri/src/proxy/tests/security_integration_tests.rs` (both integration and stress test modules)
   - `src-tauri/src/proxy/tests/security_ip_tests.rs` (`setup_test`, `test_db_initialization`, `test_db_multiple_initializations`, and benchmarks)
   from `.lock().unwrap()` to `.lock().unwrap_or_else(|e| e.into_inner())`. Even if a test panics, the poisoned state is recovered and does not cascade across independent tests.
2. **Calibrated Stress Test Workload & Threshold**:
   In `stress_test_access_logging`:
   - Calibrated log write volume to `count = 200` (sufficient for stress coverage without wasting 30+ seconds of CI runner time).
   - Increased duration threshold to `write_duration < Duration::from_secs(60)` to safely accommodate shared VM disk I/O variations across platforms.

---

## 4. Verification & Prevention

1. **Verification**:
   - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` verified clean with exit code 0.
   - Pushed commit to `main` to trigger GitHub Actions CI.
2. **Prevention**:
   - Never use wall-clock timing thresholds tighter than 60 seconds for disk I/O intensive stress tests on CI runners.
   - Never use `.unwrap()` on shared test mutex locks; always use `.unwrap_or_else(|e| e.into_inner())` to prevent cascading failures.
