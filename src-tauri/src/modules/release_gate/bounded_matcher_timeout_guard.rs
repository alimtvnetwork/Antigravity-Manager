//! Bounded Matcher & Timeout Guard.
//!
//! Enforces string matching and asynchronous wait discipline per AGENTS.md:
//! - "Detect broadly, act narrowly: a matcher may recognize a whole class of problems,
//!   while its effect stays inside the intended data — not across line breaks, tags,
//!   or other clauses."
//! - "Bound every wait with a timeout."
//!
//! Provides boundary validation across `SingleLineOnly`, `ClauseBounded`, `TagEnclosed`,
//! and `CustomDelimiter` rules, guards against unbounded or infinite timeouts (`None` or `0`),
//! and offers static code analysis to detect naked sleeps or unbounded loop waits.

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

/// Regex detecting HTML/XML tags (e.g. `<div>`, `</span>`, `<br/>`, `<input ...>`).
static HTML_TAG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"</?[a-zA-Z][a-zA-Z0-9:-]*(?:\s+[^>]*)?/?>")
        .expect("valid html tag regex")
});

/// Boundary check rules governing how narrowly a matcher effect must be bounded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryCheckType {
    /// Match effect must not cross line breaks (`\n`, `\r\n`, `\r`).
    SingleLineOnly,
    /// Match effect must stay within a single sentence or clause (must not cross `;`, `. `, `? `, `! `).
    ClauseBounded,
    /// Match effect must stay within tag boundaries (must not cross `<...>` tags).
    TagEnclosed,
    /// Match effect must not cross a specified custom delimiter character.
    CustomDelimiter(char),
}

impl fmt::Display for BoundaryCheckType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SingleLineOnly => write!(f, "SingleLineOnly"),
            Self::ClauseBounded => write!(f, "ClauseBounded"),
            Self::TagEnclosed => write!(f, "TagEnclosed"),
            Self::CustomDelimiter(c) => write!(f, "CustomDelimiter('{c}')"),
        }
    }
}

/// Represents the character/byte span of a match within full text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchSpan {
    pub start: usize,
    pub end: usize,
    pub matched_text: String,
}

impl MatchSpan {
    /// Creates a new `MatchSpan` from start index, end index, and matched text.
    pub fn new(start: usize, end: usize, matched_text: impl Into<String>) -> Self {
        Self {
            start,
            end,
            matched_text: matched_text.into(),
        }
    }

    /// Finds the first occurrence of a pattern in full text and constructs a `MatchSpan`.
    pub fn from_first_occurrence(full_text: &str, pattern: &str) -> Option<Self> {
        let start = full_text.find(pattern)?;
        let end = start + pattern.len();
        Some(Self {
            start,
            end,
            matched_text: pattern.to_string(),
        })
    }

    /// Returns the length in bytes of the span.
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Returns true if the span is empty.
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

/// Structured boundary validation report assessing span confinement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundaryValidationReport {
    pub is_bounded: bool,
    pub spans_line_breaks: bool,
    pub crosses_html_tags: bool,
    pub crosses_clauses: bool,
    pub violations: Vec<String>,
}

impl BoundaryValidationReport {
    /// Returns true if the span is strictly bounded across line breaks, HTML tags, and clauses.
    pub fn is_strictly_bounded(&self) -> bool {
        !self.spans_line_breaks && !self.crosses_html_tags && !self.crosses_clauses
    }
}

/// Violation encountered when a matcher effect crosses forbidden boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoundaryViolation {
    /// Span bounds are invalid (start > end or end exceeds full text length).
    InvalidSpanRange {
        start: usize,
        end: usize,
        text_length: usize,
    },
    /// The slice at `[start..end]` in full text does not match `matched_text`.
    SpanContentMismatch {
        expected: String,
        actual: String,
    },
    /// Match span spans line breaks (`\n` or `\r`), violating `SingleLineOnly`.
    SpansLineBreaksForbidden {
        line_breaks_count: usize,
        matched_text: String,
        report: BoundaryValidationReport,
    },
    /// Match span crosses HTML/XML tags, violating `TagEnclosed`.
    CrossesHtmlTagsForbidden {
        tags: Vec<String>,
        matched_text: String,
        report: BoundaryValidationReport,
    },
    /// Match span crosses sentence or clause boundaries, violating `ClauseBounded`.
    CrossesClausesForbidden {
        delimiters: Vec<char>,
        matched_text: String,
        report: BoundaryValidationReport,
    },
    /// Match span crosses the specified custom delimiter character.
    CrossesCustomDelimiterForbidden {
        delimiter: char,
        count: usize,
        matched_text: String,
        report: BoundaryValidationReport,
    },
    /// General boundary rule breach.
    BoundaryBreach {
        rule: BoundaryCheckType,
        report: BoundaryValidationReport,
    },
}

