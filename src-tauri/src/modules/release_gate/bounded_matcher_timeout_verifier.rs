//! Bounded Matcher & Timeout Verifier (Imp-664).
//!
//! Enforces string matching and asynchronous wait discipline per AGENTS.md:
//! - "Detect broadly, act narrowly: a matcher may recognize a whole class of problems,
//!   while its effect stays inside the intended data — not across line breaks, tags,
//!   or other clauses."
//! - "Bound every wait with a timeout."
//!
//! File: src-tauri/src/modules/release_gate/bounded_matcher_timeout_verifier.rs

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

/// Default minimum permitted wait timeout floor (50ms).
pub const DEFAULT_MIN_TIMEOUT_MS: u64 = 50;

/// Default maximum permitted wait timeout ceiling (60,000ms).
pub const DEFAULT_MAX_TIMEOUT_MS: u64 = 60_000;

/// Boundary scope constraint types for string matching operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatcherBoundaryType {
    SingleLineOnly,
    WithinTagBound,
    ClauseBounded,
    BoundedSpan(usize),
}

/// Backwards-compatible alias for MatcherBoundaryType.
pub type MatcherBoundaryKind = MatcherBoundaryType;

impl fmt::Display for MatcherBoundaryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SingleLineOnly => write!(f, "SingleLineOnly"),
            Self::WithinTagBound => write!(f, "WithinTagBound"),
            Self::ClauseBounded => write!(f, "ClauseBounded"),
            Self::BoundedSpan(limit) => write!(f, "BoundedSpan({limit})"),
        }
    }
}

/// Wait operation classification types for asynchronous wait timeouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeoutValidationType {
    PollingLoop,
    RetryLadder,
    NetworkWait,
    CustomWait,
}

impl fmt::Display for TimeoutValidationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PollingLoop => write!(f, "PollingLoop"),
            Self::RetryLadder => write!(f, "RetryLadder"),
            Self::NetworkWait => write!(f, "NetworkWait"),
            Self::CustomWait => write!(f, "CustomWait"),
        }
    }
}

/// Structured violation report types for bounded matcher and timeout verifications.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatcherViolationType {
    UnboundedMultilineSpan {
        pattern: String,
    },
    TagBoundaryCrossing {
        pattern: String,
    },
    BareWildcard {
        pattern: String,
    },
    ZeroTimeoutForbidden {
        wait_type: TimeoutValidationType,
    },
    TimeoutBelowFloor {
        wait_type: TimeoutValidationType,
        timeout_ms: u64,
        min_ms: u64,
    },
    TimeoutExceedsCeiling {
        wait_type: TimeoutValidationType,
        timeout_ms: u64,
        max_ms: u64,
    },
    BoundaryBreach {
        boundary_type: MatcherBoundaryType,
        detail: String,
    },
}

/// Backwards-compatible alias for MatcherViolationType.
pub type MatcherBoundaryViolation = MatcherViolationType;

impl fmt::Display for MatcherViolationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnboundedMultilineSpan { pattern } => {
                write!(f, "Unbounded multiline span detected in pattern: '{pattern}'")
            }
            Self::TagBoundaryCrossing { pattern } => {
                write!(f, "Unconstrained wildcard crossing tag boundary in: '{pattern}'")
            }
            Self::BareWildcard { pattern } => {
                write!(f, "Bare unanchored wildcard detected in pattern: '{pattern}'")
            }
            Self::ZeroTimeoutForbidden { wait_type } => {
                write!(f, "Zero timeout forbidden for wait type '{wait_type}'")
            }
            Self::TimeoutBelowFloor { wait_type, timeout_ms, min_ms } => {
                write!(f, "Timeout {timeout_ms}ms below floor {min_ms}ms for '{wait_type}'")
            }
            Self::TimeoutExceedsCeiling { wait_type, timeout_ms, max_ms } => {
                write!(f, "Timeout {timeout_ms}ms exceeds ceiling {max_ms}ms for '{wait_type}'")
            }
            Self::BoundaryBreach { boundary_type, detail } => {
                write!(f, "Boundary breach for '{boundary_type}': {detail}")
            }
        }
    }
}

impl std::error::Error for MatcherViolationType {}

