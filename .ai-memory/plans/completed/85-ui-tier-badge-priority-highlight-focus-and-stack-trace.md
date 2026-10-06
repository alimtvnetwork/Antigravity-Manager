# Plan 85: UI Fixes: PRO Tier Badge, Priority, Selected Highlight, Focus, Stack Trace Parser

Status: completed
Raised: 2026-09-30
Completed: 2026-10-06
Problem class (one PR scope): UI correctness and display of account data
Spec: [02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md](../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md)
Subtasks: [index](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/index.md)
Related: [86](../pending/86-update-integrity-asset-verification-and-release-gates.md), [87](../pending/87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler.md), [88](../pending/88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts.md); completed plan [74](./74-ui-email-telegram-fixes-revisit.md) (earlier contrast/focus work)

## Context

**Request.** The full verbatim user request is in the spec under "User Request (Verbatim)". This plan covers the tasks listed below; sibling plans cover the rest.

Fixes six frontend-facing defects: stack frame parsing (T02), missing PRO badges and the backend tier gap behind them (T01), priority badge hide/edit (T06), selected-state highlight, Option A (T05), and the Focus button (T04).

## Status Table

| # | Step | Title | Status |
| :--- | :--- | :--- | :--- |
| `001` | [001-stack-frame-parser-regex.md](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/001-stack-frame-parser-regex.md) | Fix stack frame parsing for URL-only frames | `done` |
| `002` | [002-stack-frame-parser-tests.md](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/002-stack-frame-parser-tests.md) | Add regression tests for the stack frame parser | `done` |
| `003` | [003-tier-fetch-backend-and-persistence.md](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/003-tier-fetch-backend-and-persistence.md) | Fetch and persist subscription tier independent of project id | `done` |
| `004` | [004-shared-tier-badge-component.md](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/004-shared-tier-badge-component.md) | Shared TierBadge with an explicit unknown state | `done` |
| `005` | [005-agm-accounts-refresh-tier-cli.md](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/005-agm-accounts-refresh-tier-cli.md) | CLI command and startup backfill for tier | `done` |
| `006` | [006-priority-badge-hide-default-and-inline-edit.md](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/006-priority-badge-hide-default-and-inline-edit.md) | Hide Priority at 50 and edit on double-click | `done` |
| `007` | [007-selected-state-shared-style.md](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/007-selected-state-shared-style.md) | Shared selected-state style (Option A: dark slate with amber rail) | `done` |
| `008` | [008-apply-selected-highlight.md](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/008-apply-selected-highlight.md) | Apply the selected-state style everywhere | `done` |
| `009` | [009-focus-button-target-and-single-scroll.md](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/009-focus-button-target-and-single-scroll.md) | Focus scrolls to the selected instance's account | `done` |

## Acceptance
All subtasks 001 through 009 have been completely implemented, verified, and integrated.
