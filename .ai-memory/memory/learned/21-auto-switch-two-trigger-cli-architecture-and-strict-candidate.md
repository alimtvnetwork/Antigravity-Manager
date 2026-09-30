# Learned Memory: Auto-Switch Two-Trigger CLI Architecture and Strict Candidate

Date: 2026-09-30
Source: [spec 85](../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md), plans 85-88.

## Decisions (confirmed by the user)
1. Auto-switch selects only strict 100% candidates; the sub-100% fallback is removed.
2. The CLI (`agm switch-if-low-credit`) is the only actor that performs a switch.
3. Two triggers call that one actor: a reactive trigger (quota refresh or threshold change, asynchronous) and a small scheduled checker (`agm auto tick`, about every minute, registered through `agm auto schedule`).
4. A notification email that already exists is skipped, never re-added (normalized `LOWER(TRIM(email))`).
5. Update announcements must be backed by a verified platform asset; installers pre-check and skip releases without binaries.
6. Scripts must check exit codes, resolve paths relative to themselves, and never embed keys.

## Why it matters
Each item was a past claim of "fixed" that failed in the field; the decisions above replace ad hoc fixes with one actor, one helper, or one gate.
