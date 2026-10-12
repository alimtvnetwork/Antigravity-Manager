---
plan: superbase-information-removal
subtask: "02"
title: Source Code, Scripts & Test Fixtures Sanitization
domain: backend/source-and-scripts
depends_on:
  - 01-repo-secrets-vault-and-oneliner.md
citations:
  app_spec: 02-spec/21-app/superbase-information-removal/02-credential-scrub-spec.md
  architecture_spec: 02-spec/21-app/superbase-information-removal/01-architecture-spec.md
  coding_guidelines: 02-spec/02-coding-guidelines/02-canonical-size-tier.md
  strictly_avoid: 02-spec/02-coding-guidelines/strictly-avoid.md
target_files:
  - src-tauri/src/modules/supabase_sync/config_a.rs
  - src-tauri/src/modules/json_envelope/tests.rs
  - scripts/setup-supabase.ps1
  - scripts/supabase-setup.ps1
  - scripts/supabase-endpoints.json
  - scripts/supabase-accounts.json
  - scripts/verify-supabase-and-ui-ux.ps1
status: pending
---

# 02 — Source Code, Scripts & Test Fixtures Sanitization

## 1. Context & Objectives

During feature development, live Supabase endpoint URLs and publishable anon keys were hardcoded into Rust backend source code (`config_a.rs`), Rust test cases (`json_envelope/tests.rs`), administrative setup scripts (`setup-supabase.ps1`, `supabase-setup.ps1`), configuration templates (`supabase-endpoints.json`, `supabase-accounts.json`), and test verification scripts (`verify-supabase-and-ui-ux.ps1`).

This subtask implements the surgical sanitization of all active executable code, scripts, templates, and tests:
1. **Rust Core Configuration**: Eradicate hardcoded URLs and API keys in `default_seed_endpoints()`; transition to dynamic runtime discovery via environment variables and external `repo-secrets` vault files.
2. **Rust Test Fixtures**: Replace live project URLs in `json_envelope/tests.rs` with synthetic mock domains (`https://mock-cluster-01.supabase.co`).
3. **PowerShell Setup Scripts**: Eradicate hardcoded secrets in parameter defaults; replace with environment variable fallbacks and dynamic vault resolution.
4. **JSON Configuration Templates**: Convert `supabase-endpoints.json` and `supabase-accounts.json` into clean templates using canonical placeholders (`YOUR_PROJECT_REF`, `YOUR_KEY_HERE`).
5. **Verification Test Suite**: Update `verify-supabase-and-ui-ux.ps1` to assert zero leaks in `config_a.rs` and validate dynamic discovery logic.

Strictly relative paths must be used throughout all code and documentation. All secrets in this document are strictly redacted.

---

## 2. Target Files & Transformation Specifications

### 2.1 Rust Backend: `src-tauri/src/modules/supabase_sync/config_a.rs`
- **Location**: Function `default_seed_endpoints()` (Lines 401–429).
- **Current State**: Hardcodes Primary Lovable (`https://[REDACTED_LOVABLE_PROJECT].supabase.co`, `sb_publishable_[REDACTED_LOVABLE_KEY]`) and Secondary Personal (`https://[REDACTED_PERSONAL_PROJECT].supabase.co`, `sb_publishable_[REDACTED_PERSONAL_KEY]`).
- **Target State**: Eradicate static strings. Replace with dynamic loader:
  1. Check environment variables: `SUPABASE_ROOT_URL`, `SUPABASE_ROOT_KEY`, `SUPABASE_SECONDARY_URL`, `SUPABASE_SECONDARY_KEY`.
  2. If unset, probe external vault via `auto_seed_from_repo_secrets()`.
  3. If no external secrets are present, return an empty vector or unseeded defaults without valid keys.
- **Invariant**: The core application must compile cleanly and function without panic even if external credentials are absent.

### 2.2 Rust Test Suite: `src-tauri/src/modules/json_envelope/tests.rs`
- **Location**: Test `test_v2_envelope_variable_interpolation()` (Lines 41–67).
- **Current State**: Contains `"baseUrl": "https://[REDACTED_LOVABLE_PROJECT].supabase.co"` and asserts against the raw host.
- **Target State**: Replace with synthetic mock URL:
  ```rust
  "baseUrl": "https://mock-cluster-01.supabase.co",
  "restUrl": "${baseUrl}/rest/v1/"
  ```
  Update the assertion in Line 61 to assert `"https://mock-cluster-01.supabase.co/rest/v1/"`.

### 2.3 PowerShell Setup: `scripts/setup-supabase.ps1`
- **Location**: Param block (Lines 10–18).
- **Current State**: Defaults for `$RootUrl`, `$RootKey`, `$SecondaryUrl`, and `$SecondaryKey` contain live secrets.
- **Target State**: Replace default values with environment variable fallbacks:
  ```powershell
  param (
      [string]$RootUrl = $(if ($env:SUPABASE_ROOT_URL) { $env:SUPABASE_ROOT_URL } else { "" }),
      [string]$RootKey = $(if ($env:SUPABASE_ROOT_KEY) { $env:SUPABASE_ROOT_KEY } else { "" }),
      [string]$SecondaryUrl = $(if ($env:SUPABASE_SECONDARY_URL) { $env:SUPABASE_SECONDARY_URL } else { "" }),
      [string]$SecondaryKey = $(if ($env:SUPABASE_SECONDARY_KEY) { $env:SUPABASE_SECONDARY_KEY } else { "" }),
      [string]$ConfigFile = "",
      [string]$NodeAlias = "",
      [switch]$SkipTest = $false
  )
  ```
  Add dynamic fallback: If `$RootUrl` is empty and `-ConfigFile` is not passed, attempt reading `../repo-secrets/02-antigravity-manager/vault/supabase_config.json` before prompting or warning.

