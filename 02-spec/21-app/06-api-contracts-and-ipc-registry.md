# API Contracts, IPC Command Registry, and Protocol Specifications

> **Specification:** `02-spec/21-app/06-api-contracts-and-ipc-registry.md`
> **Status:** Production-Ready
> **Source Files:** `src-tauri/src/lib.rs`, `src-tauri/src/commands/`, `src-tauri/src/proxy/server.rs`, `src/utils/request.ts`, `src/types/`

---

## 1. Overview

Antigravity-Manager maintains two distinct communication boundaries:
1. **Tauri IPC Command Layer:** High-speed, typed binary bridge connecting the React frontend to the Rust native backend via `tauri::generate_handler!` and `src/utils/request.ts`.
2. **HTTP Reverse Proxy Gateway:** REST & Server-Sent Events (SSE) API server consumed by external AI code assistants (Cursor, VS Code, Roo Code).

---

## 2. Tauri IPC Command Registry (105 Handlers)

All commands are declared with `#[tauri::command]` and exposed in `src-tauri/src/lib.rs:572-742`.

### 2.1 Account & Identity Commands (`src-tauri/src/commands/mod.rs`)
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `list_accounts` | (none) | `Result<Vec<Account>, String>` | Lists accounts with live quota reset timers blended from memory. |
| `add_account` | `_email: String, refresh_token: String` | `Result<Account, String>` | Persists account, refreshes quota, and reloads proxy rotation pool. |
| `delete_account` | `account_id: String` | `Result<(), String>` | Deletes account and associated session state. |
| `delete_accounts` | `account_ids: Vec<String>` | `Result<(), String>` | Batch deletion of multiple accounts and tray menu update. |
| `reorder_accounts` | `account_ids: Vec<String>` | `Result<(), String>` | Updates prioritization sequence for round-robin rotation. |
| `switch_account` | `account_id: String, target_ide: Option<String>` | `Result<(), String>` | Sets active IDE account and clears stale session bindings. |
| `get_current_account` | (none) | `Result<Option<Account>, String>` | Retrieves current active account. |
| `export_accounts` | `account_ids: Vec<String>` | `Result<AccountExportResponse, String>` | Exports non-sensitive and encrypted account credentials. |
| `toggle_proxy_status` | `account_id: String, enable: bool, reason: Option<String>` | `Result<(), String>` | Enables or disables account participation in proxy routing pool. |
| `warm_up_all_accounts` | (none) | `Result<String, String>` | Proactively triggers token refresh for all accounts in pool. |
| `warm_up_account` | `account_id: String` | `Result<String, String>` | Warms up session token for a single account. |
| `update_account_label` | `account_id: String, label: String` | `Result<(), String>` | Sets custom descriptive label on account card. |

### 2.2 Device Fingerprint Commands
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `get_device_profiles` | `account_id: String` | `Result<DeviceProfiles, String>` | Reads active and bound storage device fingerprints. |
| `bind_device_profile` | `account_id: String` | `Result<(), String>` | Binds current IDE device fingerprint to account. |
| `bind_device_profile_with_profile` | `account_id: String, profile: DeviceProfile` | `Result<(), String>` | Explicitly binds provided profile to account. |
| `preview_generate_profile` | (none) | `Result<DeviceProfile, String>` | Generates hypothetical random fingerprint for preview. |
| `apply_device_profile` | `account_id: String` | `Result<(), String>` | Writes account fingerprint into target IDE storage. |
| `restore_original_device` | (none) | `Result<(), String>` | Restores pristine backup device profile to IDE storage. |
| `list_device_versions` | `account_id: String` | `Result<Vec<DeviceVersion>, String>` | Lists historical fingerprint versions for rollback. |
| `restore_device_version` | `account_id: String, version_id: String` | `Result<(), String>` | Reverts device profile to specific version ID. |
| `delete_device_version` | `account_id: String, version_id: String` | `Result<(), String>` | Purges historical profile version snapshot. |
| `open_device_folder` | (none) | `Result<(), String>` | Opens OS file explorer at device fingerprint directory. |

### 2.3 Quota & Usage Commands
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `fetch_account_quota` | `account_id: String` | `AppResult<QuotaData>` | Fetches live 5-hour and weekly quota buckets from upstream. |
| `refresh_all_quotas` | (none) | `Result<RefreshStats, String>` | Concurrent upstream quota sweep; returns aggregate stats. |

