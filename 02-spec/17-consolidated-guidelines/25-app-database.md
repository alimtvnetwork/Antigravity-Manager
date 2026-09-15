# Consolidated: App Database — SQLite Partitioned Schema Reference

**Version:** 4.7.0
**Updated:** 2026-09-10
**Source Spec:** [`02-spec/23-app-db/01-index.md`](../23-app-db/01-index.md)

---

## 1. Storage Architecture & Partitioned Topology

**Antigravity-Manager** implements a partitioned local storage topology using **SQLite 3** (via `rusqlite` bundled). Rather than storing all application state in a monolithic database, storage is partitioned into three isolated SQLite databases located in the OS App Data directory:

| Database File | Rust Module Source | Primary Responsibility | Data Retention / Lifecycle |
|---|---|---|---|
| `proxy_logs.db` | `src-tauri/src/modules/proxy_db.rs` | Reverse proxy request telemetry, token usage, latency metrics, and payload streaming logs | Pruned via log retention policies / user clear command |
| `security.db` | `src-tauri/src/modules/security_db.rs` | Client IP access history, CIDR blacklist/whitelist rules, and rate limit tracking | Persistent firewall rules; rolling access history |
| `user_tokens.db` | `src-tauri/src/modules/user_token_db.rs` | Multi-user tokens, quotas, curfew timeframes, and client IP binding limits | Persistent auth credentials; cascaded usage logs |

### Architectural Benefits of Partitioning
- **Contention Elimination:** High-frequency write transactions from proxy streaming do not lock security rules or user authentication tokens.
- **Independent Maintenance:** Request logs can be cleared, vacuumed, or backed up without disrupting active auth sessions.
- **Fail-Safe Isolation:** Database corruption in ephemeral logs cannot affect firewall integrity or user tokens.

---

## 2. Mandatory Engine Pragmas & Concurrency Tuning

All SQLite connections opened across `proxy_db.rs`, `security_db.rs`, and `user_token_db.rs` MUST execute the following pragma sequence immediately upon connection initialization:

```sql
PRAGMA journal_mode = WAL;
PRAGMA busy_timeout = 5000;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
```

### Rationale
- **`journal_mode = WAL`:** Write-Ahead Logging allows concurrent readers to query logs without blocking active write transactions from proxy workers.
- **`busy_timeout = 5000`:** Enforces a 5000ms driver retry window to eliminate immediate `SQLITE_BUSY` errors during parallel request bursts.
- **`synchronous = NORMAL`:** Provides full durability against app crashes while avoiding costly per-commit fsync operations.
- **`foreign_keys = ON`:** Guarantees relational integrity and enforces `ON DELETE CASCADE` across foreign key relationships.

---

## 3. Database Schema Definitions

### 3.1 Proxy Logs Database (`proxy_logs.db`)

```sql
CREATE TABLE IF NOT EXISTS request_logs (
    id TEXT PRIMARY KEY,
    timestamp INTEGER NOT NULL,
    method TEXT NOT NULL DEFAULT 'POST',
    url TEXT NOT NULL DEFAULT '',
    status INTEGER NOT NULL,
    duration INTEGER NOT NULL DEFAULT 0,
    account_email TEXT NOT NULL DEFAULT '',
    mapped_model TEXT NOT NULL DEFAULT '',
    protocol TEXT NOT NULL DEFAULT '',
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    cached_tokens INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    client_ip TEXT NOT NULL DEFAULT '',
    username TEXT NOT NULL DEFAULT '',
    request_body TEXT,
    response_body TEXT,
    error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_timestamp ON request_logs (timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_status ON request_logs (status);
```

### 3.2 Security Database (`security.db`)

```sql
CREATE TABLE IF NOT EXISTS ip_access_logs (
    id TEXT PRIMARY KEY,
    client_ip TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    method TEXT,
    path TEXT,
    user_agent TEXT,
    status INTEGER,
    duration INTEGER,
    api_key_hash TEXT,
    blocked INTEGER DEFAULT 0,
    block_reason TEXT,
    username TEXT
);

CREATE TABLE IF NOT EXISTS ip_blacklist (
    id TEXT PRIMARY KEY,
    ip_pattern TEXT NOT NULL UNIQUE,
    reason TEXT,
    created_at INTEGER NOT NULL,
    expires_at INTEGER,
    created_by TEXT DEFAULT 'manual',
    hit_count INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS ip_whitelist (
    id TEXT PRIMARY KEY,
    ip_pattern TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_ip_access_ip ON ip_access_logs (client_ip);
CREATE INDEX IF NOT EXISTS idx_ip_access_timestamp ON ip_access_logs (timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_ip_access_blocked ON ip_access_logs (blocked);
CREATE INDEX IF NOT EXISTS idx_blacklist_pattern ON ip_blacklist (ip_pattern);
```

