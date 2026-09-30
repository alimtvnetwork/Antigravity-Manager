# Subtasks — Plan 85 UI, tier badge, priority, highlight, focus, stack trace

Parent plan: [85-ui-tier-badge-priority-highlight-focus-and-stack-trace.md](../../pending/85-ui-tier-badge-priority-highlight-focus-and-stack-trace.md)

Order follows data flow: contracts and scaffolds first, callers after, UI and scripts last. Execute **one step per run**; mark the row `done` in the parent plan's status table, then self-loop.

| # | File | Title | Status |
| :--- | :--- | :--- | :--- |
| `001` | [001-stack-frame-parser-regex.md](./001-stack-frame-parser-regex.md) | Fix stack frame parsing for URL-only frames | `pending` |
| `002` | [002-stack-frame-parser-tests.md](./002-stack-frame-parser-tests.md) | Add regression tests for the stack frame parser | `pending` |
| `003` | [003-tier-fetch-backend-and-persistence.md](./003-tier-fetch-backend-and-persistence.md) | Fetch and persist subscription tier independent of project id | `pending` |
| `004` | [004-shared-tier-badge-component.md](./004-shared-tier-badge-component.md) | Shared TierBadge with an explicit unknown state | `pending` |
| `005` | [005-agm-accounts-refresh-tier-cli.md](./005-agm-accounts-refresh-tier-cli.md) | CLI command and startup backfill for tier | `pending` |
| `006` | [006-priority-badge-hide-default-and-inline-edit.md](./006-priority-badge-hide-default-and-inline-edit.md) | Hide Priority at 50 and edit on double-click | `pending` |
| `007` | [007-selected-highlight-options-preview.md](./007-selected-highlight-options-preview.md) | Render the three selected-state options for user review | `pending` |
| `008` | [008-apply-selected-highlight.md](./008-apply-selected-highlight.md) | Apply the chosen selected-state design everywhere | `pending` |
| `009` | [009-focus-button-target-and-single-scroll.md](./009-focus-button-target-and-single-scroll.md) | Focus scrolls to the selected instance's account | `pending` |

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
