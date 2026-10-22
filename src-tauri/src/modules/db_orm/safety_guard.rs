//! SQL Safety Execution Guard for developer workbench and data forensics.
//! Enforces read-only query boundaries and blocks accidental or malicious destructive operations.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SqlSafetyError {
    #[error("Empty SQL query is not allowed")]
    EmptyQuery,
    #[error("Destructive statement '{0}' is forbidden in read-only mode")]
    ForbiddenKeyword(String),
    #[error("Multiple statements detected in single query execution")]
    MultipleStatementsForbidden,
    #[error("Comment bypass pattern detected")]
    SuspiciousCommentPattern,
}

const FORBIDDEN_MUTATING_KEYWORDS: &[&str] = &[
    "DROP", "DELETE", "TRUNCATE", "ALTER", "UPDATE", "INSERT", "REPLACE", "CREATE", "VACUUM",
    "ATTACH", "DETACH", "REINDEX",
];

/// Validates whether a given SQL statement is safe to execute.
/// In read-only mode (`allow_mutations = false`), only SELECT, EXPLAIN, and read pragmas are permitted.
pub fn validate_safe_query(sql: &str, allow_mutations: bool) -> Result<(), SqlSafetyError> {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return Err(SqlSafetyError::EmptyQuery);
    }

    // Check for comment injection attempts like -- or /* */
    let sanitized = strip_sql_comments(trimmed);
    let normalized = sanitized.trim();
    if normalized.is_empty() {
        return Err(SqlSafetyError::EmptyQuery);
    }

    // Check for semicolon-separated multiple statements
    let statements: Vec<&str> = normalized
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if statements.len() > 1 {
        return Err(SqlSafetyError::MultipleStatementsForbidden);
    }

    if allow_mutations {
        return Ok(());
    }

    let upper = normalized.to_ascii_uppercase();
    let tokens: Vec<&str> = upper
        .split_whitespace()
        .map(|t| t.trim_matches(|c: char| !c.is_alphanumeric()))
        .collect();

    for keyword in FORBIDDEN_MUTATING_KEYWORDS {
        if tokens.contains(keyword) {
            return Err(SqlSafetyError::ForbiddenKeyword((*keyword).to_string()));
        }
    }

    // Verify leading keyword is SELECT, EXPLAIN, or PRAGMA
    if let Some(first) = tokens.first() {
        match *first {
            "SELECT" | "EXPLAIN" | "WITH" | "PRAGMA" => Ok(()),
            other => Err(SqlSafetyError::ForbiddenKeyword(other.to_string())),
        }
    } else {
        Err(SqlSafetyError::EmptyQuery)
    }
}

/// Strips line comments (--) and block comments (/* */) from SQL string.
fn strip_sql_comments(sql: &str) -> String {
    let mut result = String::with_capacity(sql.len());
    let chars: Vec<char> = sql.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Line comments: --
        if i + 1 < len && chars[i] == '-' && chars[i + 1] == '-' {
            i += 2;
            while i < len && chars[i] != '\n' && chars[i] != '\r' {
                i += 1;
            }
            continue;
        }

        // Block comments: /* ... */
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2; // Skip closing */
            continue;
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}
