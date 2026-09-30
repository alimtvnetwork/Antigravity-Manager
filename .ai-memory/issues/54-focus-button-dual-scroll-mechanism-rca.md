# Issue 54: Focus button does not scroll to the account

Status: open (planned, not fixed)
Raised: 2026-09-30
Spec: [02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md](../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md)
Plan: [85](../plans/pending/85-ui-tier-badge-priority-highlight-focus-and-stack-trace.md)

## 1. Symptom
Clicking Focus does not bring the intended account row into view.

## 2. Trigger
Focus pressed while the account is on another page or hidden by a filter, or when the selected instance is bound to a different account than the global current account.

## 3. Root cause
`Accounts.tsx::handleFocusActiveAccount` targets the global `currentAccount`, and a separate state-driven effect also scrolls; the two race and neither clears the filter or navigates the page first.

## 4. Why it escaped
Plan 74 added scroll polling but tested only the same-page case.

## 5. Fix (planned)
Resolve the target (selected instance's bound account, fallback current), clear filters, navigate to its page, scroll once, highlight briefly. See plan 85 subtask 009.

## 6. Prevention
Single scroll mechanism.

## 7. Regression check
Manual checklist plus a unit test for the target resolver.
