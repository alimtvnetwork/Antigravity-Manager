# Subtask 02: Structured Audit Logging, End-to-End Tests & Verification

- **Task Identifier**: `123-prompt-running-instance-detection-and-audit-logging`
- **Subtask Slug**: `02-audit-logging-e2e-tests-and-verification`
- **Worker Assignment**: Worker 02
- **Master Plan**: [123-prompt-running-instance-detection-and-audit-logging.md](../../123-prompt-running-instance-detection-and-audit-logging.md)
- **Architecture Spec**: [01-architecture-spec.md](../../../02-spec/21-app/123-prompt-running-instance-detection-and-audit-logging/01-architecture-spec.md)
- **Component Spec**: [02-component-spec.md](../../../02-spec/21-app/123-prompt-running-instance-detection-and-audit-logging/02-component-spec.md)
- **Root Cause Analysis**: [123-per-instance-prompt-running-detection-root-cause.md](../../../02-spec/22-app-issues/123-per-instance-prompt-running-detection-root-cause.md)
- **Target Files**:
  - `src-tauri/src/modules/logger.rs` (Audit log formatting and emission verification)
  - `src-tauri/src/modules/repo_db.rs` (Structured audit logging in dispatchers and candidate gates)
  - `src-tauri/tests/per_instance_prompt_liveness_test.rs` (Integration test suite covering Cases A through F)

---

## 1. Objectives & Executive Scope

Worker 02 is responsible for:
1. **Expanding Structured Audit Logging in `repo_db.rs`**:
   - Calling `log_instance_prompt_audit` at all critical decision boundaries:
     * Prompt dispatchers (`dispatch_running_prompts`, `resend_running_commands_for_instance`, `check_and_dispatch_enqueued_prompts`).
     * Candidate directory evaluations and Gate 0–4 evaluations in `is_prompt_running_for_project`.
     * Tree evaluation verdict in `compute_project_conversation_tree`.
   - Adhering strictly to standardized criteria strings and rationale codes.
2. **Implementing and Updating End-to-End Integration Tests in `per_instance_prompt_liveness_test.rs`**:
   - **Case A (Sequence 1 `default`)**: `Antigravity-Manager` = RUNNING; `SpecBuilder` = IDLE; `coding-guidelines` = IDLE.
   - **Case B (Sequence 2 `8159` / `default-copy-8159`)**: `coding-guidelines` = RUNNING; `Antigravity-Manager` = IDLE; `SpecBuilder` = IDLE.
   - **Case C (Suffix Alias Resolution)**: `'8159'` and `'-8159'` resolve to `'default-copy-8159'`.
   - **Case D (Primary Key Isolation)**: Namespaced composite key `{base}__{instance_id}` prevents row collisions in `running_projects`.
   - **Case E (Stale & Queued Prompts Gating)**: Queued prompts (`status = 'queued'`) do not trigger running; stale turns (`age > 900s`) force idle.
   - **Case F (Audit Log Formatting & Emission)**: Exact validation of structured log line formatting and fields.
3. **Execution of Pre-Flight Verification & Quality Gates**:
   - Targeted integration test execution via `cargo test --test per_instance_prompt_liveness_test`.
   - Rust lint gate via `cargo clippy --all-targets --all-features`.
   - Rust format gate via `cargo fmt -- --check`.
   - Frontend compilation check via `npm run build`.

---

## 2. Detailed Implementation Specifications

### 2.1 Expanding Structured Audit Logging in `repo_db.rs`

Worker 02 must ensure that `crate::modules::logger::log_instance_prompt_audit` is called at all required evaluation points with canonical parameters.

#### 2.1.1 Function Contract
```rust
pub fn log_instance_prompt_audit(
    instance_id: &str,
    resolved_name: &str,
    project_name: &str,
    repo_path: &str,
    db_path_evaluated: &str,
    process_pid: Option<u32>,
    criteria_evaluated: &str,
    is_running: bool,
    rationale: &str,
);
```

