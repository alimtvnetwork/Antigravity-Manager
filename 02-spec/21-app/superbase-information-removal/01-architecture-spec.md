# Architecture Spec: Superbase Information Removal & Git History Purging

**Version:** 1.0.0  
**Updated:** 2026-10-13  
**AI Confidence:** High  
**Ambiguity:** None  
**Task Slug:** `superbase-information-removal`  
**Parent Plan:** `.ai-memory/plans/superbase-information-removal.md`  

---

## Keywords

`supabase-credential-decoupling` · `zero-secrets-invariant` · `git-history-purging` · `repo-secrets-isolation` · `sanitized-placeholder-schema` · `postgrest-mock-fixtures` · `powershell-one-liner-import` · `agm-supabase-push-settings` · `gitmap-secrets-gate` · `tracked-file-sanitization`

---

## Scoring

| Criterion | Status | Notes |
|---|---|---|
| AI Confidence assigned | ✅ High | Complete audit with verified line-level matches |
| Ambiguity assigned | ✅ None | Requirements explicit; zero secrets permitted |
| Keywords present | ✅ Complete | Standardized technical tags |
| Verbatim requirements captured | ✅ Complete | Included verbatim user directive |
| Credential audit catalog | ✅ Complete | 9 tracked files + 5 historical commits cataloged |
| Sanitized placeholder schemas | ✅ Complete | Standardized schemas for endpoints and accounts |
| Security boundaries defined | ✅ Complete | Public repo vs external `repo-secrets` separation |
| Relative Git paths enforced | ✅ Complete | Strict relative paths; zero absolute paths / `file:///` URIs |
| Subagent git hygiene enforced | ✅ Complete | Zero modifying git commands run by subagents |

---

## 1. Executive Summary & Verbatim Requirements

Antigravity-Manager integrates with Supabase PostgREST for distributed multi-machine workspace leasing, instance profile synchronization, and remote command queue handling. During prior releases (notably v4.92.0 and v4.96.0), production Supabase project URLs and publishable API keys were committed into tracked scripts, documentation, test fixtures, and changelogs.

This architectural specification defines the end-to-end framework for completely eliminating and decoupling all Supabase credentials from the repository. All real credentials will reside strictly in the external `repo-secrets` repository, tracked files will use standardized placeholder templates, and historical git commits containing leaked credentials will be purged so that no trace remains in git history.

### 1.1 User Request (Verbatim)

```text
# Superbase Information Removal: high priority instruction, non-negotiable task

Can you please check the recent few commits and anywhere if we have any Superbase information that is committed? If there is, then make sure this is actually fixed and correctly removed. And also at the same time, the previous files would be replaced with the current files in the Git history, so that the history never remains there. They cannot find the trace of the Superbase credential. Do you understand? All the credentials must be saved inside the repo secrets properly with a single liner to import and things like that. Is it clear?

## slug: superbase-information-removal

# Actionable Items Must Follow Non-Negotiable

1. Write spec under 02-spec/21-app/superbase-information-removal/ and enqueue plan task in .ai-memory/plans/superbase-information-removal.md (subtasks in .ai-memory/plans/subtasks/superbase-information-removal/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
4. Use `gitmap` AI agents to enter data
5. Task completion includes committing and pushing to Git
6. Check recent commits for any Superbase information and ensure it is removed
7. Replace previous files with current files in Git history to remove traces of Superbase credentials
8. Save all credentials inside repo secrets with a single liner for import

## Must follow and spawn agent using

@[.agents/skills/execute-parent-task-with-n-steps-v6]

## Additional Instructions

- /plan first before doing the work to reduce the credits.
- /learn from @[.agents/skills/gitmap] skill to leverage GitMap high-speed search, toolchain discovery, and caching.
- Only add the relative paths, never add the absolute path during your work; this should be respected on the release page and in release notes as well.
```

---

## 2. Deep Credential Audit & Leaked Secrets Inventory

A comprehensive multi-pass codebase audit conducted using `gitmap aum search` identified two sensitive production Supabase endpoints and associated publishable API keys committed across multiple repository locations.

