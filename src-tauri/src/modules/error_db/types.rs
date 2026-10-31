//! Data types and transfer objects for SQLite Speed Error Database (`errors.db`).

use serde::{Deserialize, Serialize};

/// Stored error record in `errors.db`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecord {
    pub id: String,
    pub error_code: String,
    pub severity: String,
    pub message: String,
    pub stack_trace: Option<String>,
    pub source: Option<String>,
    pub trigger_action: Option<String>,
    pub metadata_json: Option<String>,
    pub created_at: i64,
}

/// Incoming error payload for insertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewErrorRecord {
    pub id: Option<String>,
    pub error_code: String,
    pub severity: String,
    pub message: String,
    pub stack_trace: Option<String>,
    pub source: Option<String>,
    pub trigger_action: Option<String>,
    pub metadata_json: Option<String>,
    pub created_at: Option<i64>,
}

/// Query filter options for errors listing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorQueryOptions {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub severity: Option<String>,
    pub error_code: Option<String>,
    pub search: Option<String>,
}

/// Diagnostic health probe check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticCheck {
    pub name: String,
    pub category: String,
    pub passed: bool,
    pub message: String,
    pub auto_fixed: bool,
}

/// Self-healing diagnostic report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfHealingReport {
    pub timestamp: String,
    pub total_checks: usize,
    pub passed_checks: usize,
    pub fixed_count: usize,
    pub checks: Vec<DiagnosticCheck>,
    pub fixes_applied: Vec<String>,
    pub warnings: Vec<String>,
    pub is_healthy: bool,
}
