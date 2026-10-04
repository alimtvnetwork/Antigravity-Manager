---
plan: 118-instance-prompt-execution-status-fix
subtask: "01"
title: Backend Detection Remediation, Worker Scoping, Clone Sanitization & Structured Audit Logging
domain: backend-rust
depends_on: none
citations:
  app_spec: ../../../../02-spec/21-app/118-instance-prompt-execution-status-fix/01-architecture-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/repo_db.rs
  - src-tauri/src/modules/instance.rs
  - src-tauri/src/modules/logger.rs
status: pending
---

# Subtask 01: Backend Detection Remediation, Worker Scoping, Clone Sanitization & Structured Audit Logging

## 1. Context & Problem Statement
In multi-profile Antigravity topologies (Default Profile vs Cloned Instance `8159`), prompt and project running states were falsely reported and contaminated across instances:
- **Default Profile**: Actively executing **ONLY** `Antigravity-Manager`. SpecBuilder and coding-guidelines are strictly **IDLE**.
- **Instance 8159**: Actively executing **ONLY** `coding-guidelines`. Antigravity-Manager and SpecBuilder are strictly **IDLE**.
- SpecBuilder is idle in both instances.

Static analysis revealed 5 backend root causes:
1. `get_active_agy_workers()` in `is_prompt_running_for_project` checked `key.contains(project_id)` without scoping to `instance_id`.
2. `conversation_summaries.db` evaluation used inverted OR logic `not_fully_idle != 0 || status.contains("RUNNING")`, treating idle sessions as running if status had stale text.
3. Cloned instances inherited running states: `clone_instance_repo_rows` copied `is_running = 1` and `status = 'running'` directly from the source instance, and cloned directories carried active session summaries.
4. `compute_project_conversation_tree` treated `queued` and `backed_up` prompts as `is_running = true`, falsely marking idle projects as running.
5. In-flight `active_prompts` lacked TTL expiration and terminal transitions when unhandled failures occurred.

This subtask remediates the backend evaluation logic, enforces strict per-instance worker scoping, ensures complete sanitization during instance cloning, and implements verbose structured audit logging.

---

## 2. Target Files and Exact Symbols / Functions

### 2.1 `src-tauri/src/modules/logger.rs`
- **Symbols**:
  - `pub fn log_instance_prompt_audit(instance_id: &str, project_name: &str, repo_path: &str, is_instance_active: bool, is_running: bool, active_tasks: usize, rationale: &str)`
- **Role**: Standardizes structured audit logging for forensic traceability across all liveness checks.

### 2.2 `src-tauri/src/modules/repo_db.rs`
- **Symbols**:
  - `pub fn is_prompt_running_for_project(project_id: &str, instance_id: &str) -> bool`
    - Scope worker keys to `format!("{}:", instance_id)`.
    - Enforce explicit idle supremacy for `conversation_summaries.db`: if `not_fully_idle == 0` or status contains `IDLE`/`COMPLETED`/`FAILED`/`CANCELLED`, evaluate to `false`.
    - Check terminal transition and enforce active TTL for SQLite `active_prompts`.
    - Emit `log_instance_prompt_audit` with exact evaluation rationale.
  - `pub fn compute_project_conversation_tree(...) -> Vec<AgmProjectTreeNode>`
    - Correct `is_run` logic: `let is_run = is_inst_alive && ap.status == "running";` (exclude `"queued"` and `"backed_up"`).
    - Ensure conversation nodes for queued/backed-up tasks have `is_running: false`.
    - Project-level `proj_is_running` must reflect only true active execution.
  - `pub(crate) fn clone_repo_rows_on(conn: &Connection, source_id: &str, target_id: &str) -> Result<usize, String>`
    - Force `is_running = 0` when inserting cloned `running_projects`.
    - Sanitize `status` when inserting cloned `active_prompts` (map `"running"` and `"dispatched"` to `"completed"`).

### 2.3 `src-tauri/src/modules/instance.rs`
- **Symbols**:
  - `pub fn copy_instance_with_options(...) -> Result<InstanceConfig, String>`
  - `pub fn copy_instance_projects(from_id: &str, to_id: &str) -> Result<usize, String>`
  - Helper `sanitize_cloned_instance_summaries(target_id: &str) -> Result<(), String>`
    - Sanitize any copied `conversation_summaries.db` in the target instance data and home trees: set `not_fully_idle = 0` and status to `'IDLE'` so a newly cloned instance never starts with fake running sessions.