### 2.4 OAuth Lifecycle Commands
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `prepare_oauth_url` | (none) | `Result<OAuthPrepareResponse, String>` | Constructs PKCE challenge and authorization redirect URL. |
| `start_oauth_login` | (none) | `Result<String, String>` | Launches local loopback listener for OAuth callback. |
| `complete_oauth_login` | `code: String` | `Result<Account, String>` | Exchanges authorization code for refresh tokens. |
| `cancel_oauth_login` | (none) | `Result<(), String>` | Cancels pending background OAuth loopback listener. |
| `submit_oauth_code` | `code: String` | `Result<Account, String>` | Manual authorization code submission fallback. |
| `list_oauth_clients` | (none) | `Result<Vec<OAuthClientInfo>, String>` | Lists configured OAuth client credentials. |
| `get_active_oauth_client` | (none) | `Result<OAuthClientInfo, String>` | Returns currently active OAuth client preset. |
| `set_active_oauth_client` | `client_id: String` | `Result<(), String>` | Activates specific OAuth client configuration. |

### 2.5 Migration & Import Commands
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `import_v1_accounts` | `json_data: String` | `Result<usize, String>` | Ingests legacy v1 JSON export schema. |
| `import_from_db` | (none) | `Result<usize, String>` | Scans local default database for discoverable accounts. |
| `import_custom_db` | `db_path: String` | `Result<usize, String>` | Ingests accounts from specified custom SQLite database. |
| `sync_account_from_db` | (none) | `Result<usize, String>` | Synchronizes account token states from background database. |

### 2.6 OS, Window, Config & Cache Commands
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `greet` | `name: &str` | `String` | Health-check greeting command. |
| `load_config` / `save_config` | `(none)` / `config: AppConfig` | `Result<AppConfig/(), String>` | Reads / writes `config.json` application settings. |
| `save_text_file` / `read_text_file` | File path & content args | File text ops | Low-level configuration file access. |
| `clear_log_cache` / `clear_antigravity_cache` | (none) | `Result<(), String>` | Truncates runtime log files and temporary IDE cache. |
| `get_antigravity_cache_paths` | (none) | `Result<Vec<String>, String>` | Resolves host cache filesystem paths. |
| `open_data_folder` / `get_data_dir_path` | (none) | Path / Folder ops | Locates application data directory on host OS. |
| `show_main_window` / `set_window_theme` | `(none)` / `theme: String` | Window control | Manages webview visibility and title bar theme sync. |
| `get_antigravity_path` / `get_antigravity_cli_path` | (none) | Path resolutions | Resolves native binary locations on host system. |
| `get_antigravity_args` | (none) | `Result<Vec<String>, String>` | Returns process CLI launch arguments. |
| `get_http_api_settings` / `save_http_api_settings` | `(none)` / `settings` | Settings ops | Manages port, admin password, and bind settings. |

### 2.7 Updates & Package Management
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `check_for_updates` | (none) | `Result<UpdateCheckResult, String>` | Queries remote GitHub Releases API for updates. |
| `check_homebrew_installation` | (none) | `Result<bool, String>` | Detects whether app was installed via Homebrew Cask. |
| `check_appimage_installation` | (none) | `Result<bool, String>` | Detects whether app is running as Linux AppImage. |
| `brew_upgrade_cask` | (none) | `Result<String, String>` | Triggers `brew upgrade --cask` command execution. |
| `get_update_settings` / `save_update_settings` | `(none)` / `settings` | Update settings ops | Configures auto-check schedule and release channel. |
| `should_check_updates` / `update_last_check_time` | (none) | Periodic check ops | Evaluates interval elapsed time and marks check timestamp. |