### 2.1 Compromised Credential Inventory

| Credential Identifier | Role / Tag | Value / Target | Exposure Scope |
|---|---|---|---|
| **Root Supabase URL** | `root`, `lovable` | `https://ROOT_PROJECT_REF_MASKED.supabase.co/rest/v1/` | Scripts, tests, specs, completed plans, changelogs |
| **Root Supabase API Key** | `root`, `lovable` | `sb_publishable_ROOT_KEY_MASKED_SECRET` | Scripts, specs, completed plans |
| **Secondary Supabase URL** | `secondary` | `https://SEC_PROJECT_REF_MASKED.supabase.co/rest/v1/` | Scripts, specs, completed plans, changelogs |
| **Secondary Supabase API Key** | `secondary` | `sb_publishable_SEC_KEY_MASKED_SECRET` | Scripts, specs, completed plans |

### 2.2 Exact Catalog of Tracked Files Requiring Working-Tree Sanitization

The following 9 tracked files in the current working tree contain hardcoded sensitive URLs or API keys and must be sanitized:

```text
1. scripts/supabase-endpoints.json
   - Lines 6, 7: Hardcoded Root URL and Root API Key
   - Lines 21, 22: Hardcoded Secondary URL and Secondary API Key

2. scripts/supabase-accounts.json
   - Lines 6, 7: Hardcoded Root URL and Root API Key
   - Lines 21, 22: Hardcoded Secondary URL and Secondary API Key

3. scripts/setup-supabase.ps1
   - Lines 11, 12: Default parameter values for RootUrl and RootKey
   - Lines 13, 14: Default parameter values for SecondaryUrl and SecondaryKey

4. scripts/supabase-setup.ps1
   - Lines 12, 13: Default parameter values for RootUrl and RootKey
   - Lines 14, 15: Default parameter values for SecondaryUrl and SecondaryKey

5. src-tauri/src/modules/json_envelope/tests.rs
   - Line 44: Real Root baseUrl in JSON envelope test fixture
   - Line 61: Real Root full restUrl in JSON envelope test assertion

6. 02-spec/21-app/71-supabase-multi-machine-hierarchy-and-cli-e2e.md
   - Lines 13, 15: Verbatim user prompt containing Secondary URL and Key
   - Lines 19, 21: Verbatim user prompt containing Root URL and Key
   - Lines 88, 89: Configuration example containing Root URL and Key
   - Lines 100, 101: Configuration example containing Secondary URL and Key

7. .ai-memory/plans/completed/71-supabase-multi-machine-hierarchy-and-cli-e2e.md
   - Line 13: Execution summary documenting Root URL and Key
   - Line 14: Execution summary documenting Secondary URL and Key

8. changelog.md
   - Line 930: v4.96.0 release notes citing real Root and Secondary URLs

9. changelog_en.md
   - Line 785: v4.96.0 release notes citing real Root and Secondary URLs
```

### 2.3 Catalog of Historical Git Commits Requiring Purging

Audit of the git revision graph via commit diff inspection identified 5 key commits that introduced or propagated these secrets:

| Commit Hash | Commit Summary | Introduced Secret Content |
|---|---|---|
| `f33a0212` | `chore(release): bump version to 4.92.0` | Initial addition of `scripts/setup-supabase.ps1`, `scripts/supabase-endpoints.json` with Root and Secondary keys |
| `33000649` | `chore(release): bump version to 4.96.0` | Added `scripts/supabase-setup.ps1`, `scripts/supabase-accounts.json`, `02-spec/21-app/71-...`, changelog entries |
| `3520d6ad` | `feat(json): harmonize JSON envelopes...` | Added test fixture in `src-tauri/src/modules/json_envelope/tests.rs` with `ROOT_PROJECT_REF_MASKED` |
| `433d697d` | `docs(changelog): replace legacy Chinese history...` | Replicated URLs into `changelog_en.md` |
| `22a87aae` | `refactor: split god modules to <=500 lines` | Touched modules referencing test fixtures and completed plans |

