# Migration & Verification Specification: FIFO Queue SQLite Composite Index Analyzer (`Imp-663`)

## Executive Summary
This specification defines the migration strategy, index verification criteria, table schemas, and automated remediation query generation for prompt queue databases within Antigravity-Manager.

In accordance with the strict architectural mandate codified in `AGENTS.md`:
> **FIFO Restore & Queue Invariant**: All prompt restore, dispatch, and queue processing queries across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`, `ORDER BY created_at DESC`) are strictly forbidden in dispatch logic.

To guarantee $O(\log N)$ index seek performance and eliminate runtime table scans and temporary B-tree sorting overhead (`USE TEMP B-TREE FOR ORDER BY`), each prompt queue table must maintain a dedicated composite index on `(created_at ASC, id ASC)`.

---

## 1. Architectural Problem Analysis & Motivation

### 1.1 The Performance Risk of Unindexed FIFO Queries
Prompt queue dispatch engines execute frequent poll and restore queries targeting the earliest pending entries:
```sql
SELECT id, prompt_text, created_at, status
FROM active_prompts
WHERE status = 'pending'
ORDER BY created_at ASC, id ASC
LIMIT 1;
```

In SQLite, if only a primary key index on `id` exists:
1. SQLite must execute a full table scan (`SCAN TABLE active_prompts`).
2. SQLite allocates an ephemeral in-memory B-tree structure (`USE TEMP B-TREE FOR ORDER BY`).
3. All candidate rows are copied into the temporary B-tree and sorted on `(created_at, id)`.
4. As prompt history grows beyond thousands of records, this operation causes CPU throttling, page cache eviction, and extended read lock duration.

### 1.2 The Composite Index Solution
A composite index defined as `(created_at ASC, id ASC)` provides:
- **Clustered Ordering**: Records are physically traversed in canonical FIFO order directly from the index B-tree.
- **Zero Temporary Sort**: The query planner resolves `ORDER BY created_at ASC, id ASC` with zero temporary structures.
- **Deterministic Tie-Breaking**: When multiple prompts share identical millisecond timestamps, `id ASC` deterministically breaks ties without nondeterministic jitter across pages or pagination boundaries.

---

## 2. Target Table Schemas & Index Definitions

### 2.1 Table Schemas Under Audit

#### `active_prompts`
Main active queue table tracking in-flight and pending prompts dispatched to IDE instances.
```sql
CREATE TABLE IF NOT EXISTS active_prompts (
    id TEXT PRIMARY KEY NOT NULL,
    prompt_text TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER,
    status TEXT NOT NULL,
    instance_id TEXT,
    project_name TEXT
);
```

#### `prompt_backups`
Historical backup table tracking prompt snapshots preserved across app restarts and crashes.
```sql
CREATE TABLE IF NOT EXISTS prompt_backups (
    id TEXT PRIMARY KEY NOT NULL,
    prompt_id TEXT NOT NULL,
    prompt_text TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    backed_up_at INTEGER NOT NULL,
    status TEXT NOT NULL,
    metadata_json TEXT
);
```

#### `repo_prompts`
Repository-scoped prompt log tracking prompt executions attached to local workspaces.
```sql
CREATE TABLE IF NOT EXISTS repo_prompts (
    id TEXT PRIMARY KEY NOT NULL,
    repo_path TEXT NOT NULL,
    prompt_text TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    executed_at INTEGER,
    status TEXT NOT NULL
);
```

### 2.2 Canonical Composite Index Specifications
Each prompt table requires a corresponding composite index following standard naming conventions:

| Target Table | Index Name | Columns & Sort Direction |
| :--- | :--- | :--- |
| `active_prompts` | `idx_active_prompts_created_at_id` | `(created_at ASC, id ASC)` |
| `prompt_backups` | `idx_prompt_backups_created_at_id` | `(created_at ASC, id ASC)` |
| `repo_prompts` | `idx_repo_prompts_created_at_id` | `(created_at ASC, id ASC)` |

---

## 3. Index Verification Criteria & Risk Analysis

The `FifoQueueIndexAnalyzer` introspects the SQLite schema using `PRAGMA index_list` and `PRAGMA index_xinfo` (with fallback to `PRAGMA index_info`).

### 3.1 Verification Status Classification (`FifoIndexStatusType`)

1. **`FullyCompliantType`**:
   - The table contains an index where leading columns match `created_at` (column 0) and `id` (column 1).
   - Both column sort directions are strictly ascending (`desc == 0`).
   - Risk Level: `NoneType`.
   - Remediation: None required.

2. **`MissingCompositeIndexType`**:
   - The table exists, but no index includes both `created_at` and `id` in leading positions.
   - Risk Level: `HighType` (or `CriticalType` if table has > 10,000 entries).
   - Remediation: Generate automated `CREATE INDEX` SQL statement.

3. **`ImproperColumnOrderType`**:
   - An index exists containing both columns, but in reversed order (e.g. `(id ASC, created_at ASC)`).
   - Risk Level: `HighType`.
   - Remediation: Generate automated `CREATE INDEX` SQL statement for `(created_at ASC, id ASC)`.

4. **`ImproperSortDirectionType`**:
   - An index exists on `(created_at, id)`, but with descending ordering (e.g. `(created_at DESC, id ASC)`).
   - This indexes for LIFO instead of FIFO, violating `AGENTS.md`.
   - Risk Level: `HighType`.
   - Remediation: Generate automated `CREATE INDEX` SQL statement.

5. **`TableNotFoundType`**:
   - The inspected table does not exist in the database.
   - Risk Level: `MediumType`.
   - Remediation: Table creation required before index generation.

### 3.2 Risk Level Matrix (`IndexRiskLevelType`)

| Risk Level | Trigger Condition | Operational Impact | Recommended Action |
| :--- | :--- | :--- | :--- |
| `NoneType` | Fully compliant composite index present | Optimal query execution ($O(\log N)$) | No action needed |
| `LowType` | Auxiliary partial index present | Minor query plan deviation | Optional index optimization |
| `MediumType` | Table not yet initialized | Initialization pending | Create table and index on startup |
| `HighType` | Table exists without composite index | Full table scan + temp B-tree sort | Apply automated remediation SQL |
| `CriticalType` | High-frequency queue table lacking index | Immediate risk of lock timeouts | Execute immediate index migration |

---

## 4. Migration Strategy & Automated Remediation

### 4.1 Automated Remediation Query Generation
For any table diagnosed with `MissingCompositeIndexType`, `ImproperColumnOrderType`, or `ImproperSortDirectionType`, the analyzer automatically produces idempotent DDL queries:

```sql
CREATE INDEX IF NOT EXISTS idx_<table_name>_created_at_id ON <table_name>(created_at ASC, id ASC);
```

Specifically:
```sql
CREATE INDEX IF NOT EXISTS idx_active_prompts_created_at_id ON active_prompts(created_at ASC, id ASC);
CREATE INDEX IF NOT EXISTS idx_prompt_backups_created_at_id ON prompt_backups(created_at ASC, id ASC);
CREATE INDEX IF NOT EXISTS idx_repo_prompts_created_at_id ON repo_prompts(created_at ASC, id ASC);
```

### 4.2 Migration Execution Phases
1. **Introspection Phase**:
   - At startup or database connection opening, `FifoQueueIndexAnalyzer::analyze(&conn)` evaluates all queue tables.
   - Generates a structured `FifoIndexAnalysisReport`.

2. **Remediation Phase**:
   - If `report.is_fully_indexed` is `false`, `FifoQueueIndexAnalyzer::apply_remediations(&conn, &report)` executes each statement within a transaction or directly on the connection.
   - Uses `IF NOT EXISTS` to ensure strict idempotency across multiple process instances.

3. **Re-verification Phase**:
   - Post-remediation inspection validates that `report.missing_index_count == 0` and all tables report `FullyCompliantType`.

---

## 5. Verification Protocol

### 5.1 SQLite Query Plan Verification (`EXPLAIN QUERY PLAN`)
The success of composite index migration is verified by examining the SQLite query plan:

#### Before Index Creation:
```text
QUERY PLAN
|--SCAN active_prompts
`--USE TEMP B-TREE FOR ORDER BY
```

#### After Index Creation:
```text
QUERY PLAN
`--SCAN active_prompts USING INDEX idx_active_prompts_created_at_id
```

### 5.2 Unit & Integration Test Matrix
Located at `src-tauri/src/modules/db_orm/tests_fifo_queue_index_analyzer.rs`:
- `test_generate_remediation_sql`: Confirms exact SQL text syntax.
- `test_empty_database_table_not_found`: Confirms `TableNotFoundType` handling.
- `test_missing_composite_index_detected`: Confirms detection of unindexed tables.
- `test_existing_composite_index_compliant`: Confirms detection of compliant indexes.
- `test_improper_sort_direction_detected`: Intercepts `DESC` sort indexes.
- `test_improper_column_order_detected`: Intercepts reversed column indexes.
- `test_partial_single_column_index_detected`: Intercepts single-column `created_at` indexes lacking `id`.
- `test_full_analysis_report_all_tables`: Verifies multi-table summary metrics.
- `test_apply_remediations_achieves_full_compliance`: Confirms that remediation achieves 100% compliance.
