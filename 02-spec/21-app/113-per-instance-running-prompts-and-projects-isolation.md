# Specification: Strict Per-Instance Project & Prompt Liveness Isolation

> **Spec ID:** `113-per-instance-running-prompts-and-projects-isolation`  
> **Status:** APPROVED & IN PROGRESS  
> **Scope:** Per-Instance WorkspaceStorage Mapping, Process PID Gating for Prompt Liveness, Frontend Card Scoping, Audit Logging, and Isolated Verification  

---

## 1. Executive Summary

This specification enforces strict per-instance isolation for projects, conversation trees, and active prompt liveness across Antigravity Manager. It resolves the critical cross-instance state bleed where both Card #1 (Default) and Card #2 (8159) erroneously mirrored the exact same projects (`Antigravity-Manager`, `spec-builder`, `coding-guidelines`) with identical turn counts and false `[RUNNING]` indicators.

### Core Objectives:
1. **Per-Instance WorkspaceStorage Scoping**: Discovered projects must originate strictly from the specific instance's `data_dir/User/workspaceStorage`. A project belongs to an instance only if its workspace definition exists within that instance's storage.
2. **Process PID Gating for Liveness**: Eliminate blind 10-minute recency markers (`is_recency_active`) when deciding `is_running`. A conversation or project is `[RUNNING]` on an instance **if and only if**:
   - The instance's process (PID) is actively running in the OS (`find_pids_for_data_dir` / `is_instance_running`), AND
   - The project is associated with that running instance, AND
   - The conversation has an active, uncompleted prompt or active heartbeat.
3. **Parametric `get_project_conversation_tree` IPC**:
   - Update `get_project_conversation_tree` to accept `instance_id: Option<String>`.
   - When provided, filter and cache strictly for that instance (`prompt_tree_cache` table partitioned by `instance_id`).
4. **Frontend Instance Card Filtering & Liveness Badging**:
   - In `src/pages/Instances.tsx`, scope project rendering strictly to `inst.config.id` (or matching `inst.config.data_dir`).
   - Eliminate loose name matching (`node.instance_name === inst.config.name`).
   - Display `[RUNNING]` only when the project is confirmed active on that specific instance.
5. **Structured Audit Logging & Traceability**:
   - Add structured `[PROMPT_LIVENESS_PROBE]` logs capturing instance ID, PID, workspace path, and computed liveness state.
6. **End-to-End Verification Test**:
   - Add `src-tauri/tests/per_instance_prompt_liveness_test.rs` (annotated with `#[ignore]`).
7. **Root Cause Analysis Documentation**:
   - Persist full RCA in `02-spec/22-app-issues/20-cross-instance-running-prompts-bleed-rca.md`.

---

## 2. User Request (Verbatim)

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

## 3. Architecture & Technical Strategy

### 3.1 Backend: `get_project_conversation_tree` Refactoring
- **Signature**:
  ```rust
  #[tauri::command]
  pub fn get_project_conversation_tree(
      instance_id: Option<String>,
      max_words: Option<usize>,
      only_running: Option<bool>,
      force: Option<bool>,
  ) -> Result<Vec<crate::modules::repo_db::AgmProjectTreeNode>, String>
  ```
- **Caching**:
  Partition cache keys by instance: `tree:{instance_id}:{max_words}:{only_running}`.
- **Process Verification**:
  When evaluating `AgmProjectTreeNode.is_running`:
  Check whether the instance owning the project has an active OS process:
  ```rust
  let is_inst_alive = crate::modules::instance::is_instance_running(
      &inst.id,
      &inst.data_dir,
      inst.pid,
  );
  ```
  If `!is_inst_alive`, `is_running` is strictly `false`.
  If alive, verify that the project is currently open in that instance or has active in-flight prompts in SQLite.

### 3.2 Frontend: `Instances.tsx` & `PromptTreeViewModal.tsx`
- In `Instances.tsx`:
  Pass `instance_id` to query per-instance trees or filter strictly by `node.instance_id === inst.config.id` (with default fallback for legacy records).
  Remove `node.instance_name === inst.config.name` loose string match.

---

## 4. Verification & Acceptance Criteria
- [ ] Card #1 (Default) shows only projects running in the Default instance (`Antigravity-Manager` marked `[RUNNING]`, other projects not running).
- [ ] Card #2 (8159) shows only projects running in 8159 (`coding-guidelines` marked `[RUNNING]`, other projects not running).
- [ ] Backend `get_project_conversation_tree` accepts `instance_id: Option<String>` and scopes cache and scan results.
- [ ] No project is marked `[RUNNING]` if its parent instance process is not running.
- [ ] Audit logs (`[PROMPT_LIVENESS_PROBE]`) log resolution rationale.
- [ ] Root cause analysis documented in `02-spec/22-app-issues/20-cross-instance-running-prompts-bleed-rca.md`.
- [ ] Unit/Integration test written in `src-tauri/tests/per_instance_prompt_liveness_test.rs`.
