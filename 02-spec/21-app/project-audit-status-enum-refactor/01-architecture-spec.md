# Architecture Specification: Project Audit Status & Criteria Enum Refactoring

## 1. Context & Problem Statement
In `src-tauri/src/modules/repo_db/detection.rs` (lines 110–126), the instance prompt audit invocation embeds ad-hoc string ternary branches and magic string literals:
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
Similar ad-hoc string ternaries and magic strings appear in:
- `src-tauri/src/modules/repo_db/dispatch.rs` (lines 89–93): `"PROMPT_DISPATCH_MATCHED"` vs `"PROMPT_DISPATCH_REJECTED"`
- `src-tauri/src/modules/repo_db/queue.rs` (lines 107–111): `"PROMPT_DISPATCH_MATCHED"` vs `"PROMPT_DISPATCH_REJECTED"`
- `src-tauri/src/modules/repo_db/resend.rs` (lines 91–95): `"PROMPT_DISPATCH_MATCHED"` vs `"PROMPT_DISPATCH_REJECTED"`
- `src-tauri/src/modules/repo_db/liveness.rs`: Repeated magic string audit rationales and gate criteria.

## 2. Core Architectural Design

### 2.1. `ProjectAuditRationaleType`
Defined in `src-tauri/src/modules/repo_db/models.rs`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProjectAuditRationaleType {
    ActiveInFlightTasks,
    InstanceProcessDead,
    IdleNoActiveTasks,
    ActivePromptMemory,
    ActiveWorkerMatched,
    ActivePromptDbRunning,
    ConversationSummaryActiveTurn,
    PromptDispatchMatched,
    PromptDispatchRejected,
}
```
- **Constructor**:
  `pub fn from_liveness(is_project_active: bool, is_instance_active: bool) -> Self`
  `pub fn from_dispatch(is_match: bool) -> Self`
- **Presentation & String Accessors**:
  `pub const fn as_str(&self) -> &'static str`
  `pub const fn display_label(&self) -> &'static str`
  Implements `std::fmt::Display` and `std::str::FromStr`
- **Predicates**:
  `pub const fn is_active(&self) -> bool`
  `pub const fn is_dead(&self) -> bool`
  `pub const fn is_idle(&self) -> bool`

### 2.2. `PromptAuditCriteriaType`
Defined in `src-tauri/src/modules/repo_db/models.rs`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PromptAuditCriteriaType {
    WorkspaceStorageScanAndProcessLiveness,
    PromptDispatcherInstanceMatching,
    Gate0HostProcessLiveness,
    Gate1MemoryPromptsMap,
    Gate2ActiveAgyWorkers,
    Gate3ActivePromptsSqlite,
    Gate4ConversationSummariesLiveTurn,
    Gate0To4AllEvaluationsCompleted,
    ProjectConversationTreeLiveness,
}
```
- **Presentation & String Accessors**:
  `pub const fn as_str(&self) -> &'static str`
  Implements `std::fmt::Display` and `std::str::FromStr`

## 3. Scope Boundaries & Owned Files
- **Worker 01**:
  - `src-tauri/src/modules/repo_db/models.rs` (Enum definitions, constructors, methods, Display & FromStr traits, unit tests)
  - `src-tauri/src/modules/repo_db.rs` (Re-exports of new enums)
- **Worker 02**:
  - `src-tauri/src/modules/repo_db/detection.rs` (Eliminate ternary in `detect_running_projects`)
  - `src-tauri/src/modules/repo_db/dispatch.rs` (Eliminate ternary in `dispatch_running_prompts`)
  - `src-tauri/src/modules/repo_db/queue.rs` (Eliminate ternary in `check_and_dispatch_enqueued_prompts`)
  - `src-tauri/src/modules/repo_db/resend.rs` (Eliminate ternary in `resend_running_commands_for_instance`)
  - `src-tauri/src/modules/repo_db/liveness.rs` (Use typed enums for audit calls)

## 4. Quality Gates
1. `cargo fmt -- --check`
2. `cargo check --lib`
3. `cargo test --lib modules::repo_db::models::tests`
4. Targeted regression tests: `cargo test --lib modules::repo_db::tests`
