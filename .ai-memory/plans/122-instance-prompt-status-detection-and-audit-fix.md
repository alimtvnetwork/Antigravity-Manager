# Plan: 122-instance-prompt-status-detection-and-audit-fix

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

## Technical Context & Scope

- **Problem Statement**:
  - `default` profile: Truly running ONLY `Antigravity-Manager`. Must NOT report `SpecBuilder` or `coding-guidelines` as running.
  - `default-copy-8159` (8159 / sequence 2): Truly running ONLY `coding-guidelines`. Must NOT report `SpecBuilder` or `Antigravity-Manager` as running.
  - The detection logic across `src-tauri/src/modules/repo_db.rs` suffers from:
    1. Missing 15-minute TTL check in `compute_project_conversation_tree` for conversation turns.
    2. Missing `"antigravity-cli"` in `gemini_dirs_for_instance`.
    3. Global path-only matching in `get_live_project_execution_info` (`live_map` keyed only by `clean_path` without `instance_id`).
    4. Unpartitioned `seen_tree_cids` global deduplication.
    5. Overly permissive `proj_is_running` fallback without per-instance scoping.
    6. Stale `prompt_tree_cache` preserving stale running flags.
- **Goals**:
  1. Author specifications under `02-spec/21-app/122-instance-prompt-status-detection-and-audit-fix/`.
  2. Research codebase using GitMap high-speed commands.
  3. Fix the running prompt detection condition and logic for all instances (`default`, `8159`, etc.).
  4. Implement clear audit logs and debug logs with full evaluation rationale.
  5. Author end-to-end tests verifying the exact isolation requirements.
  6. Document comprehensive Root Cause Analysis for future reference in `02-spec/21-app/122-instance-prompt-status-detection-and-audit-fix/03-root-cause-analysis.md` and `02-spec/22-app-issues/122-instance-prompt-running-detection-root-cause.md`.

---

## Multi-Agent Execution Breakdown (A = 2, H = 2)

- **Phase 1 Planning**: A = 2 `research` Discovery Subagents
  - Research 01: Core running detection architecture (`is_prompt_running_for_project`, `compute_project_conversation_tree`, `get_live_project_execution_info`).
  - Research 02: Instance resolution, candidate directory resolution, and frontend tree aggregation.
- **Phase 1 Spec**: Author architecture, component specs, and RCA.
- **Phase 2 Execution**: A = 2 `self` Worker Subagents
  - Worker 01: Refactor `repo_db.rs` detection logic, TTL enforcement, and per-instance scoping.
  - Worker 02: Enhance structured audit logging in `logger.rs` & `repo_db.rs`; expand companion end-to-end tests.
- **Phase 3 Consolidation**: Targeted linters, index updates, and single atomic GitMap commit.
