---
plan: 127-accounts-ui-supabase-sync-instance-compact-and-release
subtask: "03"
title: Supabase Secrets Auto-Discovery and CLI Configuration Parity
domain: backend-rust
depends_on: none
citations:
  app_spec: ../../../../02-spec/21-app/127-accounts-ui-supabase-sync-instance-compact-and-release/02-component-spec.md#21-supabase-secrets-auto-discovery-supabase_syncrs--supabase_clientrs
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/supabase_sync.rs
  - src-tauri/src/modules/supabase_client.rs
  - src-tauri/src/bin/agm.rs
status: pending
---

# 03 — Supabase Secrets Auto-Discovery and CLI Configuration Parity

## 1. Context & Motivation
In Antigravity Manager, distributed instance synchronization and account concurrency protection rely on Supabase as the central root ledger. However, upon fresh clone or initial setup, developers often have credentials placed under `D:\work\repo-secrets\03-supabase\01-own\supabase-credentials.json` or defined via the environment variable `REPO_SECRETS_DIR`. If the application fails to auto-discover and load these credentials when `supabase_config.json` is missing or has empty endpoints, synchronization remains disabled and distributed leases fail to register.

Furthermore, CLI management via `agm supabase set-config --cooldown <mins>` historically updated only `account_lockout_window_minutes` while leaving `account_cooldown_minutes` unsynchronized, causing behavioral drift between local switcher logic and distributed lock windows.

## 2. Target Files & Symbols
- `src-tauri/src/modules/supabase_sync.rs`
  - `candidate_repo_secrets_paths() -> Vec<PathBuf>`
  - `auto_seed_from_repo_secrets(cfg: &mut SupabaseConfig) -> bool`
  - `load_config() -> Result<SupabaseConfig, AppError>`
- `src-tauri/src/modules/supabase_client.rs`
  - `normalize_supabase_url(raw: &str) -> String`
  - `SupabaseEndpoint::new_endpoint(...)`
- `src-tauri/src/bin/agm.rs`
  - `cmd_supabase_set_config(args: &[String])`

---

## 3. Implementation Steps

### 3.1 Verify & Enhance Candidate Path Probing in `supabase_sync.rs`
1. In `candidate_repo_secrets_paths()`:
   - Check `std::env::var("REPO_SECRETS_DIR")`. If defined:
     - Push raw path if it is a JSON file.
     - Push `03-supabase/01-own/supabase-credentials.json`.
     - Push `03-supabase/02-lovable/supabase-credentials.json`.
     - Push `supabase-credentials.json`.
   - Ensure the standard drive locations are probed:
     - `D:/work/repo-secrets/03-supabase/01-own/supabase-credentials.json`
     - `D:/work/repo-secrets/03-supabase/02-lovable/supabase-credentials.json`
     - `C:/work/repo-secrets/03-supabase/01-own/supabase-credentials.json`
     - `C:/work/repo-secrets/03-supabase/02-lovable/supabase-credentials.json`
   - Include monorepo relative candidate paths:
     - `../repo-secrets/03-supabase/01-own/supabase-credentials.json`
     - `../../repo-secrets/03-supabase/01-own/supabase-credentials.json`
     - `repo-secrets/03-supabase/01-own/supabase-credentials.json`
   - Check `dirs::home_dir()` subpaths.

### 3.2 UTF-8 BOM Stripping & Envelope Extraction
1. When reading candidate files in `auto_seed_from_repo_secrets`:
   - Strip leading UTF-8 BOM (`\u{feff}`) before parsing with `serde_json::from_str`.
   - Use `crate::modules::json_envelope::unpack_envelope` to handle both raw JSON maps and envelope-wrapped formats (`agm/supabase-credentials`).
   - Extract API URL, project reference, and keys (`service_role_key`, `secret_key`, or `anon_key`).

### 3.3 Endpoint Normalization & Deduplication
1. For every candidate endpoint:
   - Run URL through `normalize_supabase_url(url)` to strip trailing `/` and `/rest/v1`.
   - Check whether `cfg.endpoints` already contains an endpoint with the same normalized URL or ID.
   - If not found, append `new_ep`.
2. If at least one endpoint was newly added:
   - Set `cfg.is_sync_enabled = true`.
   - Sort `cfg.endpoints` by `priority` ascending.
   - Return `true` so `load_config` saves the populated configuration to disk immediately.

### 3.4 CLI Dual-Field Synchronization in `agm.rs`
1. Locate `cmd_supabase_set_config` in `src-tauri/src/bin/agm.rs`.
2. When parsing the `--cooldown <minutes>` or `-c <minutes>` flag:
   ```rust
   if let Some(cd) = cooldown_mins {
       app_cfg.auto_profile_switcher.account_lockout_window_minutes = cd;
       app_cfg.auto_profile_switcher.account_cooldown_minutes = cd;
       app_changed = true;
   }
   ```
3. Update the help message and status display in `cmd_supabase_set_config` and `status` to clearly print both `Account Lockout Window` and `Account Cooldown Minutes`.

---

## 4. Constraints & Invariants
- **Zero-Build & Zero-Test Execution**: Do not run `cargo build` or `cargo test` during this authoring task.
- **No Git Commands**: Under no circumstances execute `git add`, `git commit`, or `git push`.
- **Preserve Existing Documentation**: Do not remove comments, docstrings, or license headers in modified files.
- **Fail-Safe Fallbacks**: If repo-secrets files cannot be read or are missing, log a clean `tracing::warn` and proceed without panicking.

---

## 5. Acceptance Criteria

| # | Condition | Expected Outcome |
| :--- | :--- | :--- |
| **AC-1** | Empty `supabase_config.json` with existing `D:/work/repo-secrets/03-supabase/01-own/supabase-credentials.json` | `load_config()` automatically detects the file, registers the endpoint, sets `is_sync_enabled = true`, and persists to disk. |
| **AC-2** | Credentials file contains trailing slashes or `/rest/v1` | `normalize_supabase_url` cleanses the URL to base format (e.g., `https://xyz.supabase.co`). |
| **AC-3** | Duplicate credentials files in multiple candidate paths | Deduplication logic prevents duplicate entries in `cfg.endpoints`. |
| **AC-4** | Running `agm supabase set-config --cooldown 45` | Both `account_lockout_window_minutes` and `account_cooldown_minutes` are updated to `45` in `app_config.json`. |
| **AC-5** | `REPO_SECRETS_DIR` set to custom folder | Candidate search successfully probes the custom directory and subpaths. |
