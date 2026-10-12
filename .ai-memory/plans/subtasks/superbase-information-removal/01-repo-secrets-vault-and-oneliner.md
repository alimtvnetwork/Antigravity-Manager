---
plan: superbase-information-removal
subtask: "01"
title: Repo-Secrets Vault Storage & Single-Liner Import Protocol
domain: security-vault-and-cli
depends_on: none
citations:
  app_spec: 02-spec/21-app/superbase-information-removal/01-architecture-spec.md
  coding_guidelines: 02-spec/02-coding-guidelines/02-canonical-size-tier.md
  strictly_avoid: 02-spec/02-coding-guidelines/strictly-avoid.md
target_files:
  - scripts/import-supabase-credentials.ps1
  - scripts/setup-supabase.ps1
  - src-tauri/src/modules/cli/supabase.rs
  - src-tauri/src/modules/supabase_sync/config_a.rs
status: pending
---

# 01 — Repo-Secrets Vault Storage & Single-Liner Import Protocol

## 1. Context & Objectives

During prior feature integrations, Supabase endpoint configurations containing live project URLs and publishable anon keys were embedded in parameter defaults of helper scripts and fallback constants. While publishable keys are scoped by PostgreSQL Row Level Security (RLS), publishing these connection endpoints within public source control exposes infrastructure identifiers and creates attack surfaces for credential scraping.

This subtask establishes the external vault storage protocol and provides a zero-hardcoding import workflow. All live credentials reside exclusively in the unversioned external repository `../repo-secrets/` under `02-antigravity-manager/vault/` and `03-supabase/`. Operators import credentials into local user storage via the native AGM CLI (`agm supabase load-secrets`) or a clean companion PowerShell script (`scripts/import-supabase-credentials.ps1`) using a single terminal command.

No credentials or live project URLs may ever be printed to stdout, logged to debug files, or committed to Git.

## 2. Target Files & Symbols

- `scripts/import-supabase-credentials.ps1` — New PowerShell companion script executing CLI import and automated connection health testing.
- `scripts/setup-supabase.ps1` — Replace legacy script with a thin redirection wrapper delegating to `scripts/import-supabase-credentials.ps1`.
- `src-tauri/src/modules/cli/supabase.rs` — Enhance `handle_supabase_subcommand` to support `load-secrets` subcommand alias alongside `load-json`.
- `src-tauri/src/modules/supabase_sync/config_a.rs` — Ensure `candidate_repo_secrets_paths()` and `auto_seed_from_repo_secrets()` properly traverse candidate paths.

## 3. External Vault Storage Specification

Operational credentials are maintained across three validated files in sibling unversioned storage `../repo-secrets/`:

1. **Unified Dual-Cluster Vault**:
   `../repo-secrets/02-antigravity-manager/vault/supabase_config.json`
   Contains dual-cluster definitions (Primary Lovable cluster with priority 1, and Secondary Personal cluster with priority 2).
2. **Dedicated Secondary / Personal Cluster Vault**:
   `../repo-secrets/03-supabase/01-own/supabase-credentials.json`
   Contains credentials for developer-specific command queueing and telemetry.
3. **Dedicated Primary / Lovable Cluster Vault**:
   `../repo-secrets/03-supabase/02-lovable/supabase-credentials.json`
   Contains credentials for fleet node leases and public prompt image hosting.

All vault files conform to JSON Envelope v2.0 schema:
- `_envelope.version`: `"2.0"`
- `_envelope.dataType`: `"agm/supabase-endpoints"`
- `endpoints[].url`: `[REDACTED_LOVABLE_URL]` or `[REDACTED_PERSONAL_URL]`
- `endpoints[].anonKey`: `[REDACTED_LOVABLE_KEY]` or `[REDACTED_PERSONAL_KEY]`

## 4. Single-Liner Import Workflow

### 4.1 Native CLI Command: `agm supabase load-secrets`
The AGM CLI handles secret loading natively:
```bash
# Ingest from auto-discovered external vault paths
agm supabase load-secrets

# Or specify explicit vault file path
agm supabase load-secrets --vault ../repo-secrets/02-antigravity-manager/vault/supabase_config.json
```

The CLI:
- Reads the JSON envelope from the external vault.
- Sanitizes endpoint URLs (stripping trailing slashes and `/rest/v1` prefixes).
- Persists normalized config to local runtime storage (`AppData/Roaming/com.antigravity.manager/supabase_config.json`).
- Outputs endpoint count and masked summary. Never prints raw keys.