#### 2.1.2 Audit Hook Locations & Parameters

1. **In `dispatch_running_prompts`** (`src-tauri/src/modules/repo_db.rs`):
   - Hook at prompt matching loop:
     ```rust
     let is_match = prompt_inst == target_inst;
     crate::modules::logger::log_instance_prompt_audit(
         target_inst,
         &instance.name,
         &repo_name,
         &prompt.repo_path,
         "repo_prompts.db:active_prompts",
         pids.first().copied(),
         "PromptDispatcher:InstanceMatching",
         false,
         if is_match {
             "PROMPT_DISPATCH_MATCHED"
         } else {
             "PROMPT_DISPATCH_REJECTED"
         },
     );
     ```

2. **In `resend_running_commands_for_instance`** (`src-tauri/src/modules/repo_db.rs`):
   - Hook when evaluating candidates for dispatch:
     ```rust
     let is_match = prompt_inst == target_inst;
     crate::modules::logger::log_instance_prompt_audit(
         target_inst,
         target_inst,
         &prompt.repo_path,
         &prompt.repo_path,
         "repo_prompts.db:active_prompts",
         None,
         "PromptDispatcher:InstanceMatching",
         false,
         if is_match {
             "PROMPT_DISPATCH_MATCHED"
         } else {
             "PROMPT_DISPATCH_REJECTED"
         },
     );
     ```

3. **In `check_and_dispatch_enqueued_prompts`** (`src-tauri/src/modules/repo_db.rs`):
   - Hook when matching queued prompts to instances:
     ```rust
     crate::modules::logger::log_instance_prompt_audit(
         target_inst,
         target_inst,
         &prompt.repo_path,
         &prompt.repo_path,
         "repo_prompts.db:active_prompts",
         None,
         "PromptDispatcher:InstanceMatching",
         false,
         if is_match {
             "PROMPT_DISPATCH_MATCHED"
         } else {
             "PROMPT_DISPATCH_REJECTED"
         },
     );
     ```

4. **In `is_prompt_running_for_project` (Gate 0 through Gate 4)** (`src-tauri/src/modules/repo_db.rs`):
   - Ensure all gate transitions emit audit logs:
     * **Gate 0 (Process Dead)**: criteria `"Gate0:HostProcessLiveness"`, rationale `"INSTANCE_PROCESS_DEAD"`.
     * **Gate 1 (Memory Map)**: criteria `"Gate1:MemoryPromptsMap"`, rationale `"ACTIVE_PROMPT_MEMORY"`.
     * **Gate 2 (Active AGY Worker)**: criteria `"Gate2:ActiveAgyWorkers"`, rationale `"ACTIVE_WORKER_MATCHED"`.
     * **Gate 3 (Active Prompts SQLite)**: criteria `"Gate3:ActivePromptsSQLite"`, rationale `"ACTIVE_PROMPT_DB_RUNNING"`.
     * **Gate 4 (Candidate Dir Turn Freshness)**: criteria `"Gate4:ConversationSummariesLiveTurn"`, rationale `"CONVERSATION_SUMMARY_ACTIVE_TURN"`.
     * **Gate 0–4 Fallthrough (Idle)**: criteria `"Gate0-4:AllEvaluationsCompleted"`, rationale `"IDLE_NO_ACTIVE_TASKS"`.

5. **In `compute_project_conversation_tree`** (`src-tauri/src/modules/repo_db.rs`):
   - Hook at project node verdict:
     ```rust
     crate::modules::logger::log_instance_prompt_audit(
         &proj.instance_id,
         &instance_name,
         &proj.repo_name,
         &proj.repo_path,
         proj.workspace_storage_path.as_deref().unwrap_or("none"),
         inst_pid,
         "ProjectConversationTreeLiveness",
         proj_is_running,
         &rationale,
     );
     ```

---

### 2.2 End-to-End Integration Tests (`per_instance_prompt_liveness_test.rs`)

