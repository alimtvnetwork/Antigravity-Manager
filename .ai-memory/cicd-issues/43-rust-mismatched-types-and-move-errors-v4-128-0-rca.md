# CI/CD RCA 43: Rust Mismatched Types and Drop Move Errors in Release v4.128.0

## 1. Symptoms

GitHub Actions release and CI workflows failed across all platforms on commit `bcbefca` (Run `#37123726276` and `#37123722175`):

```text
error[E0308]: mismatched types
   --> src/modules/instance.rs:3642:9
   |
   3642 | /         crate::modules::process::get_antigravity_executable_path(None)
   3643 | |             .or_else(|| crate::modules::process::get_antigravity_executable_path(Some("ide")))
   3644 | |             .unwrap_or_default()
   | |________________________________^ expected `String`, found `PathBuf`

error[E0308]: mismatched types
   --> src/modules/account.rs:1722:13
   |
   1722 |             ide_path,
   |             ^^^^^^^^ expected `String`, found `PathBuf`

error[E0509]: cannot move out of type `AuditTask`, which implements the `Drop` trait
   --> src/modules/task_history_db.rs:393:8
   |
   393 |     Ok(task.id)
   |        ^^^^^^^ cannot move out of here

error[E0509]: cannot move out of type `AuditTask`, which implements the `Drop` trait
   --> src/modules/task_history_db.rs:418:8
   |
   418 |     Ok(task.id)
   |        ^^^^^^^ cannot move out of here
```

## 2. Root Cause

1. `get_antigravity_executable_path()` returns `Option<PathBuf>`, but `SwitchFacts.ide_path` expects a `String`. In both `instance.rs` and `account.rs`, `unwrap_or_default()` yielded a `PathBuf` without calling `.map(|p| p.to_string_lossy().to_string())`.
2. In `task_history_db.rs`, `record_scheduler_event` and `record_requeue_event` returned `Ok(task.id)`. Because `AuditTask` implements `Drop`, fields cannot be moved out of `task`.

## 3. Surgical Fix

- In `src-tauri/src/modules/instance.rs`: mapped `get_antigravity_executable_path()` via `.map(|p| p.to_string_lossy().to_string())`.
- In `src-tauri/src/modules/account.rs`: mapped `get_antigravity_executable_path()` via `.map(|p| p.to_string_lossy().to_string())`.
- In `src-tauri/src/modules/task_history_db.rs`: returned `Ok(task.id.clone())` on lines 393 and 418.

## 4. Verification

- `cd src-tauri && cargo fmt -- --check`: Exit 0.
- `npm run build`: Exit 0.
- `gitmap pe -t`: Confirmed telemetry diagnosis and failure tree.