---

## 3. Step-by-Step Remediation Instructions

### Step 1: Refine Structured Audit Logging in `logger.rs`
Verify and refine `log_instance_prompt_audit`:
```rust
pub fn log_instance_prompt_audit(
    instance_id: &str,
    project_name: &str,
    repo_path: &str,
    is_instance_active: bool,
    is_running: bool,
    active_tasks: usize,
    rationale: &str,
) {
    info!(
        "[InstancePromptAudit] instance='{}' project='{}' path='{}' is_instance_active={} is_running={} active_tasks={} rationale='{}'",
        instance_id, project_name, repo_path, is_instance_active, is_running, active_tasks, rationale
    );
}
```

### Step 2: Remediate Worker Scoping & Terminal Transitions in `repo_db.rs`
In `is_prompt_running_for_project(project_id: &str, instance_id: &str) -> bool`:
1. **Worker Map Scoping**:
   ```rust
   let target_inst = if instance_id.is_empty() || instance_id == "__default__" {
       "default"
   } else {
       instance_id
   };
   let expected_prefix = format!("{}:", target_inst);
   
   if let Ok(mut workers) = get_active_agy_workers().lock() {
       let mut dead_keys = Vec::new();
       let mut found_running_worker = false;
       for (key, &pid) in workers.iter() {
           let matches_inst = target_inst == "all" || key.starts_with(&expected_prefix);
           if matches_inst && key.contains(project_id) {
               // Verify OS process liveness for PID
               let mut sys = sysinfo::System::new();
               let target_pid = sysinfo::Pid::from_u32(pid);
               sys.refresh_processes_specifics(
                   sysinfo::ProcessesToUpdate::Some(&[target_pid]),
                   sysinfo::ProcessRefreshKind::new().with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
               );
               if sys.process(target_pid).is_some() {
                   found_running_worker = true;
                   break;
               } else {
                   dead_keys.push(key.clone());
               }
           }
       }
       for k in dead_keys {
           workers.remove(&k);
       }
       if found_running_worker {
           crate::modules::logger::log_instance_prompt_audit(
               instance_id, project_id, project_id, true, true, 1, "ACTIVE_WORKER_MATCHED",
           );
           return true;
       }
   }
   ```

2. **SQLite Terminal Transition & Liveness TTL**:
   ```rust
   if let Ok(conn) = connect_db() {
       let running_count: usize = if has_terminal_transition {
           0 // Terminal transition ('completed'/'failed') overrides stale running rows
       } else {
           conn.query_row(
               "SELECT COUNT(*) FROM active_prompts 
                WHERE (project_id = ?1 OR repo_path = ?1) 
                  AND (?2 = '' OR ?2 = 'all' OR instance_id = ?2 OR ((?2 = 'default' OR ?2 = '__default__') AND (instance_id = 'default' OR instance_id = '__default__' OR instance_id IS NULL OR instance_id = '')))
                  AND status = 'running'
                  AND updated_at >= ?3",
               params![project_id, instance_id, now - 300], // 5-minute active TTL
               |r| r.get(0),
           )
           .unwrap_or(0)
       };
       if running_count > 0 {
           crate::modules::logger::log_instance_prompt_audit(
               instance_id, project_id, project_id, true, true, running_count, "ACTIVE_PROMPT_DB_RUNNING",
           );
           return true;
       }
   }
   ```

### Step 3: Remediate `conversation_summaries.db` Logic in `repo_db.rs`
Replace inverted OR logic with strict explicit idle supremacy:
```rust
for item in rows.flatten() {
    let (status, not_fully_idle, ws_uris_opt, _last_time_str) = item;
    
    // Strict idle supremacy rule:
    let is_explicit_idle = not_fully_idle == 0
        || status.contains("IDLE")
        || status.contains("COMPLETED")
        || status.contains("FAILED")
        || status.contains("CANCELLED");
        
    let is_conv_running = if is_explicit_idle {
        false
    } else {
        not_fully_idle != 0 && status.contains("RUNNING")
    };
    
    if is_conv_running {
        if let Some(ws_uris_raw) = ws_uris_opt {
            let ws_uris: Vec<String> = serde_json::from_str(&ws_uris_raw).unwrap_or_default();
            for u in ws_uris {
                let clean_p = normalize_path_for_compare(&decode_uri_to_path(&u));
                if !clean_target.is_empty()
                    && (clean_p == clean_target
                        || clean_p.contains(&clean_target)
                        || clean_target.contains(&clean_p))
                {
                    crate::modules::logger::log_instance_prompt_audit(
                        instance_id, project_id, &clean_p, true, true, 1, "CONVERSATION_SUMMARY_ACTIVE_TURN",
                    );
                    return true;
                }
            }
        }
    }
}
```

