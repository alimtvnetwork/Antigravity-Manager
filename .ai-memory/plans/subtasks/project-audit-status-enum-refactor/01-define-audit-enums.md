---
id: subtask-01-define-audit-enums
task_id: Task-02
owner: Worker 01
status: DONE
files_owned:
  - src-tauri/src/modules/repo_db/models.rs
  - src-tauri/src/modules/repo_db.rs
---

# Subtask 01: Define Audit Status & Criteria Enums

## Scope
1. In `src-tauri/src/modules/repo_db/models.rs`:
   - Implement `ProjectAuditRationaleType`:
     - Variants:
       - `ActiveInFlightTasks` -> `"ACTIVE_IN_FLIGHT_TASKS"`
       - `InstanceProcessDead` -> `"INSTANCE_PROCESS_DEAD"`
       - `IdleNoActiveTasks` -> `"IDLE_NO_ACTIVE_TASKS"`
       - `ActivePromptMemory` -> `"ACTIVE_PROMPT_MEMORY"`
       - `ActiveWorkerMatched` -> `"ACTIVE_WORKER_MATCHED"`
       - `ActivePromptDbRunning` -> `"ACTIVE_PROMPT_DB_RUNNING"`
       - `ConversationSummaryActiveTurn` -> `"CONVERSATION_SUMMARY_ACTIVE_TURN"`
       - `PromptDispatchMatched` -> `"PROMPT_DISPATCH_MATCHED"`
       - `PromptDispatchRejected` -> `"PROMPT_DISPATCH_REJECTED"`
     - Constructors:
       - `from_liveness(is_project_active: bool, is_instance_active: bool) -> Self`
       - `from_dispatch(is_match: bool) -> Self`
     - Methods: `as_str()`, `display_label()`, `is_active()`, `is_dead()`, `is_idle()`
     - Traits: `std::fmt::Display`, `std::str::FromStr`
   - Implement `PromptAuditCriteriaType`:
     - Variants:
       - `WorkspaceStorageScanAndProcessLiveness` -> `"WorkspaceStorageScan + ProcessLiveness"`
       - `PromptDispatcherInstanceMatching` -> `"PromptDispatcher:InstanceMatching"`
       - `Gate0HostProcessLiveness` -> `"Gate0:HostProcessLiveness"`
       - `Gate1MemoryPromptsMap` -> `"Gate1:MemoryPromptsMap"`
       - `Gate2ActiveAgyWorkers` -> `"Gate2:ActiveAgyWorkers"`
       - `Gate3ActivePromptsSqlite` -> `"Gate3:ActivePromptsSQLite"`
       - `Gate4ConversationSummariesLiveTurn` -> `"Gate4:ConversationSummariesLiveTurn"`
       - `Gate0To4AllEvaluationsCompleted` -> `"Gate0-4:AllEvaluationsCompleted"`
       - `ProjectConversationTreeLiveness` -> `"ProjectConversationTreeLiveness"`
     - Methods: `as_str()`, `display_label()`
     - Traits: `std::fmt::Display`, `std::str::FromStr`
   - Add unit tests for both enums in `repo_db/models.rs` testing all variants, constructors, string mappings, and round-trips.

2. In `src-tauri/src/modules/repo_db.rs`:
   - Re-export `ProjectAuditRationaleType` and `PromptAuditCriteriaType`.

## Quality Gates
- `cargo fmt -- --check`
- `cargo check --lib`
- `cargo test --lib modules::repo_db::models::tests`
