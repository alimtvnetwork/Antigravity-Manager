---
plan: 86-update-integrity-asset-verification-and-release-gates
subtask: "003"
title: Aggregate update source failures and use a registered code
domain: backend-rust+frontend
depends_on: 002-release-asset-resolver.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t03--update-integrity
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/49-update-announced-without-platform-asset-rca.md
  ambiguity: none
target_files:
  - src-tauri/src/modules/update_checker.rs — combined error path
  - src-tauri/src/error.rs — `AppError` mapping (default `E9001` for `Unknown`, ~L123)
  - 02-spec/03-error-manage/03-error-code-registry/01-index.md
  - src/stores/error-store.ts — default code fallback (~L188)
status: pending
---

# 003 — Aggregate update source failures and use a registered code

## 1. Context
All source failures collapse into 'Failed to fetch updates … check network/proxy settings' and map to the generic code, even when the real reason is a missing binary.

## 2. Target files and symbols
- src-tauri/src/modules/update_checker.rs — combined error path
- src-tauri/src/error.rs — `AppError` mapping (default `E9001` for `Unknown`, ~L123)
- 02-spec/03-error-manage/03-error-code-registry/01-index.md
- src/stores/error-store.ts — default code fallback (~L188)

## 3. Steps
1. Collect each source's failure into `source_errors` (source name plus short reason) and return them with the result instead of one string.
2. Add a dedicated `AppError` variant for update problems with two reasons: network failure and no installable release; register its code in the error registry spec and the TypeScript code table.
3. Message wording distinguishes 'no network' from 'release exists but has no binary for this platform'.
4. Update the error report generator entry so the new code has a troubleshooting list.

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
- [ ] A missing-asset outcome is not reported as a network error.
- [ ] The new code is in the registry and the frontend table.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