impl fmt::Display for BoundaryViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSpanRange {
                start,
                end,
                text_length,
            } => {
                write!(
                    f,
                    "Invalid match span range [{start}..{end}] for text length {text_length}"
                )
            }
            Self::SpanContentMismatch { expected, actual } => {
                write!(
                    f,
                    "Span content mismatch: expected '{expected}', found slice '{actual}'"
                )
            }
            Self::SpansLineBreaksForbidden {
                line_breaks_count,
                matched_text,
                ..
            } => {
                write!(
                    f,
                    "Match span spans {line_breaks_count} line breaks, violating SingleLineOnly boundary rule: '{matched_text}'"
                )
            }
            Self::CrossesHtmlTagsForbidden {
                tags,
                matched_text,
                ..
            } => {
                write!(
                    f,
                    "Match span crosses HTML tags {tags:?}, violating TagEnclosed boundary rule: '{matched_text}'"
                )
            }
            Self::CrossesClausesForbidden {
                delimiters,
                matched_text,
                ..
            } => {
                write!(
                    f,
                    "Match span crosses clauses (delimiters: {delimiters:?}), violating ClauseBounded boundary rule: '{matched_text}'"
                )
            }
            Self::CrossesCustomDelimiterForbidden {
                delimiter,
                count,
                matched_text,
                ..
            } => {
                write!(
                    f,
                    "Match span crosses custom delimiter '{delimiter}' {count} time(s): '{matched_text}'"
                )
            }
            Self::BoundaryBreach { rule, report } => {
                write!(
                    f,
                    "Match span violates boundary rule {rule}: {:?}",
                    report.violations
                )
            }
        }
    }
}

impl std::error::Error for BoundaryViolation {}

/// Helper to detect clause boundary delimiters in a text snippet.
/// Recognizes semicolons (`;`), sentence terminals (`. `, `? `, `! `, or at end of text),
/// but intentionally ignores decimal points in numbers (e.g. `4.8.2`) or Rust path qualifiers (`::`).
pub fn extract_clause_delimiters(text: &str) -> Vec<char> {
    let mut delimiters = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    for i in 0..len {
        let ch = chars[i];
        if ch == ';' {
            delimiters.push(';');
        } else if ch == '?' {
            delimiters.push('?');
        } else if ch == '!' {
            delimiters.push('!');
        } else if ch == '.' {
            // Check if it's a clause-ending period rather than a version/decimal number
            let next_char = chars.get(i + 1);
            let prev_char = if i > 0 { chars.get(i - 1) } else { None };

            let is_decimal = match (prev_char, next_char) {
                (Some(p), Some(n)) => p.is_ascii_digit() && n.is_ascii_digit(),
                _ => false,
            };

            if !is_decimal {
                match next_char {
                    Some(c) if c.is_whitespace() => delimiters.push('.'),
                    None => delimiters.push('.'),
                    _ => {}
                }
            }
        }
    }

    delimiters
}

