# CI/CD Issue RCA: Security Test Setup Return Type Tuple Rustfmt Divergence

> **Version:** 1.0.0
> **Date:** 2026-09-18
> **Failed Runs:** [#35259953276](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35259953276) (CI)
> **Trigger Commit:** `ce5f2ab8`
> **Fixed In:** `HEAD`

---

## 4-Part Root Cause Analysis (RCA)

### 1. Symptom
In GitHub Actions CI pipeline [#35259953276](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35259953276), all three `Check Rust Code` matrix jobs (`macos-latest`, `ubuntu-latest`, and `windows-2025`) failed during the `cargo fmt -- --check` step:
```text
Diff in src-tauri/src/proxy/tests/security_integration_tests.rs:12:
     use std::time::Duration;
 
     /// 辅助函数：初始化测试并加锁隔离
-    fn setup_test() -> (std::sync::MutexGuard<'static, ()>, std::sync::MutexGuard<'static, ()>) {
+    fn setup_test() -> (
+        std::sync::MutexGuard<'static, ()>,
+        std::sync::MutexGuard<'static, ()>,
+    ) {

Diff in src-tauri/src/proxy/tests/security_integration_tests.rs:363:
     use std::time::{Duration, Instant};
 
     /// 辅助函数：初始化测试并加锁隔离
-    fn setup_test() -> (std::sync::MutexGuard<'static, ()>, std::sync::MutexGuard<'static, ()>) {
+    fn setup_test() -> (
+        std::sync::MutexGuard<'static, ()>,
+        std::sync::MutexGuard<'static, ()>,
+    ) {

Diff in src-tauri/src/proxy/tests/security_ip_tests.rs:27:
     }
 
     /// 辅助函数：初始化测试并加锁隔离
-    fn setup_test() -> (std::sync::MutexGuard<'static, ()>, std::sync::MutexGuard<'static, ()>) {
+    fn setup_test() -> (
+        std::sync::MutexGuard<'static, ()>,
+        std::sync::MutexGuard<'static, ()>,
+    ) {
```

### 2. Root Cause
In commit `ce5f2ab8`, `setup_test()` in both security integration test modules and security IP test module was updated to return a pair of guard locks `(std::sync::MutexGuard<'static, ()>, std::sync::MutexGuard<'static, ()>)` in order to isolate `TEST_DATA_DIR_MUTEX` alongside `TEST_SECURITY_DB_MUTEX`.
While the single-line declaration was 98 characters (under the nominal 100 column limit), `rustfmt` formats compound tuple return types with long type parameters across multiple indented lines. Because this formatting was written on a single line, `cargo fmt -- --check` exited with status code 1 on all platforms.

### 3. Resolution
1. Reformatted `fn setup_test()` return type signature in `src-tauri/src/proxy/tests/security_integration_tests.rs` (both occurrences at lines 15 and 366) to multiline format matching `rustfmt`.
2. Reformatted `fn setup_test()` return type signature in `src-tauri/src/proxy/tests/security_ip_tests.rs` (line 30) to multiline format matching `rustfmt`.

### 4. Verification & Prevention
- Checked the exact diff against rustfmt logs from CI job logs.
- Verified that all lines conform to standard rustfmt formatting conventions for tuple returns.
- Pushed commit to `main` and triggered CI to verify format check passes cleanly.
