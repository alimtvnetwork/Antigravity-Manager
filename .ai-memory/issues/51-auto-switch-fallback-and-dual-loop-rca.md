# Issue 51: Auto-switch does not behave as specified

Status: open (planned, not fixed)
Raised: 2026-09-30
Spec: [02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md](../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md)
Plan: [87](../plans/pending/87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler.md)

## 1. Symptom
Auto-switch does not trigger a switch reliably and behaves differently in the desktop app and headless.

## 2. Trigger
Quota at or below the threshold with no 100% candidate, or GUI vs daemon execution.

## 3. Root cause
`auto_switcher.rs` falls back to sub-100% candidates instead of requiring a strict candidate. `BackgroundTaskRunner.tsx` runs a separate React loop that returns early outside Tauri, duplicating the Rust daemon. `cooldown_seconds` exists in config but is not honored. Target model is resolved differently in different paths. There is no scheduled checker usable without the GUI.

## 4. Why it escaped
Unit tests covered scoring; nothing asserted the decision end to end or parity between loops.

## 5. Fix (planned)
Strict 100% candidate; one CLI actor; two triggers (reactive and a 1-minute scheduled tick); `agm auto tick --explain`; honor cooldown; same model resolution everywhere. See plan 87 subtasks 007-009.

## 6. Prevention
Decision function is pure and unit tested; both triggers call the same actor.

## 7. Regression check
`cargo test modules::auto_switcher` and `agm auto tick --explain --json` on a fixture profile set.
