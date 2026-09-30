# Which repo-secrets vault folder holds the Supabase/AGM config, and what alias should this machine get?

Slug: vault-folder-and-machine-alias-for-supabase
Status: resolved
Raised: 2026-09-30
Blocking: 88 (subtasks 005, 007, 010)

## Question
Which repo-secrets vault folder holds the Supabase/AGM config, and what alias should this machine get?

## Context
The vault index lists `02-antigravity-and-event-manager/` with a `vault/` folder and `03-supabase/01-own/` with base64 credentials. The existing connector script looks for a different folder name (`02-antigravity-manager`). The user said the config is in the Antigravity Manager vault and asked to alias the current machine but gave no alias.

## Options considered
- A: `02-antigravity-and-event-manager/vault/supabase_config.json` first, then `03-supabase/01-own/supabase-credentials.json`.
- B: only `03-supabase/01-own/`.
- Alias: (1) user-provided value, (2) hostname-derived `<hostname>`, (3) keep the stored alias.

## Impact if guessed wrong
Loading the wrong file would point the machine at the wrong endpoint or register an unwanted alias. No secret is exposed either way; the alias is visible to the fleet.

## Interim provisional default
Option A with fallback to `03-supabase/01-own/`; alias taken from `-Alias`, else the existing stored alias, else the hostname. The script never invents a numeric alias.

## Resolution

Answered: 2026-09-30
Answer: The Supabase keys are imported through the AGM CLI from an explicit vault path (`--vault <path>` or `AGM_SUPABASE_VAULT`). No auto-discovery of vault folders. No machine alias: do not set, read, or print an alias anywhere. The existing PowerShell scripts in the vault repo are fine; the deliverable is a one-liner PowerShell file that calls the AGM CLI.
Applied solution: Plan 88: `agm supabase push-settings --vault <path>`, `scripts/setup-supabase.ps1` reduced to a one-liner wrapper, alias steps removed.
