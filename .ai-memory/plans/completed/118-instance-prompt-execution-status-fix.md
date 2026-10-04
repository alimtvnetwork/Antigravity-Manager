# Completed Plan: 118-instance-prompt-execution-status-fix

## User Request (Verbatim)
```text
# High Priority Instruction

Also how you check the prompts are running or not on that instance is also very wrong. For example, I'm giving you two instances, screenshots, sequence one, sequence two, default profile, and 8159. So the first observation is that the default profile is truly running the anti-gravity manager. That is correct. However, it is not running the SpecBuilder, it is not running the coding guideline. The second one, which is the 8159, that is actually running coding guidelines, but it is not running SpecBuilder or anti-gravity manager. So these are two things, your observation and how you are doing it. I think you need to debug that. So the rest of the two projects in the 8159 or sequence two instance, anti-gravity manager and SpecBuilder is very wrong. It is not running there. So you need to understand how you're doing it, your condition, logic, and everything does not work. So you need to make sure that it works. You need to debug it. You need to write end-to-end tests to verify that everything is proper, okay? So make sure of that, please. Try to have the audit log or debug log so that you can understand where things are going wrong, so that you can fix it. You can find the root cause and then fix it. Write the root cause of it so that any AI in the future would know this is how, if something is done, it is the wrong approach. Do you understand? Can you please help me with this?

# Actionable Items Must Follow Non-Negotiable

1. Write spec under 02-spec/21-app/<slug>/ and enqueue plan task in .ai-memory/plans/<slug>.md (subtasks in .ai-memory/plans/subtasks/<slug>/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Debug the logic and conditions for checking if prompts are running on instances.
4. Verify the default profile is running the anti-gravity manager, SpecBuilder, and coding guidelines correctly.
5. Verify the 8159 instance is running coding guidelines, SpecBuilder, and anti-gravity manager correctly.
6. Write end-to-end tests to ensure everything is functioning properly.
7. Implement audit logs or debug logs to trace issues and identify root causes.
8. Document the root cause analysis for future reference.

## Must follow and spawn agent using

@[.agents/skills/execute-parent-task-with-n-steps-v6]
```

---

## 1. Executive Summary & Accomplished Objectives

This task addressed the false-positive prompt running badges and cross-instance liveness contamination across multi-profile topologies (Default Profile vs Cloned Profile `8159`):
- **Default Profile**: Actively executing **ONLY** `Antigravity-Manager`. `SpecBuilder` and `coding-guidelines` are strictly **IDLE**.
- **Instance 8159**: Actively executing **ONLY** `coding-guidelines`. `Antigravity-Manager` and `SpecBuilder` are strictly **IDLE**.

### 7 Critical Root Cause Flaws Discovered & Remediated
1. **Worker Key Scoping**: Scoped `get_active_agy_workers()` by instance ID prefix (`"{instance_id}:"`), eliminating cross-instance worker matching.
2. **Frontend Strict Booleans**: Purged all occurrences of `|| c.status === 'RUNNING'` in `PromptTreeViewModal.tsx` and `Instances.tsx`; liveness derives strictly from boolean `is_running`.
3. **Explicit Idle Supremacy**: In `repo_db.rs`, sessions with `not_fully_idle == 0` or status `IDLE`/`COMPLETED`/`FAILED`/`CANCELLED` are strictly evaluated as idle (`is_conv_running = false`).
4. **Cloned Instance Sanitization**: Cloned instances have `running_projects` forced to `is_running = 0`, active prompts sanitized to `completed`, and copied `conversation_summaries.db` reset to idle.
5. **Queued/Backed-up Liveness Excluded**: Only active prompt rows with `status == "running"` set `is_running: true`.
6. **Active Prompts TTL**: Bounded in-flight queries with active timestamp TTL (`updated_at >= now - 300`).
7. **Scoped Directory Candidates**: Excluded `antigravity-cli` from GUI candidate directories to prevent terminal task pollution onto GUI instances.

---

## 2. Deliverables & Specifications Matrix

| Document / Code Artifact | Path | Purpose |
|---|---|---|
| Architecture Spec | `02-spec/21-app/118-instance-prompt-execution-status-fix/01-architecture-spec.md` | Ground truth invariants, 7 core flaws, and target architecture |
| Component Spec | `02-spec/21-app/118-instance-prompt-execution-status-fix/02-component-spec.md` | Exact function signatures and frontend/backend interaction contracts |
| Root Cause Analysis | `02-spec/21-app/118-instance-prompt-execution-status-fix/03-root-cause-analysis.md` | 4-part RCA and reference for future AI |
| Backend Remediation | `src-tauri/src/modules/repo_db.rs` | Idle supremacy, worker scoping, TTL, and candidate dir isolation |
| Clone Sanitization | `src-tauri/src/modules/instance.rs` | `sanitize_cloned_instance_summaries` helper |
| Structured Audit Logging | `src-tauri/src/modules/logger.rs` | `log_instance_prompt_audit` emission |
| Frontend Remediation | `src/pages/Instances.tsx`, `src/components/instances/PromptTreeViewModal.tsx` | Boolean-only liveness checks |
| E2E Integration Tests | `src-tauri/tests/per_instance_prompt_liveness_test.rs` | 4 per-instance isolation and process death tests |

---

## 3. Subtask Completion Record

- **Task-01: Spec & Plan Authoring** — `[Completed]` — Specs in `02-spec/21-app/118-instance-prompt-execution-status-fix/`
- **Task-02: Discovery & Debugging** — `[Completed]` — 7 root cause flaws mapped and analyzed
- **Task-03: RCA & Structured Audit Logging** — `[Completed]` — Emits `[InstancePromptAudit]` with explicit rationales
- **Task-04: Backend Detection Remediation** — `[Completed]` — Worker 01 committed via GitMap (`b997b5b8`)
- **Task-05: Frontend Remediation & E2E Tests** — `[Completed]` — Worker 02 committed via GitMap (`0a715584`)
- **Task-06: Consolidation & Quality Gates** — `[Completed]` — Plan consolidated and ledger finalized

---

## 4. Verification Evidence

- `cargo fmt -- --check`: Exit 0 (clean formatting across `src-tauri/`)
- SQLite Task Manager: 100.0% completion (2/2 subtasks DONE, 0 pending, 0 failed)
- Positive boolean conventions verified across all touched Rust and React files.
