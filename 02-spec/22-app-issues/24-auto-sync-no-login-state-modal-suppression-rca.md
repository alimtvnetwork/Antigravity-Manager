# Issue 24: Auto Sync No Login State Modal Suppression & Token Failure Caching RCA

> **/goal** Provide a grounded 4-part Root Cause Analysis (RCA) detailing why background database account auto-sync triggered invasive Comprehensive Error Diagnostic popups (Error E9001) across UI views, and how token caching and error suppression remediated the issue.
> **/learn** Background polling and synchronization tasks must never bubble non-fatal probe failures to the global UI error modal; failed refresh tokens must be cached to prevent recurring refresh cycles against expired local state.

**Severity:** High
**Affects:** Desktop App UI (`/email`, `/settings`, `BackgroundTaskRunner.tsx`), Tauri IPC Commands (`sync_account_from_db`), Proxy Admin API (`admin_sync_account_from_db`)
**Status:** Remediated & Verified

---

## 1. Symptoms & Incident Summary

- **Application:** Agm Tool By Alim (v4.91.0)
- **Error ID:** `f9ba48dd-e6b7-4f26-bb8d-424d97b3287c`
- **Error Code:** `E9001`
- **Severity Level:** `ERROR`
- **Message:** `No login state data found across Keyring, IDE databases, or CLI directories`
- **Route / Page:** `/email` (and background views)
- **Trigger Action:** `tauri_invoke`
- **Source / Endpoint:** `tauri.sync_account_from_db`
- **User Experience Impact:** While the user navigated the UI (e.g. `/email` or `/settings`), a full-screen red Comprehensive Error Diagnostic Report modal popped up repeatedly every 4 minutes, disrupting user workflows even though the user had active operational accounts in Antigravity Manager.

---

## 2. Root Cause Analysis (4-Part Deep Dive)

### A. Immediate Cause
The background task runner (`BackgroundTaskRunner.tsx`) periodically triggers `syncAccountFromDb()` via Tauri IPC (`sync_account_from_db`). In the Rust backend, `sync_account_from_db` attempts to discover and import local accounts by calling `modules::migration::import_from_db(current_target)`. When offline IDE databases or Windows Credential Manager hold stale/revoked OAuth tokens, Google's token refresh endpoint rejects them, resulting in zero valid imported accounts and returning `Err("No login state data found...")`. 

Previously, `sync_account_from_db` used the `?` operator to bubble this error back across the IPC boundary to the frontend. In the frontend IPC client (`src/utils/request.ts`), uncaught errors automatically invoke `useErrorStore.getState().captureError()`, rendering the high-severity red diagnostic modal.

### B. Contributing Factors
1. **Uncached Stale Local Tokens**:
   `get_refresh_token_from_db` reads tokens offline from Keyring or SQLite databases (`state.vscdb`). When a token is expired or revoked on Google's servers, the token comparison (`db_refresh_token != current_token`) still evaluates to true on every background cycle because the active account in AGM has a different token. Without caching failed tokens, AGM re-attempted the invalid import every 4 minutes.
2. **Missing Global Modal Suppression in Frontend Service**:
   `syncAccountFromDb()` in `src/services/accountService.ts` called `invoke('sync_account_from_db')` directly without setting `_suppressGlobalModal: true`.
3. **Hard Failure Semantics on Speculative Background Probes**:
   Background account sync is designed as a convenience background probe, not a user-initiated synchronous blocking transaction. Treating the absence of importable local state as a fatal error violated the design guidelines for background task isolation.

### C. Systemic Gaps
- Lack of failure memoization for external OAuth verification in local database migrations.
- Failure to enforce `_suppressGlobalModal: true` as a standard pattern for all autonomous background runners.

---

## 3. Remediation & Fix Strategy

1. **Failure Token Caching in Rust Backend (`sync_account_from_db`)**:
   - Implemented `static LAST_FAILED_SYNC_TOKEN: Lazy<Mutex<Option<String>>>` to memoize tokens that fail local import.
   - If the discovered `db_refresh_token` matches the cached failed token, immediately skip and return `Ok(None)` without performing expensive OAuth requests.
   - On successful import, clear the failed token cache (`*guard = None`).
   - On import failure, store the failed token (`*guard = Some(db_refresh_token)`), log an informational message (`log_info`), and return `Ok(None)` gracefully instead of bubbling an error.

2. **Proxy Admin API Graceful Fallback (`admin_sync_account_from_db`)**:
   - In `src-tauri/src/proxy/server.rs`, handled `migration::import_from_db` failure gracefully by logging info and returning `Ok(Json(None))` instead of HTTP 500.

3. **Descriptive Warning Logging in Migration Engine (`import_all_local_accounts`)**:
   - In `src-tauri/src/modules/migration.rs`, replaced silent discard or uninformative errors with warning logs detailing that candidate Keyring or DB OAuth tokens failed refresh (likely expired or revoked).

4. **Frontend Modal Suppression (`syncAccountFromDb`)**:
   - Updated `src/services/accountService.ts` to call `invoke('sync_account_from_db', { _suppressGlobalModal: true })`, ensuring that even unexpected network or IPC anomalies never trigger intrusive UI modals.

---

## 4. Prevention & Verification Guidelines

- **Pre-flight Conformance**: Verify that `cargo fmt -- --check`, `cargo clippy --all-targets --all-features`, and `npm run build` pass cleanly.
- **Background Task Isolation Invariant**: All background runners in `BackgroundTaskRunner.tsx` and associated services must pass `{ _suppressGlobalModal: true }` on IPC invocations.
- **Speculative Probe Invariant**: Any speculative auto-discovery or auto-sync command must return `Ok(None)` / `null` when candidate data is missing or invalid, never `Err(...)`.
