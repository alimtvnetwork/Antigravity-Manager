//! Strict FIFO Queue SQLite Composite Index Analyzer (`Imp-663`).
//!
//! Enforces the `AGENTS.md` database indexing invariant:
//! "FIFO Restore & Queue Invariant: All prompt restore, dispatch, and queue processing queries
//! across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce
//! strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO
//! queries (`ORDER BY updated_at DESC`, `ORDER BY created_at DESC`) are strictly forbidden in dispatch logic."
//!
//! Introspects table index schemas, detects missing or improper composite indexes on `(created_at ASC, id ASC)`,
//! flags performance and ordering risks, and generates automated idempotent remediation SQL.

use crate::error::AppError;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fmt;
use tracing::{debug, warn};

/// Target column names required for strict FIFO composite index coverage.
pub const CANONICAL_FIFO_INDEX_COLUMNS: &[&str] = &["created_at", "id"];

/// Canonical prompt queue tables subject to strict FIFO composite index validation.
pub const DEFAULT_PROMPT_QUEUE_TABLES: &[&str] = &[
    "active_prompts",
    "prompt_backups",
    "repo_prompts",
];

/// Compliance status of a table's index schema relative to strict FIFO ordering requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FifoIndexStatusType {
    /// Composite index on `(created_at ASC, id ASC)` is present and compliant.
    FullyCompliantType,
    /// Table exists, but composite index covering `(created_at ASC, id ASC)` is missing.
    MissingCompositeIndexType,
    /// Index exists but columns are reversed (e.g. `(id, created_at)`).
    ImproperColumnOrderType,
    /// Index exists on `(created_at, id)` but with descending sort direction (`DESC`).
    ImproperSortDirectionType,
    /// The target table does not exist in the inspected SQLite database.
    TableNotFoundType,
}

impl fmt::Display for FifoIndexStatusType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FullyCompliantType => write!(f, "Fully Compliant"),
            Self::MissingCompositeIndexType => write!(f, "Missing Composite Index"),
            Self::ImproperColumnOrderType => write!(f, "Improper Column Order"),
            Self::ImproperSortDirectionType => write!(f, "Improper Sort Direction"),
            Self::TableNotFoundType => write!(f, "Table Not Found"),
        }
    }
}

/// Risk level classification for missing or sub-optimal FIFO queue indexing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IndexRiskLevelType {
    /// Zero risk: composite index provides O(log N) traversal without temporary sorting.
    NoneType,
    /// Low risk: table has secondary indexes that partially reduce table scan cost.
    LowType,
    /// Medium risk: table is uninitialized or single-column index is present without tie-breaker.
    MediumType,
    /// High risk: full table scan and temporary B-tree sort required for FIFO queries.
    HighType,
    /// Critical risk: high-volume table with zero index coverage, causing lock starvation.
    CriticalType,
}

impl fmt::Display for IndexRiskLevelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoneType => write!(f, "None"),
            Self::LowType => write!(f, "Low"),
            Self::MediumType => write!(f, "Medium"),
            Self::HighType => write!(f, "High"),
            Self::CriticalType => write!(f, "Critical"),
        }
    }
}

/// Internal evaluation result when matching an index against canonical FIFO requirements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexMatchResultType {
    /// Perfect match on `(created_at ASC, id ASC)`.
    MatchesCompositeFifoType,
    /// Columns are present but in reversed order (e.g. `(id, created_at)`).
    InvertedColumnOrderType,
    /// Columns match but one or both are indexed in descending order.
    ImproperSortDirectionType,
    /// Partial match (e.g. only `created_at` indexed without `id`).
    PartialMatchType,
    /// Completely unrelated index.
    NoMatchType,
}

/// Detailed column specification within an indexed schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexColumnDetail {
    /// Sequence rank of the column within the index (0-based).
    pub seqno: i32,
    /// Column name.
    pub name: String,
    /// Whether the index sorts this column in ascending order.
    pub is_ascending: bool,
}

impl IndexColumnDetail {
    pub fn new(seqno: i32, name: String, is_ascending: bool) -> Self {
        Self { seqno, name, is_ascending }
    }
}