Worker 02 must implement or update the comprehensive test cases in `src-tauri/tests/per_instance_prompt_liveness_test.rs`:

#### 2.2.1 Case A: Sequence 1 (`default`) Isolation
- **Function**: `test_case_a_sequence_1_default_running_agm_only`
- **Scenario**:
  - Instance: `"default"` with alive host PID.
  - Workspaces in `conversation_summaries.db`:
    * `d:/work/Antigravity-Manager`: `not_fully_idle = 1`, `status = "CASCADE_RUN_STATUS_RUNNING"`, `turn_age = 30s`.
    * `d:/work/SpecBuilder`: `not_fully_idle = 0`, `status = "CASCADE_RUN_STATUS_COMPLETED"`, `turn_age = 60s`.
    * `d:/work/coding-guidelines`: `not_fully_idle = 0`, `status = "CASCADE_RUN_STATUS_IDLE"`, `turn_age = 120s`.
- **Assertions**:
  - `Antigravity-Manager` evaluates to `is_running = true`.
  - `SpecBuilder` evaluates to `is_running = false`.
  - `coding-guidelines` evaluates to `is_running = false`.
  - Filtering with `only_running = true` yields exactly 1 project (`Antigravity-Manager`).

#### 2.2.2 Case B: Sequence 2 (`8159` / `default-copy-8159`) Isolation
- **Function**: `test_case_b_sequence_2_instance_8159_running_cg_only`
- **Scenario**:
  - Instance: `"default-copy-8159"` with alive host PID.
  - Workspaces in `conversation_summaries.db`:
    * `d:/work/coding-guidelines`: `not_fully_idle = 1`, `status = "CASCADE_RUN_STATUS_RUNNING"`, `turn_age = 45s`.
    * `d:/work/Antigravity-Manager`: cloned dormant workspace, `not_fully_idle = 0`, `status = "CASCADE_RUN_STATUS_IDLE"`.
    * `d:/work/SpecBuilder`: cloned dormant workspace, `not_fully_idle = 0`, `status = "CASCADE_RUN_STATUS_IDLE"`.
- **Assertions**:
  - `coding-guidelines` evaluates to `is_running = true`.
  - `Antigravity-Manager` evaluates to `is_running = false`.
  - `SpecBuilder` evaluates to `is_running = false`.
  - Filtering with `only_running = true` yields exactly 1 project (`coding-guidelines`).
  - **Cross-Instance Mutual Exclusion**:
    * Default running AGM does NOT cause 8159 to report AGM running.
    * 8159 running CG does NOT cause Default to report CG running.

#### 2.2.3 Case C: Suffix Alias Resolution
- **Function**: `test_case_c_suffix_alias_resolution_for_cloned_instances`
- **Scenario**:
  - Mock instance registry with:
    * `id: "default"`, `name: "Default Profile"`.
    * `id: "default-copy-8159"`, `name: "Instance 8159"`.
- **Assertions**:
  - `resolve_instance_id("8159")` returns `Ok("default-copy-8159")`.
  - `resolve_instance_id("-8159")` returns `Ok("default-copy-8159")`.
  - `resolve_instance_id("inst-8159")` returns `Ok("default-copy-8159")`.
  - `resolve_instance_id("default")` returns `Ok("default")`.

#### 2.2.4 Case D: Primary Key Composite Isolation (`{base}__{instance_id}`)
- **Function**: `test_case_d_database_primary_key_composite_isolation`
- **Scenario**:
  - Initialize SQLite database with table `running_projects`.
  - Same repository path (`d:/work/Antigravity-Manager`) registered under both `default` and `default-copy-8159`.
  - Composite IDs:
    * `antigravity-manager-hash__default`
    * `antigravity-manager-hash__default-copy-8159`
- **Assertions**:
  - Insert record for default with `is_running = 1`.
  - Insert record for 8159 with `is_running = 0`.
  - Total records count in `running_projects` is `2`.
  - Updating default's record does not overwrite or mutate 8159's record.

