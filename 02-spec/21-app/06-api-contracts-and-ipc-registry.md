# API Contracts, IPC Command Registry, and Protocol Specifications

> **Specification:** `02-spec/21-app/06-api-contracts-and-ipc-registry.md`
> **Status:** Production-Ready
> **Source Files:** `src-tauri/src/lib.rs`, `src-tauri/src/commands/`, `src-tauri/src/proxy/server.rs`, `src/utils/request.ts`, `src/types/`

---

## 1. Overview

Antigravity-Manager maintains two distinct communication boundaries:
1. **Tauri IPC Command Layer:** High-speed typed binary bridge connecting the React frontend to the Rust native backend via `tauri::generate_handler!` and `src/utils/request.ts`. All payload structs implement `serde::Deserialize`/`Serialize` with `#[serde(rename_all = "camelCase")]`.
2. **HTTP Reverse Proxy Gateway:** REST & Server-Sent Events (SSE) API server consumed by external AI code assistants (Cursor, VS Code, Roo Code).

---

## 2. Tauri IPC Command Registry (153 Handlers)

All 153 unique commands are declared with `#[tauri::command]` and exposed in `src-tauri/src/lib.rs:572-742`.

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
| `restore_original_device` | (none) | `Result<String, String>` | Restores pristine backup device profile to IDE storage. |
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
| `complete_oauth_login` | (none) | `Result<Account, String>` | Finalizes loopback callback and exchanges auth code for tokens. |
| `cancel_oauth_login` | (none) | `Result<(), String>` | Cancels pending background OAuth loopback listener. |
| `submit_oauth_code` | `code: String, state: Option<String>` | `Result<(), String>` | Manual authorization code submission fallback. |
| `list_oauth_clients` | (none) | `Result<Vec<OAuthClientInfo>, String>` | Lists configured OAuth client credentials. |
| `get_active_oauth_client` | (none) | `Result<String, String>` | Returns currently active OAuth client preset key. |
| `set_active_oauth_client` | `client_key: String` | `Result<(), String>` | Activates specific OAuth client configuration. |

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
| `load_config` | (none) | `Result<AppConfig, String>` | Reads `config.json` application settings from disk. |
| `save_config` | `config: AppConfig` | `Result<(), String>` | Writes updated application configuration to disk. |
| `save_text_file` | `path: String, content: String` | `Result<(), String>` | Low-level configuration file writing within allowed user scope. |
| `read_text_file` | `path: String` | `Result<String, String>` | Low-level configuration file reading within allowed user scope. |
| `clear_log_cache` | (none) | `Result<(), String>` | Truncates runtime log files. |
| `clear_antigravity_cache` | (none) | `Result<ClearResult, String>` | Purges temporary IDE application cache. |
| `get_antigravity_cache_paths` | (none) | `Result<Vec<String>, String>` | Resolves host cache filesystem paths for inspection. |
| `open_data_folder` | (none) | `Result<(), String>` | Opens host OS file manager at application data directory. |
| `get_data_dir_path` | (none) | `Result<String, String>` | Returns absolute path to application data directory. |
| `show_main_window` | (none) | `Result<(), String>` | Unhides and focuses the primary Tauri application window. |
| `set_window_theme` | `theme: String` | `Result<(), String>` | Synchronizes OS window frame theme (`light` / `dark`). |
| `get_antigravity_path` | `bypass_config: Option<bool>` | `Result<String, String>` | Locates Antigravity IDE executable binary on host. |
| `get_antigravity_cli_path` | `bypass_config: Option<bool>` | `Result<String, String>` | Locates Antigravity CLI (`agy`) binary on host. |
| `get_antigravity_args` | (none) | `Result<Vec<String>, String>` | Extracts runtime arguments from running Antigravity process. |
| `get_http_api_settings` | (none) | `Result<HttpApiSettings, String>` | Reads HTTP API server port and authorization settings. |
| `save_http_api_settings` | `settings: HttpApiSettings` | `Result<(), String>` | Persists HTTP API server configuration. |

