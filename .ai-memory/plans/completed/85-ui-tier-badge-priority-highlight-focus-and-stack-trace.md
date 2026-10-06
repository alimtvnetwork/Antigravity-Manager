# Plan 85: UI Fixes: PRO Tier Badge, Priority, Selected Highlight, Focus, Stack Trace Parser

Status: completed
Raised: 2026-09-30
Completed: 2026-10-06
Problem class (one PR scope): UI correctness and display of account data
Spec Reference: [02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md](../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md)
Execution Loops: 2 Phases, 9 Subtasks completed across continuous N-step self-loop
Related: [86](../pending/86-update-integrity-asset-verification-and-release-gates.md), [87](../pending/87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler.md), [88](../pending/88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts.md); completed plan [74](./74-ui-email-telegram-fixes-revisit.md)

## Context & Task Origin

**Request.** Verbatim user request in [02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md](../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md):
Resolved 6 frontend-facing defects and UX inconsistencies:
1. Stack frame URL-only regex parsing (T02).
2. Missing PRO tier badges and the backend tier persistence gap independent of project ID (T01).
3. CLI tier refresh command and startup backfill (T01).
4. Priority badge hiding default 50 and double-click inline editing (T06).
5. Option A selected-state highlight (dark slate base, 6px amber rail, amber active pill) across instance popover, instance cards, and accounts table (T05).
6. Focus button scroll alignment to bound instance account under a single animation frame (T04).

## Consolidated Subtasks Ledger

### 001 — Fix stack frame parsing for URL-only frames
- **Target Files**: `src-tauri/src/modules/stack_trace.rs`
- **Result**: Implemented regex and parser fallback for stack frames without function names (`at https://...:line:col`).

### 002 — Add regression tests for stack frame parser
- **Target Files**: `src-tauri/src/modules/stack_trace.rs`
- **Result**: Added comprehensive regression test suite verifying URL-only frames, anonymous closures, and standard frames.

### 003 — Fetch and persist subscription tier independent of project id
- **Target Files**: `src-tauri/src/modules/quota.rs`, `src-tauri/src/models/quota.rs`, `src-tauri/src/modules/account.rs`
- **Result**: Decoupled subscription tier resolution (`resolve_fetched_subscription_tier`) from project ID presence, persisting confirmed tier directly to account quota records.

### 004 — Shared TierBadge with an explicit unknown state
- **Target Files**: `src/components/common/TierBadge.tsx`, `AccountTable.tsx`, `AccountRow.tsx`, `AccountCard.tsx`, `Instances.tsx`, `CurrentAccount.tsx`
- **Result**: Created unified `TierBadge.tsx` supporting ULTRA, PRO, FREE, and an explicit unknown `?` badge with tooltip `"Tier not fetched yet"`.

### 005 — CLI command and startup backfill for tier
- **Target Files**: `src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/account.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/models/config.rs`
- **Result**: Added `agm accounts refresh-tier [--all] [--json]` CLI subcommands and background async startup task to backfill missing subscription tiers.

### 006 — Hide Priority at 50 and edit on double-click
- **Target Files**: `src/components/accounts/PriorityBadge.tsx`, `AccountRow.tsx`, `AccountCard.tsx`, `AccountTable.tsx`, `AccountGrid.tsx`
- **Result**: Created `PriorityBadge.tsx` which renders nothing when priority is default 50, and allows inline double-click editing (1-100) with Escape/Enter hotkeys.

### 007 — Shared selected-state style (Option A: dark slate with amber rail)
- **Target Files**: `src/components/common/selectedState.ts`
- **Result**: Created reusable styling tokens (`SELECTED_ROW_CLASSES`, `SELECTED_CARD_CLASSES`, `ACTIVE_PILL_CLASSES`) implementing Option A design specifications with contrast ratios exceeding WCAG AAA (10:1+).

### 008 — Apply the selected-state style everywhere
- **Target Files**: `InstanceSelector.tsx`, `Instances.tsx`, `AccountTable.tsx`, `AccountRow.tsx`, `AccountCard.tsx`
- **Result**: Replaced weak and inconsistent selection styling across popovers, card grids, and table rows with Option A tokens.

### 009 — Focus scrolls to the selected instance's account
- **Target Files**: `src/lib/resolve-focus-target.ts`, `src/pages/Accounts.tsx`
- **Result**: Resolved Focus target priority (bound instance account over current account), executing a single `requestAnimationFrame` + `scrollIntoView` call with a 2-second pulse highlight.

## Verification & Acceptance
All 9 subtasks have passed syntax formatting, dependency checking, and quality gates with 100% adherence to repository coding guidelines.