/// Detailed metadata describing an existing SQLite table index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableIndexDetail {
    /// Name of the SQLite index.
    pub name: String,
    /// Whether the index enforces a uniqueness constraint.
    pub is_unique: bool,
    /// Columns covered by the index in sequence.
    pub columns: Vec<IndexColumnDetail>,
}

impl TableIndexDetail {
    pub fn new(name: String, is_unique: bool, columns: Vec<IndexColumnDetail>) -> Self {
        Self { name, is_unique, columns }
    }
}

/// Audit status for a single prompt queue table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FifoTableIndexStatus {
    /// Table name evaluated.
    pub table_name: String,
    /// Resulting compliance status.
    pub status: FifoIndexStatusType,
    /// Whether the table possesses a valid composite index on `(created_at ASC, id ASC)`.
    pub has_composite_index: bool,
    /// Name of the matching composite index if present.
    pub matching_index_name: Option<String>,
    /// Evaluated performance and lock contention risk level.
    pub risk_level: IndexRiskLevelType,
    /// Idempotent SQL remediation statement if an index is missing or defective.
    pub remediation_sql: Option<String>,
    /// All indexes discovered on the table during inspection.
    pub existing_indices: Vec<TableIndexDetail>,
}

impl FifoTableIndexStatus {
    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    pub fn has_composite_index(&self) -> bool {
        self.has_composite_index
    }

    pub fn is_compliant(&self) -> bool {
        matches!(self.status, FifoIndexStatusType::FullyCompliantType)
    }

    pub fn risk_level(&self) -> IndexRiskLevelType {
        self.risk_level
    }

    pub fn remediation_sql(&self) -> Option<&str> {
        self.remediation_sql.as_deref()
    }
}

/// Structured summary report aggregating index inspection results across all queue tables.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FifoIndexAnalysisReport {
    /// Per-table index analysis outcomes.
    pub inspected_tables: Vec<FifoTableIndexStatus>,
    /// Total count of tables lacking a compliant composite index.
    pub missing_index_count: usize,
    /// Whether all inspected tables meet strict FIFO indexing requirements.
    pub is_fully_indexed: bool,
    /// Ordered list of remediation SQL queries required to achieve compliance.
    pub remediation_sql: Vec<String>,
}

impl FifoIndexAnalysisReport {
    pub fn is_fully_indexed(&self) -> bool {
        self.is_fully_indexed
    }

    pub fn missing_index_count(&self) -> usize {
        self.missing_index_count
    }

    pub fn inspected_tables(&self) -> &[FifoTableIndexStatus] {
        &self.inspected_tables
    }

    pub fn remediation_sql(&self) -> &[String] {
        &self.remediation_sql
    }

    pub fn combined_remediation_script(&self) -> String {
        self.remediation_sql.join("\n")
    }
}

/// Analyzer engine inspecting SQLite index schemas and enforcing strict FIFO composite indexing.
#[derive(Debug, Clone)]
pub struct FifoQueueIndexAnalyzer {
    target_tables: Vec<String>,
}

impl Default for FifoQueueIndexAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl FifoQueueIndexAnalyzer {
    /// Constructs a new analyzer initialized with the canonical prompt queue tables.
    pub fn new() -> Self {
        let tables = DEFAULT_PROMPT_QUEUE_TABLES
            .iter()
            .map(|&s| s.to_string())
            .collect();
        Self { target_tables: tables }
    }

    /// Constructs an analyzer configured for custom table targets.
    pub fn with_tables(tables: &[&str]) -> Self {
        let target_tables = tables.iter().map(|&s| s.to_string()).collect();
        Self { target_tables }
    }

    /// Returns the target table identifiers monitored by this analyzer.
    pub fn target_tables(&self) -> &[String] {
        &self.target_tables
    }

    /// Generates canonical idempotent DDL SQL to create the composite index for a table.
    pub fn generate_remediation_sql(table_name: &str) -> String {
        format!(
            "CREATE INDEX IF NOT EXISTS idx_{}_created_at_id ON {}(created_at ASC, id ASC);",
            table_name, table_name
        )
    }