### 2.7 Updates & Package Management
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `check_for_updates` | (none) | `Result<UpdateInfo, String>` | Queries remote GitHub Releases API for new versions. |
| `check_homebrew_installation` | (none) | `Result<bool, String>` | Detects whether app was installed via Homebrew Cask. |
| `check_appimage_installation` | (none) | `Result<bool, String>` | Detects whether app is running as Linux AppImage. |
| `brew_upgrade_cask` | (none) | `Result<String, String>` | Executes `brew upgrade --cask` command execution. |
| `get_update_settings` | (none) | `Result<UpdateSettings, String>` | Loads update checker configuration and release channel. |
| `save_update_settings` | `settings: UpdateSettings` | `Result<(), String>` | Persists auto-check interval and channel preferences. |
| `should_check_updates` | (none) | `Result<bool, String>` | Evaluates interval elapsed time against last check time. |
| `update_last_check_time` | (none) | `Result<(), String>` | Updates last update check timestamp to current time. |

### 2.8 Proxy Controls & Scheduling (`src-tauri/src/commands/proxy.rs`)
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `start_proxy_service` | `config: ProxyConfig` | `Result<ProxyStatus, String>` | Binds Axum HTTP server socket and starts proxy loop. |
| `stop_proxy_service` | (none) | `Result<(), String>` | Halts proxy engine and transitions state to stopped. |
| `get_proxy_status` | (none) | `Result<ProxyStatus, String>` | Returns port, bound address, running boolean, active accounts. |
| `get_proxy_stats` | (none) | `Result<ProxyStats, String>` | Returns total requests, active connections, and uptime. |
| `check_proxy_health` | (none) | `Result<ProxyPoolConfig, String>` | Validates upstream connectivity and account pool readiness. |
| `reload_proxy_accounts` | (none) | `Result<usize, String>` | Refreshes account token pool without socket interruption. |
| `update_model_mapping` | `config: ProxyConfig` | `Result<(), String>` | Dynamically hot-reloads model alias translation tables. |
| `fetch_zai_models` | `zai: ZaiConfig, upstream_proxy: UpstreamProxyConfig, request_timeout: u64` | `Result<Vec<String>, String>` | Queries remote catalog of upstream model IDs. |
| `get_proxy_scheduling_config` | (none) | `Result<StickySessionConfig, String>` | Reads session stickiness and routing affinity settings. |
| `update_proxy_scheduling_config` | `config: StickySessionConfig` | `Result<(), String>` | Updates session stickiness and cooldown configurations. |
| `clear_proxy_session_bindings` | (none) | `Result<(), String>` | Evicts all conversation-to-account stickiness mappings. |
| `set_preferred_account` | `account_id: Option<String>` | `Result<(), String>` | Configures fixed account routing lock. |
| `get_preferred_account` | (none) | `Result<Option<String>, String>` | Queries current fixed account routing lock. |
| `clear_proxy_rate_limit` | `account_id: String` | `Result<bool, String>` | Clears cooldown timer for an individual account. |
| `clear_all_proxy_rate_limits` | (none) | `Result<(), String>` | Clears all active lockout timers across token manager. |
| `generate_api_key` | (none) | `String` | Creates cryptographically secure master API key. |
| `get_proxy_pool_config` | (none) | `Result<ProxyPoolConfig, String>` | Reads multi-proxy upstream pool definitions. |

