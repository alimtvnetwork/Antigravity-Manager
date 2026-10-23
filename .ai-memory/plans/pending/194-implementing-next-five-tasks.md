# Parent Plan — 194 Implementing Next Five Tasks

Status: IN_PROGRESS (TAKEN)
Phase: 1 (claimed & assigned to subagents)
Owner: Agent-194-Lead
Specs: `02-spec/21-app/194-implementing-next-five-tasks/spec.md`
E2E Master Plan: `02-spec/21-app/194-implementing-next-five-tasks/e2e_master_plan.md`
Subtasks: `.ai-memory/plans/subtasks/194-implementing-next-five-tasks/`
Backlog Delta: 70 steps -> 65 steps remaining (-5 steps claimed)

## 1. Selected Five Tasks (Claimed & In-Progress)

| Task # | Slug | Module Target | Status | Implementation Details |
|---|---|---|---|---|
| 01 | `fifo-queue-ordering` | `src-tauri/src/modules/release_gate/fifo_queue_ordering_guard.rs` | `IN_PROGRESS` | Enforce `ORDER BY created_at ASC, id ASC` across SQLite dispatch queries, strictly forbidding LIFO. |
| 02 | `capsule-dropdown-styling` | `src-tauri/src/modules/release_gate/capsule_dropdown_styling_guard.rs` | `IN_PROGRESS` | Enforce contiguous `rounded-full` capsules and borderless inline `<select>` elements. |
| 03 | `origin-tier-hierarchy` | `src-tauri/src/modules/release_gate/origin_tier_hierarchy_guard.rs` | `IN_PROGRESS` | Enforce 4-tier origin badges (`USER_PROMPT`, `SUBAGENT_INSTRUCTION`, `SYSTEM_MESSAGE`, `TOOL_OUTPUT`) and tree glyphs. |
| 04 | `truncation-callout-parser` | `src-tauri/src/modules/release_gate/truncation_callout_parser_guard.rs` | `IN_PROGRESS` | Parse `<truncated N bytes>` and render interactive dark-glass omission banners. |
| 05 | `no-silent-errors-linter` | `src-tauri/src/modules/release_gate/no_silent_errors_linter.rs` | `IN_PROGRESS` | Enforce error propagation, `record_ignored` + `// Justification:` comments, and TypeScript error store routing. |

## 2. Invariants & Rules
- **Zero-Build & Zero-Test Invariant**: Strictly DO NOT build code or run tests during this phase.
- **Repository-Relative Paths**: Exclusively relative paths.
- **Anti-Collision Commit**: Committed immediately as taken to prevent multi-agent collisions.
