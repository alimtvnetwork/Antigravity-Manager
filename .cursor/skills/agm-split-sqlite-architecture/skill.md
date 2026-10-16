---
name: agm-split-sqlite-architecture
description: Specialized skill for managing Antigravity-Manager decoupled SQLite databases, WAL concurrency mode, schema migrations, and AES-256-GCM encrypted security vaults.
---

# AGM Split-Database SQLite Architecture

This skill provides comprehensive architectural guidance, schema rules, migration standards, and encryption specifications for the Split SQLite Database subsystem in Antigravity-Manager.

---

## 1. Subsystem Architecture Overview

To eliminate database locking bottlenecks and prevent high-frequency proxy logs from blocking credential vaults or prompt synchronization, Antigravity-Manager uses a decoupled SQLite split-database design:

```
<data_dir>/
├── repo_prompts.db         # Workspace prompts & active project registry
├── backup-prompts.db       # Prompt backup snapshots with Base64 image payloads
├── security.db             # Reverse proxy IP access logs, blacklist & whitelist
├── user_tokens.db          # Downstream client API tokens, rate limits & IP bindings
├── token_stats.db          # Aggregated hourly/daily token usage telemetry
├── proxy.db                # Proxy request logs & L2 ThinkingStore sessions
├── email_vault.db          # Email accounts, notification recipients & inbound audits
├── email_passwords.db      # AES-256-GCM encrypted credential vault (passwords & keys)
├── instances.db            # Active instance process PID registry
└── instances/<id>/data/.../state.vscdb  # Protobuf OAuth token table (VS Code internal)
```

---

## 2. Key Files & Core Responsibilities

| File Path | Database Handled | Core Responsibilities |
|---|---|---|
| `src-tauri/src/modules/repo_db.rs` | `repo_prompts.db` | Active project registration, prompt status tracking, and GitMap sequence numbering (`[AGM:P001 \| GM:#1]`). |
| `src-tauri/src/modules/backup_prompts_db.rs` | `backup-prompts.db` | Parallel prompt snapshots, image payload preservation (`has_images`), batch restoration, and prompt re-injection. |
| `src-tauri/src/modules/security_db.rs` | `security.db` | IP access logging, CIDR subnet matching, blacklist/whitelist enforcement, and attack mitigation. |
| `src-tauri/src/modules/user_token_db.rs` | `user_tokens.db` | Downstream client authentication, token generation, rate limits, and curfew access rules. |
| `src-tauri/src/modules/token_stats.rs` | `token_stats.db` | Aggregated token I/O statistics, model-based cost tracking, and quota consumption analytics. |
| `src-tauri/src/proxy/proxy_db.rs` | `proxy.db` | L2 thinking store cache persistence, full request/response log auditing. |
| `src-tauri/src/modules/email_vault_db.rs` | `email_vault.db` & `email_passwords.db` | Dual-DB email configuration and AES-256-GCM encrypted secret management. |
| `src-tauri/src/modules/instance.rs` | `instances.db` | Instance process tracking and PID state across reboots. |
| `src-tauri/src/modules/db.rs` | `state.vscdb` | VS Code `ItemTable` SQLite reader and Protobuf credential serializer. |

---

## 3. Mandatory SQLite Pragmas & Concurrency Rules

Every SQLite connection opened across the Rust backend **must** execute these initialization pragmas:

```sql
PRAGMA journal_mode = WAL;
PRAGMA busy_timeout = 5000;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
```

### Concurrency Rules:
1. **WAL Mode**: Write-Ahead Logging allows concurrent readers without being blocked by writers.
2. **Busy Timeout**: Always set `busy_timeout = 5000` (5 seconds) to handle transient contention gracefully without throwing `SQLITE_BUSY`.
3. **Database Segregation**: Never combine disparate functional concerns into a single SQLite file. High-frequency write tables (`request_logs`, `ip_access_logs`) must remain strictly isolated from configuration tables.

---

## 4. Encrypted Password Vault (`email_passwords.db`)

Sensitive credentials (IMAP passwords, SMTP passwords, SSH private keys) are strictly isolated from general configuration and encrypted at rest:

```sql
CREATE TABLE IF NOT EXISTS email_credentials (
    account_id TEXT PRIMARY KEY,
    encrypted_secret TEXT NOT NULL,
    salt TEXT NOT NULL,
    rsa_public_fingerprint TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

- **Encryption Standard**: AES-256-GCM with PBKDF2-derived keys and unique random salts per record.
- **Physical Separation**: `email_passwords.db` has restricted OS file permissions and is never included in plain configuration exports.

---

## 5. Migration & Schema Versioning Invariants

1. **Idempotent Table Creation**: All table initialization queries must use `CREATE TABLE IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS`.
2. **Column Additions**: When adding fields to existing tables, check `PRAGMA table_info(<table_name>)` prior to executing `ALTER TABLE ADD COLUMN`.
3. **PascalCase & Positive Booleans**:
   - Table column names follow standardized conventions.
   - Boolean columns must use positive affirmative naming (e.g. `is_active`, `is_restored`, `has_images`, `is_whitelisted`).