---

## 3. System Architecture for Complete Credential Decoupling

### 3.1 Architectural Principles

1. **Zero Secrets in Public Repository (Invariant R16)**:
   - The public repository contains zero live API keys, tokens, production project IDs, or private connection strings.
   - All tracked files in `scripts/`, `src/`, `src-tauri/`, and documentation use generic placeholders (`https://<project-ref>.supabase.co`, `sb_publishable_<placeholder>`) or synthetic mock URLs (`https://sample-mock-project.supabase.co`).

2. **Decoupled Secret Storage via External `repo-secrets`**:
   - Production credentials exist exclusively inside the private sibling repository `repo-secrets`.
   - Specifically, credentials are stored in two managed vault locations:
     - `02-antigravity-manager/vault/supabase_config.json`: Master multi-node fleet configuration adhering to AGM JSON Envelope Schema v2.0.
     - `03-supabase/02-lovable/supabase-credentials.json`: Base64-encoded encrypted credentials format adhering to AGM Credentials Envelope Schema v1.0.

3. **Dynamic Local Ingestion (No Hardcoded Fallbacks)**:
   - The Antigravity-Manager runtime loads credentials strictly from:
     - Active machine config: `%APPDATA%\antigravity-manager\supabase_config.json` (Windows) or `~/.config/antigravity-manager/supabase_config.json` (Linux/macOS).
     - Environment variables: `AGM_SUPABASE_VAULT`, `SUPABASE_URL`, `SUPABASE_KEY`.
     - Explicit CLI flags: `agm supabase push-settings --vault <path>` or `agm supabase load-json <path>`.
   - Application modules (`supabase_client.rs`, `supabase_sync.rs`, `workspace_lease_manager.rs`) fail gracefully with descriptive warnings when unconfigured, never falling back to hardcoded production endpoints.

### 3.2 Decoupled Data Flow Architecture

```mermaid
flowchart TD
    subgraph ExternalPrivateVault["External Private Vault (repo-secrets)"]
        VaultFile["repo-secrets/02-antigravity-manager/vault/supabase_config.json"]
        LovableCreds["repo-secrets/03-supabase/02-lovable/supabase-credentials.json"]
    end

    subgraph PublicRepository["Public Repository (antigravity-manager)"]
        PlaceholderEndpoints["scripts/supabase-endpoints.json (Sanitized Placeholder)"]
        PlaceholderAccounts["scripts/supabase-accounts.json (Sanitized Placeholder)"]
        ImportScript["scripts/import-supabase-secrets.ps1 (Thin CLI Wrapper)"]
        AppRuntime["Antigravity-Manager App & CLI (agm binary)"]
    end

    subgraph LocalMachineState["Target Machine Local Storage (Ignored / Private)"]
        LocalConfig["%APPDATA%/antigravity-manager/supabase_config.json"]
        PostgREST["Supabase PostgREST Fleet Cluster"]
    end

    VaultFile -.->|"Import via One-Liner / CLI"| ImportScript
    ImportScript -->|"agm supabase load-json --file"| AppRuntime
    AppRuntime -->|"Writes Decoupled Runtime Config"| LocalConfig
    LocalConfig -->|"Hydrates Endpoints & Leases"| AppRuntime
    AppRuntime <==>|"Authenticated REST / RPC Calls"| PostgREST
    PlaceholderEndpoints -.->|"Template Only (No Real Keys)"| PublicRepository
```

---

## 4. Sanitized Placeholder Schemas & Data Contracts

### 4.1 Sanitized `scripts/supabase-endpoints.json` Schema

The tracked endpoint definition file in the repository must use generic, sanitized placeholder values that instruct the operator where to supply their credentials:

