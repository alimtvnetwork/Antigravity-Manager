# Subtask 03: Batch 3 (Imp 463–468: Split-SQLite Index Integrity, Page Budget & Query Profiling)

- **Parent Plan**: `.ai-memory/plans/pending/172-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/172-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Improvements to Implement
1. **Imp-463: Composite Index Coverage Verifier** (`src-tauri/src/modules/db_orm/index_coverage_verifier.rs`)
   - Audits table indexes against frequent multi-column query patterns in `active_prompts` and `task_history` split databases.
2. **Imp-464: Dynamic Page Cache Memory Budget Controller** (`src-tauri/src/modules/db_orm/page_budget_controller.rs`)
   - Calculates optimal SQLite `PRAGMA cache_size` based on system available memory and active split database count.
3. **Imp-465: Dynamic Busy Timeout Scaler** (`src-tauri/src/modules/db_orm/busy_timeout_scaler.rs`)
   - Scales `PRAGMA busy_timeout` between 1,000ms and 10,000ms according to recent lock contention history.
4. **Imp-466: Foreign Key Action Rule Validator** (`src-tauri/src/modules/db_orm/foreign_key_action_validator.rs`)
   - Verifies `ON DELETE CASCADE` and `ON UPDATE NO ACTION` foreign key rules across table definitions.
5. **Imp-467: Query Latency Histogram Collector** (`src-tauri/src/modules/db_orm/query_latency_histogram.rs`)
   - Bucketizes query execution times into fixed latency histograms (<1ms, 1-5ms, 5-20ms, >20ms) for runtime telemetry.
6. **Imp-468: WAL Frame Header Integrity Inspector** (`src-tauri/src/modules/db_orm/wal_header_inspector.rs`)
   - Validates low-level 32-byte WAL file headers, salt values, and page checksums before reading WAL frames.
