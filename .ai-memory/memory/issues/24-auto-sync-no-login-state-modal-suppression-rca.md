# Auto Sync No Login State Modal Suppression & Token Failure Caching RCA

- **ID:** `E9001`
- **Symptom:** In `/email` and other views, background sync periodically popped up an intrusive red diagnostic modal with `No login state data found across Keyring, IDE databases, or CLI directories` from `tauri.sync_account_from_db`.
- **Root Cause:** Speculative background sync called `import_from_db` on stale/revoked OAuth tokens found in local storage, which failed at Google's token endpoint with zero imported accounts. The error was bubbled with `?` across Tauri IPC without modal suppression or failed token memoization, causing repeated error popups every 4 minutes.
- **Resolution:**
  1. Added `LAST_FAILED_SYNC_TOKEN` in `src-tauri/src/commands/mod.rs` to cache tokens that fail refresh, bypassing expensive repeated queries.
  2. Handled `import_from_db` failures gracefully in `sync_account_from_db` and `admin_sync_account_from_db`, returning `Ok(None)` instead of throwing.
  3. Added `{ _suppressGlobalModal: true }` to `syncAccountFromDb()` in `src/services/accountService.ts`.
  4. Enhanced descriptive warnings in `src-tauri/src/modules/migration.rs` when candidate local OAuth tokens fail refresh.
- **Reference:** [`02-spec/22-app-issues/24-auto-sync-no-login-state-modal-suppression-rca.md`](../../../02-spec/22-app-issues/24-auto-sync-no-login-state-modal-suppression-rca.md)
