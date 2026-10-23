//! Safe Query Read-Only Sandbox Guard.
//! Enforces read-only safety verification preventing mutations in diagnostic query explorers.

use serde::{Deserialize, Serialize};

pub const FORBIDDEN_MUTATING_KEYWORDS: &[&str] = &[
    "DROP", "ALTER", "TRUNCATE", "INSERT", "UPDATE", "DELETE", "ATTACH", "DETACH", "REPLACE",
    "CREATE", "VACUUM", "REINDEX",
];

/// Result of read-only query sandbox validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadSandboxCheck {
    pub is_safe: bool,
    pub sanitized_sql: String,
    pub violation_reason: Option<String>,
}

/// Validates that an arbitrary SQL statement is strictly read-only and free of destructive statements.
pub fn validate_readonly_query(raw_sql: &str) -> ReadSandboxCheck {
    let trimmed = raw_sql.trim();
    if trimmed.is_empty() {
        return ReadSandboxCheck {
            is_safe: false,
            sanitized_sql: String::new(),
            violation_reason: Some("Query cannot be empty".to_string()),
        };
    }

    // Strip trailing semicolon if single statement
    let stripped = trimmed.trim_end_matches(';').trim();

    // Reject compound semicolon queries
    if stripped.contains(';') {
        return ReadSandboxCheck {
            is_safe: false,
            sanitized_sql: stripped.to_string(),
            violation_reason: Some(
                "Multiple SQL statements delimited by semicolon are forbidden".to_string(),
            ),
        };
    }

    // Extract first keyword
    let first_token = stripped
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_uppercase();

    let allowed_leading = ["SELECT", "EXPLAIN", "WITH", "PRAGMA"];
    if !allowed_leading.contains(&first_token.as_str()) {
        return ReadSandboxCheck {
            is_safe: false,
            sanitized_sql: stripped.to_string(),
            violation_reason: Some(format!(
                "Read-only queries must start with SELECT, EXPLAIN, WITH, or PRAGMA (found '{}')",
                first_token
            )),
        };
    }

    // Scan for forbidden mutating keywords in word boundaries
    let upper = stripped.to_uppercase();
    for kw in FORBIDDEN_MUTATING_KEYWORDS {
        let pattern = format!(" {} ", kw);
        if upper.contains(&pattern) || upper.starts_with(&format!("{} ", kw)) {
            return ReadSandboxCheck {
                is_safe: false,
                sanitized_sql: stripped.to_string(),
                violation_reason: Some(format!("Forbidden mutating keyword detected: '{}'", kw)),
            };
        }
    }

    ReadSandboxCheck {
        is_safe: true,
        sanitized_sql: stripped.to_string(),
        violation_reason: None,
    }
}
