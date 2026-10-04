# Master Plan: 120-isolate-cross-instance-running-detection

## User Request (Verbatim)
> Also how you check the prompts are running or not on that instance is also very wrong. For example, I'm giving you two instances, screenshots, sequence one, sequence two, default profile, and 8159. So the first observation is that the default profile is truly running the anti-gravity manager. That is correct. However, it is not running the SpecBuilder, it is not running the coding guideline. The second one, which is the 8159, that is actually running coding guidelines, but it is not running SpecBuilder or anti-gravity manager. So these are two things, your observation and how you are doing it. I think you need to debug that. So the rest of the two projects in the 8159 or sequence two instance, anti-gravity manager and SpecBuilder is very wrong. It is not running there. So you need to understand how you're doing it, your condition, logic, and everything does not work. So you need to make sure that it works. You need to debug it. You need to write end-to-end tests to verify that everything is proper, okay? So make sure of that, please. Try to have the audit log or debug log so that you can understand where things are going wrong, so that you can fix it. You can find the root cause and then fix it. Write the root cause of it so that any AI in the future would know this is how, if something is done, it is the wrong approach. Do you understand? Can you please help me with this?

---

## 1. Executive Summary & Problem Classification
In multi-instance environments:
- **Default Profile (Sequence 1)**: Actively running ONLY `Antigravity-Manager`. Projects `SpecBuilder` and `coding-guidelines` are idle.
- **Instance 8159 (Sequence 2)**: Actively running ONLY `coding-guidelines`. Projects `Antigravity-Manager` and `SpecBuilder` are idle.

Root cause discovery confirmed 6 distinct systemic bugs:
1. **Primary Key Collision in `running_projects` SQLite Table**: Project IDs (`{repo_name}-{hash}`) lack `instance_id`. Cloned profiles overwrite rows of earlier instances via `ON CONFLICT(id) DO UPDATE`.
2. **Global CID Deduplication Across Instances**: `seen_tree_cids` deduplicates across all profiles, dropping cloned conversations on secondary instances.
3. **Loose Path Containment in `is_prompt_running_for_project`**: `clean_target.contains(&clean_p)` matches all sub-projects under parent workspace paths.
4. **Missing Liveness TTL & Inverted OR Logic in `conversation_summaries.db` Query**: Stale turns never expire; `not_fully_idle != 0 || status.contains("RUNNING")` evaluates to true for idle turns.
5. **Worker Key Matching Normalization**: Loose `.contains()` on unnormalized path strings.
6. **Instance Alias Resolution (`8159` vs `default-copy-8159`)**: Suffix matching missing in instance resolver.

---

## 2. Discrete Deliverables & Subtasks Matrix

| Subtask Code | Title | Assigned Agent | Owned Files |
|---|---|---|---|
| `Task-01` | 01-architecture-and-rca | Worker 01 | `02-spec/21-app/120-isolate-cross-instance-running-detection/01-architecture-spec.md`, `02-spec/21-app/120-isolate-cross-instance-running-detection/03-root-cause-analysis.md`, `.ai-memory/plans/subtasks/120-isolate-cross-instance-running-detection/01-architecture-and-rca.md` |
| `Task-02` | 02-component-and-plan | Worker 02 | `02-spec/21-app/120-isolate-cross-instance-running-detection/02-component-spec.md`, `.ai-memory/plans/subtasks/120-isolate-cross-instance-running-detection/02-component-and-plan.md` |
| `Task-03` | 03-backend-running-isolation | Worker 01 | `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/instance.rs`, `.ai-memory/plans/subtasks/120-isolate-cross-instance-running-detection/03-backend-running-isolation.md` |
| `Task-04` | 04-frontend-isolation-and-e2e | Worker 02 | `src/pages/Instances.tsx`, `src-tauri/tests/test_instance_prompt_running_isolation.rs`, `.ai-memory/plans/subtasks/120-isolate-cross-instance-running-detection/04-frontend-isolation-and-e2e.md` |

---

## 3. Ground Truth Verification Invariants

```text
+-----------------------+---------------------+----------------+-----------------------+
| Instance              | Project             | Expected State | Required Liveness     |
+-----------------------+---------------------+----------------+-----------------------+
| Default Profile       | Antigravity-Manager | RUNNING        | true  (active pulse)  |
| Default Profile       | SpecBuilder         | IDLE           | false (no badge)      |
| Default Profile       | coding-guidelines   | IDLE           | false (no badge)      |
| Instance 8159         | Antigravity-Manager | IDLE           | false (no badge)      |
| Instance 8159         | SpecBuilder         | IDLE           | false (no badge)      |
| Instance 8159         | coding-guidelines   | RUNNING        | true  (active pulse)  |
+-----------------------+---------------------+----------------+-----------------------+
```

---

## 4. Execution Pipeline Phases
- **Phase 1 Planning**: Research discovery (Completed), Master Plan & SQLite manifest initialization.
- **Phase 1 Spec Step**: Spawn 2 authoring subagents for modular spec authoring (Architecture, RCA, Component specs).
- **Phase 2 Execution Step**: Spawn 2 worker subagents for backend fixes, audit logging, frontend alignment, and real E2E tests.
- **Phase 3 Consolidation & Verification**: Targeted checks, secrets gate, and atomic GitMap commit (`gitmap cpb "repo-db - isolate cross-instance prompt running detection and add audit logging"`).