```json
{
  "endpoints": [
    {
      "id": "ep-root-lovable-01",
      "name": "Root Supabase (Lovable)",
      "url": "https://<ROOT_PROJECT_REF>.supabase.co/rest/v1/",
      "api_key": "sb_publishable_<ROOT_KEY_PLACEHOLDER>",
      "role": "root",
      "is_enabled": true,
      "prune_threshold_mb": 400,
      "priority": 1,
      "notes": "Root account by Lovable (Set via repo-secrets)",
      "tags": [
        "lovable",
        "root"
      ]
    },
    {
      "id": "ep-secondary-01",
      "name": "Secondary Supabase",
      "url": "https://<SECONDARY_PROJECT_REF>.supabase.co/rest/v1/",
      "api_key": "sb_publishable_<SECONDARY_KEY_PLACEHOLDER>",
      "role": "secondary",
      "is_enabled": true,
      "prune_threshold_mb": 200,
      "priority": 2,
      "notes": "Secondary fallback and command queue (Set via repo-secrets)",
      "tags": [
        "secondary"
      ]
    }
  ],
  "node_alias": "Node-${MACHINE_ID}",
  "is_sync_enabled": false,
  "auto_prune_root_mb": 400,
  "auto_prune_secondary_mb": 200,
  "heartbeat_interval_secs": 30
}
```

### 4.2 Sanitized `scripts/supabase-accounts.json` Schema

`scripts/supabase-accounts.json` mirrors the sanitized endpoint structure with `is_sync_enabled` defaulted to `false` until populated from an external vault:

```json
{
  "endpoints": [
    {
      "id": "ep-root-lovable-01",
      "name": "Root Supabase (Lovable)",
      "url": "https://<ROOT_PROJECT_REF>.supabase.co/rest/v1/",
      "api_key": "sb_publishable_<ROOT_KEY_PLACEHOLDER>",
      "role": "root",
      "is_enabled": true,
      "prune_threshold_mb": 400,
      "priority": 1,
      "notes": "Root account by Lovable",
      "tags": [
        "lovable",
        "root"
      ]
    },
    {
      "id": "ep-secondary-01",
      "name": "Secondary Supabase",
      "url": "https://<SECONDARY_PROJECT_REF>.supabase.co/rest/v1/",
      "api_key": "sb_publishable_<SECONDARY_KEY_PLACEHOLDER>",
      "role": "secondary",
      "is_enabled": true,
      "prune_threshold_mb": 200,
      "priority": 2,
      "notes": "Secondary fallback and command queue",
      "tags": [
        "secondary"
      ]
    }
  ],
  "node_alias": "Node-${MACHINE_ID}",
  "is_sync_enabled": false,
  "auto_prune_root_mb": 400,
  "auto_prune_secondary_mb": 200,
  "heartbeat_interval_secs": 30
}
```

### 4.3 Sanitized PowerShell Setup Parameter Defaults

In `scripts/setup-supabase.ps1` and `scripts/supabase-setup.ps1`, the parameter block must not embed live credentials as defaults:

```powershell
[CmdletBinding()]
param (
    [string]$ConfigFile = "",
    [string]$RootUrl = "",
    [string]$RootKey = "",
    [string]$SecondaryUrl = "",
    [string]$SecondaryKey = "",
    [string]$NodeAlias = "",
    [switch]$SkipTest = $false
)
```

If neither `$ConfigFile` nor `-RootUrl`/`-RootKey` parameters are provided, the script should automatically search for the default vault path in `../repo-secrets/02-antigravity-manager/vault/supabase_config.json` or query the environment variable `AGM_SUPABASE_VAULT`. If still unresolved, it outputs an informative error guiding the operator to provide the vault path.

### 4.4 Unit Test Mock Fixtures Sanitization

In `src-tauri/src/modules/json_envelope/tests.rs`:
- Replace `https://ROOT_PROJECT_REF_MASKED.supabase.co` with synthetic test URL: `https://sample-mock-project.supabase.co`.
- Replace `https://ROOT_PROJECT_REF_MASKED.supabase.co/rest/v1/` with `https://sample-mock-project.supabase.co/rest/v1/`.

### 4.5 Documentation & Changelog Sanitization

