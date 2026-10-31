# Plan: Project Audit Status & Criteria Enum Refactoring

## User Request (Verbatim)
> "Fix it with proper enum please"
> Image: `src-tauri/src/modules/repo_db/detection.rs:110-126` containing ad-hoc string ternary:
> `if is_project_active { "ACTIVE_IN_FLIGHT_TASKS" } else if !is_instance_active { "INSTANCE_PROCESS_DEAD" } else { "IDLE_NO_ACTIVE_TASKS" }`

---

## Deliverables & Subtask Breakdown

### Task-01: Architectural Specification & Master Ledger (Lead)
- **Status:** `[DONE]`
- **Evidence:** `02-spec/21-app/project-audit-status-enum-refactor/01-architecture-spec.md`

### Task-02: Implementation of Typed Enums in `repo_db/models.rs` & Re-exports in `repo_db.rs` (Worker 01)
- **Status:** `[DONE]`
- **Target Files:**
  - `src-tauri/src/modules/repo_db/models.rs`
  - `src-tauri/src/modules/repo_db.rs`
- **Scope:**
  - Define `ProjectAuditRationaleType` with variants:
    - `ActiveInFlightTasks`, `InstanceProcessDead`, `IdleNoActiveTasks`, `ActivePromptMemory`, `ActiveWorkerMatched`, `ActivePromptDbRunning`, `ConversationSummaryActiveTurn`, `PromptDispatchMatched`, `PromptDispatchRejected`.
  - Add constructors:
    - `from_liveness(is_project_active: bool, is_instance_active: bool) -> Self`
    - `from_dispatch(is_match: bool) -> Self`
  - Add methods: `as_str()`, `display_label()`, `is_active()`, `is_dead()`, `is_idle()`.
  - Implement `Display` and `FromStr`.
  - Define `PromptAuditCriteriaType` with variants:
    - `WorkspaceStorageScanAndProcessLiveness`, `PromptDispatcherInstanceMatching`, `Gate0HostProcessLiveness`, `Gate1MemoryPromptsMap`, `Gate2ActiveAgyWorkers`, `Gate3ActivePromptsSqlite`, `Gate4ConversationSummariesLiveTurn`, `Gate0To4AllEvaluationsCompleted`, `ProjectConversationTreeLiveness`.
  - Add methods: `as_str()`, `Display`, `FromStr`.
  - Re-export both in `src-tauri/src/modules/repo_db.rs`.
  - Add unit tests for both enums in `repo_db/models.rs`.

### Task-03: Elimination of Ad-hoc String Ternary & Magic String Logic (Worker 02)
- **Status:** `[DONE]`
- **Target Files:**
  - `src-tauri/src/modules/repo_db/detection.rs`
  - `src-tauri/src/modules/repo_db/dispatch.rs`
  - `src-tauri/src/modules/repo_db/queue.rs`
  - `src-tauri/src/modules/repo_db/resend.rs`
  - `src-tauri/src/modules/repo_db/liveness.rs`
- **Scope:**
  - In `detection.rs:117-125`: Replace magic strings and if-else ladder with:
    `let criteria = PromptAuditCriteriaType::WorkspaceStorageScanAndProcessLiveness;`
    `let rationale = ProjectAuditRationaleType::from_liveness(is_project_active, is_instance_active);`
  - In `dispatch.rs:87-93`: Replace dispatch string ternary with `ProjectAuditRationaleType::from_dispatch(is_match)` and typed criteria.
  - In `queue.rs:105-111`: Replace dispatch string ternary with `ProjectAuditRationaleType::from_dispatch(is_match)` and typed criteria.
  - In `resend.rs:89-95`: Replace dispatch string ternary with `ProjectAuditRationaleType::from_dispatch(is_match)` and typed criteria.
  - In `liveness.rs`: Replace raw magic string audit rationales and gate criteria with typed enum references.

### Task-04: Quality Gates & Verification (Lead & Workers)
- **Status:** `[DONE]`
- `cargo fmt -- --check` (PASSED)
- `cargo check --lib` (PASSED)
- `cargo test --lib modules::repo_db::models::tests` (PASSED 5/5)
- `cargo test --lib modules::repo_db::` (PASSED 30/30)
- Atomic Git commit & push
