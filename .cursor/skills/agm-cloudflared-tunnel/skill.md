---
name: agm-cloudflared-tunnel
description: Specialized skill for managing Cloudflared Zero Trust tunnels, Quick tunnel URL extraction, Named Auth tunnels, automated binary acquisition, and remote web access in src-tauri/src/modules/cloudflared.rs and commands/cloudflared.rs.
---

# AGM Cloudflared Zero Trust Tunnel Subsystem

This skill provides comprehensive architectural guidance, process management procedures, and configuration standards for Cloudflared Zero Trust tunnels in **Antigravity-Manager (AGM)**.

---

## 1. Subsystem Architecture Overview

To enable remote Web UI access and API proxy usage from anywhere without opening firewall ports or acquiring public IPv4 addresses, AGM provides integrated Cloudflare Tunnel orchestration:

```
+----------------------------------------------------------------------------------------------------+
|                                      Cloudflared Tunnel Engine                                     |
|                              src-tauri/src/modules/cloudflared.rs                                  |
|    - Automatic Binary Discovery: Checks system PATH, then checks local %APPDATA%/data/cloudflared  |
|    - Automatic Download: Fetches platform-specific binary from GitHub Releases                     |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                           Tunnel Modes                                             |
|    1. Quick Tunnel (TunnelMode::Quick):                                                            |
|       - Invokes `cloudflared tunnel --url http://localhost:8045 --http2`                           |
|       - Stream-parses stderr via regex `https://[a-zA-Z0-9-]+\.trycloudflare\.com`                 |
|       - Exposes temporary public HTTPS URL without user account                                    |
|    2. Auth / Named Tunnel (TunnelMode::Auth):                                                      |
|       - Invokes `cloudflared tunnel run --token <token>`                                           |
|       - Connects to user's pre-configured Cloudflare Zero Trust domain                             |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                    Process Group Management                                        |
|    - Windows: `CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP` flags                                   |
|    - Unix: Direct SIGTERM / kill child process tracking via tokio::process::Child                  |
+----------------------------------------------------------------------------------------------------+
```

---

## 2. Key Files & Core Responsibilities

| File Path | Core Responsibilities |
|---|---|
| `src-tauri/src/modules/cloudflared.rs` | `CloudflaredManager`, `CloudflaredConfig`, `TunnelMode` enum (`quick`, `auth`), `download_binary()`, `check_installed()`, async stderr reader with URL extraction regex, and process termination. |
| `src-tauri/src/commands/cloudflared.rs` | Tauri IPC commands: `cloudflared_check`, `cloudflared_install`, `cloudflared_start`, `cloudflared_stop`, `cloudflared_status`, and `cloudflared_save_config`. |
| `src/components/security/` & `Settings.tsx` | Cloudflared UI configuration card, status indicators, start/stop buttons, and URL copy action. |

---

## 3. Core Invariants & Rules

1. **Non-Blocking URL Resolution**:
   - `start()` launches `cloudflared` in the background and continuously scans stderr via `tokio::io::BufReader`.
   - The method must yield the discovered URL as soon as the `trycloudflare.com` pattern is detected, or timeout cleanly after 30 seconds without hanging the Tauri command worker.

2. **Windows Process Group Isolation**:
   - When spawning `cloudflared` on Windows, flags `CREATE_NO_WINDOW (0x08000000)` and `CREATE_NEW_PROCESS_GROUP (0x00000200)` must be set to prevent transient black console windows from interrupting the user.

3. **HTTP/2 Protocol Enforcement**:
   - Tunnels proxying port 8045 must include `--http2` when `use_http2` is enabled (default: true) to maintain high-throughput streaming SSE connections for AI responses.

4. **Security DB Interoperability**:
   - Requests arriving over the tunnel still pass through the Axum proxy IP filter (`security.db`) and User Token authentication (`user_tokens.db`).

---

## 4. Verification & Testing

```bash
# 1. Quality pre-flight checks
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features

# 2. Check Cloudflared command line integration
cd src-tauri && cargo test -- test_cloudflared
```