### 2.8 Proxy Controls & Scheduling (`src-tauri/src/commands/proxy.rs`)
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `start_proxy_service` | `config: ProxyConfig` | `Result<ProxyStatus, String>` | Binds Axum HTTP server socket and starts proxy loop. |
| `stop_proxy_service` | (none) | `Result<ProxyStatus, String>` | Halts proxy engine and transitions state to stopped. |
| `get_proxy_status` | (none) | `Result<ProxyStatus, String>` | Returns port, bound address, running boolean, active accounts. |
| `get_proxy_stats` | (none) | `Result<ProxyStats, String>` | Returns total requests, active connections, and tokens. |
| `check_proxy_health` | (none) | `Result<HealthCheckResult, String>` | Validates upstream connectivity and account readiness. |
| `reload_proxy_accounts` | (none) | `Result<(), String>` | Refreshes account token pool without socket interruption. |
| `update_model_mapping` | `mapping: HashMap<String, String>` | `Result<(), String>` | Dynamically hot-reloads model alias translation tables. |
| `clear_proxy_session_bindings` | (none) | `Result<(), String>` | Evicts all conversation-to-account stickiness mappings. |
| `clear_proxy_rate_limit` | `account_id: String` | `Result<(), String>` | Clears cooldown timer for an individual account. |
| `clear_all_proxy_rate_limits` | (none) | `Result<(), String>` | Clears all active lockout timers across token manager. |
| `set_preferred_account` / `get_preferred_account` | `account_id` | Affinity ops | Configures or queries fixed account routing lock. |
| `fetch_zai_models` | (none) | `Result<Vec<String>, String>` | Queries remote catalog of upstream model IDs. |
| `get_proxy_scheduling_config` / `update_proxy_scheduling_config` | `(none)` / `config` | Scheduling ops | Manages rotation weights, quota guards, and cooldowns. |
| `generate_api_key` | (none) | `Result<String, String>` | Creates cryptographically secure master API key. |

### 2.9 Proxy Logs & Inspection
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `get_proxy_logs` | (none) | `Result<Vec<ProxyRequestLog>, String>` | Returns recent proxy logs from memory buffer. |
| `get_proxy_logs_paginated` | `limit: Option<usize>, offset: Option<usize>` | `Result<Vec<ProxyRequestLog>, String>` | Queries paginated log records from `proxy_logs.db`. |
| `get_proxy_log_detail` | `log_id: String` | `Result<ProxyRequestLog, String>` | Fetches complete request/response bodies and headers. |
| `get_proxy_logs_count` | (none) | `Result<u64, String>` | Returns total recorded log entries count. |
| `export_proxy_logs` / `export_proxy_logs_json` | File path and optional data | `Result<usize, String>` | Writes log database to disk as structured JSON. |
| `get_proxy_logs_count_filtered` / `get_proxy_logs_filtered` | `filter: LogFilter` | Filtered log queries | Searches logs by model, status code, date, or query text. |
| `set_proxy_monitor_enabled` | `enabled: bool` | `Result<(), String>` | Toggles real-time request recording on or off. |
| `clear_proxy_logs` | (none) | `Result<(), String>` | Truncates `request_logs` table in `proxy_logs.db`. |

### 2.10 Proxy Pool Bindings & Autostart
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `get_proxy_pool_config` | (none) | `Result<ProxyPoolConfig, String>` | Reads multi-proxy upstream pool definitions. |
| `bind_account_proxy` | `account_id: String, proxy_url: String` | `Result<(), String>` | Binds dedicated egress proxy tunnel to specific account. |
| `unbind_account_proxy` | `account_id: String` | `Result<(), String>` | Removes dedicated egress proxy binding. |
| `get_account_proxy_binding` | `account_id: String` | `Result<Option<String>, String>` | Reads dedicated egress proxy binding for account. |
| `get_all_account_bindings` | (none) | `Result<HashMap<String, String>, String>` | Returns all account-to-egress proxy assignments. |
| `toggle_auto_launch` / `is_auto_launch_enabled` | `(none)` | `Result<bool, String>` | Toggles and checks host OS autostart on system boot. |

### 2.11 IDE & Client Sync Integrations
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `get_cli_sync_status` / `execute_cli_sync` / `execute_cli_restore` / `get_cli_config_content` | CLI sync args | Status & exec | Manages Antigravity CLI configuration sync. |
| `get_opencode_sync_status` / `get_canonical_families` / `execute_opencode_sync` / `execute_opencode_restore` / `get_opencode_config_content` / `execute_opencode_clear` | OpenCode sync args | Status & exec | Manages OpenCode client configuration and models. |
| `get_droid_sync_status` / `execute_droid_sync` / `execute_droid_restore` / `get_droid_config_content` | Droid sync args | Status & exec | Manages Droid assistant sync and model bindings. |

