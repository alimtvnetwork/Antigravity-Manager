# Plan 85: UI Fixes: PRO Tier Badge, Priority, Selected Highlight, Focus, Stack Trace Parser

Status: pending
Raised: 2026-09-30
Problem class (one PR scope): UI correctness and display of account data
Spec: [02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md](../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md)
Subtasks: [index](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/index.md)
Related: [86](./86-update-integrity-asset-verification-and-release-gates.md), [87](./87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler.md), [88](./88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts.md); completed plan [74](../completed/74-ui-email-telegram-fixes-revisit.md) (earlier contrast/focus work)

## Context

**Request.** The full verbatim user request is in the spec under "User Request (Verbatim)". This plan covers the tasks listed below; sibling plans cover the rest.

Fixes six frontend-facing defects: stack frame parsing (T02), missing PRO badges and the backend tier gap behind them (T01), priority badge hide/edit (T06), selected-state highlight with three options for the user to pick (T05), and the Focus button (T04).



**Mandatory reading.** Before any step, read `.ai-memory/coding-guidelines.md`, `.ai-memory/strictly-avoid.md`, and `.ai-memory/folder-structure.md`. Project rules in `AGENTS.md` apply: pipeline-first fixes, headless/CLI parity, cross-platform, root-cause fixes, no release except on explicit command, PR = one problem class with individually revertable commits.

**Release policy (verbatim, law).**

Individual task runs NEVER release. No version bump, no changelog entry, no
release-notes update, no root `readme.md` version pin on a per-task basis. A run
that touches the version while sibling tasks are pending is auto-reject.

The release fires ONLY when the ENTIRE plan is finished, meaning every task and
subtask of this plan has moved out of `.ai-memory/plans/pending/` into
`.ai-memory/plans/completed/` with `Status: completed`, AND the user explicitly commands the release. At that moment, and only
then:

- Bump the MINOR version per the release ceremony in `01-prompts/17-release-management/04-release.md`.
- Add one changelog entry covering the whole plan, never a single task.
- Update release notes.
- Pin the new version in the root `readme.md`.

**Open ambiguities.**
- [04-frontend-unit-test-runner.md](../../ambiguous-questions/01-new-ambiguity/04-frontend-unit-test-runner.md)
- [05-selected-state-highlight-option-choice.md](../../ambiguous-questions/01-new-ambiguity/05-selected-state-highlight-option-choice.md)
- [07-focus-button-target-account.md](../../ambiguous-questions/01-new-ambiguity/07-focus-button-target-account.md)

## Execution Model (RULE 0C)

- One step per run. Exactly one step is executed per run. Never batch two steps into a single run.
- Every step is standalone: read the step file plus the files it cites; nothing else is assumed.
- Self-loop after verify: when a step's `## 6. Verify` passes and `## 7. Done When` is satisfied, mark the step `done` in the status table below, then re-read this file, pick the next unstarted step, and begin a fresh run.
- Concurrency ceiling: at most 2 spawned agents, each with at most 3 parallel threads.
- Pre-flight for the plan (once, before the PR, not per step): `cd src-tauri && cargo fmt -- --check`, `cd src-tauri && cargo clippy --all-targets --all-features`, `npm run build` when `src/` changed. CI compiles tests without running them.

## Status Table

| # | Step | Title | Status |
| :--- | :--- | :--- | :--- |
| `001` | [001-stack-frame-parser-regex.md](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/001-stack-frame-parser-regex.md) | Fix stack frame parsing for URL-only frames | `pending` |
| `002` | [002-stack-frame-parser-tests.md](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/002-stack-frame-parser-tests.md) | Add regression tests for the stack frame parser | `pending` |
| `003` | [003-tier-fetch-backend-and-persistence.md](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/003-tier-fetch-backend-and-persistence.md) | Fetch and persist subscription tier independent of project id | `pending` |
| `004` | [004-shared-tier-badge-component.md](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/004-shared-tier-badge-component.md) | Shared TierBadge with an explicit unknown state | `pending` |
| `005` | [005-agm-accounts-refresh-tier-cli.md](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/005-agm-accounts-refresh-tier-cli.md) | CLI command and startup backfill for tier | `pending` |
| `006` | [006-priority-badge-hide-default-and-inline-edit.md](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/006-priority-badge-hide-default-and-inline-edit.md) | Hide Priority at 50 and edit on double-click | `pending` |
| `007` | [007-selected-highlight-options-preview.md](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/007-selected-highlight-options-preview.md) | Render the three selected-state options for user review | `pending` |
| `008` | [008-apply-selected-highlight.md](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/008-apply-selected-highlight.md) | Apply the chosen selected-state design everywhere | `pending` |
| `009` | [009-focus-button-target-and-single-scroll.md](../subtasks/85-ui-tier-badge-priority-highlight-focus-and-stack-trace/009-focus-button-target-and-single-scroll.md) | Focus scrolls to the selected instance's account | `pending` |

## Order and Dependencies
001 then 002. 003 before 004 and 005. 006 and 009 are independent. 007 precedes 008; 008 is blocked until the user chooses an option (ambiguity 05).

## Acceptance
See the spec section "Acceptance Criteria"; this plan owns the items named in its summary.

## Attachments
Screenshots from the original request are not stored in the repo (an email is visible in at least one). See the spec "Attachments" section.

## Rollback
Each subtask is one commit that can be reverted independently. Configuration and database migrations introduced here are additive; the dedupe `--apply` path writes a timestamped DB backup first.
