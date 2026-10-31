---
id: subtask-02-injection-refactor-and-tests
task_id: Task-03
owner: Worker 02
status: DONE
files_owned:
  - src-tauri/src/modules/telegram_inbound/injection.rs
  - src-tauri/src/modules/telegram_inbound/tests.rs
---

# Subtask 02: Injection Refactor & Unit Tests

## Scope
1. In `src-tauri/src/modules/telegram_inbound/injection.rs`:
   - Import `PromptExecutionStatusType` and `RemoteDispatchStatusType` from `super::types`.
   - In `execute_prompt_injection`:
     - Replace:
       `let exec_status = if spawned { "Executing now" } else { "Queued" };`
     - With:
       `let exec_status = PromptExecutionStatusType::from_spawned(spawned);`
     - In the format template at line 270:
       `<b>Status:</b> {}\n\` -> `exec_status` (implements `Display`, rendering `"Executing now"` or `"Queued"`).
   - In `execute_remote_prompt_injection`:
     - Replace lines 364-370:
       ```rust
       let dispatch_status = if dispatched_gitmap {
           "Sent via GitMap Cluster SSH"
       } else if enqueued_supabase {
           "Queued on the secondary database — the node will pick it up"
       } else {
           "Queued (the node recorded your instruction)"
       };
       ```
     - With:
       `let dispatch_status = RemoteDispatchStatusType::resolve(dispatched_gitmap, enqueued_supabase);`
     - In the format template:
       `dispatch_status` (implements `Display`, rendering the appropriate label).

2. In `src-tauri/src/modules/telegram_inbound/tests.rs`:
   - Add unit tests verifying:
     - `PromptExecutionStatusType::from_spawned(true)` == `ExecutingNow` & display_label == `"Executing now"`.
     - `PromptExecutionStatusType::from_spawned(false)` == `Queued` & display_label == `"Queued"`.
     - `RemoteDispatchStatusType::resolve(true, false)` == `GitMapClusterSsh`.
     - `RemoteDispatchStatusType::resolve(false, true)` == `SupabaseQueued`.
     - `RemoteDispatchStatusType::resolve(false, false)` == `LocalRecorded`.

## Quality Gates
- `cargo fmt -- --check`
- `cargo check --lib`
- `cargo test --lib modules::telegram_inbound::tests`
