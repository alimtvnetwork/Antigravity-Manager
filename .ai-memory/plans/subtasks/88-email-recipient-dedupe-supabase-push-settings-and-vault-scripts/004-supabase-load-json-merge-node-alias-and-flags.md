---
plan: 88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts
subtask: "004"
title: `load-json` applies `node_alias` and sync flags
domain: backend-rust
depends_on: none
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t08t11--supabase-and-vault-scripts
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/53-vault-and-supabase-scripts-silent-failures-rca.md
  ambiguity: ../../../ambiguous-questions/01-new-ambiguity/02-vault-folder-and-machine-alias-for-supabase.md
target_files:
  - src-tauri/src/bin/agm.rs — `cmd_supabase_load_json` (~L6164-6302), `set-alias` (~L5766-5774)
  - src-tauri/src/modules/supabase_sync.rs (~L27-64, ~L182-260)
status: pending
---

# 004 — `load-json` applies `node_alias` and sync flags

## 1. Context
Vault JSON can carry `node_alias` and `is_sync_enabled`, but loading only registers endpoints.

## 2. Target files and symbols
- src-tauri/src/bin/agm.rs — `cmd_supabase_load_json` (~L6164-6302), `set-alias` (~L5766-5774)
- src-tauri/src/modules/supabase_sync.rs (~L27-64, ~L182-260)

## 3. Steps
1. Merge `node_alias` and `is_sync_enabled` from the JSON into the stored config when present; never overwrite an existing alias with an empty value.
2. Accept both envelope schemas already supported (`agm/supabase-endpoints`, `agm/supabase-credentials`) and the plain config shape.
3. Print key names and counts only; never print endpoint keys.
4. Unit tests with temp files for each shape.

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
cd src-tauri && cargo test modules::supabase_sync
```

## 7. Done When
- [ ] Alias and sync flag are applied from JSON.
- [ ] No key material appears in output.

## 8. Ambiguities and interim defaults
- Ambiguity 02 default alias rule: explicit value, else stored alias, else hostname.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
