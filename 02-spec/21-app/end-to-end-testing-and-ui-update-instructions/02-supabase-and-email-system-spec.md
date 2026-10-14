# Supabase Credential Diagnostics, One-Liner Import/Export & Language-Agnostic Email System Specification

## 1. Supabase Credential Root-Cause Analysis & Fixes

### 1.1 Root Causes Identified
1. **Local Endpoint Pollution & Domain Typo in `~/.antigravity_tools/supabase_config.json`**:
   - The local configuration contained 4 endpoints instead of the 2 canonical endpoints:
     - Two stale mock placeholder endpoints (`ep-root-lovable-01` -> `https://mock-root.supabase.co` with `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.mock-root` and `ep-secondary-01` -> `https://mock-secondary.supabase.co`).
     - One endpoint with a character-transposition typo in its subdomain (`ep-lovable-root-supabase` -> `https://pezjuuddecbyrmqxytrv.supabase.co` instead of `https://pezjuuddecbyfmqxytrv.supabase.co`, `rmq` vs `fmq`).
   - `auto_seed_from_repo_secrets` in `src-tauri/src/modules/supabase_sync/config_a.rs` and `cmd_supabase_load_json` in `src-tauri/src/bin/agm/supabase_config_cmds.rs` checked only `cfg.endpoints.iter().any(|e| e.id == ep.id)` without updating existing mock entries in-place or purging placeholder endpoints.
   - `../repo-secrets/02-antigravity-and-event-manager/vault/supabase_config.json` had an empty `"endpoints": []` array while `../repo-secrets/02-antigravity-manager/vault/supabase_config.json` held the 2 canonical endpoints.
   - `scripts/import-supabase-secrets.ps1` read `03-supabase/01-own/supabase-credentials.json` without decoding its `"encoding_format": "base64"` payload fields (`supabase_endpoint`, `anon_public_key`, `project_id`), and `scripts/setup-supabase.ps1` had duplicate `$RootUrl`/`$RootKey`/`$PersonalUrl`/`$PersonalKey` entries in its `param(...)` block causing a PowerShell parser error.
2. **Remote Cloud Project Status (`NXDOMAIN` on Paused Free-Tier Projects)**:
   - Public DNS resolution (`Resolve-DnsName`) for both `pezjuuddecbyfmqxytrv.supabase.co` and `ikwmurmjynhxdpmhzekt.supabase.co` returns `DNS name does not exist` (`NXDOMAIN`), while `supabase.co` resolves normally (`76.76.21.21`).
   - Supabase Cloud pauses free-tier projects after 7 days of inactivity, unpublishing `<project_ref>.supabase.co` DNS records until restored in the Supabase/Lovable dashboard (`supa-base-agm-v2@pxdmail.com`).

### 1.2 Fixes Applied
- **`src-tauri/src/modules/supabase_client/types.rs`**:
  - Extended `normalize_supabase_url` to normalize `pezjuuddecbyrmqxytrv.supabase.co` -> `pezjuuddecbyfmqxytrv.supabase.co`.
  - Added `is_placeholder_or_mock_endpoint(&SupabaseEndpoint)` to detect `mock-*`, `your-project.supabase.co`, or `.mock` API keys.
- **`src-tauri/src/modules/supabase_sync/config_a.rs` & `config_b.rs`**:
  - Updated `auto_seed_from_repo_secrets` and `auto_discover_supabase_credentials` to update matching endpoints in-place by `id` or normalized `url`, purge placeholder/mock endpoints when real endpoints exist, and deduplicate by normalized URL.
- **`src-tauri/src/bin/agm/supabase_config_cmds.rs`**:
  - Updated `cmd_supabase_load_json` to normalize URLs, support `url`/`api_key`/`role`/`tag`/`notes`, purge mock endpoints, and deduplicate by normalized URL.
