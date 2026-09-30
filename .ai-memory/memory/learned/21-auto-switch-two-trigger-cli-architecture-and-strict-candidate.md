# Learned Memory: Auto-Switch Two-Trigger CLI Architecture and Strict Candidate

Date: 2026-09-30
Source: [spec 85](../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md), plans 85-88.

## Decisions (confirmed by the user)
1. Auto-switch selects only strict 100% candidates; the sub-100% fallback is removed.
2. The CLI (`agm switch-if-low-credit`) is the only actor that performs a switch.
3. Two triggers call that one actor: a reactive trigger (quota refresh or threshold change, asynchronous) and a small scheduled checker (`agm auto tick`, about every minute, registered through `agm auto schedule`).
4. A notification email that already exists is skipped, never re-added (normalized `LOWER(TRIM(email))`).
5. Update announcements must be backed by a verified platform asset; installers pre-check and skip releases without binaries.
6. Scripts written in this work check exit codes and never embed keys. Supabase keys are imported through the AGM CLI (`agm supabase push-settings --vault <path>`) from an explicit path; no machine alias. The existing vault PowerShell scripts are fine and stay untouched.
7. `npm run test` is the frontend test runner (tsx via npx, no new dependency).
8. Selected-state UI: Option A (dark slate, amber left rail, amber ACTIVE pill).

## Working style (user feedback, 2026-09-30)
- Ask questions first, at the start of a run, numbered 1, 2, 3, with the options written inline in the message. Never point at an option that lives in another file.
- After the questions, complete the task end to end: build, run tests, run the live E2E verification. Do not stop mid-run to ask.
- When the user gives permission to commit and push, do it without waiting for a second confirmation.

## Why it matters
Each item was a past claim of "fixed" that failed in the field; the decisions above replace ad hoc fixes with one actor, one helper, or one gate.