/// Structured audit report compiling matcher boundary and timeout validations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundedMatcherAuditReport {
    pub is_compliant: bool,
    pub has_valid_boundaries: bool,
    pub has_valid_timeouts: bool,
    pub inspected_pattern_count: usize,
    pub violation_count: usize,
    pub violations: Vec<String>,
}

impl BoundedMatcherAuditReport {
    /// Constructs a clean, compliant initial audit report.
    pub fn new() -> Self {
        Self {
            is_compliant: true,
            has_valid_boundaries: true,
            has_valid_timeouts: true,
            inspected_pattern_count: 0,
            violation_count: 0,
            violations: Vec::new(),
        }
    }

    /// Records an inspected pattern increment.
    pub fn increment_pattern_count(&mut self) {
        let current_count = self.inspected_pattern_count;
        let next_count = current_count.saturating_add(1);
        self.inspected_pattern_count = next_count;
    }

    /// Records a pattern boundary violation and updates compliance status.
    pub fn record_boundary_violation(&mut self, violation: String) {
        self.is_compliant = false;
        self.has_valid_boundaries = false;
        let current_count = self.violation_count;
        self.violation_count = current_count.saturating_add(1);
        self.violations.push(violation);
    }

    /// Records a timeout violation and updates compliance status.
    pub fn record_timeout_violation(&mut self, violation: String) {
        self.is_compliant = false;
        self.has_valid_timeouts = false;
        let current_count = self.violation_count;
        self.violation_count = current_count.saturating_add(1);
        self.violations.push(violation);
    }

    /// Returns true if the audit detected zero boundary or timeout violations.
    pub fn is_compliant(&self) -> bool {
        let is_ok = self.is_compliant;
        let has_zero_violations = self.violation_count == 0;
        let has_empty_list = self.violations.is_empty();
        is_ok && has_zero_violations && has_empty_list
    }

    /// Returns true if all inspected patterns passed boundary checks.
    pub fn has_valid_boundaries(&self) -> bool {
        self.has_valid_boundaries
    }

    /// Returns true if all inspected wait timeouts conformed to limits.
    pub fn has_valid_timeouts(&self) -> bool {
        self.has_valid_timeouts
    }

    /// Returns the total count of identified violations.
    pub fn violation_count(&self) -> usize {
        self.violation_count
    }
}

impl Default for BoundedMatcherAuditReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Enforces bounded string matchers and timeout validation rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundedMatcherTimeoutVerifier {
    pub min_timeout: Duration,
    pub max_timeout: Duration,
}

impl Default for BoundedMatcherTimeoutVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl BoundedMatcherTimeoutVerifier {
    /// Creates a default verifier with standard 50ms min and 60,000ms max bounds.
    pub fn new() -> Self {
        let min_bound = Duration::from_millis(DEFAULT_MIN_TIMEOUT_MS);
        let max_bound = Duration::from_millis(DEFAULT_MAX_TIMEOUT_MS);
        Self {
            min_timeout: min_bound,
            max_timeout: max_bound,
        }
    }

    /// Creates a verifier with customized minimum and maximum timeout bounds.
    pub fn with_bounds(min_timeout: Duration, max_timeout: Duration) -> Result<Self, AppError> {
        if min_timeout.is_zero() {
            let msg = "Minimum timeout bound cannot be zero".to_string();
            return Err(AppError::Config(msg));
        }
        if min_timeout > max_timeout {
            let msg = format!(
                "Minimum timeout ({min_timeout:?}) cannot exceed maximum timeout ({max_timeout:?})"
            );
            return Err(AppError::Config(msg));
        }
        Ok(Self {
            min_timeout,
            max_timeout,
        })
    }

    /// Audits a regex pattern string to ensure it remains strictly bounded.
    pub fn audit_pattern(&self, pattern: &str) -> Result<(), String> {
        let p = pattern.trim();
        if has_multiline_dotall(p) {
            return Err(format!("Unconstrained multiline dot-all in: '{pattern}'"));
        }
        if has_multiline_character_class(p) {
            return Err(format!("Unconstrained character class in: '{pattern}'"));
        }
        if has_tag_boundary_crossing(p) {
            return Err(format!("Unconstrained tag crossing in: '{pattern}'"));
        }
        if has_bare_unanchored_wildcard(p) {
            return Err(format!("Bare unanchored wildcard in: '{pattern}'"));
        }
        Ok(())
    }

