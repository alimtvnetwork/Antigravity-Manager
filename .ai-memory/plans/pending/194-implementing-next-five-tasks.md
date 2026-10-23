# Parent Plan — 194 Implementing Next Five Tasks

Status: IN PROGRESS (TAKEN)
Phase: 1 (claim committed & spec authored)
Owner: Agent-194-Lead
Specs: `02-spec/21-app/194-implementing-next-five-tasks/spec.md`
Subtasks: `.ai-memory/plans/subtasks/194-implementing-next-five-tasks/`
Backlog Delta: 70 steps -> 65 steps remaining (-5 steps claimed)

## 1. Selected Five Tasks (Claimed & In-Progress)

| Task # | Slug | Module Target | Status | Implementation Details |
|---|---|---|---|---|
| 01 | `fifo-queue-ordering` | `src-tauri/src/modules/release_gate/fifo_queue_ordering_guard.rs` | `IN PROGRESS (TAKEN)` | Enforce strict FIFO `ORDER BY created_at ASC, id ASC` across SQLite query dispatchers; reject LIFO ordering. |
| 02 | `capsule-dropdown-styling` | `src-tauri/src/modules/release_gate/capsule_dropdown_styling_guard.rs` | `IN PROGRESS (TAKEN)` | Validate segmented capsule window controls and transparent borderless inline select dropdowns. |
| 03 | `origin-tier-hierarchy` | `src-tauri/src/modules/release_gate/origin_tier_hierarchy_guard.rs` | `IN PROGRESS (TAKEN)` | Enforce 4 origin tiers (USER_PROMPT, SUBAGENT_INSTRUCTION, SYSTEM_MESSAGE, TOOL_OUTPUT) and branch glyph hierarchy. |
| 04 | `truncation-callout-parser` | `src-tauri/src/modules/release_gate/truncation_callout_parser_guard.rs` | `IN PROGRESS (TAKEN)` | Transform raw `<truncated N bytes/lines>` markers into structured omission banners with expansion metadata. |
| 05 | `no-silent-errors-linter` | `src-tauri/src/modules/release_gate/no_silent_errors_linter.rs` | `IN PROGRESS (TAKEN)` | Detect unrouted errors, bare `let _ =`, bare `.ok()`, and bare `catch {}`; enforce `record_ignored` justification comments. |

## 2. Invariants & Rules Upheld
- **Zero-Build & Zero-Test Invariant**: Strictly preserved; zero compilers invoked.
- **Repository-Relative Paths**: 100% compliant.
- **Anti-Collision**: Initial `IN PROGRESS (TAKEN)` commitment registered and synchronized so parallel AI agents do not duplicate.
