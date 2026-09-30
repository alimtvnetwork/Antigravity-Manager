---
plan: 86-update-integrity-asset-verification-and-release-gates
subtask: "002"
title: Resolve the newest release that has a platform asset
domain: backend-rust
depends_on: 001-update-info-contract.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t03--update-integrity
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/49-update-announced-without-platform-asset-rca.md
  ambiguity: none
target_files:
  - src-tauri/src/modules/update_checker.rs — `check_for_updates_internal` (~L172), updater.json path (~L249-334, hardcoded owner at ~L313-316), GitHub API path (~L370-452), static fallback (~L460-520)
  - src-tauri/src/modules/update_asset_resolver.rs (new)
status: pending
---

# 002 — Resolve the newest release that has a platform asset

## 1. Context
`has_update` is decided by version comparison only. Nothing checks the platform asset, and one URL contains a hardcoded non-origin owner.

## 2. Target files and symbols
- src-tauri/src/modules/update_checker.rs — `check_for_updates_internal` (~L172), updater.json path (~L249-334, hardcoded owner at ~L313-316), GitHub API path (~L370-452), static fallback (~L460-520)
- src-tauri/src/modules/update_asset_resolver.rs (new)

## 3. Steps
1. Create `update_asset_resolver.rs` with `resolve_installable_release(releases, platform) -> Resolution { selected, skipped }`: walk releases newest to oldest, skip drafts and prereleases unless the channel allows them, select the first whose asset list contains a file for this OS/arch (suffix/pattern table in one place).
2. For the updater.json and static sources, verify the asset URL with an HTTP HEAD (timeout 5 s, follow redirects) before accepting it; a non-2xx adds the version to `skipped_versions` with reason.
3. Replace the hardcoded owner with the repository constant used elsewhere in the module.
4. `has_update` is true only when a verified candidate is newer than the current version; `asset_url` is that candidate's URL.
5. Fixture unit tests: newest release without asset is skipped and the previous one is selected; every release lacking assets yields `has_update = false` with reasons; prerelease ignored on stable.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`.
- Relative paths only; no hardcoded repository owner, read it from one constant.
- Cross-platform: Windows, macOS, Linux behavior must match; no platform-only assumption without a fallback.
- Fail closed: a network error during verification records a `source_error`; it never marks an asset verified.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo test modules::update_asset_resolver
cd src-tauri && cargo test modules::update_checker
```

## 7. Done When
- [ ] A release with missing asset is never reported as an update.
- [ ] Tests above pass.
- [ ] No hardcoded foreign owner remains.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
