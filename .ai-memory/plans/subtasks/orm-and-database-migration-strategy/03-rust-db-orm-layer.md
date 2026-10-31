# Subtask 03: Rust Type-Safe ORM Abstraction Layer (`db_orm` module)

**Task ID:** `Task-03`  
**Parent Plan:** `orm-and-database-migration-strategy`  
**Owner:** Worker 01  
**Status:** PENDING  

## Objective
Implement `src-tauri/src/modules/db_orm/` containing core ORM traits, a type-safe `QueryBuilder`, connection management with WAL and 5000ms busy timeout, and error handling.

## Target Files
- `src-tauri/src/modules/db_orm/mod.rs`
- `src-tauri/src/modules/db_orm/traits.rs`
- `src-tauri/src/modules/db_orm/builder.rs`
- `src-tauri/src/modules/db_orm/connection.rs`
- `src-tauri/src/modules/mod.rs` (register `pub mod db_orm;`)

## Invariants
- `QueryBuilder` supports `select::<T>()`, `insert(&entity)`, `update(&entity)`, `delete_by_id(id)`.
- Enforces FIFO ordering queries (`ORDER BY CreatedAt ASC, ActivePromptId ASC`).
- Zero unwrap without error logging (`AppError`).
- Compatible with existing `rusqlite = "0.32"`.