### 3.3 User Token Database (`user_tokens.db`)

```sql
CREATE TABLE IF NOT EXISTS user_tokens (
    id TEXT PRIMARY KEY,
    token TEXT UNIQUE NOT NULL,
    username TEXT NOT NULL,
    description TEXT,
    enabled BOOLEAN NOT NULL DEFAULT 1,
    expires_type TEXT NOT NULL,
    expires_at INTEGER,
    max_ips INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    last_used_at INTEGER,
    total_requests INTEGER NOT NULL DEFAULT 0,
    total_tokens_used INTEGER NOT NULL DEFAULT 0,
    curfew_start TEXT,
    curfew_end TEXT
);

CREATE TABLE IF NOT EXISTS token_ip_bindings (
    id TEXT PRIMARY KEY,
    token_id TEXT NOT NULL,
    ip_address TEXT NOT NULL,
    first_seen_at INTEGER NOT NULL,
    last_seen_at INTEGER NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 0,
    user_agent TEXT,
    FOREIGN KEY(token_id) REFERENCES user_tokens(id) ON DELETE CASCADE,
    UNIQUE(token_id, ip_address)
);

CREATE TABLE IF NOT EXISTS token_usage_logs (
    id TEXT PRIMARY KEY,
    token_id TEXT NOT NULL,
    ip_address TEXT,
    model TEXT,
    input_tokens INTEGER,
    output_tokens INTEGER,
    request_time INTEGER NOT NULL,
    status INTEGER,
    FOREIGN KEY(token_id) REFERENCES user_tokens(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_token_usage_logs_token_id ON token_usage_logs(token_id);
CREATE INDEX IF NOT EXISTS idx_token_usage_logs_request_time ON token_usage_logs(request_time);
```

---

## 4. Migration & Schema Evolution Strategy

### Forward-Only Additive Migrations
- **Idempotent DDL:** Every table creation uses `CREATE TABLE IF NOT EXISTS`.
- **Column Evolution:** New fields are added during module initialization via non-failing `ALTER TABLE {table} ADD COLUMN {column} {type}` statements wrapped in `if let Err(...)` ignore guards.
- **Index Safety:** Indexes are defined with `CREATE INDEX IF NOT EXISTS`.

### Startup Data Sanitization
- Upon opening `user_tokens.db`, an initialization cleanup pass executes:
  ```sql
  UPDATE user_tokens SET
      total_requests = COALESCE(total_requests, 0),
      total_tokens_used = COALESCE(total_tokens_used, 0),
      max_ips = COALESCE(max_ips, 0)
  WHERE total_requests IS NULL OR total_tokens_used IS NULL OR max_ips IS NULL;
  ```
  This guarantees safe deserialization into Rust structs without runtime panics.

---

## 5. Rusqlite Driver Patterns

1. **Parameterized Queries:** Dynamic query concatenation is strictly forbidden. All parameters must bind via `rusqlite::params![...]`.
2. **Prepared Statements:** Repeated queries (e.g. log insertion in proxy hot paths) must utilize cached prepared statements.
3. **Transaction Batching:** Bulk log flushes or batch account deletions wrap operations in `conn.transaction()?` to reduce disk sync overhead.

---

## 6. Verification Criteria

- **AC-ADB-001 (Pragma Verification):** Every new SQLite connection must verify WAL mode, busy timeout = 5000, and foreign keys = ON.
- **AC-ADB-002 (Migration Verification):** Startup migrations must execute cleanly on both empty databases and older version schemas without crashing.
- **AC-ADB-003 (Cascade Verification):** Deleting a record from `user_tokens` must cascade to all associated records in `token_ip_bindings` and `token_usage_logs`.

---

*Consolidated app database — Antigravity-Manager v4.7.0*
