# Subtask 02: Structured Audit Logging & End-to-End Tests

## Objective
Implement structured audit logging in `src-tauri/src/modules/logger.rs` and `repo_db.rs`, and author comprehensive end-to-end integration tests in `src-tauri/tests/per_instance_prompt_liveness_test.rs` to verify prompt running isolation across instances.

## Scope of Work
1. **Structured Audit Logs**:
   - Ensure `log_instance_prompt_audit` emits clear, traceable logs with:
     - `instance_id`
     - `instance_name`
     - `project_id`
     - `repo_path`
     - `source_location`
     - `host_pid`
     - `evaluation_gate` (e.g. `Gate0:HostProcessLiveness`, `Gate4:ConversationSummariesLiveTurn`, `TreeComputation:TurnTTL`)
     - `is_running`
     - `rationale` (e.g. `TURN_STALE_TTL_EXPIRED`, `ACTIVE_IN_FLIGHT_TASKS`, `INSTANCE_PROCESS_DEAD`, `IDLE_NO_ACTIVE_TASKS`)
2. **End-to-End Tests in `src-tauri/tests/per_instance_prompt_liveness_test.rs`**:
   - Test default instance: running `Antigravity-Manager` does not bleed into `SpecBuilder` or `coding-guidelines`.
   - Test 8159 instance: running `coding-guidelines` does not bleed into `Antigravity-Manager` or `SpecBuilder`.
   - Test TTL expiration: turn older than 900 seconds with `RUNNING` status is strictly evaluated as `is_running = false`.
   - Test process death: instance with no active OS process has all projects forced to `is_running = false`.
3. **Root Cause Analysis (RCA)**:
   - Finalize `02-spec/22-app-issues/122-instance-prompt-running-detection-root-cause.md` and `02-spec/21-app/122-instance-prompt-status-detection-and-audit-fix/03-root-cause-analysis.md`.

## Acceptance Criteria
- [ ] Structured audit logs record all evaluation gates.
- [ ] End-to-end tests compile cleanly and pass targeted verification.
- [ ] RCA document details the root causes and architectural preventative measures.
