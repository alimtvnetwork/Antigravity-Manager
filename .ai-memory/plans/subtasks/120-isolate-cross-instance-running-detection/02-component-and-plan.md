---
plan: 120-isolate-cross-instance-running-detection
subtask: "02"
title: Component Specification and Subtasks Plan
domain: full-stack-spec
depends_on: "01"
citations:
  component_spec: ../../../../02-spec/21-app/120-isolate-cross-instance-running-detection/02-component-spec.md
  architecture_spec: ../../../../02-spec/21-app/120-isolate-cross-instance-running-detection/01-architecture-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - 02-spec/21-app/120-isolate-cross-instance-running-detection/02-component-spec.md
  - .ai-memory/plans/subtasks/120-isolate-cross-instance-running-detection/02-component-and-plan.md
  - .ai-memory/plans/subtasks/120-isolate-cross-instance-running-detection/03-backend-running-isolation.md
  - .ai-memory/plans/subtasks/120-isolate-cross-instance-running-detection/04-frontend-isolation-and-e2e.md
status: pending
---

# 02 — Component Specification & Subtasks Plan

## 1. Context & Purpose
Task 120 resolves cross-instance running state contamination and establishes concrete process-gated liveness evaluation across multi-instance Antigravity Manager profiles. This subtask authors the formal component specification (`02-component-spec.md`) and the granular, actionable execution plans for subsequent implementation subtasks (`03-backend-running-isolation.md` and `04-frontend-isolation-and-e2e.md`).

## 2. Key Component Architectural Contracts
1. **`src-tauri/src/modules/repo_db.rs`**:
   - `detect_running_projects`: Scopes workspace storage strictly to the target instance data directory; marks active only if `is_instance_active && is_prompt_running_for_project`.
   - `is_prompt_running_for_project`: Gated by host instance process liveness; enforces strict prefix matching on background workers (`{instance_id}:`); enforces idle supremacy rule in `conversation_summaries.db`.
   - `compute_project_conversation_tree`: Associates conversations strictly with matching `(instance_id, repo_path)` pairs; forces all nodes to idle when host instance is dead.
   - `get_project_conversation_tree_cached`: Partitions cache keys by `instance_id` to prevent cross-profile cache poisoning.
2. **`src-tauri/src/modules/instance.rs`**:
   - `resolve_instance_id`: Adds suffix matching so identifiers like `"8159"` or `"-8159"` deterministically resolve to `"default-copy-8159"`.
3. **`src-tauri/src/modules/logger.rs`**:
   - `log_instance_prompt_audit`: Emits structured `[InstancePromptAudit]` logs capturing target instance, project name, path, liveness flags, task count, and standardized rationale strings.
4. **`src/pages/Instances.tsx`**:
   - `fetchRunningTasks`, `hasActiveTask`, and `isProjRunning`: Purges all residual references to `c.status === 'RUNNING'`; evaluates running states exclusively from verified `is_running` boolean properties gated by `Boolean(inst.is_running)`.
5. **`src-tauri/tests/test_instance_prompt_running_isolation.rs`**:
   - Deterministic integration test suite verifying that Default Profile running `Antigravity-Manager` leaves other projects idle, while Instance 8159 running `coding-guidelines` leaves `Antigravity-Manager` idle with zero cross-instance bleed.

## 3. Subtask Decomposition Matrix
- **Subtask 01 (`01-architecture-and-rca.md`)**: Architectural analysis and 4-part root cause documentation.
- **Subtask 02 (`02-component-and-plan.md`)**: Comprehensive component specification and subtasks plan definition (this document).
- **Subtask 03 (`03-backend-running-isolation.md`)**: Backend implementation in Rust (`repo_db.rs`, `instance.rs`, `logger.rs`).
- **Subtask 04 (`04-frontend-isolation-and-e2e.md`)**: Frontend alignment in React (`Instances.tsx`) and Rust integration test creation (`test_instance_prompt_running_isolation.rs`).

## 4. Coding & Architectural Constraints
- **Strictly Positive Booleans**: Always use positive boolean identifiers (`is_running`, `is_instance_active`, `is_alive`). Never use negated names like `is_not_running` or `uncompleted`.
- **LF Line Endings**: All created files must strictly enforce UNIX LF (`\n`) line endings.
- **Strict Relative Paths**: Use repo-relative file paths without absolute filesystem prefixes in documentation and tests.
- **Git Command Ban**: Never execute git CLI commands (`git status`, `git add`, `git commit`, `git diff`). Use GitMap commands (`gitmap aum search`, `gitmap lf`, `gitmap find`) exclusively.

## 5. Verification Checklist
- [x] Author `02-spec/21-app/120-isolate-cross-instance-running-detection/02-component-spec.md`.
- [x] Author `.ai-memory/plans/subtasks/120-isolate-cross-instance-running-detection/02-component-and-plan.md`.
- [x] Author `.ai-memory/plans/subtasks/120-isolate-cross-instance-running-detection/03-backend-running-isolation.md`.
- [x] Author `.ai-memory/plans/subtasks/120-isolate-cross-instance-running-detection/04-frontend-isolation-and-e2e.md`.
- [x] Record all file authoring actions in SQLite task database.
