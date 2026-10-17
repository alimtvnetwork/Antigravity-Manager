# Subtask Plan 01: Error Logs Remediation Implementation Plan

## Metadata & Context
- **Parent Plan**: `fleet-node-navigation-and-error-log-plan`
- **Specification Reference**: `02-spec/21-app/fleet-node-navigation-and-error-log-plan/01-error-logs-rca-and-remediation-spec.md`
- **Execution Mode**: PLAN MODE (Ready for future execution once code freeze lifts)
- **Problem Class**: Root Cause Remediation for 5 Systemic Error Log Patterns (20,700+ log events)

---

## File-by-File Remediation Checklist & Detailed Implementation Steps

```
[ ] Phase 1: Pattern 1 Remediation (Prompt Queue Retry Storm & Test Isolation)
    [ ] 1.1 `src-tauri/src/modules/backup_prompts_db/restore.rs`
    [ ] 1.2 `src-tauri/src/modules/backup_prompts_db/tests.rs`
    [ ] 1.3 `src-tauri/src/modules/repo_db/queue.rs`
    [ ] 1.4 Database Cleanup Script / Migration for Corrupt Test Prompts

[ ] Phase 2: Pattern 2 Remediation (Supabase Polling Storm & Log Verbosity)
    [ ] 2.1 `src/components/instances/fleet/useFleetMachines.ts`
    [ ] 2.2 `src-tauri/src/modules/supabase_client/client.rs`

[ ] Phase 3: Pattern 3 Remediation (Broken Pipe Panic Suppression)
    [ ] 3.1 `src-tauri/src/modules/logger.rs`
    [ ] 3.2 `src-tauri/src/bin/agm/main.rs`

[ ] Phase 4: Pattern 4 Remediation (SQLite Lock Upgrade Deadlock Elimination)
    [ ] 4.1 `src-tauri/src/modules/repo_db/sequences.rs`
    [ ] 4.2 `src-tauri/src/modules/repo_db/schema.rs`

[ ] Phase 5: Pattern 5 Remediation (Self-Healing storage.json & Profile Pre-seeding)
    [ ] 5.1 `src-tauri/src/modules/device/profiles.rs`
    [ ] 5.2 `src-tauri/src/modules/instance/create.rs`
    [ ] 5.3 `src-tauri/src/modules/instance/clone.rs`
```

---

### Phase 1: Pattern 1 Remediation (Prompt Queue Retry Storm & Test Isolation)

#### Step 1.1: Parameterize Backup DB Restore with Optional Custom Repo DB
- **File**: `src-tauri/src/modules/backup_prompts_db/restore.rs`
- **Action**: Add `custom_repo_db: Option<&str>` parameter to `restore_running_prompts_for_instance`.
- **Implementation Detail**:
  1. Update signature:
     ```rust
     pub fn restore_running_prompts_for_instance(
         instance_id: Option<&str>,
         keep_backup: bool,
         custom_file: Option<&str>,
         custom_repo_db: Option<&str>,
     ) -> Result<Vec<PromptBackupRecord>, String>
     ```
  2. In `restore_running_prompts`, update backward-compatible wrapper to pass `None` for `custom_repo_db`.
  3. When re-injecting prompts into `active_prompts`, call `repo_db::save_or_requeue_prompt_in_db(&active_p, custom_repo_db)`. If `custom_repo_db` is provided, connect to `custom_repo_db` instead of the global `repo_prompts.db`.

#### Step 1.2: Isolate Unit Tests from Production DB
- **File**: `src-tauri/src/modules/backup_prompts_db/tests.rs`
- **Action**: Pass an isolated temporary DB path in `test_restore_scoping_across_instances`.
- **Implementation Detail**:
  1. Create a temporary `repo_db_path`:
     ```rust
     let repo_db_path = temp_dir.join("test-repo-prompts.db");
     let custom_repo_file = repo_db_path.to_str().unwrap();
     ```
  2. Pass `Some(custom_repo_file)` into all calls to `restore_running_prompts_for_instance`.
  3. Verify that running `cargo test --bin antigravity-manager modules::backup_prompts_db` creates and cleans up the temporary files and does NOT modify the host's actual `repo_prompts.db`.

