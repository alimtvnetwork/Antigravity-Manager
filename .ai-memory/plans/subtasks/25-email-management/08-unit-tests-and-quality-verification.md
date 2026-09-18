# Subtask 08: Unit Tests & Quality Verification

> **Parent Plan:** `.ai-memory/plans/pending/25-email-management-split-security-db-and-remote-control.md`
> **Specification:** `02-spec/21-app/16-email-dispatch-mailbox-remote-management-and-split-security-db.md`
> **Status:** Completed
> **Files:** `src-tauri/src/modules/email_vault_db.rs`, `src-tauri/src/modules/email_io.rs`, `src-tauri/src/modules/email_inbound.rs`

---

## Objective

Author comprehensive unit tests in Rust modules to test database schema migrations, password hash/encryption round-trips, JSON/CSV/XLSX import-export parser correctness, command matcher edge cases, and watcher rule evaluations without triggering unmocked system/network calls.

## Requirements

1. **Test Coverage:**
   - DB initializations, WAL pragma application, foreign key constraint behavior.
   - Credentials encryption and salted key derivation verification.
   - CSV and JSON serialization / deserialization round-trip.
   - Inbound email command parsing (`Project: <name>`, `exec: <ip>`, `instance: new`, `rotate: accounts`, `help`).
   - Watcher sensor threshold calculations and idle project detection.
2. **Quality Verification:**
   - Strictly follow coding guidelines:
     - No explicit `== true`.
     - Positive booleans (`is_active`, `is_default`, `is_enabled`).
     - Functions kept small (8–15 lines where possible).
     - No local runners or build commands executed locally (defer to CI/CD pipeline).
