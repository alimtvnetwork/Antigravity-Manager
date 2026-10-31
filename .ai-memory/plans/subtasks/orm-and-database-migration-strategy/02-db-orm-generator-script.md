# Subtask 02: Python Code Generator Script (`51-db-orm-generator.py`)

**Parent Plan:** `.ai-memory/plans/orm-and-database-migration-strategy.md`  
**Status:** IN PROGRESS  
**Target File:** `03-ai-scripts/51-db-orm-generator.py`  

## Objective
Implement an automated Python script (`03-ai-scripts/51-db-orm-generator.py`) that reads declarative SQLite schema definitions and generates type-safe Rust code for the `db_orm` module:
1. `entities.rs`: Strongly typed Rust structs for each table, with `serde` PascalCase attributes, `FromRow`, and `ToValues` implementations.
2. `tables.rs`: Table name constants, column name constants, and DDL SQL strings.
3. `views.rs`: Readonly view models with `FromRow` implementations.
4. `queries.rs`: CRUD helper traits and implementations for each entity.

## Acceptance Criteria
- Script runs with `python 03-ai-scripts/51-db-orm-generator.py --out-dir src-tauri/src/modules/db_orm/generated`.
- Output Rust files compile with `cargo clippy --lib`.
- Enforces singular PascalCase for all tables, columns, and view names.
- Enforces affirmative boolean prefixes (`Is...`, `Has...`).
