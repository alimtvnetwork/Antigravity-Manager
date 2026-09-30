---
plan: 86-update-integrity-asset-verification-and-release-gates
subtask: "001"
title: Extend UpdateInfo with asset verification fields
domain: backend-rust+frontend
depends_on: none
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#update-check-result-rust-updateinfo-and-typescript-mirror
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/49-update-announced-without-platform-asset-rca.md
  ambiguity: none
target_files:
  - src-tauri/src/modules/update_checker.rs — `UpdateInfo` (~L36)
  - src/stores/use-update-store.ts — `UpdateInfo` (~L4)
  - src/components/UpdateNotification.tsx — local `UpdateInfo` (~L7; replace with the store type)
status: pending
---

# 001 — Extend UpdateInfo with asset verification fields

## 1. Context
`UpdateInfo` exists in Rust and twice in TypeScript with no asset information, so `has_update` cannot express 'newer but not installable'.

## 2. Target files and symbols
- src-tauri/src/modules/update_checker.rs — `UpdateInfo` (~L36)
- src/stores/use-update-store.ts — `UpdateInfo` (~L4)
- src/components/UpdateNotification.tsx — local `UpdateInfo` (~L7; replace with the store type)

## 3. Steps
1. Add to the Rust struct: `asset_verified: bool`, `asset_url: Option<String>`, `candidates: Vec<String>` (verified installable versions, newest first), `skipped_versions: Vec<SkippedVersion>`, `source_errors: Vec<SourceError>`; define the two small structs in their own file `update_types.rs`.
2. All new fields use `#[serde(default)]` so older clients and cached JSON still deserialize.
3. Mirror the fields in `use-update-store.ts`; make `UpdateNotification.tsx` import that type and delete its local copy.
4. No behavior change yet: set `asset_verified = true` where the code currently reports an update so existing flows keep working until subtask 002.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`.
- Relative paths only; no hardcoded repository owner, read it from one constant.
- Cross-platform: Windows, macOS, Linux behavior must match; no platform-only assumption without a fallback.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
npm run build
```

## 7. Done When
- [ ] Fields exist in Rust and TypeScript with defaults.
- [ ] Only one TypeScript `UpdateInfo` remains.
- [ ] Build and clippy pass.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