### 4.2 Companion Script: `scripts/import-supabase-credentials.ps1`
A clean PowerShell wrapper that automates loading and health checking:
```powershell
[CmdletBinding()]
param(
    [Parameter(Mandatory = $false)]
    [string]$VaultPath = ""
)

$ErrorActionPreference = "Stop"

Write-Host "=== Antigravity-Manager: Supabase Credential Import ===" -ForegroundColor Cyan

$agmArgs = @("supabase", "load-secrets")
if ($VaultPath -ne "") {
    $agmArgs += @("--vault", $VaultPath)
}

Write-Host "-> Importing credentials via AGM CLI..." -ForegroundColor Yellow
& agm @agmArgs
if ($LASTEXITCODE -ne 0) {
    Write-Error "Failed to load secrets via AGM CLI (exit code: $LASTEXITCODE)"
    exit $LASTEXITCODE
}

Write-Host "-> Validating connection health..." -ForegroundColor Yellow
& agm supabase test
if ($LASTEXITCODE -ne 0) {
    Write-Error "Supabase connectivity verification failed (exit code: $LASTEXITCODE)"
    exit $LASTEXITCODE
}

Write-Host "=== Supabase Credentials Successfully Loaded & Verified ===" -ForegroundColor Green
exit 0
```

### 4.3 Execution Single-Liner
Operators run either of the following one-liners:
```powershell
# Direct CLI one-liner
pwsh -NoProfile -Command "agm supabase load-secrets; agm supabase test"

# Or via companion script
pwsh -NoProfile -File ./scripts/import-supabase-credentials.ps1
```

## 5. Implementation Steps

1. **Verify External Vault Existence & Structure**:
   Validate that `../repo-secrets/02-antigravity-manager/vault/supabase_config.json` exists with valid envelope formatting and credentials.
2. **Implement CLI `load-secrets` Subcommand**:
   Update `src-tauri/src/modules/cli/supabase.rs` to register `load-secrets` as a recognized subcommand that routes to `auto_seed_from_repo_secrets()` or explicit `--vault <path>` parsing.
3. **Author Companion Script `scripts/import-supabase-credentials.ps1`**:
   Create the non-hardcoded PowerShell script with strict error handling (`$ErrorActionPreference = "Stop"` and `$LASTEXITCODE` checks).
4. **Refactor Legacy `scripts/setup-supabase.ps1`**:
   Replace the old parameter default blocks with a delegation call to `scripts/import-supabase-credentials.ps1`.
5. **Verify Zero Key Leakage**:
   Ensure `agm supabase load-secrets` and `agm supabase test` output only masked endpoint URLs (e.g. `https://pe...supabase.co`) and never expose key tokens.

## 6. Constraints & Coding Standards

- **Strictly Relative Paths**: All paths in scripts and documentation MUST use relative paths (e.g., `../repo-secrets/`, `scripts/`, `src-tauri/`). Never use absolute paths.
- **Zero Key Printing**: Never print, echo, or log secret tokens.
- **No Silent Errors**: Fallible operations in Rust must route through `AppError` and `record_ignored` where justifiable. PowerShell scripts must check `$LASTEXITCODE` and exit with non-zero on failure.
- **PowerShell Compatibility**: Compatible with both Windows PowerShell 5.1 and PowerShell Core 7+.

## 7. Out of Scope

- Amending git history (handled in Subtask 04).
- Sanitizing legacy test fixtures and documentation (handled in Subtasks 02 & 03).
- Remote force push (handled in Subtask 05).

## 8. Verification & Acceptance Criteria

```bash
# 1. Verify AGM CLI builds cleanly
cd src-tauri && cargo clippy --all-targets --all-features

# 2. Run single-liner import
pwsh -NoProfile -File ./scripts/import-supabase-credentials.ps1

# 3. Confirm connectivity passes
agm supabase test
```

## 9. Done When

- [ ] `../repo-secrets/` external vault storage protocol is documented and verified.
- [ ] `agm supabase load-secrets` is implemented and functional.
- [ ] `scripts/import-supabase-credentials.ps1` exists, contains 0 hardcoded credentials, and executes cleanly.
- [ ] `scripts/setup-supabase.ps1` delegates cleanly without embedding default secrets.
- [ ] Connectivity tests return HTTP 200 without leaking raw keys.
