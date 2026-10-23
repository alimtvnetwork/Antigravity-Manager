//! Bounded Matcher & Timeout Verifier.
//!
//! Enforces string matching and asynchronous wait discipline per AGENTS.md:
//! - "Detect broadly, act narrowly: a matcher may recognize a whole class of problems,
//!   while its effect stays inside the intended data — not across line breaks, tags,
//!   or other clauses."
//! - "Bound every wait with a timeout."

use serde::{Deserialize, Serialize};
use std::fmt;

/// Boundary scope constraint kinds for string matching operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatcherBoundaryKind {
    SingleLineOnly,
    WithinTagBound,
    BoundedSpan(usize),
}

/// Violations identified when auditing matcher boundaries or asynchronous timeouts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatcherBoundaryViolation {
    UnboundedSpanDetected { pattern: String },
    CrossesNewlineDisallowed { matched_snippet: String },
    UnboundedWaitTimeout { timeout_ms: u64 },
}

impl fmt::Display for MatcherBoundaryViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnboundedSpanDetected { pattern } => {
                write!(
                    f,
                    "Unbounded span detected in matcher pattern or snippet: '{pattern}'"
                )
            }
            Self::CrossesNewlineDisallowed { matched_snippet } => {
                write!(
                    f,
                    "Matcher crossed disallowed newline boundary in matched snippet: '{matched_snippet}'"
                )
            }
            Self::UnboundedWaitTimeout { timeout_ms } => {
                write!(
                    f,
                    "Unbounded or invalid wait timeout detected: {timeout_ms} ms (0 or exceeds cap)"
                )
            }
        }
    }
}

impl std::error::Error for MatcherBoundaryViolation {}

/// Structured audit report compiling matcher boundary and timeout validations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundedMatcherAuditReport {
    pub is_bounded: bool,
    pub patterns_audited: usize,
    pub timeouts_audited: usize,
    pub violations: Vec<MatcherBoundaryViolation>,
}

impl BoundedMatcherAuditReport {
    /// Creates a new empty audit report.
    pub fn new() -> Self {
        Self {
            is_bounded: true,
            patterns_audited: 0,
            timeouts_audited: 0,
            violations: Vec::new(),
        }
    }

    /// Returns true if all audited items are bounded and conformant.
    pub fn is_bounded(&self) -> bool {
        self.is_bounded
    }

    /// Returns true if any boundary or timeout violations were recorded.
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Returns the total count of recorded violations.
    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }

    /// Records the result of a pattern boundary verification.
    pub fn record_pattern_result(&mut self, res: Result<(), MatcherBoundaryViolation>) {
        self.patterns_audited += 1;
        if let Err(v) = res {
            self.violations.push(v);
            self.is_bounded = false;
        }
    }

    /// Records the result of a timeout verification.
    pub fn record_timeout_result(&mut self, res: Result<(), MatcherBoundaryViolation>) {
        self.timeouts_audited += 1;
        if let Err(v) = res {
            self.violations.push(v);
            self.is_bounded = false;
        }
    }
}

impl Default for BoundedMatcherAuditReport {
    fn default() -> Self {
        Self::new()
    }
}

fn has_dotall_wildcard(p: &str) -> bool {
    (p.contains("(?s)") && (p.contains(".*") || p.contains(".+"))) || p.contains("(?s:")
}

fn has_multiline_character_class(p: &str) -> bool {
    p.contains(r"[\s\S]*")
        || p.contains(r"[\s\S]+")
        || p.contains(r"[\w\W]*")
        || p.contains(r"[\d\D]*")
}

fn is_bare_unanchored_wildcard(p: &str) -> bool {
    p == ".*" || p == ".+" || p == "^.*$" || p == "^.+$"
}

/// Verifies that a matcher regex pattern is strictly bounded.
///
/// Flags regex patterns containing `(?s).*`, inline dotall flags, unanchored
/// multi-line wildcard spans (`[\s\S]*`, etc.), or bare wildcards (`.*`, `.+`).
pub fn verify_matcher_pattern_bounded(pattern: &str) -> Result<(), MatcherBoundaryViolation> {
    let p = pattern.trim();
    if has_dotall_wildcard(p) || has_multiline_character_class(p) || is_bare_unanchored_wildcard(p) {
        return Err(MatcherBoundaryViolation::UnboundedSpanDetected {
            pattern: pattern.to_string(),
        });
    }
    Ok(())
}

/// Verifies that an asynchronous wait timeout is strictly bounded.
///
/// Flags timeout_ms == 0 (unbounded wait) or timeout_ms > max_cap_ms.
pub fn verify_wait_timeout(
    timeout_ms: u64,
    max_cap_ms: u64,
) -> Result<(), MatcherBoundaryViolation> {
    if timeout_ms == 0 || timeout_ms > max_cap_ms {
        return Err(MatcherBoundaryViolation::UnboundedWaitTimeout { timeout_ms });
    }
    Ok(())
}

/// Verifies that a matched text snippet adheres strictly to the specified boundary kind.
pub fn verify_matched_snippet(
    snippet: &str,
    kind: MatcherBoundaryKind,
) -> Result<(), MatcherBoundaryViolation> {
    match kind {
        MatcherBoundaryKind::SingleLineOnly => {
            if snippet.contains('\n') || snippet.contains('\r') {
                return Err(MatcherBoundaryViolation::CrossesNewlineDisallowed {
                    matched_snippet: snippet.to_string(),
                });
            }
        }
        MatcherBoundaryKind::WithinTagBound => {
            if snippet.contains('\n') || snippet.contains('\r') {
                return Err(MatcherBoundaryViolation::CrossesNewlineDisallowed {
                    matched_snippet: snippet.to_string(),
                });
            }
            if snippet.contains('<') || snippet.contains('>') {
                return Err(MatcherBoundaryViolation::UnboundedSpanDetected {
                    pattern: format!("Snippet crossed tag delimiter: '{snippet}'"),
                });
            }
        }
        MatcherBoundaryKind::BoundedSpan(max_len) => {
            if snippet.len() > max_len {
                return Err(MatcherBoundaryViolation::UnboundedSpanDetected {
                    pattern: format!(
                        "Snippet length {} exceeded maximum bound {}",
                        snippet.len(),
                        max_len
                    ),
                });
            }
        }
    }
    Ok(())
}

/// Aggregates audits across a collection of patterns and timeouts into a unified report.
pub fn audit_matchers_and_timeouts(
    patterns: &[&str],
    timeouts: &[(u64, u64)],
) -> BoundedMatcherAuditReport {
    let mut report = BoundedMatcherAuditReport::new();
    for &pattern in patterns {
        report.record_pattern_result(verify_matcher_pattern_bounded(pattern));
    }
    for &(timeout_ms, max_cap_ms) in timeouts {
        report.record_timeout_result(verify_wait_timeout(timeout_ms, max_cap_ms));
    }
    report
}