    /// Audits a pattern string and propagates violations directly via AppError.
    pub fn audit_pattern_or_error(&self, pattern: &str) -> Result<(), AppError> {
        self.audit_pattern(pattern)
            .map_err(AppError::Config)
    }

    /// Validates an asynchronous wait timeout against minimum and maximum bounds.
    pub fn validate_timeout(
        &self,
        timeout: Duration,
        wait_type: TimeoutValidationType,
    ) -> Result<(), String> {
        if timeout.is_zero() {
            return Err(format!("{wait_type}: Zero timeout is forbidden"));
        }
        let (cur, min, max) = (
            timeout.as_millis(),
            self.min_timeout.as_millis(),
            self.max_timeout.as_millis(),
        );
        if timeout < self.min_timeout {
            return Err(format!("{wait_type}: Timeout {cur}ms below floor {min}ms"));
        }
        if timeout > self.max_timeout {
            return Err(format!("{wait_type}: Timeout {cur}ms exceeds ceiling {max}ms"));
        }
        Ok(())
    }

    /// Audits a slice of patterns into an audit report.
    pub fn audit_patterns(&self, patterns: &[&str], report: &mut BoundedMatcherAuditReport) {
        for pattern in patterns {
            report.increment_pattern_count();
            if let Err(violation) = self.audit_pattern(pattern) {
                report.record_boundary_violation(violation);
            }
        }
    }

    /// Audits a slice of timeouts into an audit report.
    pub fn audit_timeouts(
        &self,
        timeouts: &[(Duration, TimeoutValidationType)],
        report: &mut BoundedMatcherAuditReport,
    ) {
        for (timeout, wait_type) in timeouts {
            if let Err(violation) = self.validate_timeout(*timeout, *wait_type) {
                report.record_timeout_violation(violation);
            }
        }
    }

    /// Audits both patterns and timeouts, returning a structured audit report.
    pub fn audit_all(
        &self,
        patterns: &[&str],
        timeouts: &[(Duration, TimeoutValidationType)],
    ) -> BoundedMatcherAuditReport {
        let mut report = BoundedMatcherAuditReport::new();
        self.audit_patterns(patterns, &mut report);
        self.audit_timeouts(timeouts, &mut report);
        report
    }

    /// Enforces compliance and returns an AppError if any violations are found.
    pub fn verify_and_enforce(
        &self,
        patterns: &[&str],
        timeouts: &[(Duration, TimeoutValidationType)],
    ) -> Result<BoundedMatcherAuditReport, AppError> {
        let report = self.audit_all(patterns, timeouts);
        if !report.is_compliant() {
            let count = report.violation_count();
            let msg = format!("Verifier found {count} violations: {:?}", report.violations);
            return Err(AppError::Config(msg));
        }
        Ok(report)
    }
}

fn has_multiline_dotall(p: &str) -> bool {
    let has_s_flag = p.contains("(?s)") && (p.contains(".*") || p.contains(".+"));
    let has_s_colon = p.contains("(?s:") && (p.contains(".*") || p.contains(".+"));
    let has_s_inline = p.contains("(?s-") || p.contains("(?-m:.*)");
    has_s_flag || has_s_colon || has_s_inline
}

fn has_multiline_character_class(p: &str) -> bool {
    let has_space_class = p.contains(r"[\s\S]*") || p.contains(r"[\s\S]+");
    let has_word_class = p.contains(r"[\w\W]*") || p.contains(r"[\w\W]+");
    let has_digit_class = p.contains(r"[\d\D]*") || p.contains(r"[\d\D]+");
    has_space_class || has_word_class || has_digit_class
}

fn has_tag_boundary_crossing(p: &str) -> bool {
    let has_simple_tag = p.contains("<.*>") || p.contains("<.+>");
    let has_lazy_tag = p.contains("<.*?>") || p.contains("<.+?>");
    let has_compound_tag = p.contains("<[^>]*.*>") || p.contains("<.*[^>]*>");
    has_simple_tag || has_lazy_tag || has_compound_tag
}

