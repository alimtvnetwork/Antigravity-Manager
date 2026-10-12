# Subtask 004: Repo Secrets Ingestion & Single-Liner Import Provisioning

- **Code:** `Task-04`
- **Parent Plan:** `.ai-memory/plans/superbase-information-removal.md`
- **Status:** `COMPLETED`
- **Owner:** Worker 02
- **Completed At:** 2026-10-13T01:28:00+08:00

## Objectives
1. Ensure actual Supabase credentials are documented for `repo-secrets` vault storage. (Completed - verified at `../repo-secrets/02-antigravity-manager/vault/supabase_config.json`)
2. Author standalone helper `scripts/import-supabase-secrets.ps1` that pulls from local vault/environment or takes arguments without committing them. (Completed)
3. Provide single-liner import commands for operators to bootstrap local `supabase_config.json`. (Completed)

## Implementation Summary
- **Created Artifact:** `scripts/import-supabase-secrets.ps1`
- **Parameter Support:** `-VaultFile`, `-RootUrl`, `-RootKey`, `-SecondaryUrl`, `-SecondaryKey`, `-NodeAlias`, `-ConfigFile`, `-SkipTest`, `-AutoDiscover`.
- **4-Tier Precedence Ladder:**
  1. Tier 1: Explicit CLI parameters (`-RootUrl`, `-RootKey`, `-SecondaryUrl`, `-SecondaryKey`)
  2. Tier 2: Explicit configuration/vault file (`-ConfigFile`, `-VaultFile`)
  3. Tier 3: Sibling directory auto-discovery (`repo-secrets/02-antigravity-manager/vault/supabase_config.json` and relative variants)
  4. Tier 4: Environment variables (`$env:AGM_SUPABASE_ROOT_URL`, `$env:AGM_SUPABASE_ROOT_KEY`, etc.)
- **URL Normalization:** Enforces stripping of redundant `/rest/v1` and trailing slashes so stored URLs are clean root endpoints (e.g. `https://<project-ref>.supabase.co`).
- **Target Destinations:**
  - Linux/macOS: `~/.config/antigravity-manager/supabase_config.json`
  - Windows: `%APPDATA%\antigravity-manager\supabase_config.json`
  - AGM Tools: `~/.antigravity_tools/supabase_config.json` (if present)
- **Operator One-Liners Provided:**
  - Auto-discovery: `pwsh -NoProfile -ExecutionPolicy Bypass -File ./scripts/import-supabase-secrets.ps1 -AutoDiscover`
  - Sibling vault: `pwsh -NoProfile -ExecutionPolicy Bypass -Command "& { [string]$v = if (Test-Path '../repo-secrets/02-antigravity-manager/vault/supabase_config.json') { '../repo-secrets/02-antigravity-manager/vault/supabase_config.json' } elseif (Test-Path 'd:\work\repo-secrets\02-antigravity-manager\vault\supabase_config.json') { 'd:\work\repo-secrets\02-antigravity-manager\vault\supabase_config.json' } else { '' }; if ($v) { & ./scripts/import-supabase-secrets.ps1 -VaultFile $v } else { Write-Error 'Vault not found' } }"`
  - Environment variables: `pwsh -NoProfile -ExecutionPolicy Bypass -Command "& ./scripts/import-supabase-secrets.ps1 -RootUrl $env:AGM_SUPABASE_ROOT_URL -RootKey $env:AGM_SUPABASE_ROOT_KEY -SecondaryUrl $env:AGM_SUPABASE_SECONDARY_URL -SecondaryKey $env:AGM_SUPABASE_SECONDARY_KEY"`

## Verification
- Verified Tier 1 (explicit params) via PowerShell `pwsh` runner: PASS
- Verified Tier 2 (explicit vault file `-VaultFile`) via `pwsh`: PASS
- Verified Tier 3 (auto-discovery from `../repo-secrets/02-antigravity-manager/vault/supabase_config.json`): PASS
- Verified Tier 4 (environment variables): PASS
- Verified fail-safe error handling when unresolved: PASS
- Verified URL normalization stripping `/rest/v1`: PASS