#### Step 1.3: Add Pre-Dispatch Directory Validation & Dead-Letter Status in Queue Dispatcher
- **File**: `src-tauri/src/modules/repo_db/queue.rs`
- **Action**: Validate `prompt.repo_path` existence BEFORE writing resume task snapshot and spawning `agy`.
- **Implementation Detail**:
  1. In `check_and_dispatch_enqueued_prompts`, at line 188 (before building `task_file`):
     ```rust
     let repo_path = Path::new(&prompt.repo_path);
     if !repo_path.is_dir() {
         crate::modules::logger::log_warn(&format!(
             "[PromptQueueScheduler] Workspace directory does not exist: {:?}. Quarantining prompt '{}' to 'failed_invalid_workspace'",
             repo_path, prompt.id
         ));
         crate::error::record_ignored(
             conn.execute(
                 "UPDATE active_prompts SET status = 'failed_invalid_workspace', updated_at = ?1 WHERE id = ?2",
                 params![now, &prompt.id],
             ),
             "quarantine prompt with missing workspace",
         );
         continue;
     }
     ```
  2. This guarantees that `fs::write(&task_file, ...)` is NEVER called with a non-existent parent directory, completely eliminating the `os error 3` storm.
  3. Furthermore, updating status to `'failed_invalid_workspace'` removes the prompt from `'queued'`/`'backed_up'`, allowing `count_enqueued_prompts(None)` to drop to 0, which transitions the scheduler from the fast 5-second ticker into the 30-second idle backoff.

#### Step 1.4: Purge Stuck Contaminated Prompts from Production DB
- **Action**: Add a one-time migration or cleanup helper in `src-tauri/src/modules/repo_db/schema.rs`:
  ```sql
  DELETE FROM active_prompts WHERE repo_path = '/nonexistent/repo' OR id LIKE 'prompt_inst_%';
  ```

---

### Phase 2: Pattern 2 Remediation (Supabase Polling Storm & Log Verbosity)

#### Step 2.1: Frontend Exponential Backoff on Network Failures
- **File**: `src/components/instances/fleet/useFleetMachines.ts`
- **Action**: Implement dynamic polling interval with exponential backoff on consecutive query failures.
- **Implementation Detail**:
  1. Add `consecutiveErrors` ref/state:
     ```typescript
     const [consecutiveErrors, setConsecutiveErrors] = useState<number>(0);
     ```
  2. Compute interval dynamically:
     ```typescript
     // Base 15s; doubles on each error: 15s -> 30s -> 60s -> 120s -> max 300s (5m)
     const currentInterval = Math.min(POLL_INTERVAL_SECONDS * Math.pow(2, consecutiveErrors), 300);
     ```
  3. In `fetchMachines`:
     - On catch / empty response due to transport error: `setConsecutiveErrors((prev) => Math.min(prev + 1, 5))`.
     - On successful response with valid data: `setConsecutiveErrors(0)`.
  4. Display backoff state cleanly in the UI countdown timer so the user sees when a paused Supabase project is cooling down.

#### Step 2.2: Downgrade Expected Network Transport Errors to `tracing::warn!`
- **File**: `src-tauri/src/modules/supabase_client/client.rs`
- **Action**: In `log_transport_error`, differentiate between normal connection drops / DNS NXDOMAIN and genuine unexpected errors.
- **Implementation Detail**:
  ```rust
  pub(crate) fn log_transport_error(&self, op: &str, url: &str, err: &reqwest::Error) {
      if err.is_connect() || err.is_timeout() {
          tracing::warn!(
              target: "supabase_client",
              operation = op,
              url = url,
              error = %err,
              is_timeout = err.is_timeout(),
              is_connect = err.is_connect(),
              "Supabase client {} transport warning on {}: {}",
              op,
              url,
              err
          );
      } else {
          let backtrace = std::backtrace::Backtrace::capture();
          tracing::error!(
              target: "supabase_client",
              operation = op,
              url = url,
              error = %err,
              status = ?err.status(),
              backtrace = %backtrace,
              "Supabase client {} unexpected transport error on {}: {}",
              op,
              url,
              err
          );
      }
  }
  ```