### 2.4 PowerShell Setup: `scripts/supabase-setup.ps1`
- **Location**: Param block (Lines 11–19).
- **Current State**: Contains identical hardcoded default parameters as `setup-supabase.ps1`.
- **Target State**: Apply identical parameter sanitization as `setup-supabase.ps1`.

### 2.5 JSON Template: `scripts/supabase-endpoints.json`
- **Location**: Lines 6–7 and 21–22.
- **Current State**: Embedded live publishable anon keys and URLs.
- **Target State**: Convert into a template file with placeholder values:
  ```json
  {
    "endpoints": [
      {
        "id": "ep-root-lovable-01",
        "name": "Root Supabase (Lovable)",
        "url": "https://YOUR_ROOT_PROJECT.supabase.co/rest/v1/",
        "api_key": "sb_publishable_YOUR_ROOT_KEY_HERE",
        "role": "root",
        "is_enabled": true,
        "prune_threshold_mb": 400,
        "priority": 1,
        "notes": "Template placeholder for Root cluster",
        "tags": ["lovable", "root"]
      },
      {
        "id": "ep-secondary-01",
        "name": "Secondary Supabase",
        "url": "https://YOUR_SECONDARY_PROJECT.supabase.co/rest/v1/",
        "api_key": "sb_publishable_YOUR_SECONDARY_KEY_HERE",
        "role": "secondary",
        "is_enabled": true,
        "prune_threshold_mb": 200,
        "priority": 2,
        "notes": "Template placeholder for Secondary cluster",
        "tags": ["secondary"]
      }
    ],
    "node_alias": "Node-Template",
    "is_sync_enabled": true,
    "auto_prune_root_mb": 400,
    "auto_prune_secondary_mb": 200,
    "heartbeat_interval_secs": 30
  }
  ```

### 2.6 JSON Template: `scripts/supabase-accounts.json`
- **Location**: Lines 6–7 and 21–22.
- **Current State**: Contains duplicate copy of live endpoints.
- **Target State**: Replace with identical sanitized template structure as `scripts/supabase-endpoints.json`.

### 2.7 Verification Suite: `scripts/verify-supabase-and-ui-ux.ps1`
- **Location**: Section 5 (Lines 138–146).
- **Current State**: Asserts that `config_a.rs` contains the exact live project URLs and keys.
- **Target State**: Refactor into negative leak assertions and positive dynamic discovery assertions:
  ```powershell
  Assert-Test "config_a.rs does not contain hardcoded Supabase project IDs or keys" {
      $content = Get-Content "src-tauri/src/modules/supabase_sync/config_a.rs" -Raw
      # Ensure no hardcoded raw publishable key strings exist
      (-not ($content -match "sb_publishable_\[REDACTED_LOVABLE_KEY\]")) -and
      (-not ($content -match "sb_publishable_\[REDACTED_PERSONAL_KEY\]")) -and
      (-not ($content -match "\[REDACTED_LOVABLE_PROJECT_ID\]")) -and
      (-not ($content -match "\[REDACTED_PERSONAL_PROJECT_ID\]"))
  }

  Assert-Test "config_a.rs supports dynamic vault discovery" {
      $content = Get-Content "src-tauri/src/modules/supabase_sync/config_a.rs" -Raw
      $content -match "SUPABASE_ROOT_URL" -or $content -match "load_endpoints_from_repo_secrets" -or $content -match "auto_seed_from_repo_secrets"
  }
  ```

---

## 3. Implementation Workflow & Execution Order

1. **Phase 1: Rust Core & Test Sanitization**:
   - Edit `src-tauri/src/modules/supabase_sync/config_a.rs` to replace hardcoded strings in `default_seed_endpoints()`.
   - Edit `src-tauri/src/modules/json_envelope/tests.rs` to replace leaked project URL with synthetic mock.
   - Run `cargo test --lib json_envelope` to verify tests pass.
2. **Phase 2: Administrative Scripts & Templates Sanitization**:
   - Edit `scripts/setup-supabase.ps1` and `scripts/supabase-setup.ps1` to sanitize default parameter blocks.
   - Replace credentials in `scripts/supabase-endpoints.json` and `scripts/supabase-accounts.json` with template placeholders.
3. **Phase 3: Verification Suite Alignment**:
   - Edit `scripts/verify-supabase-and-ui-ux.ps1` to assert zero secrets and dynamic discovery.
4. **Phase 4: Local Scan Verification**:
   - Execute `gitmap aum search` across the modified paths to ensure zero occurrences of target secret tokens.

---

## 4. Verification Test Matrix

| Test ID | Command / Action | Expected Result |
| :--- | :--- | :--- |
| **TEST-02-01** | `cargo test --lib json_envelope` | Passes 100% with synthetic mock URLs. |
| **TEST-02-02** | `cargo test --lib supabase` | Passes 100% with dynamic/empty seed endpoints. |
| **TEST-02-03** | `gitmap aum search [REDACTED_LOVABLE_PROJECT_ID] src-tauri/` | 0 hits. |
| **TEST-02-04** | `gitmap aum search [REDACTED_PERSONAL_PROJECT_ID] src-tauri/` | 0 hits. |
| **TEST-02-05** | `gitmap aum search [REDACTED_LOVABLE_PROJECT_ID] scripts/` | 0 hits. |
| **TEST-02-06** | `gitmap aum search [REDACTED_PERSONAL_PROJECT_ID] scripts/` | 0 hits. |
| **TEST-02-07** | `pwsh -File scripts/verify-supabase-and-ui-ux.ps1` | Passes all assertions including negative leak tests. |
