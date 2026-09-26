# Proxy Pool Bindings "Service not running" Modal & Configuration Fallback RCA

**Version:** 1.0.0
**Updated:** 2026-09-27
**Severity:** High
**Status:** Fixed

---

## 1. Reproduction

1. Launch Antigravity-Manager without starting the internal API proxy service.
2. Navigate to **Settings** (`/settings`) in the application navigation bar.
3. Click the **Proxy** tab (or open the **Proxy Binding Manager** modal).
4. Observe that:
   - An intrusive error diagnostic report modal pops up with:
     - **Error Code:** `E9001`
     - **Message:** `Service not running`
     - **Trigger Action:** `tauri_invoke`
     - **Source:** `tauri.get_all_account_bindings`
   - Background polling for `get_proxy_pool_config` periodically threw backend errors (`"服务未运行"`), which also risked triggering the global error capture modal.

---

## 2. Cause

1. **Missing Service-Stopped Fallback in Backend Proxy Commands**:
   - In [`src-tauri/src/commands/proxy_pool.rs`](../../src-tauri/src/commands/proxy_pool.rs) and [`src-tauri/src/commands/proxy.rs`](../../src-tauri/src/commands/proxy.rs), `get_all_account_bindings`, `get_proxy_pool_config`, and `get_preferred_account` assumed the Axum proxy service was running. When `instance_lock.as_ref()` was `None`, they returned `Err("Service not running".to_string())` or `Err("服务未运行".to_string())` instead of reading existing settings from [`AppConfig`](../../src-tauri/src/modules/config.rs).
2. **Missing Modal Suppression on Passive Settings Queries**:
   - In [`src/components/settings/ProxyPoolSettings.tsx`](../../src/components/settings/ProxyPoolSettings.tsx) and [`src/components/settings/proxy/ProxyBindingManager.tsx`](../../src/components/settings/proxy/ProxyBindingManager.tsx), `fetchBindings()` and the proxy status poller called `request('get_all_account_bindings')` and `request('get_proxy_pool_config')` without passing `{ _suppressGlobalModal: true }`.
3. **Unsanitized Control Arguments in Request Utility**:
   - In [`src/utils/request.ts`](../../src/utils/request.ts), the `_suppressGlobalModal` flag was not sanitized or stripped from arguments before passing them into Tauri's `invoke(cmd, args)` or Web `fetch` request bodies/query strings.

---

## 3. Fix

1. **Persistent Configuration Fallbacks for Proxy & Binding Commands (`src-tauri/src/commands/proxy_pool.rs`, `src-tauri/src/commands/proxy.rs`)**:
   - Updated `get_all_account_bindings`, `get_account_proxy_binding`, `bind_account_proxy`, and `unbind_account_proxy` to fall back to `load_app_config()?.proxy.proxy_pool.account_bindings` when the proxy service is offline.
   - Updated `get_proxy_pool_config`, `get_preferred_account`, and `set_preferred_account` to read and persist to `AppConfig` directly when `instance.is_none()`.
2. **Modal Suppression Flag Sanitization (`src/utils/request.ts`)**:
   - Extracted `suppressGlobalModal` and created sanitized argument objects (`cleanedArgs`, `bodyArgs`), stripping `_suppressGlobalModal` so backend IPC parameters and HTTP endpoints receive clean payloads.
   - Enforced `if (!suppressGlobalModal)` in both Tauri IPC and Web fetch catch handlers before dispatching errors to `useErrorStore.getState().captureError(...)`.
3. **Suppressed Global Modal on Passive Settings Fetches (`src/components/settings/ProxyPoolSettings.tsx`, `src/components/settings/proxy/ProxyBindingManager.tsx`)**:
   - Added `{ _suppressGlobalModal: true }` to `get_all_account_bindings` and `get_proxy_pool_config` calls in `ProxyPoolSettings.tsx` and `ProxyBindingManager.tsx`.

---

## 4. Prevention

- **Offline-First Configuration Access**: Passive configuration read/write commands must always fall back to local disk storage (`AppConfig`) when runtime service daemons are not running.
- **Background Error Isolation**: Always mark background pollers, passive mount fetches, and non-fatal telemetry checks with modal suppression flags to prevent degrading the user experience with unexpected error popups.
- **Frontend Utility Sanitization**: Any meta-flags used by frontend utilities (such as `_suppressGlobalModal`) must be stripped before forwarding payloads across IPC or network boundaries.
