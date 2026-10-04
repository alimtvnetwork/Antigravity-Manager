# Plan: 114-debug-instance-prompt-running-detection

## User Request (Verbatim)
```text
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

## Summary & Architectural Synthesis
Ground truth inspection revealed:
1. In Default profile (`~/.gemini/antigravity/conversation_summaries.db`), `Antigravity-Manager` is actively running (`CASCADE_RUN_STATUS_RUNNING`, `not_fully_idle = 1`). `spec-builder` and `coding-guidelines` are IDLE / not running.
2. In 8159 profile (`C:\Users\Administrator\.antigravity_tools\instances\default-copy-8159\home\.gemini\antigravity\conversation_summaries.db`), `coding-guidelines` is actively running (`CASCADE_RUN_STATUS_RUNNING`, `not_fully_idle = 1`). `spec-builder` and `Antigravity-Manager` are IDLE / not running.
3. The previous system mistakenly reported `spec-builder` and `coding-guidelines` as running on default, and reported `spec-builder` and `Antigravity-Manager` as running on 8159 because:
   - Candidate directories across ALL instances were aggregated into a flat list, stripping instance identity, hardcoding `"default"`, and keying conversations solely by normalized repo path (`convs_by_path`).
   - Any project matching the path inherited active conversations from foreign instances.
   - An unconditional timestamp heuristic `is_recency_active = age < 600` falsely overrode `CASCADE_RUN_STATUS_IDLE` and `not_fully_idle == 0`.
   - `detect_running_projects` assigned `is_running: is_instance_active` to all historical workspaces in an instance's `workspaceStorage`.
   - `get_project_conversation_tree_cached` ignored `instance_id` and cached trees globally.

## Traceable Subtasks
- **Task-01**: Author architecture spec in `02-spec/21-app/114-debug-instance-prompt-running-detection/01-architecture-spec.md` and subtask `01-backend-detection-refactor.md`.
- **Task-02**: Author component spec in `02-spec/21-app/114-debug-instance-prompt-running-detection/02-component-spec.md` and subtask `02-frontend-and-tests.md`.
- **Task-03**: Implement structured audit logging `[InstancePromptAudit]` in `src-tauri/src/modules/logger.rs` and document root cause analysis in `02-spec/22-app-issues/21-instance-prompt-running-check-rca.md`.
- **Task-04**: Refactor backend prompt and project detection in `src-tauri/src/modules/repo_db.rs` and `src-tauri/src/commands/instance.rs`.
- **Task-05**: Update frontend filtering and live state polling in `src/pages/Instances.tsx` and `src/components/instances/PromptTreeViewModal.tsx`.
- **Task-06**: Implement comprehensive unit & integration tests under `src-tauri/tests/per_instance_prompt_liveness_test.rs`.
- **Task-07**: Verify default and 8159 instance isolation and accuracy with live process and conversation data.
- **Task-08**: Run targeted linters, update index registries, consolidate plans, and push atomic GitMap commit.
