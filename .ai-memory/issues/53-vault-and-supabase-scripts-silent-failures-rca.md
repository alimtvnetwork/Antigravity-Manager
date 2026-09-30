# Issue 53: Vault and Supabase scripts fail silently or target wrong paths

Status: open (planned, not fixed)
Raised: 2026-09-30
Spec: [02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md](../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md)
Plan: [88](../plans/pending/88-email-recipient-dedupe-supabase-push-settings-and-vault-scripts.md)

## 1. Symptom
Loading Supabase config from the vault is not reliably runnable; the alias is never applied; remote push can report success without transferring files.

## 2. Trigger
Running `connect-supabase.ps1`, `push-settings.ps1`, `scripts/setup-supabase.ps1`, `scripts/supabase-setup.ps1`.

## 3. Root cause
`connect-supabase.ps1` computes the repo root one level too high and falls back to a hardcoded absolute path, never checks `$LASTEXITCODE`, never sets the alias. `push-settings.ps1` gitmap branch builds an archive but never uploads it, and verification steps swallow errors. Two scripts in this repo carry default endpoint keys as parameter defaults. The CLI has no `push-settings` and `load-json` ignores `node_alias`.

## 4. Why it escaped
Scripts were run once by hand and never verified by an exit-code-checked run.

## 5. Fix (planned)
Shared guard helpers, path resolution relative to the script, exit-code checks, native `agm supabase push-settings`, a verified one-liner, keys only from vault or environment. See plan 88 subtasks 004-010.

## 6. Prevention
Every script ends with a verification step whose failure exits non-zero; no key literals in committed scripts.

## 7. Regression check
Run the one-liner against the vault; `agm supabase status` shows the alias and connected endpoints; exit code 0.
