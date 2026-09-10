# API Contracts, IPC Command Registry, and Protocol Specifications

> **Specification:** `02-spec/21-app/06-api-contracts-and-ipc-registry.md`
> **Status:** Production-Ready
> **Source Files:** `src-tauri/src/lib.rs`, `src-tauri/src/commands/`, `src-tauri/src/proxy/server.rs`, `src/types/`

---

## 1. Overview

Antigravity-Manager operates two distinct communication boundaries:
1. **Tauri IPC Command Layer:** High-speed, typed binary bridge connecting the React frontend to the Rust native backend via `tauri::invoke`.
2. **HTTP Reverse Proxy Gateway:** REST & Server-Sent Events (SSE) API server consumed by external AI code assistants (Cursor, VS Code, Roo Code).

---

## 2. Tauri IPC Command Registry

All IPC commands are declared with `#[tauri::command]` and exposed through `tauri::generate_handler!`:

### 2.1 Account & Identity Commands (`commands/mod.rs`)
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `list_accounts` | (none) | `Result<Vec<Account>, String>` | Returns all configured accounts with live rate-limit reset timers blended from memory. |
| `add_account` | `account: Account` | `Result<(), String>` | Persists a new account into local storage and reloads proxy rotation pool. |
| `delete_account` | `id: String` | `Result<(), String>` | Removes an account and purges associated session bindings. |
| `delete_accounts` | `ids: Vec<String>` | `Result<(), String>` | Batch deletion of multiple accounts. |
| `reorder_accounts` | `ids: Vec<String>` | `Result<(), String>` | Updates the prioritization order for round-robin rotation. |
| `switch_account` | `id: String` | `Result<(), String>` | Sets the active default account for manual IDE interactions. |
| `get_current_account` | (none) | `Result<Option<Account>, String>` | Retrieves the currently active account. |
| `export_accounts` | (none) | `Result<String, String>` | Exports non-sensitive account metadata. |

### 2.2 Quota & Usage Commands
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `fetch_account_quota` | `id: String` | `Result<QuotaData, String>` | Queries live 5-hour and weekly quota buckets from upstream API. |
| `refresh_all_quotas` | (none) | `Result<Vec<Account>, String>` | Concurrently polls upstream quota endpoints for all active accounts. |

### 2.3 Proxy Engine Lifecycle Commands (`commands/proxy.rs`)
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `start_proxy_service` | (none) | `Result<ProxyStatus, String>` | Binds TCP socket, initializes Hyper server, and starts proxy loop. |
| `stop_proxy_service` | (none) | `Result<ProxyStatus, String>` | Emits shutdown signal via oneshot channel and closes listening socket. |
| `get_proxy_status` | (none) | `Result<ProxyStatus, String>` | Returns running state, bound address, active port, and uptime. |
| `get_proxy_stats` | (none) | `Result<ProxyStats, String>` | Returns total requests, active connections, and input/output token counts. |
| `check_proxy_health` | (none) | `Result<HealthCheckResult, String>` | Tests upstream connectivity and account health. |
| `reload_proxy_accounts` | (none) | `Result<(), String>` | Reloads account credentials without restarting the HTTP listener. |
| `update_model_mapping` | `mapping: HashMap<String, String>` | `Result<(), String>` | Dynamically updates client-to-upstream model name aliases. |
| `clear_proxy_session_bindings` | (none) | `Result<(), String>` | Evicts all in-memory conversation-to-account stickiness mappings. |
| `clear_proxy_rate_limit` | `account_id: String` | `Result<(), String>` | Manually resets rate limit lockout for a specific account. |
| `clear_all_proxy_rate_limits` | (none) | `Result<(), String>` | Resets all active account lockout timers in `TokenManager`. |

### 2.4 Proxy Logs & Inspection Commands
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `get_proxy_logs_paginated` | `page: u32, page_size: u32` | `Result<PaginatedLogs, String>` | Fetches request logs with pagination from `proxy_logs.db`. |
| `get_proxy_logs_filtered` | `filter: LogFilter` | `Result<PaginatedLogs, String>` | Queries logs by status code, model name, date range, or search string. |
| `get_proxy_log_detail` | `id: String` | `Result<Option<ProxyRequestLog>, String>` | Retrieves complete request/response bodies and headers for an entry. |
| `clear_proxy_logs` | (none) | `Result<(), String>` | Truncates the `request_logs` table in `proxy_logs.db`. |
| `export_proxy_logs_json` | (none) | `Result<String, String>` | Exports log database to JSON format. |

### 2.5 Security & Access Control Commands (`commands/security.rs`)
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `get_ip_access_logs` | `limit: u32, offset: u32` | `Result<Vec<IpAccessLog>, String>` | Returns incoming client IP connection attempts. |
| `add_ip_blacklist` | `ip: String, reason: Option<String>` | `Result<(), String>` | Adds an IP or CIDR block to the firewall blacklist. |
| `remove_ip_blacklist` | `id: String` | `Result<(), String>` | Removes an IP from the blacklist. |
| `add_ip_whitelist` | `ip: String, desc: Option<String>` | `Result<(), String>` | Adds an IP or CIDR block to the whitelist. |
| `remove_ip_whitelist` | `id: String` | `Result<(), String>` | Removes an IP from the whitelist. |
| `generate_api_key` | (none) | `Result<String, String>` | Generates a cryptographically random master API key. |

---

## 3. HTTP Reverse Proxy API Contracts

### 3.1 OpenAI Chat Completions (`POST /v1/chat/completions`)
- **Headers:**
  - `Authorization: Bearer <API_KEY>` (optional or verified against master key)
  - `Content-Type: application/json`
  - `X-Conversation-Id: <ID>` (optional conversation fingerprint override)
- **Request Body (Standard OpenAI Schema):**
  ```json
  {
    "model": "gpt-4o",
    "messages": [
      {"role": "system", "content": "You are a coding assistant."},
      {"role": "user", "content": "Hello!"}
    ],
    "stream": true,
    "temperature": 0.7
  }
  ```
- **Response Headers:**
  - `Content-Type: text/event-stream` (when `stream: true`)
  - `Retry-After: <seconds>` (when returning HTTP 429 / 503)
  - `X-Session-Generation: <int>` (internal generation counter)
- **Response Shape (SSE Chunk):**
  ```text
  data: {"id":"chatcmpl-xyz","object":"chat.completion.chunk","created":1725900000,"model":"gpt-4o","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}

  data: [DONE]
  ```

### 3.2 Anthropic Claude Messages (`POST /v1/messages`)
- **Headers:**
  - `x-api-key: <KEY>`
  - `anthropic-version: 2023-06-01`
  - `Content-Type: application/json`
- **Request Body:** Standard Anthropic Messages API format supporting system prompts, multimodal images, and tools.
- **SSE Protocol:** Emits `message_start`, `content_block_start`, `content_block_delta`, `content_block_stop`, `message_delta`, and `message_stop` events.

### 3.3 Google Gemini Content (`POST /v1beta/models/*:generateContent`)
- **Request Body:** Standard Gemini `Content` array with parts.
- **Streaming Variant:** `POST /v1beta/models/*:streamGenerateContent` returning JSON array stream.
