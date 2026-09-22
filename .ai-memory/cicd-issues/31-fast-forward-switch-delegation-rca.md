# RCA 31: Fast-Forward Button Switch Delegation Failure

## 1. Executive Summary
Button 1 (Fast-Forward `>>` in `InstanceSelector.tsx`) failed to rotate accounts because it bypassed the central `switchAccount` pipeline and invoked a low-level sandbox injector (`switchAccountToInstance`) that fails on default profiles and leaves UI stores desynchronized. Button 2 (Row Switch `⇄` in `AccountTable.tsx`) succeeds because it delegates to `useAccountStore.getState().switchAccount`, invoking the complete `switch_account` backend command with tray updates, proxy reloads, and `currentAccount` synchronization. The solution refactors Button 1 to delegate its execution directly to Button 2's proven `switchAccount` pipeline after selecting the logical candidate workspace/account.

## 2. Telemetry & Failure Trace
- **Button 1 Call Chain:**
  `InstanceSelector.tsx` (`handleSmartRotate`)
  -> `useInstanceStore.ts` (`smartRotateProfileAccount`)
  -> `instanceService.ts` (`switchAccountToInstance`)
  -> Tauri command `switch_account_to_instance`
  -> `modules::instance::switch_account_to_instance`
  -> `registry.instances.find(id == "default")` -> **Error: Target instance default not found**
- **Button 2 Call Chain:**
  `AccountTable.tsx` / `AccountRow.tsx` (`onSwitch`)
  -> `Accounts.tsx` (`handleSwitch`)
  -> `useAccountStore.ts` (`switchAccount`)
  -> Tauri command `switch_account`
  -> `commands::mod::switch_account`
  -> `AccountService::switch_account` -> Injects tokens into active IDE
  -> `tray::update_tray_menus` -> Tray synced
  -> `proxy::reload_proxy_accounts` -> Proxy reloaded
  -> `useAccountStore::fetchCurrentAccount` -> UI synced

## 3. Root Cause Analysis
1. **Parallel Execution Paths:** Two divergent code paths existed for account switching: one unified path for `AccountRow` / `AccountTable` and an incomplete bypass for `InstanceSelector`.
2. **Missing Desktop Environment Support in Instance Injector:** `switch_account_to_instance` was only built for directory-isolated sandbox instances (`~/.antigravity-manager/instances/<id>/data`) and lacked desktop IDE integration for default profiles.
3. **Store Incoherence:** The bypass in `useInstanceStore` omitted `fetchCurrentAccount()` in `useAccountStore`, preventing the UI from reflecting the newly active account badge.

## 4. Remediation Plan
1. Refactor `smartRotateProfileAccount` in `useInstanceStore.ts` to delegate to `useAccountStore.getState().switchAccount(verified.account.id, targetIdeParam)`.
2. Target IDE resolution:
   - If `instId === 'default'`, set `targetIdeParam = undefined`.
   - If `instId !== 'default'`, set `targetIdeParam = 'instance:' + instId`.
3. Synchronize instances and account stores after switch completion.
4. Harmonize `smartPlayInstance` in `useInstanceStore.ts` with the same delegation logic.
5. Add event stopping (`e.stopPropagation()`) in `InstanceSelector.tsx`.