fn has_bare_unanchored_wildcard(p: &str) -> bool {
    let is_bare = p == ".*" || p == ".+";
    let is_anchored_bare = p == "^.*$" || p == "^.+$";
    let is_grouped_bare = p == "(.*)" || p == "(.+)";
    is_bare || is_anchored_bare || is_grouped_bare
}

/// Verifies that a matched text snippet adheres strictly to the specified boundary constraint.
pub fn verify_matched_snippet(
    snippet: &str,
    boundary: MatcherBoundaryType,
) -> Result<(), String> {
    match boundary {
        MatcherBoundaryType::SingleLineOnly => check_single_line_boundary(snippet),
        MatcherBoundaryType::WithinTagBound => check_tag_boundary(snippet),
        MatcherBoundaryType::ClauseBounded => check_clause_boundary(snippet),
        MatcherBoundaryType::BoundedSpan(limit) => check_span_limit(snippet, limit),
    }
}

fn check_single_line_boundary(snippet: &str) -> Result<(), String> {
    let has_newline = snippet.contains('\n') || snippet.contains('\r');
    if has_newline {
        let msg = format!("Snippet crossed newline boundary: '{snippet}'");
        return Err(msg);
    }
    Ok(())
}

fn check_tag_boundary(snippet: &str) -> Result<(), String> {
    let has_newline = snippet.contains('\n') || snippet.contains('\r');
    if has_newline {
        return Err(format!("Snippet in tag bound crossed newline: '{snippet}'"));
    }
    let has_tag_delims = snippet.contains('<') || snippet.contains('>');
    if has_tag_delims {
        return Err(format!("Snippet crossed tag delimiter: '{snippet}'"));
    }
    Ok(())
}

fn check_clause_boundary(snippet: &str) -> Result<(), String> {
    let has_semi = snippet.contains(';');
    let has_sentence_end = snippet.contains(". ") || snippet.contains("? ") || snippet.contains("! ");
    if has_semi || has_sentence_end {
        return Err(format!("Snippet crossed clause delimiter: '{snippet}'"));
    }
    Ok(())
}

fn check_span_limit(snippet: &str, limit: usize) -> Result<(), String> {
    let actual_len = snippet.len();
    if actual_len > limit {
        let msg = format!("Snippet length {actual_len} exceeded maximum bound of {limit}");
        return Err(msg);
    }
    Ok(())
}

/// Audits a matcher regex pattern for unbounded constructs using standard defaults.
pub fn verify_matcher_pattern_bounded(pattern: &str) -> Result<(), String> {
    let verifier = BoundedMatcherTimeoutVerifier::new();
    verifier.audit_pattern(pattern)
}

/// Validates an asynchronous wait timeout in milliseconds against standard floor and ceiling.
pub fn verify_wait_timeout(timeout_ms: u64, max_cap_ms: u64) -> Result<(), String> {
    if timeout_ms == 0 {
        return Err("Zero timeout is forbidden; waits must be bounded".to_string());
    }
    if timeout_ms > max_cap_ms {
        return Err(format!("Timeout {timeout_ms}ms exceeds maximum cap {max_cap_ms}ms"));
    }
    if timeout_ms < DEFAULT_MIN_TIMEOUT_MS {
        return Err(format!("Timeout {timeout_ms}ms below floor {DEFAULT_MIN_TIMEOUT_MS}ms"));
    }
    Ok(())
}

/// Audits a list of patterns and (timeout_ms, max_cap_ms) pairs into a BoundedMatcherAuditReport.
pub fn audit_matchers_and_timeouts(
    patterns: &[&str],
    timeouts: &[(u64, u64)],
) -> BoundedMatcherAuditReport {
    let mut report = BoundedMatcherAuditReport::new();
    let verifier = BoundedMatcherTimeoutVerifier::new();
    verifier.audit_patterns(patterns, &mut report);
    for &(timeout_ms, max_cap_ms) in timeouts {
        if let Err(violation) = verify_wait_timeout(timeout_ms, max_cap_ms) {
            report.record_timeout_violation(violation);
        }
    }
    report
}