/// Inspects a `MatchSpan` against full text and a boundary rule, producing a structured report.
pub fn inspect_match_boundary(
    _full_text: &str,
    span: &MatchSpan,
    boundary_rule: BoundaryCheckType,
) -> BoundaryValidationReport {
    let text = &span.matched_text;

    let line_breaks_count = text.chars().filter(|&c| c == '\n' || c == '\r').count();
    let spans_line_breaks = line_breaks_count > 0;

    let detected_tags: Vec<String> = HTML_TAG_REGEX
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect();
    let crosses_html_tags = !detected_tags.is_empty();

    let clause_delims = extract_clause_delimiters(text);
    let crosses_clauses = !clause_delims.is_empty() || spans_line_breaks;

    let mut violations = Vec::new();

    match boundary_rule {
        BoundaryCheckType::SingleLineOnly => {
            if spans_line_breaks {
                violations.push(format!(
                    "Match span spans {line_breaks_count} line break(s): '{}' violates SingleLineOnly boundary rule",
                    span.matched_text
                ));
            }
        }
        BoundaryCheckType::ClauseBounded => {
            if crosses_clauses {
                violations.push(format!(
                    "Match span crosses clause boundaries (delimiters: {clause_delims:?}, line breaks: {line_breaks_count}): '{}' violates ClauseBounded boundary rule",
                    span.matched_text
                ));
            }
        }
        BoundaryCheckType::TagEnclosed => {
            if crosses_html_tags {
                violations.push(format!(
                    "Match span crosses HTML/XML tags {detected_tags:?}: '{}' violates TagEnclosed boundary rule",
                    span.matched_text
                ));
            }
        }
        BoundaryCheckType::CustomDelimiter(c) => {
            let count = text.chars().filter(|&ch| ch == c).count();
            if count > 0 {
                violations.push(format!(
                    "Match span crosses custom delimiter '{c}' {count} time(s): '{}' violates CustomDelimiter boundary rule",
                    span.matched_text
                ));
            }
        }
    }

    let is_bounded = violations.is_empty();

    BoundaryValidationReport {
        is_bounded,
        spans_line_breaks,
        crosses_html_tags,
        crosses_clauses,
        violations,
    }
}

/// Validates that a `MatchSpan` adheres strictly to the provided boundary rule.
/// Returns `Ok(report)` if bounded, or `Err(BoundaryViolation)` upon boundary breach or invalid span.
pub fn validate_match_boundary(
    full_text: &str,
    span: &MatchSpan,
    boundary_rule: BoundaryCheckType,
) -> Result<BoundaryValidationReport, BoundaryViolation> {
    if span.start > span.end || span.end > full_text.len() {
        return Err(BoundaryViolation::InvalidSpanRange {
            start: span.start,
            end: span.end,
            text_length: full_text.len(),
        });
    }

    let actual_slice = &full_text[span.start..span.end];
    if actual_slice != span.matched_text {
        return Err(BoundaryViolation::SpanContentMismatch {
            expected: span.matched_text.clone(),
            actual: actual_slice.to_string(),
        });
    }

    let report = inspect_match_boundary(full_text, span, boundary_rule.clone());

    if report.is_bounded {
        Ok(report)
    } else {
        match boundary_rule {
            BoundaryCheckType::SingleLineOnly => {
                let count = span.matched_text.chars().filter(|&c| c == '\n' || c == '\r').count();
                Err(BoundaryViolation::SpansLineBreaksForbidden {
                    line_breaks_count: count,
                    matched_text: span.matched_text.clone(),
                    report,
                })
            }
            BoundaryCheckType::ClauseBounded => {
                let delims = extract_clause_delimiters(&span.matched_text);
                Err(BoundaryViolation::CrossesClausesForbidden {
                    delimiters: delims,
                    matched_text: span.matched_text.clone(),
                    report,
                })
            }
            BoundaryCheckType::TagEnclosed => {
                let tags: Vec<String> = HTML_TAG_REGEX
                    .find_iter(&span.matched_text)
                    .map(|m| m.as_str().to_string())
                    .collect();
                Err(BoundaryViolation::CrossesHtmlTagsForbidden {
                    tags,
                    matched_text: span.matched_text.clone(),
                    report,
                })
            }
            BoundaryCheckType::CustomDelimiter(c) => {
                let count = span.matched_text.chars().filter(|&ch| ch == c).count();
                Err(BoundaryViolation::CrossesCustomDelimiterForbidden {
                    delimiter: c,
                    count,
                    matched_text: span.matched_text.clone(),
                    report,
                })
            }
        }
    }
}