#### 2.2.5 Case E: Stale and Queued Prompts Gating
- **Function**: `test_case_e_stale_and_queued_prompts_do_not_trigger_running`
- **Scenario**:
  - Insert row in `active_prompts` with `status = 'queued'` and current timestamp.
  - Insert row in `active_prompts` with `status = 'running'` but timestamp older than 300s.
  - Insert turn in `conversation_summaries` with `not_fully_idle = 1` but `last_modified_time` older than 900s.
- **Assertions**:
  - Queued prompt evaluates to `is_running = false`.
  - Stale active prompt (> 300s) evaluates to `is_running = false`.
  - Stale conversation turn (> 900s) evaluates to `is_running = false` with rationale `"TURN_STALE_TTL_EXPIRED"`.

#### 2.2.6 Case F: Audit Log Formatting and Emission Validation
- **Function**: `test_case_f_audit_log_formatting_and_emission`
- **Scenario**:
  - Format audit log using `format_instance_prompt_audit`.
- **Assertions**:
  - Output string starts with `"[InstancePromptAudit]"`.
  - Contains exact fields:
    * `instance_id='default'`
    * `resolved_name='Default Profile'`
    * `project='Antigravity-Manager'`
    * `repo_path='d:/work/Antigravity-Manager'`
    * `pid=11628` (or `pid=none` when None)
    * `criteria='ProjectConversationTreeLiveness'`
    * `is_running=true`
    * `rationale='ACTIVE_IN_FLIGHT_TASKS'`
  - Function executes without panic or formatting error.

---

## 3. Acceptance Criteria Checklist

- [ ] Audit logging hooks added to `dispatch_running_prompts` with `PROMPT_DISPATCH_MATCHED` and `PROMPT_DISPATCH_REJECTED`.
- [ ] Audit logging hooks added to `resend_running_commands_for_instance` with `PROMPT_DISPATCH_MATCHED` and `PROMPT_DISPATCH_REJECTED`.
- [ ] Audit logging hooks added to `check_and_dispatch_enqueued_prompts` with `PROMPT_DISPATCH_MATCHED` and `PROMPT_DISPATCH_REJECTED`.
- [ ] Audit logging hooks verified across all Gate 0 through Gate 4 transitions in `is_prompt_running_for_project`.
- [ ] Audit logging hook verified at `compute_project_conversation_tree` project node verdict with `ProjectConversationTreeLiveness`.
- [ ] Case A integration test implemented and passing: Sequence 1 (`default`) has AGM=running, SB=idle, CG=idle.
- [ ] Case B integration test implemented and passing: Sequence 2 (`8159`) has CG=running, AGM=idle, SB=idle.
- [ ] Case C integration test implemented and passing: Suffix alias resolution (`8159` -> `default-copy-8159`).
- [ ] Case D integration test implemented and passing: Primary key composite isolation `{base}__{instance_id}`.
- [ ] Case E integration test implemented and passing: Stale turns (> 900s) and queued prompts (`status = 'queued'`) do not trigger running.
- [ ] Case F integration test implemented and passing: Audit log formatting and emission exact string verification.
- [ ] Pre-flight checks executed successfully:
  - [ ] `cargo test --test per_instance_prompt_liveness_test` passes with 0 failures.
  - [ ] `cd src-tauri && cargo clippy --all-targets --all-features` passes with 0 warnings.
  - [ ] `cd src-tauri && cargo fmt -- --check` passes with 0 diffs.
  - [ ] `npm run build` passes with 0 errors.

---

## 4. Verification & Execution Commands

```powershell
# 1. Run targeted integration test suite
cargo test --test per_instance_prompt_liveness_test

# 2. Comprehensive Rust clippy gate
cd src-tauri
cargo clippy --all-targets --all-features

# 3. Rust formatting check
cargo fmt -- --check

# 4. Frontend production build
cd ..
npm run build
```