---

### Phase 3: Pattern 3 Remediation (Broken Pipe Panic Suppression)

#### Step 3.1: Panic Hook Broken Pipe Inspection & Clean Process Exit
- **File**: `src-tauri/src/modules/logger.rs`
- **Action**: In the panic hook, check for broken pipe panic signatures and exit 0 without writing crash dumps.
- **Implementation Detail**:
  1. Inspect `payload` at lines 121–127:
     ```rust
     let is_broken_pipe = payload.contains("failed printing to stdout")
         || payload.contains("The pipe is being closed")
         || payload.contains("os error 232")
         || payload.contains("os error 109")
         || payload.contains("Broken pipe")
         || payload.contains("os error 32");

     if is_broken_pipe {
         // Gracefully terminate CLI process without polluting crash logs
         std::process::exit(0);
     }
     ```
  2. This guarantees that `panic.log` and `crash_stacktrace.log` only record actual bugs and unhandled exceptions, not normal CLI pipeline terminations (`| head`, `| less`, etc.).

#### Step 3.2: CLI Main Entry Point Hygiene
- **File**: `src-tauri/src/bin/agm/main.rs`
- **Action**: Ensure all top-level commands handle pipe signals gracefully on POSIX systems via `libc::signal(libc::SIGPIPE, libc::SIG_DFL)`.

---

### Phase 4: Pattern 4 Remediation (SQLite Lock Upgrade Deadlock Elimination)

#### Step 4.1: Switch from `BEGIN DEFERRED` to `BEGIN IMMEDIATE`
- **File**: `src-tauri/src/modules/repo_db/sequences.rs`
- **Action**: Update `ensure_conversation_sequence_in_conn` and `ensure_project_sequence_in_conn`.
- **Implementation Detail**:
  1. Replace `conn.unchecked_transaction()`:
     ```rust
     let tx_res = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate);
     if let Ok(tx) = tx_res {
         // Existing query and update logic
         ...
         let _ = tx.commit();
     } else {
         // Fallback directly to single statement with busy handling
         ...
     }
     ```
  2. Because `BEGIN IMMEDIATE` acquires a `RESERVED` lock at transaction inception, no other writer can start, and no two readers holding `SHARED` locks will ever attempt a concurrent upgrade.

#### Step 4.2: Ensure Connection-Level Busy Timeout
- **File**: `src-tauri/src/modules/repo_db/schema.rs`
- **Action**: Verify that every connection created via `connect_db()` sets:
  ```rust
  conn.busy_timeout(std::time::Duration::from_millis(5000))?;
  ```

---

### Phase 5: Pattern 5 Remediation (Self-Healing storage.json & Profile Pre-seeding)

#### Step 5.1: Self-Healing Minimal Template in `write_profile`
- **File**: `src-tauri/src/modules/device/profiles.rs`
- **Action**: When `storage_path` does not exist, initialize a minimal valid JSON object instead of returning an error.
- **Implementation Detail**:
  ```rust
  pub fn write_profile(storage_path: &Path, profile: &DeviceProfile) -> Result<(), String> {
      if let Some(parent) = storage_path.parent() {
          if !parent.exists() {
              fs::create_dir_all(parent).map_err(|e| format!("create_dir_failed: {}", e))?;
          }
      }

      let mut json: Value = if storage_path.exists() {
          let content = fs::read_to_string(storage_path).map_err(|e| format!("read_failed: {}", e))?;
          serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
      } else {
          serde_json::json!({})
      };

      // Populate telemetry object and flat keys
      if !json.get("telemetry").map_or(false, |v| v.is_object()) {
          json["telemetry"] = serde_json::json!({});
      }

      if let Some(telemetry) = json.get_mut("telemetry").and_then(|v| v.as_object_mut()) {
          telemetry.insert("machineId".to_string(), Value::String(profile.machine_id.clone()));
          telemetry.insert("macMachineId".to_string(), Value::String(profile.mac_machine_id.clone()));
          telemetry.insert("devDeviceId".to_string(), Value::String(profile.dev_device_id.clone()));
          telemetry.insert("sqmId".to_string(), Value::String(profile.sqm_id.clone()));
      }

      if let Some(map) = json.as_object_mut() {
          map.insert("telemetry.machineId".to_string(), Value::String(profile.machine_id.clone()));
          map.insert("telemetry.macMachineId".to_string(), Value::String(profile.mac_machine_id.clone()));
          map.insert("telemetry.devDeviceId".to_string(), Value::String(profile.dev_device_id.clone()));
          map.insert("telemetry.sqmId".to_string(), Value::String(profile.sqm_id.clone()));
      }

      let formatted = serde_json::to_string_pretty(&json).map_err(|e| format!("serialize_failed: {}", e))?;
      fs::write(storage_path, formatted).map_err(|e| format!("write_failed: {}", e))?;
      Ok(())
  }
  ```

