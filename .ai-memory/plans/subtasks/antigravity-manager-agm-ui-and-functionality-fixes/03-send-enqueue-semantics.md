# Task-03: Send/Enqueue semantics

## Status
PENDING (needs Task-01 gap report)

## Objective
Fix the behavior: Send dispatches immediately to the IDE conversation;
Enqueue appends FIFO; both work on any project.

## Requirements
- Send = immediate dispatch to the IDE on that conversation — NOT enqueue.
  Must work whether or not the workspace/project is active, and for all
  projects (not just the active one).
- Enqueue = append to the FIFO queue. Queue reads MUST use
  `ORDER BY created_at ASC, id ASC` (AGENTS.md FIFO invariant — LIFO
  `ORDER BY updated_at DESC` is forbidden).
- Trace the current Send path (`usePromptTreePromptActions.ts` → backend):
  find where Send secretly enqueues or where it requires an active project,
  and fix at the root.
- CLI/GUI parity: the same service routes must serve both.
- No silent errors (Rust: AppError/tracing; TS: error-store).

## Acceptance
- [ ] Send dispatches immediately on active AND inactive projects.
- [ ] Enqueue appends FIFO and is visibly queued.
- [ ] `cargo fmt` + `clippy` clean; `npm run build` passes.

## Constraints
- GitMap search only. Relative paths only.
