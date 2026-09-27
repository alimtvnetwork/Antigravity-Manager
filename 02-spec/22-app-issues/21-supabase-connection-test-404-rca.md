# Issue 21: Supabase Connection Test 404 & URL Normalization RCA

## Executive Summary
On the `/settings` page under Supabase Synchronization (`SupabaseSyncSettings.tsx`), operators attempting to test a newly saved Supabase endpoint encountered an intrusive fatal application error modal (`Error ID: 6f9cb5b8-ab15-4606-9e67-4541145d660d`, Code `E1002`, `Network error: Supabase connection test failed with status: 404`). Investigation revealed two critical defects:
1. **OpenAPI Spec Root Deprecation on Supabase**: Supabase has deprecated and blocked access to the PostgREST root OpenAPI specification (`/rest/v1/`) for `anon` API keys and deployments where OpenAPI is toggled off for schema security. The previous `test_connection()` probe solely queried `GET /rest/v1/`, which directly returns HTTP 404.
2. **URL Suffix Duplication**: When operators entered or pasted full REST URLs (e.g. `https://<ref>.supabase.co/rest/v1` or `https://<ref>.supabase.co/rest/v1/`), `SupabaseClient` only trimmed trailing slashes and then blindly appended `/rest/v1/` or `/rest/v1/<table_name>`, creating invalid paths like `.../rest/v1/rest/v1/nodes` that unconditionally fail with 404.
3. **Inappropriate Error Store Capture on Diagnostic Probes**: `supabaseService.testEndpoint` wrapped IPC calls with `useErrorStore.captureError()`, treating user connectivity tests (where errors like bad credentials, network timeouts, or unmigrated schemas are expected feedback) as fatal unhandled application crashes.

---

## 1. Symptoms & Incident Walkthrough
1. **User Added Supabase Endpoint:** The user navigated to `/settings`, opened the Supabase connection modal, input the project URL, anon key, selected database role, and clicked "Save Endpoint".
2. **User Clicked "Test":** The user immediately clicked the "Test" button to verify credentials and connectivity before running schema migrations.
3. **Application Threw Error Modal:** Instead of displaying an inline toast or badge, the application captured a high-severity error modal (`E1002: Network error: Supabase connection test failed with status: 404`).

---

## 2. Root Cause Analysis (4-Part RCA)

### 2.1 Symptom
```text
Network error: Supabase connection test failed with status: 404
Route / Page: /settings
Source: SupabaseService
Error ID: 6f9cb5b8-ab15-4606-9e67-4541145d660d
```

### 2.2 Root Cause
The Supabase connection probe relied exclusively on `GET /rest/v1/` without URL normalization or multi-stage fallback, causing HTTP 404 whenever Supabase blocked OpenAPI schema inspection for `anon` keys or when operators pasted URLs ending with `/rest/v1`, while frontend diagnostic handlers erroneously forwarded test rejections to the global crash reporter.

### 2.3 Resolution
1. **Multi-Stage Resilient Probe Ladder (`SupabaseClient::test_connection`)**:
   - **Probe 1 (REST Root):** Checks `/rest/v1/`. If 200..399, connection verified. If 401/403, returns explicit authentication failure details without fatal crashes.
   - **Probe 2 (PostgREST Table & Schema Detection):** If Probe 1 returns 404, probes table endpoints (`/rest/v1/nodes?limit=0` and `/rest/v1/command_queue?limit=0`). If 200, tables exist. If 404 with PostgREST JSON signature (`PGRST204`, `PGRST205`, `relation ... does not exist`, `schema cache`), identifies that PostgREST is online and auth is valid, prompting user to run schema migrations.
   - **Probe 3 (Supabase Service Health):** Falls back to `/auth/v1/health` (GoTrue health check), verifying that the Supabase host is alive and responding.
2. **Canonical URL Normalization (`normalize_supabase_url`)**:
   - Strips whitespace, redundant trailing slashes, and `/rest/v1` or `/rest/v1/` suffixes across backend (`SupabaseClient`, `supabase_sync::save_config`, `supabase_sync::load_config`) and frontend (`SupabaseSyncSettings.tsx` onBlur, submit, and toast).
3. **Structured Non-Fatal Diagnostic Results (`EndpointTestResult`)**:
   - `test_supabase_endpoint` returns `Ok(EndpointTestResult { is_success, message, status_code })` instead of throwing IPC errors.
   - Removed intrusive `useErrorStore.captureError()` from `supabaseService.testEndpoint` and `checkEndpointTables`.

### 2.4 Prevention & Learnings
- **Diagnostic Probes Are Not Fatal Crashes**: Connection test buttons and schema verifiers must return structured result envelopes (`is_success`, `message`) rather than rejecting promises or triggering global error modals.
- **Always Normalize Provider Endpoints**: Third-party cloud providers (Supabase, Firebase, OpenRouter, Claude) exhibit varying dashboard URL formats. Always normalize base endpoints (strip trailing slashes, strip version prefixes) to ensure deterministic path concatenation.

---

## 3. Verification & Evidence
- **Rust Unit Tests:** Added 3 tests in `src-tauri/src/modules/supabase_client.rs` verifying URL normalization across 9 edge cases, table/rpc URL construction without duplication, and `is_connected` verification logic (`cargo test --lib modules::supabase_client` passed 3/3).
- **Frontend Compilation:** `npm run build` completed with zero TypeScript errors.
- **Cargo Gate:** Passed `cargo fmt -- --check` and `cargo clippy --all-targets --all-features`.