#### Step 5.2: Verification in `instance/create.rs` and `instance/clone.rs`
- **Files**: `src-tauri/src/modules/instance/create.rs`, `src-tauri/src/modules/instance/clone.rs`
- **Action**: Verify that calls to `write_profile(&storage_path, &profile)` now succeed on the first attempt without producing `WARN best_effort: write device profile: storage_json_missing`.

---

## Step-by-Step Execution Sequence for Future AI Implementer

1. **Step 1: Fix Test DB Pollution & Path Validation (Pattern 1)**:
   - Edit `src-tauri/src/modules/backup_prompts_db/restore.rs`.
   - Edit `src-tauri/src/modules/backup_prompts_db/tests.rs`.
   - Edit `src-tauri/src/modules/repo_db/queue.rs`.
   - Run targeted test: `cd src-tauri && cargo test --lib modules::backup_prompts_db::tests`.
2. **Step 2: Fix Panic Hook Broken Pipe Suppression (Pattern 3)**:
   - Edit `src-tauri/src/modules/logger.rs`.
   - Compile and verify with pipe: `agm instance list | head -n 1`.
3. **Step 3: Fix SQLite Transaction Locking (Pattern 4)**:
   - Edit `src-tauri/src/modules/repo_db/sequences.rs`.
   - Verify transaction behavior with concurrent read/write test.
4. **Step 4: Fix Self-Healing `storage.json` Creation (Pattern 5)**:
   - Edit `src-tauri/src/modules/device/profiles.rs`.
   - Run test creating a mock instance in a clean temp directory.
5. **Step 5: Fix Frontend Polling & Supabase Client Logging (Pattern 2)**:
   - Edit `src/components/instances/fleet/useFleetMachines.ts`.
   - Edit `src-tauri/src/modules/supabase_client/client.rs`.
   - Run frontend build: `npm run build`.
6. **Step 6: Pre-flight Verification**:
   - `cd src-tauri && cargo fmt -- --check`
   - `cd src-tauri && cargo clippy --all-targets --all-features`
   - `npm run build`

---

## Verification Matrix & Pre-Flight Commands

| Step | Verification Command | Expected Outcome |
| :--- | :--- | :--- |
| **P1** | `cd src-tauri && cargo test modules::backup_prompts_db` | All tests pass; host `repo_prompts.db` untouched; `active_prompts` count does not increment. |
| **P2** | `npm run build` | TypeScript builds clean without type errors; exponential backoff verified in unit/UI test. |
| **P3** | `agm prompt list \| head -n 2` | Exits with code 0; `panic.log` timestamp does not advance. |
| **P4** | `cd src-tauri && cargo test modules::repo_db::sequences` | Concurrency test executes without `SQLITE_BUSY` or deadlock. |
| **P5** | `agm instance create test_verify_p5` | Instance created; `storage.json` exists in `globalStorage`; zero warnings logged. |

---

## Rollback Strategy & Risk Mitigation
- All modifications are additive or surgical replacements:
  - If `BEGIN IMMEDIATE` causes lock timeouts under unexpected loads, it is safely bounded by the 5000ms `busy_timeout`.
  - The broken pipe exit in `logger.rs` specifically matches stdout/stderr pipe closed messages, leaving all genuine panics completely intact.
  - The `storage.json` self-healing logic creates missing files only when they do not exist, preserving existing files and their telemetry untouched.
