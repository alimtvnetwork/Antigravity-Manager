# Master Plan: 123-prompt-running-instance-detection-and-audit-logging

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

## 1. Executive Problem Summary & Ground Truth Matrix

The Google Antigravity ecosystem manages multiple parallel instances:
- Sequence 1: Default Profile (`default`)
- Sequence 2: Cloned Instance `default-copy-8159` (shorthand `8159`)

### Ground Truth Target State:
| Instance | Target Workspace | Ground Truth Status |
| :--- | :--- | :--- |
| `default` | `Antigravity-Manager` | **RUNNING** (`is_running: true`) |
| `default` | `SpecBuilder` | **IDLE** (`is_running: false`) |
| `default` | `coding-guidelines` | **IDLE** (`is_running: false`) |
| `8159` (`default-copy-8159`) | `coding-guidelines` | **RUNNING** (`is_running: true`) |
| `8159` (`default-copy-8159`) | `Antigravity-Manager` | **IDLE** (`is_running: false`) |
| `8159` (`default-copy-8159`) | `SpecBuilder` | **IDLE** (`is_running: false`) |

---

## 2. Nine Root Causes Discovered

1. **Cross-Instance Prompt Bleed via Path Matching**:
   In `resend_running_commands_for_instance` (`repo_db.rs:2981-2984`) and `dispatch_running_prompts` (`repo_db.rs:1404-1406`), backed-up prompts from `default` were matched against `default-copy-8159` simply because `default-copy-8159` had cloned workspace storage folders matching the repository paths. Prompts belonging to `default` were erroneously dispatched to `8159`, setting `is_running = 1` in `running_projects`.
2. **Project Identity Hijacking & Table Duplication in `running_projects`**:
   `detect_running_projects` previously scanned cloned workspace folders and updated rows where `id` matched `repo-hash` without the instance suffix (`ON CONFLICT(id) DO UPDATE SET instance_id = excluded.instance_id`). This hijacked `default`'s `antigravity-manager` record and assigned it `instance_id = 'default-copy-8159'`.
3. **Specifier / Suffix Resolution Omission**:
   In `detect_running_projects` (`repo_db.rs:477`), `resolve_instance_id` was not called. Passing shorthand `"8159"` resulted in `Instance '8159' not found`, causing inconsistent and stale state retention.
4. **`get_live_project_execution_info` Treats 'queued' as Live Running**:
   In `repo_db.rs:2363-2387`, `active_prompts` rows with `status = 'queued'` set `entry.0 = true`, reporting inactive queued prompts as actively executing.
5. **Cache Contamination in `prompt_tree_cache`**:
   The cache entry `tree:300:false` cached corrupted trees across instances for 60 seconds without immediate invalidation when projects or instances update.
6. **Missing Audit Logging in Production Tree Evaluation**:
   `compute_project_conversation_tree` did not emit structured audit logs (`log_instance_prompt_audit`) explaining each project's running verdict and each directory scanned.
7. **SQL Wildcard Bug in `DELETE WHERE id NOT LIKE '%__%'`**:
   In SQLite, `_` in `LIKE` matches any single character, so `'%__%'` matched any string with 2+ characters! `NOT LIKE '%__%'` only matched strings of length 0 or 1, completely failing to delete non-composite rows (`length >= 2`). This left hundreds of legacy rows with `is_running = 1` permanently stranded in `running_projects`. Fixed with `WHERE instr(id, '__') = 0` and orphaned workspace pruning.
8. **`save_or_requeue_prompt` Polluting `running_projects`**:
   Unconditionally executed `INSERT OR REPLACE INTO running_projects ... VALUES (..., NULL, 1, ...)`, persisting `is_running = 1` and `workspace_storage_path = NULL` even for `queued` or `backed_up` prompts, and without composite ID namespacing.
9. **`compute_project_conversation_tree` Arbitrary `LIMIT 40` Truncation**:
   Caused older or multi-workspace projects to return 0 conversation nodes, triggering the empty workspace fallback rather than reading actual on-disk idle state.

---

## 3. Architecture Scope & Subtask Decomposition

### Subtask 1: Backend Isolation & Database Hygiene (`01-fix-instance-prompt-matching-and-db-hygiene.md`)
- File: `src-tauri/src/modules/repo_db.rs`
- Eliminate cross-instance prompt bleed in `resend_running_commands_for_instance` and `dispatch_running_prompts`. Prompts must match `p.instance_id == target_inst` (with canonical resolution).
- Universal canonical instance normalization: wrap `instance_id` with `resolve_instance_id` across `detect_running_projects`, `gemini_dirs_tagged`, and `compute_project_conversation_tree`.
- Database hygiene migration: Prune and repair non-composite IDs in `running_projects` using `instr(id, '__') = 0` (fixing the SQLite `LIKE` wildcard defect) and prune orphaned workspace entries.
- Sanitize `save_or_requeue_prompt`: Ensure prompts with `status = 'queued'` or `'backed_up'` never insert `is_running = 1` or null out `workspace_storage_path`, and namespace primary keys with composite IDs.
- Remove arbitrary `LIMIT 40` truncation in `compute_project_conversation_tree` so multi-workspace projects receive complete conversation summaries from disk.
- Invalidate `prompt_tree_cache` upon prompt state or instance state changes.
- Ensure `get_live_project_execution_info` differentiates `running` from `queued`.

### Subtask 2: Comprehensive Audit Logging, Verification & E2E Tests (`02-audit-logging-e2e-tests-and-verification.md`)
- Files: `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/logger.rs`, `src-tauri/tests/per_instance_prompt_liveness_test.rs`
- Add structured audit logging (`log_instance_prompt_audit`) at every decision point in `compute_project_conversation_tree` and `is_prompt_running_for_project` Gate 4.
- Implement comprehensive end-to-end integration tests in `src-tauri/tests/per_instance_prompt_liveness_test.rs` validating:
  - Sequence 1 (`default`): `Antigravity-Manager` = RUNNING; `SpecBuilder` = IDLE; `coding-guidelines` = IDLE.
  - Sequence 2 (`8159` / `default-copy-8159`): `coding-guidelines` = RUNNING; `Antigravity-Manager` = IDLE; `SpecBuilder` = IDLE.
  - Shorthand suffix resolution (`"8159"` -> `"default-copy-8159"`).
  - Database primary key composite isolation.
  - Audit log output validation.

---

## 4. Phase 1 Spec Authoring Allocation (Disjoint Subagents)
- **Subagent 1 (self)**:
  - Spec: `02-spec/21-app/123-prompt-running-instance-detection-and-audit-logging/01-architecture-spec.md`
  - Subtask: `.ai-memory/plans/subtasks/123-prompt-running-instance-detection-and-audit-logging/01-fix-instance-prompt-matching-and-db-hygiene.md`
- **Subagent 2 (self)**:
  - Spec: `02-spec/21-app/123-prompt-running-instance-detection-and-audit-logging/02-component-spec.md`
  - Subtask: `.ai-memory/plans/subtasks/123-prompt-running-instance-detection-and-audit-logging/02-audit-logging-e2e-tests-and-verification.md`
