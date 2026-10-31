---
id: subtask-02-refactor-detection-and-audit-calls
task_id: Task-03
owner: Worker 02
status: DONE
files_owned:
  - src-tauri/src/modules/repo_db/detection.rs
  - src-tauri/src/modules/repo_db/dispatch.rs
  - src-tauri/src/modules/repo_db/queue.rs
  - src-tauri/src/modules/repo_db/resend.rs
  - src-tauri/src/modules/repo_db/liveness.rs
---

# Subtask 02: Refactor Detection & Audit Call Sites

## Scope
1. In `src-tauri/src/modules/repo_db/detection.rs`:
   - Import `ProjectAuditRationaleType` and `PromptAuditCriteriaType` from `super::models`.
   - In `detect_running_projects` (lines 110–126):
     Replace:
     ```rust
     crate::modules::logger::log_instance_prompt_audit(
         target_id,
         &instance.name,
         &repo_name,
         &raw_path,
         &ws_folder.to_string_lossy(),
         pid_opt,
         "WorkspaceStorageScan + ProcessLiveness",
         is_project_active,
         if is_project_active {
             "ACTIVE_IN_FLIGHT_TASKS"
         } else if !is_instance_active {
             "INSTANCE_PROCESS_DEAD"
         } else {
             "IDLE_NO_ACTIVE_TASKS"
         },
     );
     ```
     With:
     ```rust
     let criteria = PromptAuditCriteriaType::WorkspaceStorageScanAndProcessLiveness;
     let rationale = ProjectAuditRationaleType::from_liveness(is_project_active, is_instance_active);
     crate::modules::logger::log_instance_prompt_audit(
         target_id,
         &instance.name,
         &repo_name,
         &raw_path,
         &ws_folder.to_string_lossy(),
         pid_opt,
         criteria.as_str(),
         is_project_active,
         rationale.as_str(),
     );
     ```

2. In `src-tauri/src/modules/repo_db/dispatch.rs`:
   - Replace lines 87–93 string ternary with `ProjectAuditRationaleType::from_dispatch(is_match)` and `PromptAuditCriteriaType::PromptDispatcherInstanceMatching`.

3. In `src-tauri/src/modules/repo_db/queue.rs`:
   - Replace lines 105–111 string ternary with `ProjectAuditRationaleType::from_dispatch(is_match)` and `PromptAuditCriteriaType::PromptDispatcherInstanceMatching`.

4. In `src-tauri/src/modules/repo_db/resend.rs`:
   - Replace lines 89–95 string ternary with `ProjectAuditRationaleType::from_dispatch(is_match)` and `PromptAuditCriteriaType::PromptDispatcherInstanceMatching`.

5. In `src-tauri/src/modules/repo_db/liveness.rs`:
   - Replace raw magic string audit rationales and gate criteria with typed enum references:
     - Line 44 & 46: `PromptAuditCriteriaType::Gate0HostProcessLiveness.as_str()`, `ProjectAuditRationaleType::InstanceProcessDead.as_str()`
     - Line 112 & 114: `PromptAuditCriteriaType::Gate1MemoryPromptsMap.as_str()`, `ProjectAuditRationaleType::ActivePromptMemory.as_str()`
     - Line 172 & 174: `PromptAuditCriteriaType::Gate2ActiveAgyWorkers.as_str()`, `ProjectAuditRationaleType::ActiveWorkerMatched.as_str()`
     - Line 205 & 207: `PromptAuditCriteriaType::Gate3ActivePromptsSqlite.as_str()`, `ProjectAuditRationaleType::ActivePromptDbRunning.as_str()`
     - Line 327 & 329: `PromptAuditCriteriaType::Gate4ConversationSummariesLiveTurn.as_str()`, `ProjectAuditRationaleType::ConversationSummaryActiveTurn.as_str()`
     - Line 349 & 351: `PromptAuditCriteriaType::Gate0To4AllEvaluationsCompleted.as_str()`, `ProjectAuditRationaleType::IdleNoActiveTasks.as_str()`

## Quality Gates
- `cargo fmt -- --check`
- `cargo check --lib`
- `cargo test --lib modules::repo_db::tests`