### 2.9 Proxy Logs & Inspection
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `get_proxy_logs` | `limit: Option<usize>` | `Result<Vec<ProxyRequestLog>, String>` | Returns recent proxy logs from in-memory ring buffer. |
| `get_proxy_logs_paginated` | `limit: Option<usize>, offset: Option<usize>` | `Result<Vec<ProxyRequestLog>, String>` | Queries paginated log records from SQLite log store. |
| `get_proxy_log_detail` | `log_id: String` | `Result<ProxyRequestLog, String>` | Fetches complete request/response bodies and headers. |
| `get_proxy_logs_count` | (none) | `Result<u64, String>` | Returns total recorded log entries count. |
| `export_proxy_logs` | `file_path: String` | `Result<usize, String>` | Exports log database to disk as structured JSON. |
| `export_proxy_logs_json` | `file_path: String, json_data: String` | `Result<usize, String>` | Writes provided log JSON content to target file. |
| `get_proxy_logs_count_filtered` | `filter: String, errors_only: bool` | `Result<u64, String>` | Returns count of log entries matching filter criteria. |
| `get_proxy_logs_filtered` | `filter: String, errors_only: bool, limit: usize, offset: usize` | `Result<Vec<ProxyRequestLog>, String>` | Searches logs by model, status code, date, or query text. |
| `set_proxy_monitor_enabled` | `enabled: bool` | `Result<(), String>` | Toggles real-time request recording on or off. |
| `clear_proxy_logs` | (none) | `Result<(), String>` | Truncates `request_logs` table in `proxy_logs.db`. |

### 2.10 Proxy Pool Bindings & Autostart
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `bind_account_proxy` | `account_id: String, proxy_id: String` | `Result<(), String>` | Binds dedicated egress proxy tunnel to specific account. |
| `unbind_account_proxy` | `account_id: String` | `Result<(), String>` | Removes dedicated egress proxy binding. |
| `get_account_proxy_binding` | `account_id: String` | `Result<Option<String>, String>` | Reads dedicated egress proxy binding for account. |
| `get_all_account_bindings` | (none) | `Result<HashMap<String, String>, String>` | Returns all account-to-egress proxy assignments. |
| `toggle_auto_launch` | `enable: bool` | `Result<(), String>` | Configures host OS auto-launch on system startup. |
| `is_auto_launch_enabled` | (none) | `Result<bool, String>` | Checks if host OS autostart is currently enabled. |

### 2.11 Token Statistics & Analytics
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `get_token_stats_hourly` | `hours: i64` | `Result<Vec<TokenStatsAggregated>, String>` | Aggregates hourly token consumption across models. |
| `get_token_stats_daily` | `days: i64` | `Result<Vec<TokenStatsAggregated>, String>` | Aggregates daily token consumption across models. |
| `get_token_stats_weekly` | `weeks: i64` | `Result<Vec<TokenStatsAggregated>, String>` | Aggregates weekly token consumption across models. |
| `get_token_stats_by_account` | `hours: i64` | `Result<Vec<AccountTokenStats>, String>` | Summarizes token usage distribution per account. |
| `get_token_stats_summary` | `hours: i64` | `Result<TokenStatsSummary, String>` | Aggregates total prompt, completion, and cache tokens. |
| `get_token_stats_by_model` | `hours: i64` | `Result<Vec<ModelTokenStats>, String>` | Summarizes token usage distribution per model. |
| `get_token_stats_model_trend_hourly` | `hours: i64` | `Result<Vec<ModelTrendPoint>, String>` | Hourly token trends broken down by model ID. |
| `get_token_stats_model_trend_daily` | `days: i64` | `Result<Vec<ModelTrendPoint>, String>` | Daily token trends broken down by model ID. |
| `get_token_stats_account_trend_hourly` | `hours: i64` | `Result<Vec<AccountTrendPoint>, String>` | Hourly token trends broken down by account ID. |
| `get_token_stats_account_trend_daily` | `days: i64` | `Result<Vec<AccountTrendPoint>, String>` | Daily token trends broken down by account ID. |

