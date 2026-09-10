# App DB — Storage Architecture, Schemas, and Migration Specification

> **/goal** Master and enforce the architectural standards, specifications, and CI/CD validation rules for 23 App Db.
> **/learn** Read the sequentially ordered specification files in this directory, follow the actionable CI/CD checklist, and apply mandatory rules before generating code.

## 🎯 Actionable CI/CD & Agent Checklist

- [ ] `/goal` Read and understand all numbered specifications under `02-spec/23-app-db/`.
- [ ] `/learn` Adhere strictly to `.lovable/folder-structure.md` and `.lovable/strictly-avoid.md`.
- [ ] `/goal` Verify zero explicit `true` boolean evaluations and no mixed-polarity conditionals.
- [ ] `/learn` Run all local verification linters via `python 03-ai-scripts/06-cicd-local-runner.py`.

. **CRITICAL AI INSTRUCTION:** This `01-index.md` file is the primary entry point for this directory. AI agents MUST read this file first before exploring other files in this folder.

**Version:** 4.7.0
**Updated:** 2026-09-10
**AI Confidence:** Production-Ready
**Ambiguity:** None

---

## Keywords

`app-db` · `sqlite` · `wal-mode` · `proxy-logs` · `security-db` · `user-tokens` · `migrations` · `schemas`

---

## Scoring

| Criterion | Status |
|-----------|--------|
| `01-index.md` present | ✅ |
| AI Confidence assigned | ✅ |
| Ambiguity assigned | ✅ |
| Keywords present | ✅ |
| Scoring table present | ✅ |

---

## Purpose

Application-specific database (App DB) specification for **Antigravity-Manager**. The desktop gateway and reverse proxy partitions local storage into three isolated SQLite databases (`proxy_logs.db`, `security.db`, and `user_tokens.db`). Partitioning prevents locking contention, isolates high-frequency request telemetry from firewall rules and authentication state, and provides independent data retention policies.

---

## Document Inventory & Storage Topology

| # | Database File | Rust Module Source | Storage Location | Responsibility |
|---|---------------|--------------------|------------------|----------------|
| 1 | `proxy_logs.db` | `src-tauri/src/modules/proxy_db.rs` | App Data Directory | Request telemetry, token usage, latency metrics, and payload streaming logs |
| 2 | `security.db` | `src-tauri/src/modules/security_db.rs` | App Data Directory | IP access history, CIDR blacklist/whitelist rules, and rate limit enforcement |
| 3 | `user_tokens.db` | `src-tauri/src/modules/user_token_db.rs` | App Data Directory | Multi-user tokens, quotas, curfew timeframes, and client IP binding limits |

---

## Engine Pragmas & Concurrency Tuning

All SQLite connections instantiated across `proxy_db.rs`, `security_db.rs`, and `user_token_db.rs` apply the following mandatory pragma sequence upon opening:

```sql
PRAGMA journal_mode = WAL;
PRAGMA busy_timeout = 5000;
PRAGMA synchronous = NORMAL;
```

- **`journal_mode = WAL`:** Write-Ahead Logging allows concurrent readers to query logs without blocking active write transactions from proxy workers.
- **`busy_timeout = 5000`:** Sets a 5000ms driver retry window to eliminate immediate `SQLITE_BUSY` contention during parallel burst traffic.
- **`synchronous = NORMAL`:** Avoids full disk syncs on every commit in WAL mode while preserving crash safety against power loss.

---

## 1. Proxy Logs Database (`proxy_logs.db`)

Managed by `src-tauri/src/modules/proxy_db.rs`. Records inbound reverse proxy requests with full token accounting and latency metrics:

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

**Schema Migrations:**
- Added columns (`request_body`, `response_body`, `input_tokens`, `output_tokens`, `cached_tokens`, `account_email`, `mapped_model`, `protocol`, `client_ip`, `username`) via non-failing `ALTER TABLE` statements during module initialization.

---

## 2. Security Database (`security.db`)

Managed by `src-tauri/src/modules/security_db.rs`. Powers the local firewall, CIDR pattern blocking, and access auditing:

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

**Schema Migrations:**
- Added `username TEXT` to `ip_access_logs` via idempotent `ALTER TABLE`.

---

## 3. User Token Database (`user_tokens.db`)

Managed by `src-tauri/src/modules/user_token_db.rs`. Supports team deployment with token provisioning, quotas, and curfew enforcement:

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

**Schema Migrations & Data Sanitization:**
- Forward-only `ALTER TABLE` additions for `expires_type`, `expires_at`, `max_ips`, `total_requests`, `total_tokens_used`, `last_used_at`, `curfew_start`, and `curfew_end`.
- Data cleaning pass on init executes `UPDATE user_tokens SET ... WHERE ... IS NULL` to repair legacy NULL columns and guarantee deserialization safety.

---

## Cross-References

- [Backend Modules & Persistence](../21-app/04-modules-storage-and-persistence.md) — Module architecture & persistence lifecycle
- [Split DB Architecture](../05-split-db-architecture/01-index.md) — SQLite partitioning patterns and concurrency guidelines
- [Database Conventions](../04-database-conventions/01-index.md) — General naming, PK/FK, and index design standards
- [Security & Risk Model](../21-app/02-security-and-risks.md) — SQL injection defense and credential security

---

## Verification & Acceptance Criteria

### AC-ADB-001: App Database Concurrency & Pragma Configuration
- **Given:** SQLite connections opened for `proxy_logs.db`, `security.db`, or `user_tokens.db`.
- **When:** `connect_db()` initializes SQLite handles via `rusqlite`.
- **Then:** Connections execute `PRAGMA journal_mode = WAL`, `PRAGMA busy_timeout = 5000`, and `PRAGMA synchronous = NORMAL`.

### AC-ADB-002: Forward Migration & Idempotent DDL Execution
- **Given:** An existing SQLite database from an earlier application version.
- **When:** `init_db()` is invoked during application startup.
- **Then:** All DDL migrations execute forward-only without data loss, newly introduced columns default safely without runtime panic, and indices are created idempotently (`IF NOT EXISTS`).

### AC-ADB-003: Multi-Tenant Token Referential Integrity & Cascade Deletion
- **Given:** User tokens stored in `user_tokens.db` with associated `token_ip_bindings` and `token_usage_logs`.
- **When:** A user token record is deleted from `user_tokens`.
- **Then:** Foreign key cascade deletion removes all dependent IP bindings and usage records, preventing orphaned database artifacts.
