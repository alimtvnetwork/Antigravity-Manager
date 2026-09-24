# App Issue RCA 03: IP Security Statistics Null Column Conversion & IPC Access Log Argument Mismatch

> **Specification Reference:** `02-spec/22-app-issues/03-ip-security-null-and-ipc-query-fix.md`
> **Target Subsystem:** Security IP Diagnostics (`src-tauri/src/modules/security_db.rs`, `src-tauri/src/commands/security.rs`, `src/components/security/IpAccessLogs.tsx`)
> **Severity:** High
> **Status:** Fixed
> **Date:** 2026-09-24

---

## 1. Reproduction

1. **Defect A: Null Column Conversion in `get_ip_stats`:**
   - Fresh install or environment where `ip_access_logs` in `security.db` has 0 rows.
   - Navigate to `/security` page.
   - **Observed Behavior:** The global Error Modal opens with:
     `tauri.get_ip_stats -> Invalid column type Null at index: 2, name: blocked`
     The IP overview dashboard cards fail to populate.
2. **Defect B: Missing Required Key `query` in `get_ip_access_logs`:**
   - On the `/security` page under the Access Logs tab:
   - **Observed Behavior:** The logs table fails to load with error:
     `invalid args query for command get_ip_access_logs: command get_ip_access_logs missing required key query`

---

## 2. Cause

1. **Root Cause of Defect A (SQLite Aggregate NULL Semantics):**
   - In SQLite, aggregate functions `COUNT(*)` return `0` on empty tables, but `SUM(...)` over zero matching rows returns `NULL`.
   - In `security_db.rs`:
     ```rust
     SUM(CASE WHEN blocked = 1 THEN 1 ELSE 0 END) as blocked,
     SUM(CASE WHEN timestamp >= ?1 THEN 1 ELSE 0 END) as today
     ```
   - When no records exist, both `blocked` (column index 2) and `today` (column index 3) evaluate to SQL `NULL`.
   - The query mapping attempted to convert the column into Rust `u64`:
     `row.get(2)?`
   - Rusqlite threw a type conversion error `FromSqlConversionFailure(2, Integer, InvalidColumnType("Null"))`.
2. **Root Cause of Defect B (Tauri Command Serialization Mismatch):**
   - In `src-tauri/src/commands/security.rs`, the command was declared as:
     `pub async fn get_ip_access_logs(query: IpAccessLogQuery) -> Result<...>`
   - Tauri v2 command reflection expects the JSON payload to have a top-level key matching the Rust argument name: `{ "query": { ... } }`.
   - However, in `src/components/security/IpAccessLogs.tsx`, the frontend invoked:
     `invoke('get_ip_access_logs', { page, pageSize, search, blockedOnly })`
   - Because the argument `query` was marked non-optional in Rust and was missing from the root of the invoke payload, Tauri's deserializer failed before entering the function body.

---

## 3. Fix

1. **SQL `COALESCE` & Option Mapping in `security_db.rs`:**
   - Wrapped aggregate sums in `COALESCE(SUM(...), 0)`.
   - Safely deserialized all row values through `Option<u64>` with fallback to `0`:
     `row.get::<_, Option<u64>>(2)?.unwrap_or(0)`
   - Applied identical null-safe logic to `today`, `total`, and `unique_ips`.
2. **Dual Payload Compatibility in `commands/security.rs` & `IpAccessLogs.tsx`:**
   - Updated Rust command signature to accept both nested `query: Option<IpAccessLogQuery>` and direct top-level arguments (`page: Option<usize>`, `page_size: Option<usize>`, `search: Option<String>`, `blocked_only: Option<bool>`).
   - Defaulted missing parameters safely (`page = 1`, `page_size = 50`, `blocked_only = false`).
   - In `IpAccessLogs.tsx`, formatted invoke payload to supply both `{ query: { ... }, page, pageSize, search, blockedOnly }`, ensuring full compatibility across Tauri IPC and HTTP fallback routes.

---

## 4. Prevention

1. **Mandatory `COALESCE` for SQL Aggregates**:
   - Any SQL query using `SUM`, `AVG`, `MIN`, `MAX` across tables that may be empty must wrap expressions in `COALESCE(..., 0)`.
2. **Nullable Column Deserialization**:
   - Prefer `row.get::<_, Option<T>>()?.unwrap_or_default()` over strict `row.get::<_, T>()?` for aggregate projections.
3. **IPC Contract Testing**:
   - Ensure IPC command parameters provide `Option<T>` or `#[serde(default)]` whenever optional frontend queries are allowed.
