---
plan: 86-update-integrity-asset-verification-and-release-gates
subtask: "007"
title: agm update and the delegate updater use the resolved version
domain: backend-rust
depends_on: 002-release-asset-resolver.md, 005-install-ps1-precheck-and-ladder.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t03--update-integrity
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/49-update-announced-without-platform-asset-rca.md
  ambiguity: none
target_files:
  - src-tauri/src/modules/delegate_updater.rs — `run_cli_update` (~L496), `run_installer_update` in `update_checker.rs` (~L814+)
  - src-tauri/src/bin/agm.rs — `cmd_update` (~L11245)
status: pending
---

# 007 — agm update and the delegate updater use the resolved version

## 1. Context
The CLI and GUI must install the same verified version the banner showed.

## 2. Target files and symbols
- src-tauri/src/modules/delegate_updater.rs — `run_cli_update` (~L496), `run_installer_update` in `update_checker.rs` (~L814+)
- src-tauri/src/bin/agm.rs — `cmd_update` (~L11245)

## 3. Steps
1. Have `agm update` call the resolver and pass the resolved version to the installer stage explicitly.
2. If nothing is installable, print the skipped list and exit non-zero without starting the delegate.
3. Add `--json` output with the same fields as `UpdateInfo`.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`.
- Relative paths only; no hardcoded repository owner, read it from one constant.
- Cross-platform: Windows, macOS, Linux behavior must match; no platform-only assumption without a fallback.
- Headless parity: no GUI dependency.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo run --bin agm -- update --check --json
```

## 7. Done When
- [ ] CLI and GUI resolve the same version.
- [ ] Non-installable state exits non-zero with reasons.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
