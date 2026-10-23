---
name: implementing-next-five-tasks
description: Autonomously pull upstream changes, claim the next five sequential backlog tasks, synchronize git status with a taken commit, author specs and subtask plans, execute code changes across multi-agent swarms with zero builds and zero tests, and commit atomically via GitMap.
---

# Implementing Next Five Tasks (V6 Autonomous Protocol)

## Protocol Workflow
1. **Git Pull Preflight**: Run `gitmap pull` (fallback `git pull`) to guarantee a clean, synchronized upstream tree before touching any files.
2. **Backlog Discovery & Reservation**:
   - Locate next 5 sequential tasks in `.ai-memory/plans/`.
   - Update their state to `IN_PROGRESS` / `TAKEN`.
   - Stage and commit the reservation via GitMap so concurrent agents bypass these tasks.
3. **Spec & Subtask Authoring**:
   - Create specification under `02-spec/21-app/XX-implementing-next-five-tasks/`.
   - Create parent plan `.ai-memory/plans/pending/XX-implementing-next-five-tasks.md`.
   - Decompose into 5 subtask markdown plans in `.ai-memory/plans/subtasks/XX-implementing-next-five-tasks/`.
4. **Multi-Agent Swarm Dispatch (`A = 3`, `H = 2`)**:
   - Spawn `A = 3` subagents via `invoke_subagent` in strictly disjoint file boxes.
   - Enforce coding guidelines (PascalCase enums, positive booleans, no silent errors, FIFO query ordering).
5. **Execution & Finalization**:
   - Zero builds and zero full test runs per Rule R1.
   - Mark plan and subtasks as `COMPLETED` and move to `.ai-memory/plans/completed/`.
   - Final atomic commit and push via `gitmap cpf "<module> - <summary>"`.