/// Applies a surgical replacement inside `full_text` at `span`, validating that neither
/// the match span nor the replacement string bleed across forbidden boundaries.
pub fn validate_replacement_boundaries(
    full_text: &str,
    span: &MatchSpan,
    replacement: &str,
    boundary_rule: BoundaryCheckType,
) -> Result<String, BoundaryViolation> {
    // 1. Validate target span
    validate_match_boundary(full_text, span, boundary_rule.clone())?;

    // 2. Validate replacement text boundaries
    let replacement_span = MatchSpan::new(0, replacement.len(), replacement);
    validate_match_boundary(replacement, &replacement_span, boundary_rule)?;

    // 3. Assemble safely
    let mut out = String::with_capacity(full_text.len().saturating_sub(span.len()) + replacement.len());
    out.push_str(&full_text[..span.start]);
    out.push_str(replacement);
    out.push_str(&full_text[span.end..]);

    Ok(out)
}

/// Applies regex replacement across `full_text` ensuring every matched span
/// and replacement string strictly adhere to `boundary_rule`.
pub fn apply_bounded_regex_replacement(
    full_text: &str,
    regex: &Regex,
    replacement: &str,
    boundary_rule: BoundaryCheckType,
) -> Result<(String, usize), BoundaryViolation> {
    // Validate replacement first
    let replacement_span = MatchSpan::new(0, replacement.len(), replacement);
    validate_match_boundary(replacement, &replacement_span, boundary_rule.clone())?;

    // Validate all matches first
    for m in regex.find_iter(full_text) {
        let span = MatchSpan::new(m.start(), m.end(), m.as_str());
        validate_match_boundary(full_text, &span, boundary_rule.clone())?;
    }

    // Perform replacement
    let mut count = 0;
    let mut result = String::new();
    let mut last_end = 0;

    for m in regex.find_iter(full_text) {
        count += 1;
        result.push_str(&full_text[last_end..m.start()]);
        result.push_str(replacement);
        last_end = m.end();
    }
    result.push_str(&full_text[last_end..]);

    Ok((result, count))
}

/// Configuration specifying timeout bounds for an asynchronous or long-running wait operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub wait_ms: Option<u64>,
    pub max_allowed_ms: u64,
    pub operation_description: String,
}

impl TimeoutConfig {
    /// Constructs a new `TimeoutConfig`.
    pub fn new(
        wait_ms: Option<u64>,
        max_allowed_ms: u64,
        operation_description: impl Into<String>,
    ) -> Self {
        Self {
            wait_ms,
            max_allowed_ms,
            operation_description: operation_description.into(),
        }
    }

    /// Constructs a bounded `TimeoutConfig` with explicit millisecond wait.
    pub fn bounded(
        wait_ms: u64,
        max_allowed_ms: u64,
        operation_description: impl Into<String>,
    ) -> Self {
        Self {
            wait_ms: Some(wait_ms),
            max_allowed_ms,
            operation_description: operation_description.into(),
        }
    }

    /// Constructs an unbounded `TimeoutConfig` (`wait_ms == None`).
    pub fn unbounded(max_allowed_ms: u64, operation_description: impl Into<String>) -> Self {
        Self {
            wait_ms: None,
            max_allowed_ms,
            operation_description: operation_description.into(),
        }
    }
}

/// Structured inspection report evaluating a wait timeout against AGENTS.md requirements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeoutReport {
    pub is_bounded: bool,
    pub effective_timeout_ms: u64,
    pub is_unbounded_infinite: bool,
    pub exceeds_max: bool,
    pub violations: Vec<String>,
}

/// Violation encountered when an asynchronous wait lacks proper timeout bounding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeoutViolation {
    /// Unbounded wait (`wait_ms == None` or `wait_ms == Some(0)`) is strictly forbidden.
    UnboundedWaitForbidden {
        operation_description: String,
        details: String,
    },
    /// Wait timeout exceeds the maximum permitted ceiling for the operation.
    ExceedsMaxAllowed {
        operation_description: String,
        wait_ms: u64,
        max_allowed_ms: u64,
    },
    /// Asynchronous operation timed out before completion.
    OperationTimedOut {
        operation_description: String,
        wait_ms: u64,
    },
    /// Operation description cannot be empty or blank.
    EmptyOperationDescription,
}

