# Plan 80: Supabase Connection Test 404 & URL Normalization Fix

## Metadata
- **Status:** COMPLETED
- **Category:** Bug Fix / Supabase Diagnostics
- **Spec Reference:** `02-spec/21-app/60-supabase-connection-probe-and-url-normalization.md`
- **RCA Reference:** `02-spec/22-app-issues/21-supabase-connection-test-404-rca.md`
- **Date:** 2026-09-27

---

## 1. Goal
Diagnose and resolve the `E1002` error (`Supabase connection test failed with status: 404`) triggered on `/settings` when testing newly configured Supabase endpoints, eliminating false-positive error modals and ensuring robust connection testing.

---

## 2. Completed Steps

### Step 1: Root Cause Analysis
- Identified that Supabase blocks root OpenAPI specification inspection (`/rest/v1/`) for `anon` keys and projects where OpenAPI docs are disabled.
- Identified URL suffix duplication (`.../rest/v1/rest/v1`) when operators entered URLs already including `/rest/v1`.
- Identified that `useErrorStore.captureError()` was inappropriately capturing connection probe rejections.

### Step 2: Backend URL Normalization & Probe Ladder
- Added `normalize_supabase_url()` in `src-tauri/src/modules/supabase_client.rs`.
- Applied URL normalization in `SupabaseClient::new()`, `supabase_sync::save_config()`, and `supabase_sync::load_config()`.
- Implemented 3-stage resilient probe ladder in `SupabaseClient::test_connection()`:
  - Probe 1: `GET /rest/v1/`
  - Probe 2: Table probe (`/rest/v1/nodes?limit=0` & `command_queue?limit=0`) with PostgREST `PGRST` signature detection
  - Probe 3: Auth health probe (`/auth/v1/health`)
- Fixed `is_connected` logic in `verify_expected_tables`.
- Structured `test_supabase_endpoint` IPC command to return `Ok(EndpointTestResult)`.

### Step 3: Frontend Service & Settings UI Polish
- Updated `src/services/supabaseService.ts`:
  - Added `EndpointTestResult` interface.
  - Removed `useErrorStore.captureError` from `testEndpoint` and `checkEndpointTables`.
- Updated `src/components/settings/SupabaseSyncSettings.tsx`:
  - Added `normalizeSupabaseUrl()` on input blur and form submit.
  - Added helper note clarifying project URL expectations.
  - Polished inline badges and toasts with informative messages.

### Step 4: Verification
- Added 3 unit tests in `src-tauri/src/modules/supabase_client.rs` covering URL normalization, URL building, and `is_connected` logic (`cargo test --lib modules::supabase_client` passed).
- Ran `npm run build` (passed code 0).
- Ran `cargo fmt -- --check` and `cargo clippy --all-targets --all-features`.
