# RCA: Instance Account Switching & Session Isolation Failure

## 1. Symptoms & Observed Behavior

When users launch multiple Antigravity instances (e.g. "Default" and "Work" or cloned profiles), both instances display the exact same Google account email (e.g., `ashleyescobarzu@gmail.com`).
Even when:
1. Different accounts are selected or bound to specific profiles via the instance management drawer.
2. An instance profile is duplicated from an existing profile to use with a new account.
3. Multiple instance windows are opened concurrently with distinct `--user-data-dir` arguments.

The secondary instance fails to present its bound account and instead displays the primary profile's credentials and quota status.

## 2. Root Cause Analysis (Code Defect)

The defect stems from three decoupled state-propagation breakdowns across `src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/db.rs`, and `src-tauri/src/modules/integration.rs`:

### A. Missing SQLite Database Token Injection on Launch
In `src-tauri/src/modules/instance.rs`, the `launch_instance` function previously executed:
```rust
if let Some(ref account_id) = bound_acc {
    if let Ok(account) = crate::modules::account::load_account(account_id) {
        let _ = crate::modules::integration::write_to_system_keyring(&account);
    }
}
```
It synchronized the bound account's credentials solely to the global system keyring (Windows Credential Manager / macOS Keychain / Secret Service). However:
- VS Code forks and Antigravity cache the active session state, user tokens, and OAuth preferences inside the instance's isolated SQLite database:
  `{data_dir}/User/globalStorage/state.vscdb`
- The keys `antigravityUnifiedStateSync.oauthToken`, `antigravityUnifiedStateSync.userStatus`, and `telemetry.serviceMachineId` inside `state.vscdb` were never injected or updated during `launch_instance`.
- As a result, Antigravity read its existing cached session from `state.vscdb`, completely ignoring the bound account.

### B. Session Inheritance During Instance Duplication
In `copy_instance`, the directory tree `{source_data_dir}` was recursively copied to `{new_instance_data_dir}`.
Because the source instance's `state.vscdb` contained the active authentication tokens of the source user, the newly created clone inherited those exact tokens. The cloned instance was registered with `bound_account_id: None`, so `launch_instance` never overwrote the copied credentials.

### C. Missing Process Executable Isolation
For non-default instances with `executable_path: None`, `launch_instance` defaulted to executing the global base binary (`Antigravity.exe`). On Windows, launching identical executables with different `--user-data-dir` flags can lead Windows Shell and Electron single-instance locks to broker windows into the existing primary process rather than spawning an isolated runtime.

## 3. Architectural Fix

### A. Direct SQLite Token Injection in `launch_instance`
In `src-tauri/src/modules/instance.rs`, `launch_instance` now directly updates the instance's isolated SQLite database before process spawn:
```rust
if let Some(ref account_id) = bound_acc {
    if let Ok(account) = crate::modules::account::load_account(account_id) {
        let _ = crate::modules::integration::write_to_system_keyring(&account);

        let db_dir = target_data_path.join("User").join("globalStorage");
        let has_db_dir = db_dir.exists();
        if !has_db_dir {
            let _ = fs::create_dir_all(&db_dir);
        }
        let db_path = db_dir.join("state.vscdb");
        let _ = crate::modules::db::inject_token(
            &db_path,
            &account.token.access_token,
            &account.token.refresh_token,
            account.token.expiry_timestamp,
            &account.email,
            account.token.is_gcp_tos,
            account.token.project_id.as_deref(),
            account.token.id_token.as_deref(),
            account.token.oauth_client_key.as_deref(),
            None,
        );

        if let Some(ref profile) = account.device_profile {
            let _ = crate::modules::db::write_service_machine_id(&db_path, &profile.mac_machine_id);
        }
    }
}
```

### B. Cloned Instance Session Sanitization
In `src-tauri/src/modules/db.rs`, introduced `sanitize_session(db_path: &Path)`:
- Opens `state.vscdb` and deletes all authentication tokens (`antigravityUnifiedStateSync.oauthToken`, `antigravityUnifiedStateSync.userStatus`, `antigravityUnifiedStateSync.enterprisePreferences`, `jetskiStateSync.agentManagerInitState`, and `antigravityOnboarding`).
- In `copy_instance`, whenever an instance is cloned, `sanitize_session` is executed on the target's `state.vscdb` so the clone starts with a clean, unauthenticated session until an account is bound.

### C. Automatic Executable Isolation on Launch
In `launch_instance`, when `executable_path` is `None` on a non-default profile, `clone_instance_executable(instance_id)` is automatically invoked:
- Clones or hardlinks the executable to `Antigravity-{instance_id}.exe`.
- Provides fallback to `std::fs::copy` and launcher scripts if hardlinking is restricted.
- Guarantees distinct process names and binary paths in Windows Task Manager, ensuring complete OS-level process isolation.

## 4. Prevention Guardrails

1. **Mandatory DB State Injection:** All instance launch and switch routines must inject credentials directly into the target profile's `{data_dir}/User/globalStorage/state.vscdb` rather than relying solely on the system keyring.
2. **Clone Sanitization:** Any workflow that duplicates profiles or configuration trees must explicitly sanitize credentials and telemetry machine identifiers.
3. **Automated Verification:** CI/CD and unit tests must verify that `copy_instance` yields an unauthenticated database and that `launch_instance` successfully writes the bound account's email into `antigravityUnifiedStateSync.userStatus`.