### 2.12 Security, Firewall & Cloudflare Tunnel
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `get_ip_access_logs` / `get_ip_stats` / `get_ip_token_stats` / `clear_ip_access_logs` | Log query args | Stats & log ops | Manages IP connection attempts and per-IP token stats. |
| `get_ip_blacklist` / `add_ip_to_blacklist` / `remove_ip_from_blacklist` / `clear_ip_blacklist` / `check_ip_in_blacklist` | IP & Reason args | Firewall rules | Manages blocked client IP addresses and CIDR ranges. |
| `get_ip_whitelist` / `add_ip_to_whitelist` / `remove_ip_from_whitelist` / `clear_ip_whitelist` / `check_ip_in_whitelist` | IP & Desc args | Firewall rules | Manages permitted client IP addresses and CIDR ranges. |
| `get_security_config` / `update_security_config` | `(none)` / `config` | Security config ops | Configures firewall modes and abuse thresholds. |
| `cloudflared_check` / `cloudflared_install` / `cloudflared_start` / `cloudflared_stop` / `cloudflared_get_status` | Token & lifecycle args | Tunnel ops | Controls local Cloudflare Tunnel background daemon. |

### 2.13 Debug Console, Token Analytics, User Tokens & Binary Patch
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `enable_debug_console` / `disable_debug_console` / `is_debug_console_enabled` / `get_debug_console_logs` / `clear_debug_console_logs` | Debug console args | Tracing bridge | Controls internal tracing subscriber log capture. |
| `get_token_stats_hourly` / `daily` / `weekly` / `by_account` / `summary` / `by_model` / `model_trend_*` / `account_trend_*` | Token stats args | Analytics ops | Aggregates token consumption across models and accounts. |
| `list_user_tokens` / `create_user_token` / `update_user_token` / `delete_user_token` / `renew_user_token` / `get_token_ip_bindings` / `get_user_token_summary` / `query_transit_info` | User token args | Multi-tenant auth | Issues and validates downstream API consumer tokens. |
| `patch_agy_binary` | `target_path: String` | `Result<PatchResult, String>` | Patches upstream binary for custom proxy compatibility. |

---

## 3. HTTP Reverse Proxy API Contracts

### 3.1 OpenAI Chat Completions (`POST /v1/chat/completions`)
- **Headers:** `Authorization: Bearer <API_KEY>`, `Content-Type: application/json`, `X-Conversation-Id: <ID>`.
- **Request Body:** Standard OpenAI schema (`model`, `messages`, `stream`, `temperature`).
- **Response Shape (SSE Chunk):**
  ```text
  data: {"id":"chatcmpl-xyz","object":"chat.completion.chunk","created":1725900000,"model":"gpt-4o","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}

  data: [DONE]
  ```

### 3.2 Anthropic Claude Messages (`POST /v1/messages`)
- **Headers:** `x-api-key: <KEY>`, `anthropic-version: 2023-06-01`, `Content-Type: application/json`.
- **SSE Protocol:** Emits standard Anthropic stream events (`message_start`, `content_block_start`, `content_block_delta`, `message_delta`, `message_stop`).

### 3.3 Google Gemini Content (`POST /v1beta/models/*:generateContent`)
- Supports both standard JSON request/response and streaming variants (`streamGenerateContent`).

---

## 4. Verification & Acceptance Criteria

### AC-IPC-001: IPC Registry Coverage & Signature Parity
- **Given** The 105 command registrations declared in `src-tauri/src/lib.rs:572-742`.
- **When** Auditing all handler signatures against this registry specification.
- **Then** Every registered command is documented across the 13 domain tables with 1:1 parameter types and return contracts matching backend implementations in `src-tauri/src/commands/`.

### AC-IPC-002: Gateway Protocol Interoperability
- **Given** Downstream AI clients invoking `/v1/chat/completions`, `/v1/messages`, and `/v1beta/models/*`.
- **When** Proxying requests through the local Axum gateway.
- **Then** Successful calls stream SSE chunks or JSON bodies; upstream 429/503 errors yield standard envelopes with `Retry-After` headers and session regeneration counters.
