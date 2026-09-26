# Proxy Pool Bindings "Service not running" Modal & Configuration Fallback RCA

- **ID:** `E9001`
- **Symptom:** Opening Settings -> Proxy tab or Proxy Binding Manager displayed an intrusive diagnostic modal with `Service not running` from `tauri.get_all_account_bindings` / `get_proxy_pool_config`.
- **Root Cause:** Backend proxy commands returned `Err("Service not running")` / `Err("服务未运行")` when proxy service was offline instead of falling back to persistent `AppConfig`, and frontend callers lacked suppression flags in utility requests.
- **Resolution:**
  1. Updated `get_proxy_pool_config`, `get_all_account_bindings`, `get_preferred_account`, and `set_preferred_account` to safely fall back to `AppConfig` when `instance.is_none()`.
  2. Updated `src/utils/request.ts` to sanitize `_suppressGlobalModal` from IPC and Web requests and suppress global modal triggers when `suppressGlobalModal` is true.
  3. Added `{ _suppressGlobalModal: true }` to `ProxyPoolSettings.tsx` and `ProxyBindingManager.tsx`.
- **Reference:** [`02-spec/22-app-issues/13-proxy-pool-bindings-service-not-running-rca.md`](../../../02-spec/22-app-issues/13-proxy-pool-bindings-service-not-running-rca.md)
