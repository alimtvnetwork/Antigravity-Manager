# Subtask 03: Rust Type-Safe ORM Abstraction Layer (`db_orm` module)

**Parent Plan:** `.ai-memory/plans/orm-and-database-migration-strategy.md`  
**Status:** PENDING  
**Target Files:** `src-tauri/src/modules/db_orm/` (`mod.rs`, `traits.rs`, `builder.rs`, `connection.rs`)

## Objective
Implement a lightweight, type-safe Rust ORM library on top of `rusqlite` that eliminates raw SQL query strings from business logic:
1. `traits.rs`: `Entity`, `FromRow`, `ToValues`, `ViewEntity`.
2. `builder.rs`: `QueryBuilder` with fluent methods (`select`, `where_eq`, `where_in`, `order_by`, `limit`, `fetch_all`, `fetch_one`, `insert`, `update`, `delete_by_id`).
3. `connection.rs`: Standardized connection opener setting WAL mode, busy timeout 5000ms, foreign keys ON.

## Acceptance Criteria
- Full test coverage in `src-tauri/src/modules/db_orm/tests.rs` verifying fluent queries, inserts, updates, and deletes on in-memory SQLite.
- Seamless conversion to/from Rust structs.
- No silent errors rule enforced (returns `Result<T, AppError>`).
