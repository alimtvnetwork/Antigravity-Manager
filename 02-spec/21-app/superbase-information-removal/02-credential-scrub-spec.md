# Technical Specification: Credential Scrub & Zero-Secrets Sanitization Architecture

- **Specification Slug:** `superbase-information-removal`
- **Specification Path:** `02-spec/21-app/superbase-information-removal/02-credential-scrub-spec.md`
- **Target Release Version:** `v4.194.0` (Hardening upon `v4.193.0` Baseline)
- **Author:** Writer 02 (Credential Scrub Spec & Sanitization Subtasks Working Group)
- **Status:** APPROVED & MANDATED

---

## 1. Executive Summary & Zero-Secrets Mandate

During rapid implementation and end-to-end verification of the dual-cluster Supabase synchronization subsystem, concrete production credentials and project URLs were inadvertently checked into source code, test suites, automation scripts, specifications, planning memory, and release changelogs.

Specifically, two Supabase project environments were exposed:
1. **Primary Cluster (Lovable Root)**:
   - Host URL: `https://[REDACTED_LOVABLE_PROJECT].supabase.co`
   - Project Reference ID: `[REDACTED_LOVABLE_PROJECT_ID]`
   - Publishable API Key: `sb_publishable_[REDACTED_LOVABLE_KEY]`
2. **Secondary Cluster (Personal Fallback)**:
   - Host URL: `https://[REDACTED_PERSONAL_PROJECT].supabase.co`
   - Project Reference ID: `[REDACTED_PERSONAL_PROJECT_ID]`
   - Publishable API Key: `sb_publishable_[REDACTED_PERSONAL_KEY]`

### 1.1 Non-Negotiable Mandate
This specification mandates the total eradication of every trace of these credentials across all tracked repository files, historical commit entries, and future documentation artifacts. 
- **Zero-Secrets Invariant**: Not a single raw secret string, project reference ID, or publishable API key may remain in tracked files or Git commit history.
- **Dynamic Discovery Invariant**: All real credentials must be resolved dynamically at runtime from external encrypted vaults (`repo-secrets`), local environment variables, or user-supplied configuration files.
- **Sanitized Tooling Parity**: All build, test, and verification pipelines must execute cleanly using dynamic secret discovery or sanitized mock fixtures without requiring hardcoded secrets.

---

## 2. Target File Taxonomy & Eradication Inventory

A comprehensive repository audit using `gitmap aum search` identified 31 occurrences of the Primary Cluster identifier and 28 occurrences of the Secondary Cluster identifier across 12 distinct file paths in 5 major categories.

### 2.1 File Inventory Table

