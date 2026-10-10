---
name: agm-security-network-guard
description: Specialized skill for managing reverse proxy IP security, CIDR whitelist/blacklist matching, curfew access windows, downstream user token provisioning, and Cloudflared Zero Trust tunnels in Antigravity-Manager.
---

# AGM Security, Network Guard & Cloudflared Architecture

This skill provides comprehensive architectural guidance, database schema rules, IPC contracts, and operational guidelines for the Security, IP Access Monitoring, Downstream User Tokens, and Cloudflared Tunneling subsystems in Antigravity-Manager.

---

## 1. Subsystem Architecture Overview

The security perimeter protects the Axum reverse proxy (`127.0.0.1:8045`) and handles public ingress:

```
+-------------------------------------------------------------------------+
|                  Inbound Client Request (HTTP / SSE)                    |
+------------------------------------+------------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|         Axum Inbound Middleware Stack (src-tauri/src/proxy/middleware/) |
|  1. ip_filter_middleware: Whitelist / Blacklist / Curfew Access Windows |
|  2. auth_middleware: UserToken & Master API Key Validation              |
|  3. monitor_middleware: Token consumption & audit logging               |
+------------------------------------+------------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|                         Split SQLite Databases                          |
|  - security.db: IP access logs, blacklist, whitelist, curfew rules      |
|  - user_tokens.db: Client tokens, IP bindings, rate limits, quotas      |
+-------------------------------------------------------------------------+
                                     ^
                                     |
+------------------------------------+------------------------------------+
|               Cloudflared Tunnel (commands/cloudflared.rs)              |
|  - Secure Zero Trust ingress without open router ports                  |
|  - Automatic binary provisioning & lifecycle management                 |
+-------------------------------------------------------------------------+
```

---

## 2. Key Files & Core Responsibilities

| File Path | Core Responsibilities |
|---|---|
| `src-tauri/src/modules/security_db.rs` | SQLite persistence for IP access logs, CIDR whitelist/blacklist, curfew window enforcement, and stats aggregation. |
| `src-tauri/src/commands/security.rs` | Tauri IPC commands: `get_ip_access_logs`, `get_ip_stats`, `add_ip_to_blacklist`, `update_security_config`. |
| `src-tauri/src/modules/user_token_db.rs` | Downstream API token provisioning, token expiry, client token IP binding, and consumption quotas. |
| `src-tauri/src/commands/user_token.rs` | Tauri IPC commands: `list_user_tokens`, `create_user_token`, `renew_user_token`, `delete_user_token`. |
| `src-tauri/src/modules/cloudflared.rs` & `commands/cloudflared.rs` | Cloudflare tunnel lifecycle: download, run, token binding, and health monitoring. |
| `src/pages/Security.tsx` & `src/components/security/IpAccessLogs.tsx` | Frontend UI for real-time IP log inspection, blacklist management, and curfew rules. |
| `src/pages/UserToken.tsx` | Downstream API token management dashboard. |

---

## 3. CIDR & Curfew Matching Conventions

1. **IP & CIDR Parsing**:
   - Supports both exact IP matches (`192.168.1.50`) and CIDR subnets (`192.168.1.0/24`, `10.0.0.0/8`).
   - Private subnets (e.g. `127.0.0.1`, `::1`) are permitted by default unless explicitly restricted.
2. **Curfew Access Windows**:
   - Allows defining valid operational hours (e.g. `09:00 - 18:00`).
   - Requests arriving outside the allowed curfew window are rejected with `403 Forbidden` (`CURFEW_BLOCKED`).
3. **Null-Safety & Query Resilience**:
   - When querying IP access logs and token statistics in `security_db.rs`, always use `COALESCE` or handle `Option<String>` / `Option<i64>` to prevent rusqlite type conversion panics on null columns.

---

## 4. Downstream User Token Authentication

- **Master Token**: Configured in `ProxyConfig` for full administrative access.
- **Client Tokens**: Issued via `user_tokens.db`:
  - Each token can be restricted to specific IP addresses or CIDR blocks.
  - Expiration timestamps (`expires_at`) are validated on every request.
  - Quota limits (monthly/daily token budget) are checked before routing to upstream providers.
