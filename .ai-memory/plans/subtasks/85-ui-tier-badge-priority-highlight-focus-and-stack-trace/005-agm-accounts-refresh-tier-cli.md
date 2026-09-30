---
plan: 85-ui-tier-badge-priority-highlight-focus-and-stack-trace
subtask: "005"
title: CLI command and startup backfill for tier
domain: backend-rust
depends_on: 003-tier-fetch-backend-and-persistence.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#cli-surface-added-or-changed
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/47-pro-badge-missing-when-tier-not-fetched-rca.md
  ambiguity: none
target_files:
  - src-tauri/src/bin/agm.rs — accounts subcommand router and help text
  - src-tauri/src/modules/account.rs — a `refresh_missing_tiers` function reused by CLI and startup
status: pending
---

# 005 — CLI command and startup backfill for tier

## 1. Context
Existing accounts already have no tier; the UI fix alone leaves them unknown until their next refresh. Headless users need the same backfill.

## 2. Target files and symbols
- src-tauri/src/bin/agm.rs — accounts subcommand router and help text
- src-tauri/src/modules/account.rs — a `refresh_missing_tiers` function reused by CLI and startup

## 3. Steps
1. Add `refresh_missing_tiers(limit_concurrency)` in `account.rs` that fetches tiers only for accounts with no tier, bounded concurrency, returns counts (`updated`, `still_unknown`, `failed`).
2. Add `agm accounts refresh-tier [--all] [--json]` calling it; `--all` also refreshes known tiers.
3. Call the function once, in the background, after app startup (non-blocking) and expose the same behavior through the existing config switch style if a toggle is needed.
4. Add the command to `agm` help using the repo's help conventions.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function size and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`; each new component or helper lives in its own file.
- Relative paths only in code, scripts, and docs.
- Reuse existing UI conventions (pill/badge toggles, existing dialogs); no new unrelated controls.
- Headless parity: command works without the GUI; `--json` uses the unified envelope.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo run --bin agm -- accounts refresh-tier --json
```

## 7. Done When
- [ ] The command prints counts and exits 0 when no failure occurs.
- [ ] Startup backfill is non-blocking.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