    /// Inspects a single SQLite table and returns its FIFO composite index status.
    pub fn analyze_table(&self, conn: &Connection, table_name: &str) -> Result<FifoTableIndexStatus, AppError> {
        let is_present = check_table_exists(conn, table_name)?;
        if !is_present {
            return Ok(build_missing_table_status(table_name));
        }
        let indices = fetch_all_table_indices(conn, table_name)?;
        Ok(evaluate_table_status(table_name, indices))
    }

    /// Analyzes all configured target tables and generates an aggregated analysis report.
    pub fn analyze(&self, conn: &Connection) -> Result<FifoIndexAnalysisReport, AppError> {
        let mut statuses = Vec::new();
        for table in &self.target_tables {
            let status = self.analyze_table(conn, table)?;
            statuses.push(status);
        }
        Ok(compile_analysis_report(statuses))
    }

    /// Executes all recommended remediation queries against the connection to fix missing indexes.
    pub fn apply_remediations(
        &self,
        conn: &Connection,
        report: &FifoIndexAnalysisReport,
    ) -> Result<usize, AppError> {
        let mut applied_count = 0;
        for sql in &report.remediation_sql {
            conn.execute(sql, [])?;
            applied_count += 1;
        }
        debug!(applied_count, "Applied FIFO composite index remediation SQL");
        Ok(applied_count)
    }
}

/// Verifies whether a table exists within the current SQLite database schema.
pub fn check_table_exists(conn: &Connection, table_name: &str) -> Result<bool, AppError> {
    let sql = "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1;";
    let mut stmt = conn.prepare(sql)?;
    let count: i64 = stmt.query_row([table_name], |row| row.get(0))?;
    Ok(count > 0)
}

/// Queries `PRAGMA index_list` to discover all indexes declared on a table.
pub fn fetch_index_list(conn: &Connection, table_name: &str) -> Result<Vec<(String, bool)>, AppError> {
    let list_sql = format!("PRAGMA index_list('{}');", table_name);
    let mut stmt = conn.prepare(&list_sql)?;
    let mut rows = stmt.query([])?;
    let mut index_list = Vec::new();
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        let is_unique: bool = row.get::<_, i32>(2)? != 0;
        index_list.push((name, is_unique));
    }
    Ok(index_list)
}

/// Helper function to parse an `index_xinfo` row into an `IndexColumnDetail`.
fn parse_xinfo_row(row: &rusqlite::Row<'_>, current_len: usize) -> Result<Option<IndexColumnDetail>, rusqlite::Error> {
    let cid: i32 = row.get(1)?;
    let col_name_opt: Option<String> = row.get(2)?;
    let desc: i32 = row.get(3)?;
    let key: i32 = row.get(5)?;
    if key == 1 && cid >= 0 {
        if let Some(col_name) = col_name_opt {
            return Ok(Some(IndexColumnDetail::new(current_len as i32, col_name, desc == 0)));
        }
    }
    Ok(None)
}

/// Inspects index columns using `PRAGMA index_xinfo` (supporting sort direction inspection).
pub fn fetch_columns_from_xinfo(conn: &Connection, index_name: &str) -> Result<Vec<IndexColumnDetail>, AppError> {
    let xinfo_sql = format!("PRAGMA index_xinfo('{}');", index_name);
    let mut stmt = conn.prepare(&xinfo_sql)?;
    let mut rows = stmt.query([])?;
    let mut columns = Vec::new();
    while let Some(row) = rows.next()? {
        if let Some(detail) = parse_xinfo_row(row, columns.len())? {
            columns.push(detail);
        }
    }
    Ok(columns)
}

/// Fallback inspector using `PRAGMA index_info` when `index_xinfo` yields no key columns.
pub fn fetch_columns_from_info(conn: &Connection, index_name: &str) -> Result<Vec<IndexColumnDetail>, AppError> {
    let info_sql = format!("PRAGMA index_info('{}');", index_name);
    let mut stmt = conn.prepare(&info_sql)?;
    let mut rows = stmt.query([])?;
    let mut columns = Vec::new();
    while let Some(row) = rows.next()? {
        let seqno: i32 = row.get(0)?;
        let col_name: String = row.get(2)?;
        columns.push(IndexColumnDetail::new(seqno, col_name, true));
    }
    Ok(columns)
}

