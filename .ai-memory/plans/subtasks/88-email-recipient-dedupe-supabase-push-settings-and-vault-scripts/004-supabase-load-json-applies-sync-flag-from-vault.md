---
plan: 88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts
subtask: "004"
title: `load-json` imports keys and applies the sync flag from the vault file
domain: backend-rust
depends_on: none
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t08t11--supabase-and-vault-scripts
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/53-vault-and-supabase-scripts-silent-failures-rca.md
  ambiguity: ../../../ambiguous-questions/02-ambiguity-resolved/02-vault-folder-and-machine-alias-for-supabase.md
target_files:
  - src-tauri/src/bin/agm.rs — `cmd_supabase_load_json` (~L6164-6302)
  - src-tauri/src/modules/supabase_sync.rs (~L27-64, ~L182-260)
status: pending
---

# 004 — `load-json` imports keys and applies the sync flag from the vault file

## 1. Context
Decision (ambiguity 02): Supabase endpoint keys are imported through the AGM CLI from an explicitly given vault file or folder. No machine alias is set.

## 2. Target files and symbols
- src-tauri/src/bin/agm.rs — `cmd_supabase_load_json` (~L6164-6302)
- src-tauri/src/modules/supabase_sync.rs (~L27-64, ~L182-260)

## 3. Steps
1. Confirm `load-json` accepts both envelope schemas (`agm/supabase-endpoints`, `agm/supabase-credentials`) and the plain config shape; add the missing shape if any.
2. Apply `is_sync_enabled` from the JSON when present. Ignore `node_alias` in the file (do not read, write, or print it).
3. Print endpoint ids and counts only; never print keys.
4. Unit tests with temp files for each shape.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`.
- Relative paths only. The secrets repo is a sibling repository at `../repo-secrets/`; it is read, not edited, by this plan.
- NEVER print, log, echo, or commit a secret, token, key, password, or email. Print key names and counts only.
- No machine alias handling anywhere (resolved ambiguity 02).

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
- [ ] All three shapes import.
- [ ] No key material appears in output.
- [ ] `node_alias` in a file has no effect.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
