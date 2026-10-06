---
plan: 85-ui-tier-badge-priority-highlight-focus-and-stack-trace
subtask: "003"
title: Fetch and persist subscription tier independent of project id
domain: backend-rust
depends_on: none
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t01--pro-badge
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/47-pro-badge-missing-when-tier-not-fetched-rca.md
  ambiguity: none
target_files:
  - src-tauri/src/modules/quota.rs — the branch that skips `fetch_project_id` when `cached_project_id` exists (~L293-297)
  - src-tauri/src/models/account.rs — `update_quota` (~L140-184)
  - src-tauri/src/models/quota.rs — `ensure_subscription_tier`, `normalize_subscription_tier`, `resolve_subscription_tier`, `tier_priority`
status: done
---

# 003 — Fetch and persist subscription tier independent of project id

## 1. Context
When a `project_id` is cached the quota path returns `(Some(pid), None)`; the tier is then never populated, and `update_quota` only preserves a tier that already existed.

## 2. Target files and symbols
- src-tauri/src/modules/quota.rs — the branch that skips `fetch_project_id` when `cached_project_id` exists (~L293-297)
- src-tauri/src/models/account.rs — `update_quota` (~L140-184)
- src-tauri/src/models/quota.rs — `ensure_subscription_tier`, `normalize_subscription_tier`, `resolve_subscription_tier`, `tier_priority`

## 3. Steps
1. In `quota.rs`, when the tier is unknown for the account, call the tier source even if `project_id` is cached (separate the project fetch from the tier fetch; reuse the same `loadCodeAssist` response when both are needed).
2. When the tier is already known and fresh (define a TTL constant, e.g. 24h, in one place), keep skipping the call to protect rate limits.
3. In `update_quota`, persist a newly fetched tier over `None`/free, and never downgrade a known paid tier to `None` on a transient failure.
4. Normalize through `normalize_subscription_tier` before storing so `PRO`, `pro`, and `Google AI Pro` end up as one value.
5. Add Rust unit tests: cached project id plus unknown tier fetches a tier; known tier within TTL does not refetch; transient failure keeps the old tier.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function size and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`; each new component or helper lives in its own file.
- Relative paths only in code, scripts, and docs.
- Reuse existing UI conventions (pill/badge toggles, existing dialogs); no new unrelated controls.
- Backend-only step; no UI change. Do not log tokens or emails.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo test models::quota
cd src-tauri && cargo test modules::quota
```

## 7. Done When
- [x] A refresh with a cached project id yields a tier.
- [x] Tests above pass and fmt/clippy are clean.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
