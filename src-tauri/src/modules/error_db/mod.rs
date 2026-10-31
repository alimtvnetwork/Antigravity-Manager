//! Speed Error Database (`errors.db`) and AI Diagnostic Self-Healing Engine.
//! Provides SQLite WAL-mode ultra-fast insert capabilities for frontend and backend
//! error telemetry, diagnostic probing, and automated self-healing.

pub mod db;
pub mod diagnostics;
pub mod ops;
#[cfg(test)]
pub mod tests;
pub mod types;

pub use db::{connect_error_db, get_error_db_path};
pub use diagnostics::run_self_healing;
pub use ops::{
    clear_errors, count_errors, get_error_by_id, list_errors, prune_old_errors, record_error,
};
pub use types::{
    DiagnosticCheck, ErrorQueryOptions, ErrorRecord, NewErrorRecord, SelfHealingReport,
};
