---
name: agm-codebase-master-architecture
description: Comprehensive architecture guide, system topology, component interactions, and engineering guidelines for the entire Antigravity-Manager codebase.
---

# Antigravity-Manager (AGM) Master Architecture Guide

This skill serves as the foundational architectural blueprint for Antigravity-Manager. It documents the end-to-end system topology, component interactions, persistence layers, frontend state, and engineering conventions to ensure consistency across all AI-driven developments and modifications.

---

## 1. System Topology Overview

Antigravity-Manager is a unified desktop application and local proxy infrastructure built with Tauri v2 (Rust backend) and React 18 + TypeScript (Vite frontend). It orchestrates multi-instance Antigravity/VS Code sandboxes, routes AI API traffic through an Axum-based reverse proxy, and safeguards credentials across a split-SQLite storage architecture.

```
+-------------------------------------------------------------------------+
|                              Desktop Window                             |
|  React 18 + Vite + Tailwind CSS + Zustand                               |
|  - Pages: Accounts, ApiProxy, Instances, Settings, Security, Tokens     |
|  - Live Sync: window-restored, focus, visibilitychange re-fetch         |
+------------------------------------+------------------------------------+
                                     | Tauri v2 IPC (request.ts / invoke)
                                     v
+-------------------------------------------------------------------------+
|                           Tauri v2 Rust Core                            |
|  - Window Lifecycle & Win32 Foreground Lock Bypass                      |
|  - Process Sandboxing (--user-data-dir, --extensions-dir)               |
|  - Background Polling (Account quotas, health checks)                   |
+-------------------+--------------------------------+--------------------+
                    |                                |
                    v                                v
+------------------------------------+ +----------------------------------+
|      Axum Reverse Proxy (8045)     | |      Split SQLite Databases      |
|  - Protocol Translators:           | |  - proxy_logs.db (WAL)           |
|    OpenAI <-> Claude <-> Gemini    | |  - email_vault.db & passwords    |
|  - SSE Streaming & Thinking Store  | |  - user_tokens.db & stats        |
|  - IP Filtering & Rate Limiting    | |  - state.vscdb (IDE sync)        |
+------------------------------------+ +----------------------------------+
```

---

## 2. Directory Layout & Module Responsibilities

### Backend (`src-tauri/`)

- `src-tauri/src/main.rs` & `src-tauri/src/lib.rs`: Tauri application initialization, single-instance plugin registration, tray setup, and native window restore hooks.
- `src-tauri/src/proxy/`: Axum reverse proxy engine.
  - `server.rs`: Proxy listener, route dispatching, and TLS/port binding (default: `127.0.0.1:8045`).
  - `adapters/`: Protocol translation adapters (OpenAI format, Anthropic Claude format, Google Gemini format).
  - `adapters/thinking_store.rs`: Server-side reasoning and thought signature cache.
  - `pipeline/`: Outbound thinking pipeline, usage accounting, and canonical stream event diffusion.
  - `handlers/`: Route handlers for `/v1/chat/completions`, `/v1/messages`, and `/v1beta/models`.
- `src-tauri/src/modules/`: Core backend subsystems.
  - `win32.rs` & native FFI: Foreground thread attachment (`AttachThreadInput`), `ShowWindow(SW_RESTORE)`, and `SwitchToThisWindow` for guaranteed taskbar and workspace restoration.
  - `instance.rs`: Multi-instance sandbox spawning, PID management, and port allocation.
  - `account.rs`: Account authentication, token extraction from `state.vscdb`, and quota fetching.
  - `process.rs`: Cross-platform process discovery, process scoring, and termination.
- `src-tauri/src/commands/`: Registered Tauri IPC commands (`#[tauri::command]`).

### Frontend (`src/`)

- `src/pages/`:
  - `Accounts.tsx`: Account manager, live quota bars, instant switching, and custom labels.
  - `ApiProxy.tsx`: Real-time request monitoring, SSE stream inspection, and token costs.
  - `Instances.tsx`: Multi-profile sandboxes, port status, and instance launching.
  - `UserToken.tsx`: Downstream API token provisioning and expiration policies.
  - `TokenStats.tsx`: Cumulative token consumption visualizations.
  - `Security.tsx`: IP whitelist/blacklist management and curfew rules.
  - `Settings.tsx`: Thinking budgets, port settings, and theme customization.
- `src/components/`:
  - `accounts/`: `AccountTable.tsx`, `AccountRow.tsx`, `AccountCard.tsx`, `QuotaItem.tsx`.
  - `layout/`: `Layout.tsx` (top navigation, live cache invalidation listeners, error queue drawer trigger).
  - `UpdateNotification.tsx`: Compact bottom-right floating update prompt.
- `src/stores/`: Zustand state management.
  - `useAccountStore.ts`: Accounts array, active account, quota refresh.
  - `useInstanceStore.ts`: Sandboxed instances, active profile, running status.
  - `useConfigStore.ts`: Global configuration flags and proxy preferences.
- `src/utils/`:
  - `date.ts`: Polite date and time formatter (`formatDateTime` -> `DD-MMM-YY - hh:mm A`).
  - `request.ts`: Typed wrapper around Tauri `invoke`.

---

## 3. Critical Engineering Conventions & Patterns

### 1. Taskbar Restore & Win32 Foreground Lock
When a frameless Tauri desktop application is minimized or sent to the background on Windows, standard `window.unminimize()` or `window.set_focus()` calls often fail silently due to the OS **Foreground Lock Timeout**.
- Always invoke `force_restore_and_focus_win32()` in `src-tauri/src/lib.rs`.
- It connects to the active foreground thread via `AttachThreadInput`, invokes `ShowWindow(hwnd, 9)` (`SW_RESTORE`), `OpenIcon`, and brings the window to the front via `SetForegroundWindow` and `SwitchToThisWindow`.
- The frontend registers `window-restored` event listeners in `Layout.tsx` to automatically re-fetch stale account and instance data.

### 2. Live Cache Invalidation
Never rely on purely in-memory static state when the window is hidden or backgrounded:
- In `Layout.tsx`, `window-restored`, `focus`, and `visibilitychange` events trigger a throttled refresh of `fetchAccounts()`, `fetchCurrentAccount()`, and `fetchInstances()`.
- This ensures that token switches performed via CLI, remote email, or other nodes immediately reflect in the UI without a manual page reload.

### 3. Polite Date Formatting
Standardized date representation across the entire repository is:
`DD-MMM-YY - hh:mm A` (e.g., `22-Sep-26 - 10:29 AM`)
- Always use `formatDateTime(timestamp)` from `src/utils/date.ts`.
- Avoid multi-line stacked date/time blocks in compact tables.

### 4. Split-SQLite Database Integrity
- Separate high-throughput writes (`proxy_logs.db`, `token_stats.db`) from credential storage (`email_passwords.db`).
- WAL mode is mandatory on write-heavy databases.
- Passwords and secret tokens must be encrypted with machine-bound cryptography prior to persistence.

### 5. Quality Assurance & Verification
- Verify TypeScript changes: `npx tsc --noEmit`
- Verify Rust changes: `cargo check` (inside `src-tauri/`)
- Run local CI/CD quality gate runner: `python 03-ai-scripts/06-cicd-local-runner.py`
- Adhere strictly to the cross-language boolean principles (no explicit `== true`, no mixed polarity) and strictly lowercase file naming.
