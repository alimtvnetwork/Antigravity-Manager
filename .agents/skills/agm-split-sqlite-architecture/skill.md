---
name: agm-split-sqlite-architecture
description: Specialized skill for designing, migrating, and maintaining the split SQLite database architecture, encrypted security vaults, and database connections in Antigravity-Manager.
---

# AGM Split-Database SQLite Architecture

This skill guides engineering work on the multi-database persistence layer of Antigravity-Manager. To prevent SQLite write-lock contention under heavy proxy concurrency and to isolate sensitive credentials, AGM partitions storage across dedicated SQLite database files.

## Database Partitioning Directory

| Database File | Module Path | Purpose |
|---|---|---|
| `state.vscdb` | `src-tauri/src/modules/db.rs` | External IDE global storage database read to extract active tokens and profiles. |
| `proxy_logs.db` | `src-tauri/src/modules/proxy_db.rs` | High-throughput HTTP logs, request/response payloads, latency, and status codes. |
| `email_vault.db` | `src-tauri/src/modules/email_vault_db.rs` | IMAP/SMTP account configurations, polling intervals, and remote execution audit logs. |
| `email_passwords.db` | `src-tauri/src/modules/security_db.rs` | Encrypted vault isolating sensitive passwords, API keys, and private tokens. |
| `user_tokens.db` | `src-tauri/src/modules/user_token_db.rs` | Downstream client API tokens, quotas, allowed models, and rate limits. |
| `token_stats.db` | `src-tauri/src/modules/token_stats.rs` | Historical token telemetry (prompt, completion, and thinking tokens). |
| `repo.db` | `src-tauri/src/modules/repo_db.rs` | Repository paths, prompt templates, and project metadata. |

## Engineering Rules & Best Practices

### 1. Concurrency & Connection Safety
- Never share a single SQLite connection across async Tokio tasks without synchronization. Use dedicated connection pools or instantiate short-lived connections per transaction.
- Enable WAL (Write-Ahead Logging) mode on high-throughput databases (`proxy_logs.db`, `token_stats.db`):
  `PRAGMA journal_mode = WAL;`
- Set appropriate busy timeouts (`PRAGMA busy_timeout = 5000;`) to handle lock contention gracefully.

### 2. Migration Protocol
- Database schema initialization functions (`init_db()`) must be idempotent.
- Use `CREATE TABLE IF NOT EXISTS` and check for missing columns using `PRAGMA table_info` before executing `ALTER TABLE ADD COLUMN`.
- Keep migration scripts in `src-tauri/src/modules/migration.rs` or directly within the respective database module.

### 3. Credential Encryption & Vault Isolation
- Passwords and secret keys must NEVER be stored in plain text inside `email_vault.db` or `config.json`.
- All credentials belong in `email_passwords.db` / `security_db.rs`, encrypted using strong authenticated cryptography (AES-GCM / ChaCha20-Poly1305) with machine-bound key derivation.