- **`src-tauri/src/modules/supabase_client/client.rs`**:
  - Enhanced `classify_transport_error` to detect DNS `NXDOMAIN` (`os error 11001` / `no such host`) and report an explicit diagnostic explaining that the Supabase free-tier project is paused or its DNS record is unpublished.
- **`scripts/import-supabase-secrets.ps1` & `scripts/setup-supabase.ps1`**:
  - Added Base64 envelope decoding to `scripts/import-supabase-secrets.ps1`, fixed duplicate parameter declarations in `scripts/setup-supabase.ps1`, and converted all paths to relative paths.

### 1.3 Supabase Import & Export One-Liners
```bash
# Auto-discover and import from repo-secrets vault into ~/.antigravity_tools/supabase_config.json
agm supabase load-secrets

# Import from explicit JSON file
agm supabase load-json ../repo-secrets/02-antigravity-manager/vault/supabase_config.json

# Export current Supabase endpoints to portable JSON envelope
agm supabase export --file supabase-backup.json

# PowerShell one-liner (decodes Base64 03-supabase vaults and imports into AGM)
powershell -ExecutionPolicy Bypass -File scripts/import-supabase-secrets.ps1 -RepoSecretsDir "../repo-secrets/03-supabase"
```

---

## 2. Password-Only Email Import One-Liner & Language-Agnostic Email Specification

### 2.1 CLI & Vault Enhancements
- **`src-tauri/src/modules/email_io/json.rs`**:
  - Added `extract_email_and_has_password(payload)` and `import_from_json_with_password(payload, password_override)` supporting both the full `EmailExportBundle` (`agm-email-config.json`) and the compact credential spec (`email-credentials.json` or inline JSON with `smtp: {host, port, security}`, `imap: {host, port, security}`, `notify_recipient`, `settings`).
- **`src-tauri/src/bin/agm/email_arms_b.rs` & `email_arms_a.rs`**:
  - Enhanced `agm email import` to accept `--json <inline_json>`, `-f <file>`, `--password <pwd>`, and `--ask-password` (or auto-prompt on stdin when the JSON payload has no embedded password).
  - Enhanced `agm email add` to accept `--encryption SSL|TLS`.
- **`src-tauri/src/modules/email_vault_db/recipients.rs`, `accounts.rs` & `db.rs`**:
  - Deduplicated `notify_recipients` case-insensitively by `LOWER(TRIM(email))` with a unique index (`idx_notify_recipients_email_lower`) and made `upsert_email_account` reuse existing account IDs when matched by `LOWER(TRIM(email))`.

### 2.2 Single-Line Password-Only Email Import Commands
```bash
# Cross-platform AGM CLI one-liner (embeds all SMTP 465 SSL / IMAP 993 SSL / recipient / settings JSON; prompts ONLY for password)
agm email import --ask-password --json '{"account_id":"3f4733d9-b150-41ae-a219-ccc628dfc610","alias":"ai-agm-tool-v1 (hire-seoexperts.com)","email":"ai-agm-tool-v1@hire-seoexperts.com","smtp":{"host":"mail.hire-seoexperts.com","port":465,"security":"SSL"},"imap":{"host":"mail.hire-seoexperts.com","port":993,"security":"SSL"},"notify_recipient":"devorg.bd@gmail.com","is_default":true,"is_active":true,"settings":{"is_enabled":true,"polling_interval_minutes":3,"inbox_check_interval_minutes":1,"notify_on_quota_drop":true,"quota_drop_threshold_percent":15,"notify_on_workspace_switch":true,"notify_on_idle_workspace":true,"notify_on_system_update":true,"allow_remote_prompt_execution":true,"allow_remote_cli_execution":true,"allow_remote_instance_rotation":true}}'

# File-based password-only import (reads non-secret metadata from email-credentials.json and prompts ONLY for password)
agm email import --ask-password -f ../repo-secrets/02-antigravity-manager/vault/email-credentials.json

# Export full email configuration to JSON envelope
agm email export -f ../repo-secrets/02-antigravity-and-event-manager/vault/agm-email-config.json
```
