# Subtask 02: Code Generator Script (`51-db-orm-generator.py`) for Rust ORM Models

**Task ID:** `Task-02`  
**Parent Plan:** `orm-and-database-migration-strategy`  
**Owner:** Worker 01  
**Status:** PENDING  

## Objective
Implement `03-ai-scripts/51-db-orm-generator.py` following GitMap generator conventions (`35-db-struct-enum-generator.py`) to automatically generate Rust typed entity models, table constants, and view structs with PascalCase attributes and affirmative booleans.

## Target Files
- `03-ai-scripts/51-db-orm-generator.py`
- `src-tauri/src/modules/db_orm/generated/mod.rs`
- `src-tauri/src/modules/db_orm/generated/entities.rs`
- `src-tauri/src/modules/db_orm/generated/tables.rs`
- `src-tauri/src/modules/db_orm/generated/views.rs`

## Invariants
- Python script executes cleanly without external third-party dependencies beyond standard library and `02-shared-engine.py`.
- Generates 100% PascalCase Rust structs with `#[serde(rename_all = "PascalCase")]`.
- Strictly enforces affirmative booleans (`Is...`, `Has...`). Rejects negative column names.
- Outputs clean Rust code formatted with `cargo fmt`.