| Category | Relative File Path | Exact Target Areas | Nature of Leaked Artifact | Sanitization Replacement Strategy |
| :--- | :--- | :--- | :--- | :--- |
| **Production Rust** | `src-tauri/src/modules/supabase_sync/config_a.rs` | Lines 407–408, 419–420 (`default_seed_endpoints`) | Hardcoded Primary & Secondary seed URLs and API keys | Eradicate hardcoded strings; load dynamically from `repo-secrets` vault or environment variables (`SUPABASE_ROOT_URL`, `SUPABASE_ROOT_KEY`, `SUPABASE_SECONDARY_URL`, `SUPABASE_SECONDARY_KEY`). If unconfigured, return placeholder mock objects. |
| **Test Fixtures (Rust)** | `src-tauri/src/modules/json_envelope/tests.rs` | Lines 44, 61 (`test_v2_envelope_variable_interpolation`) | Hardcoded Primary project URL in variable map | Replace with synthetic mock URL: `"https://mock-cluster-01.supabase.co"` and assert interpolation against sanitized URL. |
| **Automation Scripts** | `scripts/setup-supabase.ps1` | Lines 11–14 (param block defaults) | Default parameter strings for `$RootUrl`, `$RootKey`, `$SecondaryUrl`, `$SecondaryKey` | Replace defaults with environment variable lookups (`$env:SUPABASE_ROOT_URL`, etc.) with fallback to empty strings. Prompt or load from `repo-secrets` if missing. |
| **Automation Scripts** | `scripts/supabase-setup.ps1` | Lines 12–15 (param block defaults) | Default parameter strings for `$RootUrl`, `$RootKey`, `$SecondaryUrl`, `$SecondaryKey` | Replace defaults with environment variable lookups (`$env:SUPABASE_ROOT_URL`, etc.) with fallback to empty strings. Prompt or load from `repo-secrets` if missing. |
| **Configuration Templates** | `scripts/supabase-endpoints.json` | Lines 6–7, 21–22 | Hardcoded Primary & Secondary URLs and publishable API keys | Replace with clean template tokens: `"https://YOUR_ROOT_PROJECT.supabase.co/rest/v1/"` and `"sb_publishable_YOUR_ROOT_KEY_HERE"`. |
| **Configuration Templates** | `scripts/supabase-accounts.json` | Lines 6–7, 21–22 | Hardcoded Primary & Secondary URLs and publishable API keys | Replace with clean template tokens: `"https://YOUR_ROOT_PROJECT.supabase.co/rest/v1/"` and `"sb_publishable_YOUR_ROOT_KEY_HERE"`. |
| **Verification Test Suite** | `scripts/verify-supabase-and-ui-ux.ps1` | Lines 140, 145 (test assertions) | Regular expression assertions requiring leaked project URLs and keys in `config_a.rs` | Refactor assertions to verify dynamic seeding capability and absence of hardcoded secret tokens. |
| **Specifications** | `02-spec/21-app/supabase-connection-and-ui-ux-enhancement/01-architecture-spec.md` | Lines 41–42, 48–49, 71–74, 91–92, 103–104, 157–159, 265–266 | Architectural tables, diagrams, and JSON payloads containing raw URLs & keys | Redact all credentials using `[REDACTED_LOVABLE_KEY]`, `[REDACTED_PERSONAL_KEY]`, `https://[REDACTED_LOVABLE_PROJECT].supabase.co`, and `https://[REDACTED_PERSONAL_PROJECT].supabase.co`. |
| **Specifications** | `02-spec/21-app/71-supabase-multi-machine-hierarchy-and-cli-e2e.md` | Lines 13, 15, 19, 21, 88–89, 100–101 | Specification examples and test fixtures | Redact all credentials with standard redaction tokens or mock domain names. |
| **Active & Completed Plans** | `.ai-memory/plans/supabase-connection-and-ui-ux-enhancement.md` | Lines 17, 18, 21, 22, 59 | Recorded plan context and endpoint lists | Redact all credentials with standard redaction tokens. |
| **Active & Completed Plans** | `.ai-memory/plans/subtasks/supabase-connection-and-ui-ux-enhancement/01-supabase-dual-cluster-and-stacktrace.md` | Lines 23, 40, 41, 48, 49, 156, 157 | Subtask goals and test matrix items | Redact all credentials with standard redaction tokens. |
| **Active & Completed Plans** | `.ai-memory/plans/subtasks/supabase-connection-and-ui-ux-enhancement/05-image-sharing-public-url.md` | Lines 49, 50, 87, 88 | Upload router mermaid diagrams and quota thresholds | Replace project IDs with generic cluster identifiers (`cluster-primary`, `cluster-secondary`). |
| **Active & Completed Plans** | `.ai-memory/plans/completed/71-supabase-multi-machine-hierarchy-and-cli-e2e.md` | Lines 13, 14 | Log of executed setup commands | Redact all credentials with standard redaction tokens. |
| **Release Changelogs** | `changelog.md` & `changelog_en.md` | Lines 930 (zh), 785 (en) | Release notes for v4.96.0 describing native endpoint integration | Remove raw host URLs and replace with generic references to `Root Supabase (Lovable)` and `Secondary Supabase`. |

---

## 3. Sanitization Replacement Rules & Design Patterns

### 3.1 Rule 1: Dynamic Repo-Secrets Vault Discovery in Rust Core
In production code (`src-tauri/src/modules/supabase_sync/config_a.rs`), static hardcoding in `default_seed_endpoints()` is strictly eliminated.

