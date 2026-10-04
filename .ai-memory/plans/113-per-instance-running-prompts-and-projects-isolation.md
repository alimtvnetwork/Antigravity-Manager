# Plan 113: Strict Per-Instance Project & Prompt Liveness Isolation

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

## Visual Reference
- `![Cross-Instance Projects Bleed](assets/screenshots/113-per-instance-running-prompts-and-projects-isolation-01.png)`

## Architecture Boundaries & Subtask Breakdown

### Wave 1: Subtask Plan Generation (A = 2, H = 2 Subagents)
- **Subagent A (Backend Architecture Planner)**:
  - `.ai-memory/plans/subtasks/113-per-instance-running-prompts-and-projects-isolation/001-backend-parametric-tree-and-pid-liveness.md`
  - `.ai-memory/plans/subtasks/113-per-instance-running-prompts-and-projects-isolation/002-structured-audit-logging-and-traceability.md`
- **Subagent B (Frontend & Testing Planner)**:
  - `.ai-memory/plans/subtasks/113-per-instance-running-prompts-and-projects-isolation/003-frontend-instance-card-scoping-and-badging.md`
  - `.ai-memory/plans/subtasks/113-per-instance-running-prompts-and-projects-isolation/004-e2e-isolation-test-and-rca-documentation.md`

### Wave 2: Execution (A = 2, H = 2 Worker Subagents)
- **Worker 01 (Backend Specialist)**:
  - Files:
    - `src-tauri/src/commands/instance.rs`
    - `src-tauri/src/modules/repo_db.rs`
    - `src-tauri/src/modules/instance.rs`
    - `src-tauri/tests/per_instance_prompt_liveness_test.rs`
- **Worker 02 (Frontend & Documentation Specialist)**:
  - Files:
    - `src/pages/Instances.tsx`
    - `src/components/instances/PromptTreeViewModal.tsx`
    - `02-spec/22-app-issues/20-cross-instance-running-prompts-bleed-rca.md`