impl fmt::Display for TimeoutViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnboundedWaitForbidden {
                operation_description,
                details,
            } => {
                write!(
                    f,
                    "Unbounded wait forbidden for '{operation_description}': {details}"
                )
            }
            Self::ExceedsMaxAllowed {
                operation_description,
                wait_ms,
                max_allowed_ms,
            } => {
                write!(
                    f,
                    "Wait timeout {wait_ms}ms exceeds maximum allowed {max_allowed_ms}ms for '{operation_description}'"
                )
            }
            Self::OperationTimedOut {
                operation_description,
                wait_ms,
            } => {
                write!(
                    f,
                    "Operation '{operation_description}' timed out after {wait_ms}ms"
                )
            }
            Self::EmptyOperationDescription => {
                write!(f, "Operation description cannot be empty")
            }
        }
    }
}

impl std::error::Error for TimeoutViolation {}

/// Inspects a `TimeoutConfig` and produces a structured `TimeoutReport`.
pub fn inspect_wait_timeout(config: &TimeoutConfig) -> TimeoutReport {
    let mut violations = Vec::new();

    let (is_unbounded_infinite, effective_timeout_ms) = match config.wait_ms {
        None => {
            violations.push(format!(
                "Unbounded wait forbidden: wait_ms is None for operation '{}'",
                config.operation_description
            ));
            (true, 0)
        }
        Some(0) => {
            violations.push(format!(
                "Unbounded wait forbidden: wait_ms is 0ms for operation '{}'",
                config.operation_description
            ));
            (true, 0)
        }
        Some(ms) => (false, ms),
    };

    let exceeds_max = if !is_unbounded_infinite {
        if effective_timeout_ms > config.max_allowed_ms {
            violations.push(format!(
                "Wait timeout {}ms exceeds maximum allowed threshold {}ms for operation '{}'",
                effective_timeout_ms, config.max_allowed_ms, config.operation_description
            ));
            true
        } else {
            false
        }
    } else {
        false
    };

    if config.operation_description.trim().is_empty() {
        violations.push("Operation description cannot be empty".to_string());
    }

    let is_bounded = !is_unbounded_infinite && !exceeds_max && !config.operation_description.trim().is_empty();

    TimeoutReport {
        is_bounded,
        effective_timeout_ms,
        is_unbounded_infinite,
        exceeds_max,
        violations,
    }
}

/// Validates that a wait timeout is strictly bounded and conforms to AGENTS.md rules.
/// Forbids unbounded waits (`wait_ms == None` or `wait_ms == Some(0)`).
pub fn validate_wait_timeout(config: &TimeoutConfig) -> Result<TimeoutReport, TimeoutViolation> {
    if config.operation_description.trim().is_empty() {
        return Err(TimeoutViolation::EmptyOperationDescription);
    }

    let report = inspect_wait_timeout(config);

    if report.is_unbounded_infinite {
        return Err(TimeoutViolation::UnboundedWaitForbidden {
            operation_description: config.operation_description.clone(),
            details: "Wait duration cannot be None or 0ms (unbounded waits are strictly forbidden by AGENTS.md)".to_string(),
        });
    }

    if report.exceeds_max {
        return Err(TimeoutViolation::ExceedsMaxAllowed {
            operation_description: config.operation_description.clone(),
            wait_ms: report.effective_timeout_ms,
            max_allowed_ms: config.max_allowed_ms,
        });
    }

    Ok(report)
}

/// Executes an asynchronous future bounded by `TimeoutConfig`.
/// Ensures wait is bounded and fails fast if the timeout elapses.
pub async fn execute_with_timeout_bounds<F, T>(
    config: &TimeoutConfig,
    fut: F,
) -> Result<T, TimeoutViolation>
where
    F: std::future::Future<Output = T>,
{
    let report = validate_wait_timeout(config)?;
    let duration = Duration::from_millis(report.effective_timeout_ms);

    match tokio::time::timeout(duration, fut).await {
        Ok(val) => Ok(val),
        Err(_) => Err(TimeoutViolation::OperationTimedOut {
            operation_description: config.operation_description.clone(),
            wait_ms: report.effective_timeout_ms,
        }),
    }
}

