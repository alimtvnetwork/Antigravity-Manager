# Subtask 01: Supabase Multi-Endpoint Connection & Repo-Secrets Synchronization

**Plan Slug:** `supabase-connection-and-ui-ux-enhancement`  
**Subtask File:** `.ai-memory/plans/subtasks/supabase-connection-and-ui-ux-enhancement/01-supabase-connection-and-secrets.md`  
**Associated Spec:** `02-spec/21-app/supabase-connection-and-ui-ux-enhancement/01-supabase-and-secrets-spec.md`  
**Status:** READY  
**Author:** Worker 01 (Supabase, Secrets & Image Storage Architect)  

---

## 1. Context & Objectives

This subtask operationalizes the multi-endpoint Supabase configuration, fixes typographical and DNS resolution vulnerabilities, establishes unpausing guidance for free-tier projects, and integrates the 5-tier credential discovery ladder with `repo-secrets` and GitMap.

### 1.1 Core Problems Addressed
1. **Subdomain Typo & NXDOMAIN Failures**: The project reference `pezjuuddecbyrmqxytrv` (`r` instead of `f`) crashes DNS lookups with `WSAHOST_NOT_FOUND`. The client must proactively identify this typo and supply the corrected URL.
2. **Project Pause Latency**: Inactive free-tier Supabase projects automatically pause after 7 days, resulting in 15-second timeouts or HTTP 503 responses. Transport error handlers must provide explicit dashboard restoration links.
3. **Multi-Endpoint Provisioning**: Primary Root (Lovable: `https://[REDACTED_LOVABLE_PROJECT].supabase.co/rest/v1/`) and Secondary (Personal: `https://[REDACTED_PERSONAL_PROJECT].supabase.co/rest/v1/`) endpoints must be seamlessly managed with role segregation (`root` vs `secondary`), storage quotas (400MB vs 200MB), and priority cascading.
4. **Credential Harvesting**: Seamless discovery across local configuration, `repo-secrets/03-supabase/`, parent relative paths, and GitMap environment variables.

---

## 2. File Ownership & Touchpoints

The following files are touched by this subtask:
- `src-tauri/src/modules/supabase_client/client.rs`: Enhanced transport error classification for typos, NXDOMAIN, and project pausing.
- `src-tauri/src/modules/supabase_client/tests.rs`: Unit tests verifying error classifier diagnostics for typos and connection errors.
- `src-tauri/src/modules/supabase_sync/config_a.rs`: 5-tier credential discovery ladder and repo-secrets harvester.
- `src-tauri/src/modules/supabase_sync/config_b.rs`: Configuration serialization, normalization, and endpoint defaults.
- `src-tauri/src/bin/agm/supabase_cmds.rs`: AGM CLI commands for `status`, `test`, `load-json`, and `reload-secrets`.
- `src-tauri/src/bin/agm/supabase_config_cmds.rs`: CLI endpoint management routines.

---

## 3. Step-by-Step Implementation Plan

### Step 1: Enhance Transport Error Classifier in `client.rs`
- Detect known typo subdomain `pezjuuddecbyrmqxytrv` and return an immediate, actionable fix pointing to `[REDACTED_LOVABLE_PROJECT]`.
- Detect DNS host resolution errors (`No such host is known`, `failed to lookup address`) and label them as `DNS NXDOMAIN`.
- Detect connection timeouts (`e.is_timeout()`) and provide the direct URL to the Supabase dashboard with instructions to unpause/restore the project.

### Step 2: Seed Default Endpoints in `config_b.rs`
- Ensure that fresh configurations or empty endpoint lists automatically register:
  - **Endpoint 1 (Root / Lovable)**:
    - ID: `ep-root-lovable-01`
    - URL: `https://[REDACTED_LOVABLE_PROJECT].supabase.co/rest/v1/`
    - API Key: `[REDACTED_LOVABLE_KEY]`
    - Role: `root`
    - Priority: 1
    - Prune Threshold: 400 MB
    - Tags: `["lovable", "root"]`
  - **Endpoint 2 (Secondary / Personal)**:
    - ID: `ep-secondary-personal-01`
    - URL: `https://[REDACTED_PERSONAL_PROJECT].supabase.co/rest/v1/`
    - API Key: `[REDACTED_PERSONAL_KEY]`
    - Role: `secondary`
    - Priority: 2
    - Prune Threshold: 200 MB
    - Tags: `["secondary", "personal"]`

### Step 3: Implement 5-Tier Discovery Ladder in `config_a.rs`
- Implement `auto_discover_supabase_credentials()`:
  - Tier 1: Check existing `%APPDATA%/antigravity-manager/supabase_config.json`.
  - Tier 2: Search relative path `repo-secrets/03-supabase/02-lovable/supabase-credentials.json`.
  - Tier 3: Search relative path `repo-secrets/03-supabase/01-own/supabase-credentials.json`.
  - Tier 4: Traverse parent relative paths (`../repo-secrets/...`, `../../repo-secrets/...`).
  - Tier 5: Read environment variables `AGM_SUPABASE_ROOT_URL`, `AGM_SUPABASE_ROOT_KEY`, `AGM_SUPABASE_SECONDARY_URL`, `AGM_SUPABASE_SECONDARY_KEY`.
- Merge discovered endpoints into memory without overwriting customized user settings.

### Step 4: Expand CLI Subcommands in `supabase_cmds.rs`
- Add command `agm supabase reload-secrets` to trigger immediate ladder discovery.
- Enhance `agm supabase status` to display:
  - Configured endpoints with masked keys (`sb_publishable_...`).
  - Active role assignments (`root` vs `secondary`).
  - Probe connectivity status and storage quota thresholds.
- Enhance `agm supabase load-json <path>` to accept either raw endpoint arrays or two-tier unified envelopes (`{ "attributes": ..., "data": ... }`).

### Step 5: Unit Testing & Verification
- Unit test typo classification: assert that checking typographical errors outputs actionable replacement suggestions.
- Unit test timeout classification: assert that a simulated timeout outputs unpause instructions.
- Unit test configuration round-trip: verify JSON import and normalization.

---

## 4. Acceptance Criteria & Quality Gates

- [ ] **AC-1 (Typo Diagnostic)**: Subdomain typo `pezjuuddecbyrmqxytrv` is detected with a specific error message advising the `r` -> `f` correction.
- [ ] **AC-2 (Pause Diagnostic)**: Timeout responses provide clear dashboard URLs for unpausing projects.
- [ ] **AC-3 (Dual Endpoint Persistence)**: Primary Lovable Root and Secondary Personal endpoints are cleanly registered with roles and priorities.
- [ ] **AC-4 (Discovery Ladder)**: Endpoints in `repo-secrets/03-supabase/` are correctly resolved via relative path traversal.
- [ ] **AC-5 (CLI Parity)**: `agm supabase status` and `agm supabase test` run successfully from the terminal with full tabular output.
