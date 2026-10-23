# Subtask 04: GitMap cmdpull Modularization & 8-Pillars Clean Architecture
Parent: `221-gitmap-and-antigravity-tools-manager-updates-and-testing.md`
Step: 319
Improvement: `Imp-733`
Status: PENDING

## Scope & Objective
In `d:/work/gitmap/cli/cmdpull`:
- Decompose `pull_task_db.go` (586 lines) into:
  - `pull_task_db.go` ($\le 280$ lines): task DB coordinator and lifecycle methods.
  - `pull_task_db_ops.go` ($\le 280$ lines): execution operations and record status updates.
  - `pull_task_db_query.go` ($\le 280$ lines): queries, row scanning, and filters.
- Decompose `push.go` (353 lines) into:
  - `push.go` ($\le 220$ lines): entry point and flag definitions.
  - `push_exec.go` ($\le 220$ lines): remote push execution and retry logic.
- Follow 8-Pillars: all files $\le 300$ lines, functions 8–15 lines, error handling via `*apperror.AppError`.

## Invariants
- Zero builds / zero tests during authoring.
- No modifications to behavior or public API signatures.
