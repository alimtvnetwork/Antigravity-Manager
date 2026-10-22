//! Asset-verification types for the update contract (plan 86, subtask 001).

use serde::{Deserialize, Serialize};

/// A newer release that was considered but not offered, with the reason.
/// Reported to the user so a skipped version is never silent (plan 86).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkippedVersion {
    /// Version that was skipped (no leading `v`).
    pub version: String,
    /// Human-readable reason, e.g. "no installer asset for this platform".
    pub reason: String,
}

/// A failure from one update source, aggregated instead of swallowed
/// (plan 86, subtask 003 groundwork).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceError {
    /// Source name, e.g. "GitHub API (Stable)".
    pub source: String,
    /// Failure message.
    pub message: String,
}
