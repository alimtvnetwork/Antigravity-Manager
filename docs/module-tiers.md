# Module Tier Map — `src-tauri/src/modules/`

Scope guardrail for the Antigravity-Manager backend. The project's core mission
(AGENTS.md) is a **protocol gateway**: aggregate OpenAI Responses, OpenAI Chat
Completions, Anthropic Claude, and Google Gemini into Antigravity-style Gemini
protocol output. Every module is classified into exactly one tier:

- **CORE** — the gateway cannot function without it (accounts, auth, quota,
  proxy state, instances).
- **ADJACENT** — a separate feature domain bolted onto the gateway (email,
  Telegram, Supabase, SSH, tunnels). May grow, but must never be imported by
  core proxy code (`src-tauri/src/proxy/`); see the guardrail in `proxy/mod.rs`.
- **SUPPORT** — cross-cutting infrastructure any tier may use (config, logging,
  DB utilities, i18n, platform integration).

Rule of thumb for new modules: if removing it would not break request
proxying, it is not CORE.

## CORE (protocol gateway) — 13 modules

| Module | Role |
|---|---|
| `account.rs` | Proxy account management |
| `account_service.rs` | Account service layer (Tauri-decoupled) |
| `auto_switcher.rs` | Automatic account switching on quota/failure |
| `instance.rs` | Instance profiles hosting proxy configs |
| `oauth.rs` | Google OAuth configuration (Antigravity auth) |
| `oauth_server.rs` | OAuth callback server |
| `proxy_db.rs` | Proxy state database |
| `quota.rs` | Quota API (Daily → Sandbox → Prod fallback) |
| `repo_db.rs` | Running-project / active-prompt state DB |
| `security_db.rs` | Proxy server security monitoring (IP logs) |
| `task_history_db.rs` | Task history database |
| `token_stats.rs` | Aggregated token usage statistics |
| `user_token_db.rs` | User token storage |

## ADJACENT (separate feature domains) — 17 modules

| Module | Role |
|---|---|
| `agy_cleaner.rs` | Antigravity desktop-app conversation pruning & cache cleaning |
| `backup_prompts_db.rs` | Prompt database backup |
| `cache.rs` | Antigravity application cache clearing |
| `cloudflared.rs` | Cloudflare tunnel management |
| `email_inbound.rs` | Inbound email poller / remote execution bridge |
| `email_io.rs` | Email import/export engine (JSON/CSV/XLSX/SQLite) |
| `email_sender.rs` | Outbound SMTP dispatcher |
| `email_vault_db.rs` | Email credentials vault database |
| `email_watcher.rs` | Background email watcher daemon |
| `notification_hub.rs` | Unified notifications across Email + Telegram |
| `ssh_manager.rs` | SSH connection management |
| `supabase_client.rs` | Supabase PostgREST client |
| `supabase_command_queue.rs` | Supabase inbound command queue & execution |
| `supabase_pruner.rs` | Supabase 500 MB free-tier pruning |
| `supabase_schema.rs` | Supabase schema definitions & migrations |
| `supabase_sync.rs` | Supabase node registry & heartbeat sync |
| `telegram_inbound.rs` | Telegram bot inbound watcher / remote commands |

## SUPPORT (cross-cutting infrastructure) — 24 modules

| Module | Role |
|---|---|
| `audit_action.rs` | Audit action helpers |
| `cli.rs` | CLI interface |
| `config.rs` | Configuration management |
| `db.rs` | Shared database utilities |
| `delegate_updater.rs` | Out-of-process self-updater |
| `device.rs` | Device identity |
| `git_info.rs` | Git metadata / telemetry (version, branch, commit) |
| `http_api.rs` | Local HTTP API for external programs |
| `i18n.rs` | Internationalization |
| `integration.rs` | Platform integration (macOS/Linux security) |
| `iterative_codec.rs` | Iterative Base64 codec / API-key protection |
| `json_envelope.rs` | Structured work-directory configuration |
| `lightweight.rs` | Lightweight window mode |
| `log_bridge.rs` | Tracing log bridge to frontend |
| `logger.rs` | Logging |
| `migration.rs` | Database migrations |
| `mod.rs` | Module declarations |
| `process.rs` | Process utilities |
| `scheduler.rs` | Background scheduler (warmup history) |
| `training_api.rs` | Machine training REST API engine |
| `tray.rs` | System tray |
| `update_checker.rs` | Update checking |
| `version.rs` | Version information |
| `workspace_lease_manager.rs` | Distributed workspace/account lease manager |

## Notes

- Counts: 13 CORE + 17 ADJACENT + 24 SUPPORT = 54 files.
- `proxy/` currently imports from CORE and SUPPORT modules only; the
  `proxy/mod.rs` guardrail documents that ADJACENT modules must stay out of
  the proxy import graph.
- If an ADJACENT module needs proxy data, the dependency must point
  ADJACENT → CORE/SUPPORT, never the reverse.
