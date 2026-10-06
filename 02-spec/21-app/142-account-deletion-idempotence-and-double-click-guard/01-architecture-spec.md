# Specification 142: Account Deletion Idempotence & Double-Click Guard Architecture

## 1. Executive Summary & Root Cause Analysis (RCA)

### 1.1 Error Signature
- **Error ID**: `76acaaf3-5e95-47f3-8cb2-b2650e40ed23`
- **Error Code**: `E9001`
- **Message**: `Account ID not found: e3536dda-b192-494d-ad28-fd74dd36a22f`
- **Source**: `tauri.delete_account` / `delete_account`
- **Route**: `/accounts`

### 1.2 Root Cause Analysis
1. **Frontend Double-Click & Missing In-Flight Guard**:
   In `src/pages/Accounts.tsx`, `executeDelete` was triggered asynchronously upon confirming the delete dialog. However:
   - No `isDeleting` state was tracked.
   - `ModalDialog` confirm and cancel buttons did not receive `isLoading`.
   - The user rapidly double-clicked the confirm "Delete" button (clicks at `10:48:27.544` and `10:48:27.681`, 137ms apart).
   - This dispatched two concurrent `delete_account` IPC calls with the exact same `account_id`.
2. **Backend Non-Idempotent Deletion Error**:
   In `src-tauri/src/modules/account.rs:1420-1422`:
   ```rust
   let original_len = index.accounts.len();
   index.accounts.retain(|s| s.id != account_id);
   if index.accounts.len() == original_len {
       return Err(format!("Account ID not found: {}", account_id));
   }
   ```
   The first call successfully removed the account from the index and file system.
   The second concurrent call arrived, found the account was already absent from the index, and returned a hard error `Account ID not found`, triggering an unhandled exception and error dialog `E9001`.

---

## 2. Architecture & Solution Design

### 2.1 Backend Idempotency Invariant
Resource deletion operations MUST be idempotent:
- If `account_id` is found in the index, remove it, reassign `current_account_id` if necessary, and persist `save_account_index`.
- If `account_id` is already missing from the index, log an informational/warning note and continue cleanly rather than failing with `Err`.
- If the account JSON file exists on disk, remove it.
- Emit token cleanup signal `crate::proxy::server::trigger_account_delete(account_id)`.
- Always return `Ok(())`.

### 2.2 Frontend Double-Click & Concurrency Guard
- Introduce `isDeleting` boolean state in `src/pages/Accounts.tsx`.
- Pass `isLoading={isDeleting}` to `ModalDialog`, which disables both confirm and cancel buttons and displays a spinner.
- Guard `executeDelete` and `executeBatchDelete`: return early if `isDeleting` is true.
- Close the modal or prevent any repeated triggers while deletion is in flight.

---

## 3. Strict Relative Path Invariant
All affected files strictly use relative Git paths:
- `02-spec/21-app/142-account-deletion-idempotence-and-double-click-guard/`
- `.ai-memory/plans/completed/142-account-deletion-idempotence-and-double-click-guard.md`
- `src-tauri/src/modules/account.rs`
- `src/pages/Accounts.tsx`