```rust
// Sanitized Architecture: Dynamic Vault & Environment Discovery
pub fn default_seed_endpoints() -> Vec<SupabaseEndpoint> {
    let mut endpoints = Vec::new();

    // 1. Probe environment variables first
    let root_url = std::env::var("SUPABASE_ROOT_URL").unwrap_or_default();
    let root_key = std::env::var("SUPABASE_ROOT_KEY").unwrap_or_default();
    if !root_url.is_empty() && !root_key.is_empty() {
        endpoints.push(SupabaseEndpoint {
            id: "ep-root-lovable-01".to_string(),
            name: "Primary Lovable".to_string(),
            url: normalize_supabase_url(&root_url),
            api_key: root_key,
            role: "root".to_string(),
            is_enabled: true,
            prune_threshold_mb: 400,
            priority: 1,
            notes: Some("Primary Lovable root cluster endpoint (env-discovered)".to_string()),
            tags: vec!["lovable".to_string(), "root".to_string(), "primary".to_string()],
        });
    }

    let sec_url = std::env::var("SUPABASE_SECONDARY_URL").unwrap_or_default();
    let sec_key = std::env::var("SUPABASE_SECONDARY_KEY").unwrap_or_default();
    if !sec_url.is_empty() && !sec_key.is_empty() {
        endpoints.push(SupabaseEndpoint {
            id: "ep-secondary-personal-01".to_string(),
            name: "Secondary Personal".to_string(),
            url: normalize_supabase_url(&sec_url),
            api_key: sec_key,
            role: "secondary".to_string(),
            is_enabled: true,
            prune_threshold_mb: 200,
            priority: 2,
            notes: Some("Secondary Personal cluster endpoint (env-discovered)".to_string()),
            tags: vec!["secondary".to_string(), "personal".to_string()],
        });
    }

    // 2. If environment variables are not set, attempt loading from repo-secrets vault
    if endpoints.is_empty() {
        if let Some(vault_endpoints) = load_endpoints_from_repo_secrets() {
            return vault_endpoints;
        }
    }

    // 3. Fallback to empty vector: No hardcoded secrets in source code
    endpoints
}
```

### 3.2 Rule 2: Synthetic Mock Fixtures in Test Suites
In `src-tauri/src/modules/json_envelope/tests.rs`:
- Replace any real Supabase project URL with synthetic test identifiers:
  ```rust
  "baseUrl": "https://mock-cluster-01.supabase.co",
  "restUrl": "${baseUrl}/rest/v1/"
  ```
- All test assertions check deterministic string substitution against synthetic domains without requiring network calls or real tokens.

### 3.3 Rule 3: Parameterized Script Ingestion with Fallbacks
In `scripts/setup-supabase.ps1` and `scripts/supabase-setup.ps1`:
- Remove hardcoded default parameters. Replace with:
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
- Provide automatic discovery logic: if parameters are empty, attempt reading `d:/work/repo-secrets/02-antigravity-manager/vault/supabase_config.json` before prompting the operator.

### 3.4 Rule 4: Sanitized Template JSON Configurations
In `scripts/supabase-endpoints.json` and `scripts/supabase-accounts.json`:
- Replace leaked credentials with explicit template placeholders:
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
        "notes": "Root account template placeholder",
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
        "notes": "Secondary fallback template placeholder",
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

### 3.5 Rule 5: Dynamic Verification Script Assertions
In `scripts/verify-supabase-and-ui-ux.ps1`:
- Eradicate assertions matching the leaked project IDs and keys.
- Replace with negative leak tests and positive dynamic discovery tests:
  ```powershell
  Assert-Test "config_a.rs does not contain hardcoded Supabase project IDs or keys" {
      $content = Get-Content "src-tauri/src/modules/supabase_sync/config_a.rs" -Raw
      # Ensure no hardcoded raw publishable key strings or project IDs exist
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

### 3.6 Rule 6: Redaction in Specs, Plans, and Changelogs
In all markdown documentation files (`02-spec/...`, `.ai-memory/...`, `changelog*.md`):
- Replace real tokens with explicit redaction annotations:
  - `sb_publishable_...[REDACTED_LOVABLE_KEY]...`
  - `sb_publishable_...[REDACTED_PERSONAL_KEY]...`
  - `https://[REDACTED_LOVABLE_PROJECT].supabase.co`
  - `https://[REDACTED_PERSONAL_PROJECT].supabase.co`
