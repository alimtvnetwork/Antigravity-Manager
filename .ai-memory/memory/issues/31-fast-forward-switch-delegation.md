# Issue 31: Fast-Forward Button Switch Delegation Failure

## Part 1: Why It Happened
When clicking Button 1 (the Fast-Forward `>>` button in the top navigation bar `InstanceSelector.tsx`), the application failed to switch the active account or workspace profile as expected, leaving users without the expected profile rotation and occasionally throwing silent or unhandled exceptions. In contrast, Button 2 (the row switch button `⇄` in the Accounts table `AccountRow.tsx` / `AccountTable.tsx`) reliably and instantly switched accounts, updated the active status badge, refreshed system tray menus, reloaded proxy session bindings, and persisted credentials into the running IDE's database.

## Part 2: How It Happened
1. When Button 2 (`⇄` row switch) was clicked in `AccountTable.tsx`:
   - It invoked `onSwitch()` which mapped to `handleSwitch(account.id)` in `src/pages/Accounts.tsx`.
   - `handleSwitch` called `useAccountStore.getState().switchAccount(accountId, targetIde)`.
   - In `useAccountStore.ts`, `switchAccount` invoked the Tauri backend command `switch_account`.
   - In `src-tauri/src/commands/mod.rs`, `switch_account` checked whether the target was an isolated instance or the default IDE. For default IDEs, it leveraged desktop `AccountService::switch_account` to inject credentials directly into `%APPDATA%/Antigravity/User/globalStorage/state.vscdb`, updated tray menus (`update_tray_menus`), and reloaded proxy account pools (`reload_proxy_accounts`).
   - On completion, `useAccountStore` immediately executed `fetchCurrentAccount()` to update the UI state.
2. When Button 1 (`>>` Fast-Forward) was clicked in `InstanceSelector.tsx`:
   - It triggered `handleSmartRotate(activeInstance?.config.id)`.
   - This delegated to `useInstanceStore.getState().smartRotateProfileAccount(targetInstanceId)`.
   - Although `smartRotateProfileAccount` correctly computed candidate account rankings via `rankSmartCandidates`, it bypassed the entire `useAccountStore.switchAccount` pipeline.
   - Instead, it directly called `instanceService.switchAccountToInstance(verified.account.id, instId)`.
   - In the backend (`src-tauri/src/modules/instance.rs`), `switch_account_to_instance` looked up `registry.instances` for the target instance ID. When `instId` was `"default"`, it could not find a matching isolated sandbox directory, throwing an error (`Target instance default not found`).
   - Furthermore, even if an instance was found, `switch_account_to_instance` wrote solely to sandbox directories, never updated the desktop IDE's database, never updated the system tray, never reloaded proxy accounts, and never updated `currentAccount` in `useAccountStore`.

## Part 3: Root Cause Analysis
1. **Architectural Divergence:** The fast-forward rotation in `useInstanceStore.ts` implemented a redundant, parallel account injection mechanism (`switchAccountToInstance`) rather than delegating the account switch to the unified, battle-tested `switchAccount` pipeline in `useAccountStore.ts`.
2. **Missing Default IDE Handling in Sandboxed Path:** `instance::switch_account_to_instance` assumes every target is a registered multi-instance sandbox in `registry.instances`. The default installation does not use an isolated multi-instance sandbox path, causing the sandbox injector to fail on default profiles.
3. **State Desynchronization:** Bypassing `useAccountStore.switchAccount` bypassed `fetchCurrentAccount()`, proxy session resets, and tray updates, causing the UI to show stale accounts even if files on disk were modified.

## Part 4: Corrective Action & Verification
1. **Refactor Fast-Forward to Delegate to Switch Account:**
   - In `useInstanceStore.ts` (`smartRotateProfileAccount`), after logically selecting and verifying the best candidate account via `rankSmartCandidates`, delegate the switch operation directly to `useAccountStore.getState().switchAccount(verified.account.id, targetIdeParam)`.
   - When `instId === 'default'`, pass `undefined` as `targetIdeParam` so `switch_account` routes to the standard desktop IDE installation.
   - When `instId !== 'default'`, pass `'instance:' + instId` so `switch_account` correctly targets the specific sandbox instance while still executing full lifecycle hooks (tray update, proxy reload, store synchronization).
2. **Harmonize `smartPlayInstance`:** Apply the same delegation pattern in `smartPlayInstance` to ensure consistent behavior across all instance launch actions.
3. **Event Propagation Safeguards:** Ensure `onClick` in `InstanceSelector.tsx` includes `e.stopPropagation()` to prevent unwanted parent container interference.
4. **Verification:** Run TypeScript checks, verify quality gates with `06-cicd-local-runner.py`, and validate that Button 1 executes the exact same underlying switch flow as Button 2.