/// Retrieves complete index metadata including column sequences and sort orders.
pub fn fetch_all_table_indices(conn: &Connection, table_name: &str) -> Result<Vec<TableIndexDetail>, AppError> {
    let index_list = fetch_index_list(conn, table_name)?;
    let mut details = Vec::new();
    for (name, is_unique) in index_list {
        let mut columns = fetch_columns_from_xinfo(conn, &name)?;
        if columns.is_empty() {
            columns = fetch_columns_from_info(conn, &name)?;
        }
        details.push(TableIndexDetail::new(name, is_unique, columns));
    }
    Ok(details)
}

/// Evaluates column sequence against canonical FIFO `(created_at ASC, id ASC)` requirements.
pub fn evaluate_columns_for_fifo(columns: &[IndexColumnDetail]) -> IndexMatchResultType {
    if columns.len() < 2 {
        if is_partial_created_at_lead(columns) {
            return IndexMatchResultType::PartialMatchType;
        }
        return IndexMatchResultType::NoMatchType;
    }
    let col0 = &columns[0];
    let col1 = &columns[1];
    check_two_column_order(col0, col1)
}

/// Checks whether the single leading column matches `created_at`.
pub fn is_partial_created_at_lead(columns: &[IndexColumnDetail]) -> bool {
    columns
        .first()
        .map(|c| c.name.eq_ignore_ascii_case("created_at"))
        .unwrap_or(false)
}

/// Checks ordering and sort direction across the first two indexed columns.
pub fn check_two_column_order(col0: &IndexColumnDetail, col1: &IndexColumnDetail) -> IndexMatchResultType {
    let is_created_at_first = col0.name.eq_ignore_ascii_case("created_at");
    let is_id_second = col1.name.eq_ignore_ascii_case("id");
    let is_id_first = col0.name.eq_ignore_ascii_case("id");
    let is_created_at_second = col1.name.eq_ignore_ascii_case("created_at");

    if is_created_at_first && is_id_second {
        if col0.is_ascending && col1.is_ascending {
            IndexMatchResultType::MatchesCompositeFifoType
        } else {
            IndexMatchResultType::ImproperSortDirectionType
        }
    } else if is_id_first && is_created_at_second {
        IndexMatchResultType::InvertedColumnOrderType
    } else if is_created_at_first {
        IndexMatchResultType::PartialMatchType
    } else {
        IndexMatchResultType::NoMatchType
    }
}

/// Records a full compliance match for a table's index evaluation.
fn record_full_compliance(
    idx: &TableIndexDetail,
    status: &mut FifoIndexStatusType,
    matching: &mut Option<String>,
    risk: &mut IndexRiskLevelType,
) -> bool {
    *status = FifoIndexStatusType::FullyCompliantType;
    *matching = Some(idx.name.clone());
    *risk = IndexRiskLevelType::NoneType;
    true
}

/// Records an improper sort direction defect for a table's index evaluation.
fn record_improper_direction(
    idx: &TableIndexDetail,
    status: &mut FifoIndexStatusType,
    matching: &mut Option<String>,
    risk: &mut IndexRiskLevelType,
) -> bool {
    *status = FifoIndexStatusType::ImproperSortDirectionType;
    *matching = Some(idx.name.clone());
    *risk = IndexRiskLevelType::HighType;
    false
}

/// Records an inverted column order defect for a table's index evaluation.
fn record_inverted_order(
    idx: &TableIndexDetail,
    status: &mut FifoIndexStatusType,
    matching: &mut Option<String>,
) -> bool {
    if *status == FifoIndexStatusType::MissingCompositeIndexType {
        *status = FifoIndexStatusType::ImproperColumnOrderType;
        *matching = Some(idx.name.clone());
    }
    false
}

/// Records a partial index match for a table's index evaluation.
fn record_partial_match(risk: &mut IndexRiskLevelType) -> bool {
    if *risk > IndexRiskLevelType::MediumType {
        *risk = IndexRiskLevelType::MediumType;
    }
    false
}

