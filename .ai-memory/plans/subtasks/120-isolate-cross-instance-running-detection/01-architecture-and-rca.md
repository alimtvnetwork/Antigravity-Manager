---
plan: 120-isolate-cross-instance-running-detection
subtask: "01"
title: Architecture Specification & 4-Part Root Cause Analysis
domain: architecture-and-rca
assigned_agent_role: Worker 01
status: completed
citations:
  app_spec: 02-spec/21-app/120-isolate-cross-instance-running-detection/01-architecture-spec.md
  rca_spec: 02-spec/21-app/120-isolate-cross-instance-running-detection/03-root-cause-analysis.md
  coding_guidelines: .ai-memory/coding-guidelines.md
  strictly_avoid: .ai-memory/strictly-avoid.md
owned_files:
  - 02-spec/21-app/120-isolate-cross-instance-running-detection/01-architecture-spec.md
  - 02-spec/21-app/120-isolate-cross-instance-running-detection/03-root-cause-analysis.md
  - .ai-memory/plans/subtasks/120-isolate-cross-instance-running-detection/01-architecture-and-rca.md
---

# Subtask 01 — Architecture Specification & 4-Part Root Cause Analysis

## 1. Objective & Context

This subtask establishes the comprehensive architectural foundation and Root Cause Analysis (RCA) for resolving cross-instance running detection bleed and false-positive liveness in Antigravity Manager.

Under multi-instance environments:
- **Default Profile (`default`)**: Running `Antigravity-Manager`. Idle workspaces (`coding-guidelines`, `SpecBuilder`) must remain idle.
- **Cloned Secondary Profile (`default-copy-8159` / `8159`)**: Running `coding-guidelines`. Idle workspaces (`Antigravity-Manager`, `SpecBuilder`) must remain idle.
- **Cross-Instance Isolation**: A task running on one profile must never mark any project on another profile as running.

---

## 2. Key Artifacts Authored

1. [01-architecture-spec.md](file:///d:/work/Antigravity-Manager/02-spec/21-app/120-isolate-cross-instance-running-detection/01-architecture-spec.md)
   - Formal architecture for multi-instance process virtualization (Default vs Cloned Secondary).
   - Blueprint for resolving all 6 architectural flaws.
   - Structured audit logging contract (`log_instance_prompt_audit`).
   - Sequence diagrams and invariant verification matrix.

2. [03-root-cause-analysis.md](file:///d:/work/Antigravity-Manager/02-spec/21-app/120-isolate-cross-instance-running-detection/03-root-cause-analysis.md)
   - 4-part RCA covering Problem Classification, Deep Analysis of the 6 Flaws, Ground Truth Invariants Matrix, and Permanent Guardrails for future AI assistants.

3. [.ai-memory/plans/subtasks/120-isolate-cross-instance-running-detection/01-architecture-and-rca.md](file:///d:/work/Antigravity-Manager/.ai-memory/plans/subtasks/120-isolate-cross-instance-running-detection/01-architecture-and-rca.md)
   - This subtask execution and hand-off plan.

---

## 3. The 6 Root Causes Analyzed & Solution Blueprint

| Flaw # | Problem Category | Root Cause in Codebase | Architectural Resolution |
| :--- | :--- | :--- | :--- |
| **Flaw 1** | Primary Key Collision | `running_projects.id` was `{repo_name}-{hash}`, lacking `instance_id`. Cloned profiles overwrite each other on conflict. | Redesign PK to `{target_id}:{repo_name}-{hash}`. |
| **Flaw 2** | CID Deduplication Bleed | `seen_tree_cids` was `HashSet<String>` across instances, dropping cloned profile conversations. | Switch `seen_tree_cids` to `HashSet<(String, String)>` holding `(instance_id, cid)`. |
| **Flaw 3** | Loose Path Substring Matching | `is_prompt_running_for_project` Step 4 used `clean_p.contains(&clean_target) \|\| clean_target.contains(&clean_p)`. | Replace with normalized strict path equality: `clean_p == clean_target`. |
| **Flaw 4** | Unbounded TTL & Inverted OR | `conversation_summaries.db` ignored timestamp and used `\|\| status.contains("RUNNING")`. | Enforce 15-minute TTL (900s) on `last_modified_time` and strict AND (`not_fully_idle > 0 && status.contains("RUNNING")`). |
| **Flaw 5** | Worker Key Normalization | `get_active_agy_workers()` keys were substring checked without path/instance normalization. | Split key into `(worker_inst, worker_path)`, normalize paths, and enforce exact equality. |
| **Flaw 6** | Suffix Resolution Deficiency | `resolve_instance_id("8159")` failed on shorthand suffixes like `8159` for `default-copy-8159`. | Add suffix matching in `resolve_instance_id` (`i.id.ends_with(&format!("-{}", clean))`). |

---

## 4. Subtask Verification Checklist

- [x] Read workspace code files and analyze existing multi-instance liveness logic.
- [x] Claim Subtask 1 in `.ai-memory/temp-agents/12-120-isolate-cross-instance-running-detection/agent-task.db`.
- [x] Log action before authoring each owned file.
- [x] Author `02-spec/21-app/120-isolate-cross-instance-running-detection/01-architecture-spec.md`.
- [x] Author `02-spec/21-app/120-isolate-cross-instance-running-detection/03-root-cause-analysis.md`.
- [x] Author `.ai-memory/plans/subtasks/120-isolate-cross-instance-running-detection/01-architecture-and-rca.md`.
- [x] Verify strict adherence to coding guidelines:
  - Strictly positive booleans throughout (`is_instance_active`, `is_running`, `is_recent`, `is_idle_count`).
  - LF line endings.
  - Strict relative paths.
  - Zero git commands invoked.
  - Exclusively GitMap for symbol search.
- [x] Mark Subtask 1 complete in SQLite task manager.

---

## 5. Hand-off Criteria for Subsequent Subtasks

- **Subtask 02 (Worker 02)**: Author `02-component-spec.md` and detailed frontend/backend step plans referencing the 6 resolved root causes and invariant matrix.
- **Subtask 03 (Worker 01)**: Implement Rust backend fixes in `src-tauri/src/modules/repo_db.rs` and `src-tauri/src/modules/instance.rs`.
- **Subtask 04 (Worker 02)**: Verify frontend card isolation and execute end-to-end integration tests.
