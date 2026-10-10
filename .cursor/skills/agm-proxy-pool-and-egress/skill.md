---
name: agm-proxy-pool-and-egress
description: Specialized skill for managing the Antigravity-Manager outbound egress proxy pool, SOCKS5/HTTP proxy rotation, health checks, latency scoring, strategy selection (RoundRobin, LeastUsed, Failover), and account-to-proxy bindings in src-tauri/src/proxy/proxy_pool.rs.
---

# AGM Outbound Egress Proxy Pool & Routing Subsystem

This skill provides comprehensive architectural guidance, selection algorithms, health checking protocols, and account binding procedures for the Outbound Egress Proxy Pool in **Antigravity-Manager (AGM)**.

---

## 1. Subsystem Architecture Overview

To prevent upstream IP-based rate limiting, geo-fencing, or account cross-contamination, AGM features a dynamic outbound proxy pool that routes upstream requests through multiple SOCKS5 or HTTP egress nodes:

```
+----------------------------------------------------------------------------------------------------+
|                                    Upstream Requests Initiated                                     |
|    - Google Gemini upstream calls (Daily / Sandbox / Prod)                                         |
|    - OAuth token refreshes & quota checks                                                          |
|    - ZAI PaaS vision calls                                                                         |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                  ProxyPoolManager (Singleton)                                      |
|                               src-tauri/src/proxy/proxy_pool.rs                                    |
|    - Account Binding Check: Does account_id have a dedicated pinned proxy?                         |
|      -> YES: Use bound proxy entry                                                                 |
|      -> NO: Apply selection strategy across active healthy proxies                                 |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                      Selection Strategies                                          |
|    - RoundRobin: Increments AtomicUsize across active pool                                         |
|    - LeastUsed: Selects proxy with minimum usage counter (DashMap<String, usize>)                  |
|    - Failover: Uses primary proxy; cascades to backup nodes upon failure                           |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                  Outbound TLS Client (rquest)                                      |
|    - Builds rquest::Proxy with SOCKS5 / HTTP authentication                                        |
|    - Preserves TLS fingerprinting via rquest_util Emulation                                        |
+----------------------------------------------------------------------------------------------------+
```

---

## 2. Key Files & Core Responsibilities

| File Path | Core Responsibilities |
|---|---|
| `src-tauri/src/proxy/proxy_pool.rs` | `ProxyPoolManager`, `PoolProxyConfig`, selection algorithms (`RoundRobin`, `LeastUsed`, `Failover`), `usage_counter`, account binding map, and health probe runners. |
| `src-tauri/src/commands/proxy_pool.rs` | Tauri IPC commands: `bind_account_proxy`, `unbind_account_proxy`, `get_account_proxy_binding`, `get_all_account_proxy_bindings`. Directly falls back to updating `app_config.json` when proxy service is stopped. |
| `src-tauri/src/proxy/config.rs` | `ProxyPoolConfig`, `ProxyEntry`, `ProxySelectionStrategy` enum (`round_robin`, `least_used`, `failover`). |
| `src/pages/ApiProxy.tsx` | Frontend management table for proxy pool entries, health statuses, latency metrics, and account bindings. |

---

## 3. Core Invariants & Rules

1. **State Persistence Across Restarts**:
   - Proxy pool bindings (`account_bindings`) and active proxy configurations must persist into `app_config.json`.
   - When the proxy service is not currently running, IPC commands must mutate the configuration file directly so changes are not lost.

2. **Zero Deadlock Guarantee**:
   - `ProxyPoolManager::new` must avoid blocking async locks by utilizing `try_read()` on configuration locks.
   - Usage counters and account bindings use `DashMap` for lock-free concurrent reads and updates across high-throughput proxy requests.

3. **Graceful Fallback on Health Failure**:
   - If a proxy fails its health check or encounters connection timeouts, the manager must automatically exclude it from round-robin rotation and fallback to next candidate or direct connection if enabled.

4. **Credential & Authentication Isolation**:
   - Proxy credentials (`username:password`) must be parsed and applied directly into `rquest::Proxy`, never logged in plain text in proxy logs or debug consoles.

---

## 4. Verification & Testing

```bash
# 1. Quality pre-flight checks
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features

# 2. Frontend build verification
npm run build
```