/// Evaluates a single index against FIFO requirements and updates the evaluation state.
fn apply_index_match(
    idx: &TableIndexDetail,
    status: &mut FifoIndexStatusType,
    matching: &mut Option<String>,
    risk: &mut IndexRiskLevelType,
) -> bool {
    match evaluate_columns_for_fifo(&idx.columns) {
        IndexMatchResultType::MatchesCompositeFifoType => record_full_compliance(idx, status, matching, risk),
        IndexMatchResultType::ImproperSortDirectionType => record_improper_direction(idx, status, matching, risk),
        IndexMatchResultType::InvertedColumnOrderType => record_inverted_order(idx, status, matching),
        IndexMatchResultType::PartialMatchType => record_partial_match(risk),
        IndexMatchResultType::NoMatchType => false,
    }
}

/// Evaluates all indexes on an existing table to determine FIFO composite compliance.
pub fn evaluate_table_status(table_name: &str, indices: Vec<TableIndexDetail>) -> FifoTableIndexStatus {
    let mut status = FifoIndexStatusType::MissingCompositeIndexType;
    let mut matching = None;
    let mut risk = IndexRiskLevelType::HighType;

    for idx in &indices {
        if apply_index_match(idx, &mut status, &mut matching, &mut risk) {
            break;
        }
    }
    build_evaluated_status(table_name, status, matching, risk, indices)
}

/// Resolves the optional remediation SQL query depending on whether the table is compliant.
fn resolve_remediation_sql(table_name: &str, has_composite_index: bool) -> Option<String> {
    if has_composite_index {
        None
    } else {
        Some(FifoQueueIndexAnalyzer::generate_remediation_sql(table_name))
    }
}

/// Logs a diagnostic warning whenever a table lacks a canonical FIFO composite index.
fn log_index_compliance_warning(
    table_name: &str,
    status: FifoIndexStatusType,
    risk_level: IndexRiskLevelType,
    has_composite_index: bool,
) {
    if !has_composite_index {
        warn!(
            table = table_name,
            status = %status,
            risk = %risk_level,
            "Missing canonical FIFO composite index (created_at ASC, id ASC)"
        );
    }
}

/// Constructs a status object for an existing and evaluated table.
pub fn build_evaluated_status(
    table_name: &str,
    status: FifoIndexStatusType,
    matching_index: Option<String>,
    risk_level: IndexRiskLevelType,
    existing_indices: Vec<TableIndexDetail>,
) -> FifoTableIndexStatus {
    let has_composite_index = matches!(status, FifoIndexStatusType::FullyCompliantType);
    let remediation_sql = resolve_remediation_sql(table_name, has_composite_index);
    log_index_compliance_warning(table_name, status, risk_level, has_composite_index);

    FifoTableIndexStatus {
        table_name: table_name.to_string(),
        status,
        has_composite_index,
        matching_index_name: matching_index,
        risk_level,
        remediation_sql,
        existing_indices,
    }
}

/// Constructs a status object representing an uninitialized or missing database table.
pub fn build_missing_table_status(table_name: &str) -> FifoTableIndexStatus {
    FifoTableIndexStatus {
        table_name: table_name.to_string(),
        status: FifoIndexStatusType::TableNotFoundType,
        has_composite_index: false,
        matching_index_name: None,
        risk_level: IndexRiskLevelType::MediumType,
        remediation_sql: Some(FifoQueueIndexAnalyzer::generate_remediation_sql(table_name)),
        existing_indices: Vec::new(),
    }
}

/// Aggregates per-table statuses into a unified `FifoIndexAnalysisReport`.
pub fn compile_analysis_report(inspected_tables: Vec<FifoTableIndexStatus>) -> FifoIndexAnalysisReport {
    let missing_index_count = inspected_tables
        .iter()
        .filter(|s| !s.has_composite_index)
        .count();
    let is_fully_indexed = missing_index_count == 0;
    let remediation_sql = inspected_tables
        .iter()
        .filter_map(|s| s.remediation_sql.clone())
        .collect();

    FifoIndexAnalysisReport {
        inspected_tables,
        missing_index_count,
        is_fully_indexed,
        remediation_sql,
    }
}
