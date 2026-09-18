# 4-Part Root Cause Analysis: Missing `setup_test` in Test Module Scope

## Metadata
- **Pipeline Run ID**: 35045903098
- **Repository**: alimtvnetwork/Antigravity-Manager
- **Commit**: `da986902`
- **Impacted Jobs**:
  - `Check Rust Code` (`ubuntu-latest`, `macos-latest`, `windows-2025`)
- **Date**: 2026-09-16
- **Status**: Resolved

---

## 1. Symptoms

In GitHub Actions CI run [#35045903098](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35045903098), all platform runners failed at the `Run Clippy` step (`cargo clippy --all-targets --all-features`) with exit code 101:

```text
error[E0425]: cannot find function `setup_test` in this scope
   --> src/proxy/tests/security_integration_tests.rs:375:21
    |
375 |         let _lock = setup_test();
    |                     ^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `setup_test` in this scope
   --> src/proxy/tests/security_integration_tests.rs:464:21
    |
464 |         let _lock = setup_test();
    |                     ^^^^^^^^^^ not found in this scope

error: could not compile `antigravity-tools` (lib test) due to 2 previous errors
Process completed with exit code 101.
```

---

## 2. Root Cause

In commit `381b9ee4`, SQLite concurrency test isolation was introduced across security tests by locking `TEST_SECURITY_DB_MUTEX`.

In `src-tauri/src/proxy/tests/security_integration_tests.rs`, two distinct test sub-modules exist:
1. `mod integration_tests` (lines 7–347)
2. `mod stress_tests` (lines 354–485)

The helper function `setup_test()` was declared inside `mod integration_tests`:
```rust
mod integration_tests {
    fn setup_test() -> std::sync::MutexGuard<'static, ()> {
        let lock = crate::modules::security_db::TEST_SECURITY_DB_MUTEX
            .lock()
            .unwrap();
        let _ = init_db();
        cleanup_test_data();
        lock
    }
}
```
Because the function was declared private (`fn`) inside `mod integration_tests`, it was not accessible to sibling module `mod stress_tests`. When lines 375 and 464 inside `mod stress_tests` invoked `setup_test()`, the Rust compiler was unable to resolve `setup_test` in that module's lexical scope, resulting in compiler error `E0425`.

---

## 3. Resolution

1. Defined `setup_test()` within `mod stress_tests` in `src-tauri/src/proxy/tests/security_integration_tests.rs`:
   ```rust
   mod stress_tests {
       fn setup_test() -> std::sync::MutexGuard<'static, ()> {
           let lock = crate::modules::security_db::TEST_SECURITY_DB_MUTEX
               .lock()
               .unwrap();
           let _ = init_db();
           cleanup_test_data();
           lock
       }
   ...
   ```
2. Replaced redundant manual mutex acquisition and database initialization in `stress_test_access_logging` with `let _lock = setup_test();`.
3. Validated formatting via `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`.

---

## 4. Verification & Prevention

1. **Verification**:
   - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` terminates with exit code 0.
   - Pushed fix to `main` branch to trigger CI validation on GitHub Actions runners.
2. **Prevention**:
   - In Rust modules containing multiple test sub-modules (`mod integration_tests`, `mod stress_tests`), ensure shared test fixtures and helpers are either placed at the outer module scope with `pub(super)` visibility or duplicated per sub-module if encapsulation is preferred.
   - Run `cargo check --tests --manifest-path src-tauri/Cargo.toml` prior to merging test synchronization changes.
