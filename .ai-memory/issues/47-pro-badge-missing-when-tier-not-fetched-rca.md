# Issue 47: PRO accounts render no PRO badge

Status: open (planned, not fixed)
Raised: 2026-09-30
Spec: [02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md](../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md)
Plan: [85](../plans/pending/85-ui-tier-badge-priority-highlight-focus-and-stack-trace.md)

## 1. Symptom
Accounts that are Pro in the Antigravity service show no badge in the accounts table, cards, instance popover, and current-account header.

## 2. Trigger
Any quota refresh for an account that already has a cached `project_id`.

## 3. Root cause
`modules/quota.rs` skips `loadCodeAssist` on a cached `project_id` and returns `subscription_tier = None`. `models/account.rs::update_quota` keeps a tier only when a previous one existed. The UI renders the badge only when the tier is set, so an unknown tier looks the same as a free tier.

## 4. Why it escaped
No test covered the cached-project path; the UI has no 'unknown' state, so the gap was invisible.

## 5. Fix (planned)
Fetch tier independently of project id; persist and normalize it; backfill existing accounts through `agm accounts refresh-tier` and one startup pass; add a shared `TierBadge` with an explicit unknown state. See plan 85 subtasks 003-005.

## 6. Prevention
Shared badge component used everywhere; Rust test asserts that a refresh with a cached project id still yields a tier.

## 7. Regression check
`cd src-tauri && cargo test models::quota` and `cargo test modules::quota`.