### 2.12 IDE & Client Sync Integrations
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `get_cli_sync_status` | `app_type: CliApp, proxy_url: String` | `Result<CliStatus, String>` | Checks CLI installation and sync state for Antigravity CLI. |
| `execute_cli_sync` | `app_type: CliApp, proxy_url: String, api_key: String, model: Option<String>` | `Result<(), String>` | Writes proxy routing configuration to CLI tool config. |
| `execute_cli_restore` | `app_type: CliApp` | `Result<(), String>` | Restores CLI tool configuration from backup file. |
| `get_cli_config_content` | `app_type: CliApp, file_name: Option<String>` | `Result<String, String>` | Reads configuration file contents for CLI app. |
| `get_opencode_sync_status` | `proxy_url: String` | `Result<OpencodeStatus, String>` | Checks OpenCode installation, sync state, and files. |
| `get_canonical_families` | (none) | `Vec<CanonicalFamilyDto>` | Returns canonical Gemini family definitions and aliases. |
| `execute_opencode_sync` | `proxy_url: String, api_key: String, sync_accounts: Option<bool>, models: Option<Vec<ModelInput>>` | `Result<(), String>` | Configures OpenCode providers and model mappings. |
| `execute_opencode_restore` | (none) | `Result<(), String>` | Restores OpenCode configuration from backup snapshot. |
| `get_opencode_config_content` | `request: GetOpencodeConfigRequest` | `Result<String, String>` | Reads OpenCode configuration JSON/JSONC file content. |
| `execute_opencode_clear` | `proxy_url: Option<String>, clear_legacy: Option<bool>` | `Result<(), String>` | Clears Antigravity provider and model entries from OpenCode. |
| `get_droid_sync_status` | `proxy_url: String` | `Result<DroidStatus, String>` | Checks Droid assistant installation and model sync status. |
| `execute_droid_sync` | `custom_models: Vec<Value>` | `Result<usize, String>` | Synchronizes custom models list into Droid configuration. |
| `execute_droid_restore` | (none) | `Result<(), String>` | Restores Droid assistant configuration from backup. |
| `get_droid_config_content` | (none) | `Result<String, String>` | Reads current Droid assistant configuration JSON content. |

### 2.13 Security, Firewall & Cloudflare Tunnel
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `get_ip_access_logs` | `query: IpAccessLogQuery` | `Result<IpAccessLogResponse, String>` | Queries paginated client IP access attempt logs. |
| `get_ip_stats` | (none) | `Result<IpStatsResponse, String>` | Returns total requests, unique IPs, blocked count, top IPs. |
| `get_ip_token_stats` | `limit: Option<usize>, hours: Option<i64>` | `Result<Vec<IpTokenStats>, String>` | Aggregates token consumption grouped by client IP. |
| `clear_ip_access_logs` | (none) | `Result<(), String>` | Truncates client IP access log database table. |
| `get_ip_blacklist` | (none) | `Result<Vec<IpBlacklistEntry>, String>` | Returns list of blocked client IP addresses/CIDRs. |
| `add_ip_to_blacklist` | `request: AddBlacklistRequest` | `Result<(), String>` | Adds IP or CIDR block to firewall blacklist. |
| `remove_ip_from_blacklist` | `ip_pattern: String` | `Result<(), String>` | Removes IP or CIDR block from firewall blacklist. |
| `clear_ip_blacklist` | (none) | `Result<(), String>` | Purges all firewall blacklist entries. |
| `check_ip_in_blacklist` | `ip: String` | `Result<bool, String>` | Tests whether given client IP matches any blacklist rule. |
| `get_ip_whitelist` | (none) | `Result<Vec<IpWhitelistEntry>, String>` | Returns list of permitted client IP addresses/CIDRs. |
| `add_ip_to_whitelist` | `request: AddWhitelistRequest` | `Result<(), String>` | Adds IP or CIDR block to firewall whitelist. |
| `remove_ip_from_whitelist` | `ip_pattern: String` | `Result<(), String>` | Removes IP or CIDR block from firewall whitelist. |
| `clear_ip_whitelist` | (none) | `Result<(), String>` | Purges all firewall whitelist entries. |
| `check_ip_in_whitelist` | `ip: String` | `Result<bool, String>` | Tests whether given client IP matches any whitelist rule. |
| `get_security_config` | (none) | `Result<SecurityMonitorConfig, String>` | Retrieves active firewall configuration and thresholds. |
| `update_security_config` | `config: SecurityMonitorConfig` | `Result<(), String>` | Updates firewall settings and hot-reloads middleware. |
| `cloudflared_check` | (none) | `Result<CloudflaredStatus, String>` | Checks if cloudflared daemon is installed on host. |
| `cloudflared_install` | (none) | `Result<CloudflaredStatus, String>` | Downloads and installs cloudflared executable. |
| `cloudflared_start` | `config: CloudflaredConfig` | `Result<CloudflaredStatus, String>` | Launches cloudflared tunnel daemon with token/config. |
| `cloudflared_stop` | (none) | `Result<CloudflaredStatus, String>` | Terminates running cloudflared tunnel daemon. |
| `cloudflared_get_status` | (none) | `Result<CloudflaredStatus, String>` | Returns tunnel status, active URL, and process state. |