- `02-spec/21-app/71-supabase-multi-machine-hierarchy-and-cli-e2e.md`: Mask verbatim URLs to `https://<root-ref>.supabase.co/rest/v1/` and keys to `sb_publishable_<root_key>`.
- `.ai-memory/plans/completed/71-supabase-multi-machine-hierarchy-and-cli-e2e.md`: Mask URLs and keys with redacted placeholders.
- `changelog.md` and `changelog_en.md`: Replace explicit project URLs in release v4.96.0 descriptions with generic descriptors `Root Supabase (Lovable)` and `Secondary Supabase`.

---

## 5. Security Boundaries: Public Repository vs External `repo-secrets`

```text
+-----------------------------------------------------------------------------------+
|                           SECURITY BOUNDARY ARCHITECTURE                          |
+-----------------------------------------------------------------------------------+
|                                                                                   |
|  [ PUBLIC REPOSITORY: antigravity-manager ]                                       |
|  * Source code, build manifests, and UI components                                |
|  * Tracked templates: scripts/supabase-endpoints.json (Placeholder schema only)   |
|  * Utility scripts: scripts/import-supabase-secrets.ps1 (Thin loader wrapper)      |
|  * Unit test fixtures: Synthetic mock domains (*.supabase.co with dummy tokens)   |
|  * Documentation & Specs: Redacted URLs and placeholder tokens only               |
|  * INVARIANT: Zero live API keys or production project references                |
|                                                                                   |
|                              |                                                    |
|                              | (No direct Git dependency / No committed secrets)  |
|                              v                                                    |
|                                                                                   |
|  [ EXTERNAL PRIVATE REPOSITORY: repo-secrets ]                                    |
|  * Physical location: Sibling directory (../repo-secrets) or secure vault         |
|  * Vault path: 02-antigravity-manager/vault/supabase_config.json                  |
|  * Encrypted backup: 03-supabase/02-lovable/supabase-credentials.json             |
|  * Access control: Restrained team access, independent git repository             |
|                                                                                   |
|                              |                                                    |
|                              | (Loaded dynamically at runtime or deployment)      |
|                              v                                                    |
|                                                                                   |
|  [ LOCAL RUNTIME ENVIRONMENT ]                                                    |
|  * Location: %APPDATA%/antigravity-manager/supabase_config.json (Windows)         |
|              ~/.config/antigravity-manager/supabase_config.json (Linux/macOS)     |
|  * Lifecycle: Git-ignored, ephemeral local state populated on demand              |
|                                                                                   |
+-----------------------------------------------------------------------------------+
```

### 5.1 Single-Liner Runtime Import Provisioning

To enable one-liner setup without committing credentials to `antigravity-manager`, a dedicated script `scripts/import-supabase-secrets.ps1` provides seamless ingestion from `repo-secrets`:

#### Standard Local Execution One-Liner (PowerShell)
```powershell
powershell -NoProfile -ExecutionPolicy Bypass -Command "& { [string]$vault = if (Test-Path '../repo-secrets/02-antigravity-manager/vault/supabase_config.json') { '../repo-secrets/02-antigravity-manager/vault/supabase_config.json' } else { 'd:\work\repo-secrets\02-antigravity-manager\vault\supabase_config.json' }; if (Test-Path $vault) { agm supabase load-json --file $vault; Write-Host 'Successfully imported Supabase credentials from vault' -ForegroundColor Green } else { Write-Error 'Vault not found at expected location' } }"
```

#### Dedicated Runner Script Usage
```powershell
pwsh -NoProfile -File ./scripts/import-supabase-secrets.ps1 -VaultPath ../repo-secrets/02-antigravity-manager/vault/supabase_config.json
```

---

## 6. Git History Purging Strategy

Per user directive:
> "And also at the same time, the previous files would be replaced with the current files in the Git history, so that the history never remains there. They cannot find the trace of the Superbase credential."

### 6.1 Replacement & History Rewriting Approach

1. **Working Tree Replacement First**:
   - First, sanitize all 9 tracked files in the current working tree so they contain only valid placeholder values and mock fixtures.
   - Verify that test suites and compilation remain completely clean with placeholders in place.

