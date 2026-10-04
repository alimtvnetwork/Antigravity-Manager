# 21 — Instance Prompt Running Detection and Process Isolation RCA

## 1. Executive Summary & Root Cause Analysis

When multi-instance workspaces were opened (e.g. `Sequence #1 Default Profile` and `Sequence #2 8159 Profile`):
- **User Observation**:
  - `Default Profile` was truly running `Antigravity-Manager`, but NOT running `spec-builder` or `coding-guidelines`.
  - `8159 Profile` was truly running `coding-guidelines`, but NOT running `spec-builder` or `Antigravity-Manager`.
  - The UI incorrectly showed `spec-builder` and `coding-guidelines` as running on `Default`, and showed `Antigravity-Manager` and `spec-builder` as running on `8159`.

## 2. Root Cause Breakdown

### Root Cause 1: Flat Candidate Directory Aggregation in `repo_db.rs`
`get_gemini_candidate_dirs()` scanned both `~/.gemini/antigravity` and all instances' `<instance_home>/.gemini/antigravity` and flattened them into a single list.
All conversation summaries from all instances were aggregated into `convs_by_path` with a hardcoded `"default"` instance tag. When any instance card rendered a project with a matching path, it inherited foreign conversation summaries from other instances.

### Root Cause 2: Blind 10-Minute Recency Assumption
`is_recency_active = age < 600` tagged conversations as running purely based on last modified timestamps, even if:
- The conversation was completely idle (`not_fully_idle == 0`).
- The instance process hosting the conversation was dead.
- The prompt had completed.

### Root Cause 3: `workspaceStorage` Duplication Marking All Projects as Running
`detect_running_projects` marked every single project found in an instance's `workspaceStorage` with `is_running: is_instance_active`.
When an instance was cloned, all workspace entries were copied. Thus, if the instance was running, ALL historical workspaces ever opened in that instance were stamped as `is_running: true`!

### Root Cause 4: Global `active_prompts` and Unscoped Database Queries
`is_prompt_running_for_project` checked `active_prompts` without filtering by `instance_id`, and always inspected default `~/.gemini/antigravity/conversation_summaries.db`. Thus, active prompts in the default profile caused `8159` to report `Antigravity-Manager` as running.

### Root Cause 5: Frontend Loose Name Matching and Empty-Array Fallback
In `Instances.tsx`, project matching used loose name equality `node.instance_name === inst.config.name`.
In `PromptTreeViewModal.tsx`, empty filtered results fell back to the global array (`relevant.length > 0 ? relevant : data`), leaking all global projects into instances that had zero projects.

## 3. Implemented Remedies

1. **Instance-Scoped Candidate Dirs**: When `instance_id` is supplied to `compute_project_conversation_tree`, only that instance's `.gemini` directory is scanned.
2. **Owning Instance Tagging**: Candidate directories are tagged with their true owning instance ID, and conversations only attach to projects of the matching instance.
3. **Strict Process Liveness Verification**: `is_running` is strictly `false` if `!is_instance_alive`. Recency alone never marks an entity as running.
4. **Per-Project Prompt Verification in `detect_running_projects`**: Projects in `workspaceStorage` are only marked running if `is_instance_active && is_prompt_running_for_project(...)`.
5. **Structured Audit Logging**: Emitted `[PROMPT_LIVENESS_PROBE]` logs detailing target instance ID, matched PIDs, evaluated projects, and explicit rationales (`INSTANCE_PROCESS_DEAD`, `ACTIVE_IN_FLIGHT_TASKS`, `IDLE`).
6. **Frontend Strict ID Matching & Fallback Elimination**: Gated `isProjRunning` by `inst.is_running`, matched strictly on `node.instance_id === inst.config.id`, and removed empty-array fallback leaks.

## 4. Architectural Rules for Future AI

1. **NEVER** use loose name comparisons (`node.instance_name === inst.config.name`) for instance scoping.
2. **NEVER** fall back to global collections when an instance-scoped filter returns an empty array.
3. **NEVER** mark an entity as running based on file recency without active OS process confirmation (`is_instance_alive`).
4. **ALWAYS** filter SQLite queries and in-memory caches by `instance_id`.
