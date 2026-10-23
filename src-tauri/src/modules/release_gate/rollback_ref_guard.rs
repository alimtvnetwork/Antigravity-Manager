//! Local Rollback Reference & Drift Guard.
//!
//! Enforces Git branch and history hygiene per AGENTS.md:
//! Retains local rollback refs (`backup/*`) before history rewrites,
//! confirming zero content drift via diff stat before destructive operations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DriftReport {
    pub has_drift: bool,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub summary: String,
}

/// Formats a canonical rollback ref name for a task or operation.
pub fn format_backup_ref_name(task_slug: &str, timestamp_secs: u64) -> String {
    let clean_slug = task_slug
        .trim()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
        .collect::<String>();
    format!("backup/{}-{}", clean_slug, timestamp_secs)
}

/// Validates whether a ref string conforms to the `backup/*` naming convention.
pub fn is_valid_backup_ref(ref_name: &str) -> bool {
    let s = ref_name.trim();
    if !s.starts_with("backup/") {
        return false;
    }
    let remainder = &s["backup/".len()..];
    !remainder.is_empty() && remainder.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Parses the output of `git diff --stat <backup> HEAD` to verify zero content drift.
pub fn parse_drift_stat(diff_stat_output: &str) -> DriftReport {
    let trimmed = diff_stat_output.trim();
    if trimmed.is_empty() {
        return DriftReport {
            has_drift: false,
            files_changed: 0,
            insertions: 0,
            deletions: 0,
            summary: "Zero drift: diff stat is completely empty".to_string(),
        };
    }

    // Look for lines like "3 files changed, 10 insertions(+), 5 deletions(-)"
    let mut files_changed = 0;
    let mut insertions = 0;
    let mut deletions = 0;

    for line in trimmed.lines() {
        if line.contains("file changed") || line.contains("files changed") {
            for part in line.split(',') {
                let p = part.trim();
                if p.contains("file") {
                    if let Some(num_str) = p.split_whitespace().next() {
                        files_changed = num_str.parse().unwrap_or(0);
                    }
                } else if p.contains("insertion") {
                    if let Some(num_str) = p.split_whitespace().next() {
                        insertions = num_str.parse().unwrap_or(0);
                    }
                } else if p.contains("deletion") {
                    if let Some(num_str) = p.split_whitespace().next() {
                        deletions = num_str.parse().unwrap_or(0);
                    }
                }
            }
        }
    }

    let has_drift = files_changed > 0 || insertions > 0 || deletions > 0;

    DriftReport {
        has_drift,
        files_changed,
        insertions,
        deletions,
        summary: if has_drift {
            format!("Drift detected: {} files changed, +{} -{}", files_changed, insertions, deletions)
        } else {
            "Zero drift confirmed".to_string()
        },
    }
}