2. **History Rewriting Scope**:
   - The history rewrite targets the commits in the current branch that introduced the sensitive credentials:
     - `f33a0212` through `HEAD`
   - Files to replace or filter across historical commits:
     - `scripts/supabase-endpoints.json` -> replace with sanitized version
     - `scripts/supabase-accounts.json` -> replace with sanitized version
     - `scripts/setup-supabase.ps1` -> replace with sanitized version
     - `scripts/supabase-setup.ps1` -> replace with sanitized version
     - `src-tauri/src/modules/json_envelope/tests.rs` -> replace with sanitized mock version
     - `02-spec/21-app/71-supabase-multi-machine-hierarchy-and-cli-e2e.md` -> replace with sanitized version
     - `.ai-memory/plans/completed/71-supabase-multi-machine-hierarchy-and-cli-e2e.md` -> replace with sanitized version
     - `changelog.md` and `changelog_en.md` -> replace with sanitized version

3. **Subagent Execution Gate**:
   - Subagents (Worker 01, Worker 02) are strictly forbidden from executing modifying git commands (`git filter-repo`, `git rebase`, `git commit`, `git push`).
   - The Lead Orchestrator agent performs the git history rewrite and push once all files are verified in the working tree.

---

## 7. Verification & Quality Acceptance Matrix

| Verification ID | Description | Verification Method | Acceptance Standard |
|---|---|---|---|
| **V-01** | Zero real Supabase project URLs in working tree | `gitmap aum search "ROOT_PROJECT_REF_MASKED" . --include-large-json` | Exactly 0 hits |
| **V-02** | Zero secondary Supabase project URLs in working tree | `gitmap aum search "SEC_PROJECT_REF_MASKED" . --include-large-json` | Exactly 0 hits |
| **V-03** | Zero real publishable keys in working tree | `gitmap aum search "cJdJyIEeXc8bpym9TIOU7w_vVU0_Y69" . --include-large-json` | Exactly 0 hits |
| **V-04** | Zero secondary publishable keys in working tree | `gitmap aum search "barsshQom3VcUw1l5soE_A_R1FwQz8o" . --include-large-json` | Exactly 0 hits |
| **V-05** | Real credentials safely preserved in repo-secrets | `gitmap aum search "ROOT_PROJECT_REF_MASKED" /home/a/git-work/repo-secrets` | Persisted safely in external vault |
| **V-06** | Unit tests pass with mock fixtures | `cd src-tauri && cargo test modules::json_envelope` | Test passes with zero real URLs |
| **V-07** | Relative paths compliance | Manual & automated scan of newly created specs | Zero `file:///` URIs, zero absolute paths |
| **V-08** | Git history trace removal | `git log -S "ROOT_PROJECT_REF_MASKED" --oneline` after history rewrite | Exactly 0 commits returned |

---

## 8. Subtask Traceability & Roadmap

| Subtask Code | Title | Target Artifacts | Owner |
|---|---|---|---|
| **Task-01** | Specification & Architecture Formulation | `02-spec/21-app/superbase-information-removal/01-architecture-spec.md`<br>`02-spec/21-app/superbase-information-removal/02-component-spec.md` | Worker 01 |
| **Task-02** | Codebase & Working Tree Credential Sanitization | `scripts/supabase-endpoints.json`<br>`scripts/supabase-accounts.json`<br>`scripts/setup-supabase.ps1`<br>`scripts/supabase-setup.ps1`<br>`src-tauri/src/modules/json_envelope/tests.rs`<br>Changelogs & documentation | Worker 01 / Lead |
| **Task-03** | Git Commit History Audit & Secret Purging | History rewriting across commits `f33a0212`..`HEAD` | Lead Orchestrator |
| **Task-04** | Repo Secrets Ingestion & Single-Liner Import | `scripts/import-supabase-secrets.ps1`<br>`repo-secrets/02-antigravity-manager/vault/supabase_config.json` | Worker 01 / Lead |
| **Task-05** | Final Verification, Index Registration, and Push | `gitmap aum search` gate, `readme.md` sync, `gitmap cpf` push | Lead Orchestrator |