/// Polls an asynchronous predicate at `poll_interval_ms` intervals until it returns `Some(T)`
/// or the timeout configured in `config` elapses.
pub async fn poll_until_bounded_timeout<F, Fut, T>(
    config: &TimeoutConfig,
    poll_interval_ms: u64,
    mut check: F,
) -> Result<T, TimeoutViolation>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Option<T>>,
{
    let report = validate_wait_timeout(config)?;
    let total_timeout = Duration::from_millis(report.effective_timeout_ms);
    let interval = Duration::from_millis(poll_interval_ms.max(1));
    let start = tokio::time::Instant::now();

    loop {
        if let Some(val) = check().await {
            return Ok(val);
        }

        if start.elapsed() >= total_timeout {
            return Err(TimeoutViolation::OperationTimedOut {
                operation_description: config.operation_description.clone(),
                wait_ms: report.effective_timeout_ms,
            });
        }

        tokio::time::sleep(interval).await;
    }
}

/// Static code linter that inspects source code for naked sleep and unbounded loop wait violations.
pub fn lint_wait_calls_in_source(source_code: &str) -> Vec<String> {
    let mut violations = Vec::new();
    let lines: Vec<&str> = source_code.lines().collect();
    let mut in_block_comment = false;

    for (idx, line) in lines.iter().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();

        // Handle block comments /* ... */
        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }
        if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
            continue;
        }

        // Ignore single-line comments or doc comments
        if trimmed.starts_with("//") {
            continue;
        }

        // 1. Check for naked std::thread::sleep
        if trimmed.contains("thread::sleep(") || trimmed.contains("std::thread::sleep(") {
            let has_timeout_guard = trimmed.contains("timeout")
                || (idx > 0 && lines[idx - 1].contains("// BoundedTimeout:"));
            if !has_timeout_guard {
                violations.push(format!(
                    "Line {line_num}: Naked 'thread::sleep' detected: '{trimmed}' (all waits must be bounded with a timeout per AGENTS.md)"
                ));
            }
        }

        // 2. Check for naked tokio::time::sleep
        if trimmed.contains("tokio::time::sleep(") || trimmed.contains("time::sleep(") {
            let has_timeout_guard = trimmed.contains("timeout(")
                || trimmed.contains("select!")
                || (idx > 0 && lines[idx - 1].contains("// BoundedTimeout:"));
            if !has_timeout_guard {
                violations.push(format!(
                    "Line {line_num}: Naked 'tokio::time::sleep' detected: '{trimmed}' (must be bounded with tokio::time::timeout or deadline per AGENTS.md)"
                ));
            }
        }

        // 3. Check for unbounded loop waits
        if trimmed.starts_with("loop {") || trimmed.starts_with("while true {") || trimmed.starts_with("while (true) {") {
            let mut loop_body = Vec::new();
            let mut lookahead = idx;
            let mut brace_depth = 0;
            let mut loop_started = false;

            while lookahead < lines.len() && lookahead < idx + 15 {
                let curr = lines[lookahead];
                for ch in curr.chars() {
                    if ch == '{' {
                        brace_depth += 1;
                        loop_started = true;
                    } else if ch == '}' {
                        brace_depth -= 1;
                    }
                }
                loop_body.push(curr);
                if loop_started && brace_depth == 0 {
                    break;
                }
                lookahead += 1;
            }

            let loop_content = loop_body.join("\n");
            let has_sleep = loop_content.contains("sleep(");
            let has_bounded_guard = loop_content.contains("elapsed()")
                || loop_content.contains("timeout")
                || loop_content.contains("deadline")
                || loop_content.contains("retries")
                || loop_content.contains("attempts")
                || loop_content.contains("Instant::now")
                || loop_content.contains("max_wait");

            if has_sleep && !has_bounded_guard {
                violations.push(format!(
                    "Line {line_num}: Unbounded loop wait detected: loop contains sleep/wait without a timeout, deadline, or retry counter limit"
                ));
            }
        }
    }

    violations
}