### Step 4: Fix Tree Node Liveness for Queued / Backed-up Prompts in `repo_db.rs`
In `compute_project_conversation_tree`:
```rust
// Strictly 'running' constitutes active execution; queued and backed_up are idle/waiting
let is_run = is_inst_alive && ap.status == "running";
```
Ensure `is_run` is assigned to `AgmConversationNode.is_running`.

### Step 5: Implement Cloning Sanitization in `repo_db.rs` and `instance.rs`
1. In `repo_db::clone_repo_rows_on`:
   ```rust
   // Force is_running = 0 for cloned running_projects
   conn.execute(
       "INSERT OR IGNORE INTO running_projects
        (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, 0, ?7, ?8)",
       rusqlite::params![new_id, target_id, name, path, storage, detected, updated],
   )?;
   
   // Sanitize active_prompts status
   let sanitized_status = match status.as_str() {
       "running" | "dispatched" => "completed".to_string(),
       other => other.to_string(),
   };
   conn.execute(
       "INSERT OR IGNORE INTO active_prompts
        (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
       rusqlite::params![
           new_prompt_id, new_id, target_id, path, content, model, session, sanitized_status, created, prompt_updated, image
       ],
   )?;
   ```

2. In `instance.rs`, add helper `sanitize_cloned_instance_summaries(target_id: &str)`:
   - Locates candidate `conversation_summaries.db` in `instances/{target_id}/home/.gemini/antigravity` and `instances/{target_id}/data`.
   - Executes:
     ```sql
     UPDATE conversation_summaries 
     SET not_fully_idle = 0, status = 'IDLE' 
     WHERE not_fully_idle != 0 OR status LIKE '%RUNNING%';
     ```
   - Invoke this helper at the end of `copy_instance_with_options`.

---

## 4. Architectural Constraints
- **Positive Booleans**: Name booleans positively (`is_instance_active`, `is_running`, `has_active_execution`, `matches_inst`). Never write `== true` or double negatives.
- **US English**: All identifiers, documentation, and log strings must use US English spelling (`sanitized`, `canceled`/`cancelled`).
- **No Production Panics**: Never call `.unwrap()` or `.expect()` in non-test production code. Return structured `Result` or handle errors with fallback logging.
- **Cross-Platform Path Hygiene**: Normalize paths with `normalize_path_for_compare` or forward-slash conversion before string comparison.
- **Strict Instance Isolation**: Queries and worker lookups must strictly enforce instance ownership boundaries.

---

## 5. Out of Scope
- Frontend UI modifications in `src/pages/Instances.tsx` (delegated to Worker 02 / Subtask 02).
- E2E testing suite creation in `src-tauri/tests/per_instance_prompt_liveness_test.rs` (delegated to Worker 02 / Subtask 02).
- Version bump, changelog updating, and release tagging (reserved for Master Lead).

---

## 6. Verification & Quality Gates
Execute the following verification commands upon completion of backend edits:
```powershell
cd src-tauri
cargo fmt -- --check
cargo clippy --all-targets --all-features
cargo test modules::repo_db
cargo test modules::instance
```

---

## 7. Acceptance Criteria & Done When
- [ ] `get_active_agy_workers()` in `is_prompt_running_for_project` strictly checks instance prefix `"{instance_id}:"` and prunes dead PIDs.
- [ ] `conversation_summaries.db` idle evaluation implements explicit idle supremacy: `not_fully_idle == 0` or status `IDLE`/`COMPLETED`/`FAILED`/`CANCELLED` strictly forces `is_conv_running = false`.
- [ ] `compute_project_conversation_tree` excludes `"queued"` and `"backed_up"` statuses from setting `is_running: true`.
- [ ] Cloned instances are sanitized: `clone_repo_rows_on` sets `is_running = 0` on `running_projects`, maps in-flight `active_prompts` to `"completed"`, and resets copied `conversation_summaries.db` to idle.
- [ ] Structured audit logs (`[InstancePromptAudit]`) are emitted for every liveness decision with comprehensive metadata.
- [ ] All pre-flight Rust gates (`cargo fmt -- --check` and `cargo clippy`) pass with zero errors and zero warnings.