- Replace raw identifiers in tables and code blocks with semantic descriptions.

---

## 4. Architectural Data Flow: Dynamic Discovery vs Static Leaks

```mermaid
flowchart TD
    subgraph Storage [Secure External Storage - Untracked]
        VAULT["External Vault<br/>d:/work/repo-secrets/02-antigravity-manager/vault/"]
        ENV["System Environment Variables<br/>SUPABASE_ROOT_URL / KEY<br/>SUPABASE_SECONDARY_URL / KEY"]
    end

    subgraph Runtime Core [Antigravity-Manager Runtime Core]
        DISCOVERY["Dynamic Secret Resolver<br/>(auto_seed_from_repo_secrets)"]
        CFG["In-Memory SupabaseConfig"]
        ROUTER["Multi-Cluster Router<br/>(client::ClusterManager)"]
    end

    subgraph Tracked Codebase [Git Tracked Codebase - 100% Sanitized]
        RUST_SRC["src-tauri/src/...<br/>config_a.rs (Zero Secrets)"]
        TESTS["tests.rs<br/>(Synthetic Mocks Only)"]
        SCRIPTS["scripts/*.ps1<br/>(Parameter Fallbacks)"]
        DOCS["02-spec/ & .ai-memory/<br/>(Fully Redacted)"]
    end

    VAULT -->|Loaded at Startup / One-Liner| DISCOVERY
    ENV -->|Loaded at Process Launch| DISCOVERY
    DISCOVERY --> CFG
    CFG --> ROUTER

    RUST_SRC -.->|Compiles with zero secrets| Runtime Core
    TESTS -.->|Runs tests with synthetic data| Runtime Core
    SCRIPTS -.->|Passes runtime params or loads vault| Runtime Core
```

---

## 5. Zero-Secrets Verification Protocol via GitMap

To enforce absolute zero-leak compliance across the entire repository without relying on banned tools (`rg`, `grep`, `git grep`), the verification protocol mandates using `gitmap aum search`.

### 5.1 Verification Commands
Every verification query MUST return **0 hits** across the entire codebase:

```powershell
# 1. Primary Project Identifier Query (must return 0 hits across all files)
gitmap aum search [REDACTED_LOVABLE_PROJECT_ID]

# 2. Secondary Project Identifier Query (must return 0 hits across all files)
gitmap aum search [REDACTED_PERSONAL_PROJECT_ID]

# 3. Primary Publishable Key Fragment Query (must return 0 hits across all files)
gitmap aum search [REDACTED_LOVABLE_KEY]

# 4. Secondary Publishable Key Fragment Query (must return 0 hits across all files)
gitmap aum search [REDACTED_PERSONAL_KEY]
```

### 5.2 Verification Pass/Fail Criteria
- **PASS**: Output displays `Search completed: 0 hit(s) across N file(s)`.
- **FAIL**: Any query returns 1 or more hits. The release gate is immediately intercepted.

---

## 6. Security Invariants & Risk Mitigation

1. **Strict FIFO & Stability Preservation**: No changes to queue restoration or process dispatching algorithms. The modification affects only credential retrieval.
2. **Backward Compatibility**: Existing user configurations located in `%APPDATA%\antigravity-manager\supabase_config.json` remain untouched and functional.
3. **Graceful Degradation**: If an operator runs AGM in a headless server without `repo-secrets` or environment variables, the system logs an informative debug message and disables sync rather than panicking or failing silently.
4. **No Re-Introduction Invariant**: All future pull requests, spec additions, or test fixtures touching Supabase must enforce template placeholders (`YOUR_KEY_HERE`, `mock-cluster-01`) and pass the `gitmap aum search` gate.
