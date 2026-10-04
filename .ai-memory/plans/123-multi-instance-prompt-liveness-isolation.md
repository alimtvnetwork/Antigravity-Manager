# Parent Task Plan: 123-multi-instance-prompt-liveness-isolation

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

## 1. Executive Problem Summary & Ground Truth Objectives

In a multi-instance desktop deployment with cloned profiles and concurrent running IDEs:
1. **Sequence 1 (`default` profile, PID 11628 / 12980)**:
   - Truly Running: `Antigravity-Manager`
   - Truly IDLE (NOT Running): `SpecBuilder` (must show NOT running / IDLE)
   - Truly IDLE (NOT Running): `coding-guidelines` (must show NOT running / IDLE)
2. **Sequence 2 (`8159` / `default-copy-8159`, PID 11984)**:
   - Truly Running: `coding-guidelines`
   - Truly IDLE (NOT Running): `SpecBuilder` (must show NOT running / IDLE)
   - Truly IDLE (NOT Running): `Antigravity-Manager` (must show NOT running / IDLE)

### Root Cause Synthesis:
1. **CLI Background Directory Contamination**: `gemini_dirs_for_instance` swept `antigravity-cli`. Autonomous CLI agents writing to `~/.gemini/antigravity-cli/conversation_summaries.db` for background scripts caused GUI instance cards to falsely claim those projects were running in the GUI IDE.
2. **Permissive Fallback in Conversation Tree**: In `compute_project_conversation_tree`, when `conv_nodes` had no running conversations, lines 4030-4035 invoked `has_active_prompt = is_prompt_running_for_project(...)`. Gate 4 inside `is_prompt_running_for_project` scanned candidate directories again and matched paths, flipping `proj_is_running = true`.
3. **Stale `running_projects` Persistence**: Stale `is_running = 1` rows in SQLite `running_projects` table persisted from previous test runs and clones without active turn validation.
4. **Prompt Tree Caching Cross-Contamination**: 60-second TTL cache in `prompt_tree_cache` served stale tree nodes with `is_running: true`.

---

## 2. Technical Remediation Strategy

1. **Strict GUI Candidate Directory Confinement**:
   - `gemini_dirs_for_instance(instance_id)`: Exclude `antigravity-cli` for GUI instance cards. GUI profiles only inspect `antigravity` and `antigravity-ide`.
   - Dedicated CLI helper: If CLI inspection is needed, provide `gemini_dirs_for_cli()` isolated from GUI profile cards.
2. **Strict Instance and Project Isolation**:
   - In `compute_project_conversation_tree`, evaluate running conversations strictly partitioned by `(norm_owning_inst, clean_path)`.
   - Require:
     * Process alive for the owning instance (`is_inst_alive`).
     * `not_fully_idle > 0`.
     * `status.contains("RUNNING")`.
     * Explicit idle supremacy: `!status.contains("IDLE") && !status.contains("COMPLETED") && !status.contains("FAILED") && !status.contains("CANCELLED")`.
     * Fresh TTL: `timestamp >= now - 900` (15 minutes).
   - Eliminate permissive fallback that resurrects running status when zero active conversations exist for this instance.
3. **Structured Audit & Debug Telemetry**:
   - Enhance `log_instance_prompt_audit` to log exact gate evaluations, rejection reasons (e.g. `STALE_TTL_EXPIRED`, `EXPLICIT_IDLE_SUPREMACY`, `PROCESS_DEAD`, `ZERO_ACTIVE_TURNS`), and source channel tags (`[GUI_IDE]` vs `[CLI_RUNNER]`).
4. **Comprehensive Companion Tests**:
   - Expand `src-tauri/tests/per_instance_prompt_liveness_test.rs` to assert Ground Truth Sequence 1 (`default`) and Sequence 2 (`8159`) with strict positive boolean invariants.

---

## 3. Subtask Decomposition (Disjoint File Boundaries)

| Subtask ID | Title | Assigned Agent | Target Files |
| :--- | :--- | :--- | :--- |
| **01-core-detection-and-isolation** | Core Detection, Directory Confinement & Tree Computation Fix | Worker 01 | `src-tauri/src/modules/repo_db.rs` |
| **02-audit-logging-and-e2e-tests** | Audit Logging Telemetry, RCA Documentation & E2E Tests | Worker 02 | `src-tauri/src/modules/logger.rs`, `src-tauri/tests/per_instance_prompt_liveness_test.rs`, `02-spec/22-app-issues/123-per-instance-prompt-running-detection-root-cause.md`, `02-spec/21-app/123-multi-instance-prompt-liveness-isolation/03-root-cause-analysis.md` |
