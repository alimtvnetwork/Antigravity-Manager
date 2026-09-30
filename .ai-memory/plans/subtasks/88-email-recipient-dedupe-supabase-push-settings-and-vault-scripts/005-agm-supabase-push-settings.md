---
plan: 88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts
subtask: "005"
title: `agm supabase push-settings`
domain: backend-rust
depends_on: 004-supabase-load-json-merge-node-alias-and-flags.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#cli-surface-added-or-changed
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/53-vault-and-supabase-scripts-silent-failures-rca.md
  ambiguity: ../../../ambiguous-questions/01-new-ambiguity/02-vault-folder-and-machine-alias-for-supabase.md
target_files:
  - src-tauri/src/bin/agm.rs — `cmd_supabase` router (~L5707-5783), help text `print_supabase_help`
  - src-tauri/src/modules/supabase_push.rs (new)
status: pending
---

# 005 — `agm supabase push-settings`

## 1. Context
The user wants a 'push settings' command: load the vault config, alias this machine, sync, and verify, in one step.

## 2. Target files and symbols
- src-tauri/src/bin/agm.rs — `cmd_supabase` router (~L5707-5783), help text `print_supabase_help`
- src-tauri/src/modules/supabase_push.rs (new)

## 3. Steps
1. Implement `push_settings(vault_dir, alias) -> PushReport` that: locates the config (ambiguity 02 order), loads it, applies the alias, runs sync, runs the connectivity test, returns each step's result.
2. Wire `agm supabase push-settings [--vault <dir>] [--alias <name>] [--json]`; exit non-zero if any step fails; `--json` uses the unified envelope.
3. Add environment overrides `AGM_SUPABASE_VAULT` and `AGM_SUPABASE_ALIAS` (headless parity).
4. Add to `agm supabase help` and to `agm which-format` schema list if a new envelope is emitted.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`.
- Relative paths only. The secrets repo is a sibling repository at `../repo-secrets/`; it has its own git history and is committed separately.
- NEVER print, log, echo, or commit a secret, token, key, password, or email. Print key names and counts only.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo run --bin agm -- supabase push-settings --help
```

## 7. Done When
- [ ] One command performs load, alias, sync, test.
- [ ] Failure of any step exits non-zero.

## 8. Ambiguities and interim defaults
- Ambiguity 02 defaults apply.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
