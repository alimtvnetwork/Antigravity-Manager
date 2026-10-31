# Plan: Prompt Execution & Dispatch Status Enum Refactoring

## User Request (Verbatim)
> "codes like this should be repleaced with proper enum read enum cg execute prompt and execute properly"
> Image: `let exec_status = if spawned { "Executing now" } else { "Queued" };` in `src-tauri/src/modules/telegram_inbound/injection.rs:264`.

---

## Deliverables & Subtask Breakdown

### Task-01: Architectural Specification & Master Ledger (Lead)
- **Status:** `[DONE]`
- **Evidence:** `02-spec/21-app/prompt-execution-status-enum-refactor/01-architecture-spec.md`

### Task-02: Implementation of Typed Enums in `telegram_inbound/types.rs` & `supabase_command_queue.rs` (Worker 01)
- **Status:** `[DONE]`
- **Target Files:**
  - `src-tauri/src/modules/telegram_inbound/types.rs`
  - `src-tauri/src/modules/supabase_command_queue.rs`
- **Scope:**
  - Define `PromptExecutionStatusType` (`ExecutingNow`, `Queued`) with `.from_spawned()`, `.display_label()`, `Display`.
  - Define `RemoteDispatchStatusType` (`GitMapClusterSsh`, `SupabaseQueued`, `LocalRecorded`) with `.resolve()`, `.display_label()`, `Display`.
  - Define `CommandExecutionStatusType` (`Completed`, `Failed`) with `.from_exit_code()`, `.as_str()`, `Display`.

### Task-03: Elimination of Ad-hoc String Ternary Logic in `injection.rs` (Worker 02)
- **Status:** `[DONE]`
- **Target Files:**
  - `src-tauri/src/modules/telegram_inbound/injection.rs`
  - `src-tauri/src/modules/telegram_inbound/tests.rs`
- **Scope:**
  - Replace line 264 `let exec_status = if spawned { "Executing now" } else { "Queued" };` with `let exec_status = PromptExecutionStatusType::from_spawned(spawned);`.
  - Replace lines 364-370 `let dispatch_status = if dispatched_gitmap { ... }` with `let dispatch_status = RemoteDispatchStatusType::resolve(dispatched_gitmap, enqueued_supabase);`.
  - Add unit tests for `PromptExecutionStatusType` and `RemoteDispatchStatusType`.

### Task-04: Quality Gates & Verification (Lead & Workers)
- **Status:** `[DONE]`
- `cargo fmt -- --check` (PASSED)
- `cargo check --lib` (PASSED)
- `cargo test --lib modules::telegram_inbound::tests` (PASSED 17/17)
- `cargo test --lib modules::supabase_command_queue::tests` (PASSED 3/3)
- Atomic Git commit