### 2.14 Debug Console, User Tokens, Transit & Binary Patch
| Command Name | Parameters | Return Type | Description |
|---|---|---|---|
| `enable_debug_console` | (none) | `()` | Attaches tracing subscriber ring buffer and enables bridge. |
| `disable_debug_console` | (none) | `()` | Disables log event emission across Tauri event bridge. |
| `is_debug_console_enabled` | (none) | `bool` | Checks if debug tracing console is currently active. |
| `get_debug_console_logs` | (none) | `Vec<LogEntry>` | Retrieves all buffered in-memory tracing log entries. |
| `clear_debug_console_logs` | (none) | `()` | Truncates in-memory tracing subscriber log ring buffer. |
| `list_user_tokens` | (none) | `Result<Vec<UserToken>, String>` | Lists downstream tenant API tokens and statuses. |
| `create_user_token` | `request: CreateTokenRequest` | `Result<UserToken, String>` | Generates new downstream API token with IP limits. |
| `update_user_token` | `id: String, request: UpdateTokenRequest` | `Result<(), String>` | Updates token metadata, limits, or enabled state. |
| `delete_user_token` | `id: String` | `Result<(), String>` | Revokes and deletes tenant API token. |
| `renew_user_token` | `id: String, expires_type: String` | `Result<(), String>` | Extends expiration timestamp for existing tenant token. |
| `get_token_ip_bindings` | `token_id: String` | `Result<Vec<TokenIpBinding>, String>` | Lists client IPs bound to specific tenant token. |
| `get_user_token_summary` | (none) | `Result<UserTokenStats, String>` | Aggregates active token counts and unique user totals. |
| `query_transit_info` | `url: String, key: String` | `Result<String, String>` | Queries remote upstream transit station endpoint. |
| `patch_agy_binary` | `file_path: String` | `Result<String, String>` | Binary-patches `agy` executable to bypass custom proxy checks. |

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

### AC-IPC-001: IPC Command Registration Count & Parity
- **Executable Test:** `tests::test_ipc_command_registration_count`
- **Given** The 153 unique command registrations declared in `src-tauri/src/lib.rs:572-742`.
- **When** Auditing all handler signatures against this registry specification.
- **Then** Every registered command is documented across domain tables with 1:1 typed input parameters, Serde camelCase conventions, and `Result<T, E>` return contracts matching implementations in `src-tauri/src/commands/`.

### AC-IPC-002: IPC Serde camelCase Serialization & Gateway Interoperability
- **Executable Test:** `tests::test_ipc_serde_camelcase_serialization`
- **Given** Downstream AI clients invoking `/v1/chat/completions`, `/v1/messages`, and `/v1beta/models/*` or IPC command invocations.
- **When** Payloads are serialized and deserialized across the Tauri IPC boundary and Axum gateway.
- **Then** All command structures serialize using strict camelCase conventions, successful calls stream SSE chunks or JSON bodies, and upstream errors yield standard envelopes with `Retry-After` headers and session regeneration counters.
