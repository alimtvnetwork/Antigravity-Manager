# Master Execution Plan: 118-instance-prompt-execution-status-fix

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

## 1. Executive Summary & Problem Classification

The user observed that prompt/project execution status is falsely reported and cross-contaminated across instances:
- **Default Profile (Sequence 1)**: Truly running ONLY `Antigravity-Manager`. SpecBuilder and Coding Guidelines are NOT running, but previously may have displayed as running.
- **Instance 8159 (Sequence 2)**: Truly running ONLY `coding-guidelines`. Antigravity-Manager and SpecBuilder are NOT running, but previously falsely displayed as running.

### 7 Critical Root Cause Flaws Discovered
1. **Worker Map Cross-Instance Leak**: `repo_db::is_prompt_running_for_project` checked `key.contains(project_id)` across all active workers without filtering by `instance_id`.
2. **Frontend Raw Status Override**: `src/pages/Instances.tsx` used `c.status === 'RUNNING'` which overrode the backend's definitive `is_running = false`.
3. **Inverted OR Liveness Logic**: In `repo_db.rs`, `not_fully_idle != 0 || status.contains("RUNNING")` ignored when `not_fully_idle == 0` (idle agent turn).
4. **Cloned State Pollution**: Instance cloning duplicated `running_projects`, `active_prompts` with `status = 'running'`, and `conversation_summaries.db` without clearing active runtime states.
5. **Queued/Backed-Up Marked as Running**: In `compute_project_conversation_tree`, `queued` and `backed_up` prompts were treated as `is_running = true`.
6. **Missing Terminal Transition**: Stale `active_prompts` were never transitioned to `completed`/`failed`, permanently causing `running_count > 0`.
7. **Global Tree Query Bleed**: Frontend called `get_project_conversation_tree` without `instanceId`, polluting the shared cache.

---

## 2. Architecture & Work Breakdown

- **Wave 1: Planning & Discovery** (COMPLETED via Research 01 & 02)
- **Wave 2: Spec Authoring & Subtask Decomposition** (IN PROGRESS via Spec Subagents 01 & 02)
  - Spec Subagent 01: `02-spec/21-app/118-instance-prompt-execution-status-fix/01-architecture-spec.md` & Subtask 01
  - Spec Subagent 02: `02-spec/21-app/118-instance-prompt-execution-status-fix/02-component-spec.md` & Subtask 02
  - Lead: `02-spec/21-app/118-instance-prompt-execution-status-fix/03-root-cause-analysis.md`
- **Wave 3: Execution (Workers 01 & 02)**
  - Worker 01: Backend detection remediation in `src-tauri/src/modules/repo_db.rs`, `instance.rs`, and audit logging in `logger.rs`
  - Worker 02: Frontend status display remediation in `src/pages/Instances.tsx` and comprehensive E2E tests in `src-tauri/tests/per_instance_prompt_liveness_test.rs`
- **Wave 4: Verification, Lint Gates, & Atomic GitMap Commit**
  - Run Rust fmt & clippy gates
  - Verify zero secrets
  - Atomic commit via `gitmap cpb "instance-status - fix cross-instance prompt running detection and audit logging"`
