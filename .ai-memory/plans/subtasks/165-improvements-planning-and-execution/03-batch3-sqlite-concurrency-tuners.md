# Subtask 03: Batch 3 (Imp 253–258: Split-SQLite Concurrency, Indexing & Pragma Tuners)

- **Parent Plan**: `.ai-memory/plans/pending/165-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/165-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Targeted Improvements
- [x] **Imp-253: PRAGMA Cache-Size Memory Tuner** (`src-tauri/src/modules/db_orm/cache_size_tuner.rs`)
- [x] **Imp-254: Query Plan Index Coverage Inspector** (`src-tauri/src/modules/db_orm/index_usage_inspector.rs`)
- [x] **Imp-255: Explicit Write-Ahead Log Flusher** (`src-tauri/src/modules/db_orm/write_ahead_flush.rs`)
- [x] **Imp-256: Freelist Threshold Vacuum Optimizer** (`src-tauri/src/modules/db_orm/vacuum_optimizer.rs`)
- [x] **Imp-257: Transaction Latency & Health Logger** (`src-tauri/src/modules/db_orm/transaction_logger.rs`)
- [x] **Imp-258: Split-Database Connection Pool Metrics Collector** (`src-tauri/src/modules/db_orm/connection_pool_metrics.rs`)

## Verification Summary
- `cache_size_tuner.rs`: Role-based PRAGMA cache_size calculator assigning optimal memory limits (-KiB).
- `index_usage_inspector.rs`: Query plan parser detecting unindexed table scans vs covering index lookups.
- `write_ahead_flush.rs`: Controlled WAL flush checkpoint manager (`Passive`, `Full`, `Restart`, `Truncate`).
- `vacuum_optimizer.rs`: Freelist-to-page-count ratio analyzer executing incremental vacuum only when justified.
- `transaction_logger.rs`: Transaction latency timer with warning alerts for slow commits (>50ms).
- `connection_pool_metrics.rs`: Real-time reader/writer lease and query metrics collector.
