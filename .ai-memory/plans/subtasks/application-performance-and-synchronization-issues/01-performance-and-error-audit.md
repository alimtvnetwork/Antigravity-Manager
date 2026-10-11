# Subtask 01: Application Performance & Error Swallowing Remediation

Spec Reference: [01-performance-and-error-spec.md](../../../../02-spec/21-app/application-performance-and-synchronization-issues/01-performance-and-error-spec.md)

## Objective
Eliminate thread-blocking sleeps causing UI hangs and process glitches (`wait_for_instance_prompt_channel` and `close_instance_verified`), and audit/remediate all error swallowing sites across Rust backend modules and TypeScript frontend hooks according to the repository's "No Silent Errors" invariant.

---

## Target Files & Code Locations

1. `src-tauri/src/modules/instance/projects.rs` (Lines 140–170)
2. `src-tauri/src/modules/instance/close.rs` (Lines 71–151)
3. `src-tauri/src/modules/instance/switch.rs` (Lines 79–98, 143–161, 204–215, 401–440)
4. `src-tauri/src/bin/agm/instance_fastforward.rs` (Lines 78–95, 120–135, 158, 184)
5. `src-tauri/src/commands/instance.rs` (Lines 16, 49, 101, 360–371)
6. `src/components/settings/AutoSwitcher/useAutoSwitcher.ts` (Lines 34, 55, 144)
7. `src/components/instances/settingsModal/useInstanceSettings.ts` (Lines 63–65)

---

## Actionable Implementation Steps

### Step 1: Remediate 6,000ms Sleep in `projects.rs`
- In `src-tauri/src/modules/instance/projects.rs:150-157`:
  - Current flaw: `std::thread::sleep(std::time::Duration::from_millis(6000));` blocks execution thread unconditionally for 6 seconds.
  - Action: Replace the static 6,000ms sleep with an adaptive stabilization loop that checks for instance process/socket readiness in short intervals (100ms) with a maximum cap of 1,200ms.
  - Provide both sync adaptive helper `stabilize_prompt_channel_adaptive` and async variant `wait_for_instance_prompt_channel_async` using `tokio::time::sleep`.

```rust
// Replacement in projects.rs
if running {
    crate::modules::logger::log_info(&format!(
        "[Instance] Prompt channel detected for '{}' ({} pids after {:?}); performing adaptive stabilization",
        instance_id,
        pids.len(),
        start_time.elapsed()
    ));
    // Adaptive sleep: check liveness with short backoff up to 1200ms instead of hard 6000ms freeze
    let mut stabilized = false;
    let stab_deadline = std::time::Instant::now() + std::time::Duration::from_millis(1200);
    while std::time::Instant::now() < stab_deadline {
        if is_pid_alive_targeted(pids[0]) {
            stabilized = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    if !stabilized {
        crate::modules::logger::log_warn(&format!(
            "[Instance] Prompt channel stabilization reached 1.2s timeout for '{}'; proceeding",
            instance_id
        ));
    }
    return;
}
```

### Step 2: Implement Non-Blocking `close_instance_verified_async` in `close.rs`
- In `src-tauri/src/modules/instance/close.rs`:
  - Implement `pub async fn close_instance_verified_async(instance_id: &str, timeout: std::time::Duration) -> Result<bool, String>`.
  - Replace `std::thread::sleep(std::time::Duration::from_millis(200))` inside the polling loop with `tokio::time::sleep(std::time::Duration::from_millis(100)).await`.
  - Maintain the synchronous `close_instance_verified` for CLI commands, but bound its sleep interval to 100ms with early exit.
- In `src-tauri/src/modules/instance/switch.rs:204`:
  - Update `switch_account_to_instance` (which is `async`) to call `.await` on `close_instance_verified_async`.

