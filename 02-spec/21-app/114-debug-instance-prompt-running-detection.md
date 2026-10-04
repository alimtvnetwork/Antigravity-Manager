# 114 — Debug Instance Prompt Running Detection and Multi-Instance Isolation

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

## System Overview
This specification governs the deterministic, zero-leak detection of running projects and prompt conversations across distinct Antigravity IDE instances (Default profile and cloned/secondary profiles such as 8159).

### Core Problem
In multi-profile setups:
1. Default profile was executing `Antigravity-Manager`, but AGM UI falsely marked `spec-builder` and `coding-guidelines` as `RUNNING`.
2. Cloned profile (8159) was executing `coding-guidelines`, but AGM UI falsely marked `spec-builder` and `Antigravity-Manager` as `RUNNING`.
3. Historical workspaces in `workspaceStorage` were unconditionally marked `is_running = true` whenever an instance process was alive.
4. Conversations from foreign instances bled across instance boundaries due to global directory aggregation and path-only keying.

### Architectural Solution
1. **Per-Instance Gemini Directory Resolution**: `gemini_dirs_for_instance(instance_id)` targets strictly the `.gemini/antigravity` path for that specific instance.
2. **Tagged Conversation Mapping**: Key conversation cache by `(instance_id, repo_path)`.
3. **Strict Idle Respect**: If `not_fully_idle == 0` or status is `"CASCADE_RUN_STATUS_IDLE"`, never override with recency heuristics.
4. **Active Window & Task Liveness**: A project in `workspaceStorage` is only running if it has an affirmative active conversation or task in that instance's summaries DB.
5. **Structured Audit Logging**: Emit `[InstancePromptAudit]` logs detailing the exact decision logic.
6. **E2E Integration Testing**: Hermetic tests verifying complete cross-instance isolation.
