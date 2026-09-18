# Split SQLite Logging Architecture & Task DB Migration

**Date:** 2026-09-09
**Status:** Learned & Consolidated
**Scope:** `04-code/golang/pkg/applogger`, `04-code/golang/pkg/sqlitelogger`, `02-spec/04-database-conventions/`

---

## 1. Context & Architectural Motivation

In high-throughput agentic and worker workflows, concurrent task logging into a single monolithic SQLite database creates severe lock contention, unbounded file growth, and slow queries.

To solve this, the logging infrastructure implements a **Split SQLite Database Architecture**:
1. **Root Log Database (`logs.db`):** Stores system-level, lifecycle, and global orchestration events.
2. **Task Log Databases (`tasks/<taskId>.db`):** Dedicated, isolated SQLite database file per task execution.

---

## 2. Core Principles & Path Configuration

1. **Configurable Base Directories:**
   - Both the main DB directory (`workDir`) and the task DB directory (`tasksDir`) MUST be configurable via options (`WithWorkDir(path)`, `WithTasksDir(path)`).
   - Default convention:
     - Main DB: `<workDir>/logs.db`
     - Task DBs: `<workDir>/tasks/<taskId>.db`
2. **Dynamic Task Isolation:**
   - Writing task logs opens or connects to the specific `<taskId>.db` without acquiring locks on `logs.db` or other sibling task databases.
   - Reading or querying task history operates on the task-specific database, allowing zero-contention log streaming to API consumers.

---

## 3. Mandatory Task DB Auto-Migration & Schema Self-Repair

When the system opens or interacts with any task database, it MUST NOT assume the schema is current or intact.

### Non-Negotiable Contract
- **Auto-Migration on Open:** Before performing any read, query, or append operation on a task database, the connection pipeline runs schema verification and auto-migration (`AutoMigrateTaskSchema(db)`).
- **Self-Repair:** If required columns (e.g. `SpanId`, `TraceId`, `Timestamp`) or indexes are missing due to legacy versions or abnormal termination, the migration logic safely executes `ALTER TABLE` or creates missing indexes.
- **Idempotency:** Schema migrations are strictly idempotent and non-destructive. Existing task records must never be truncated or corrupted during migration.

---

## 4. Text-Based Log Rotation

For plain-text file logging sinks, rotation mechanics prevent runaway disk utilization:
- **Triggers:** Rotates when current log file exceeds `MaxSizeMb` (default: 10MB).
- **Retention:** Retains up to `MaxBackups` (default: 5) and purges files older than `MaxAgeDays`.
- **Timestamping:** Rotated archives use deterministic UTC or local timestamps (`app.log.2026-09-09T04-00-00.gz` or `.1`, `.2` index scheme).
- **Concurrency Safety:** Thread-safe multi-process write protection via file locks.

---

## 5. Verification & Quality Standards

- 100% unit and integration test coverage for task DB creation, schema repair, concurrent writes, and rotation.
- Strict error propagation using `*appfault.AppError`.