### Step 3: Audit & Remediate Error Swallowing in `instance_fastforward.rs`
- In `src-tauri/src/bin/agm/instance_fastforward.rs`:
  - Line 78:
    ```rust
    // Replace: let _ = repo_db::backup_running_prompts(&target_instance);
    if let Err(e) = repo_db::backup_running_prompts(&target_instance) {
        crate::modules::logger::log_warn(&format!("[FastForward] Running prompt backup failed for '{}': {}", target_instance, e));
    }
    ```
  - Lines 92–93:
    ```rust
    // Replace bare let _ =
    if let Err(e) = repo_db::resend_running_commands_for_instance(Some(&target_instance), 20) {
        crate::modules::logger::log_warn(&format!("[FastForward] Resend running commands warning: {}", e));
    }
    if let Err(e) = repo_db::dispatch_running_prompts(&target_instance) {
        crate::modules::logger::log_warn(&format!("[FastForward] Dispatch running prompts warning: {}", e));
    }
    ```
  - Line 122: Replace `.ok()` with explicit match or error logging:
    ```rust
    let predicted_candidate = match auto_switcher::select_candidate_profiles(inst_ref, "gemini-2.5-pro", 15.0, &pred_exclusions) {
        Ok(candidates) => candidates.into_iter().next(),
        Err(e) => {
            crate::modules::logger::log_warn(&format!("[FastForward] Candidate selection failed: {}", e));
            None
        }
    }
    ```
  - Lines 158 & 184:
    ```rust
    // Replace unwrap_or_default() on JSON serialization
    match serde_json::to_string_pretty(&out) {
        Ok(json_str) => println!("{}", json_str),
        Err(err) => {
            eprintln!("[ERROR] JSON formatting error: {}", err);
            println!("{{\"success\":false,\"error\":\"JSON serialization failed\"}}");
        }
    }
    ```

### Step 4: Audit & Remediate Error Swallowing in `commands/instance.rs` & `switch.rs`
- In `src-tauri/src/commands/instance.rs`:
  - Lines 365–366: Replace bare `.unwrap_or_default()` on instance data dir lookup with explicit check and warning log when an instance ID is missing from registry.
- In `src-tauri/src/modules/instance/switch.rs`:
  - Line 82: If `load_account(id)` fails, log a warning instead of swallowing with `.ok()`.
  - Line 145: If `migrate_instance_workspaces` fails, log a warning:
    ```rust
    match migrate_instance_workspaces(&registry.active_instance_id, &instance.id) {
        Ok(ws) => Some(ws),
        Err(e) => {
            crate::modules::logger::log_warn(&format!("[Switch] Workspace migration failed: {}", e));
            None
        }
    }
    ```
  - Line 155: Replace `.unwrap_or(0)` with explicit error handling and warning log.
  - Line 413: Log warning if `list_running_projects()` returns an Err.

### Step 5: Remediate Frontend Error Swallowing in TypeScript
- In `src/components/settings/AutoSwitcher/useAutoSwitcher.ts`:
  - Line 34:
    ```typescript
    .catch((err) => {
        useErrorStore.getState().trackWarning(err, {
            source: 'useAutoSwitcher.fetchStatus',
            triggerAction: 'getAutoSwitcherDaemonStatus',
        });
    });
    ```
  - Line 55:
    ```typescript
    .catch((err) => {
        useErrorStore.getState().trackWarning(err, {
            source: 'useAutoSwitcher.eventListen',
            triggerAction: 'listen_status_tick',
        });
    });
    ```
  - Line 144:
    ```typescript
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'useAutoSwitcher.handleManualRotate',
            triggerAction: 'manual_rotate',
        });
        setRotationFeedback(`Error: ${e?.message || e?.toString() || 'Rotation failed'}`);
    }
    ```
- In `src/components/instances/settingsModal/useInstanceSettings.ts`:
  - Lines 63–65:
    ```typescript
    } catch (e) {
        useErrorStore.getState().trackWarning(e, {
            source: 'useInstanceSettings.initClipboard',
            triggerAction: 'read_clipboard_buffer',
        });
        return null;
    }
    ```

---

## Verification & Preflight Commands

Execute following verification commands locally:
```bash
# 1. Rust code formatting check
cd src-tauri && cargo fmt -- --check

# 2. Rust clippy gate (all features and targets)
cd src-tauri && cargo clippy --all-targets --all-features

# 3. Frontend compilation check
npm run build
```
