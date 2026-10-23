//! Co-Author Trailer Alignment Guard (`AGENTS.md`).
//!
//! Enforces Contributor Respect & Attribution discipline per AGENTS.md:
//! "Preserve authorship by preferring the contributor's own PR for squash commits,
//! or attaching explicit `Co-authored-by:` trailers on merge commits and proxy PRs."
//!
//! Validates git `Co-authored-by:` trailer syntax, ensuring full contributor
//! names and valid `<email>` brackets are present while forbidding bare handles.

use serde::{Deserialize, Serialize};

/// Represents an individual parsed Git `Co-authored-by:` trailer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoauthorTrailer {
    pub name: String,
    pub email: String,
    pub raw_line: String,
}

/// Validation findings generated when examining trailer lines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrailerValidationFinding {
    MalformedSyntax { line: usize, raw: String },
    MissingEmailBracket { line: usize, raw: String },
    InvalidEmailFormat { line: usize, email: String },
    DisallowedHandleInName { line: usize, handle: String },
}

impl std::fmt::Display for TrailerValidationFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MalformedSyntax { line, raw } => {
                write!(
                    f,
                    "Line {}: Malformed Co-authored-by trailer syntax: '{}'",
                    line, raw
                )
            }
            Self::MissingEmailBracket { line, raw } => {
                write!(
                    f,
                    "Line {}: Missing '<email>' brackets in trailer: '{}'",
                    line, raw
                )
            }
            Self::InvalidEmailFormat { line, email } => {
                write!(
                    f,
                    "Line {}: Invalid contributor email format: '{}'",
                    line, email
                )
            }
            Self::DisallowedHandleInName { line, handle } => {
                write!(
                    f,
                    "Line {}: Disallowed handle '{}' found in contributor name (use real name)",
                    line, handle
                )
            }
        }
    }
}

impl std::error::Error for TrailerValidationFinding {}

/// Detailed audit report summarizing trailer compliance in a commit message.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CoauthorAuditReport {
    pub is_valid: bool,
    pub trailers: Vec<CoauthorTrailer>,
    pub findings: Vec<TrailerValidationFinding>,
    pub total_trailers_found: usize,
}

impl CoauthorAuditReport {
    /// Returns true if valid trailers were detected and findings are clean.
    pub fn is_clean(&self) -> bool {
        self.is_valid && self.findings.is_empty()
    }

    /// Returns true if any valid trailers were captured.
    pub fn has_trailers(&self) -> bool {
        !self.trailers.is_empty()
    }
}

/// Formats a standardized git `Co-authored-by:` trailer.
pub fn format_coauthor_trailer(name: &str, email: &str) -> String {
    format!("Co-authored-by: {} <{}>", name.trim(), email.trim())
}

/// Validates that contributor name does not contain bare handles.
fn validate_contributor_name(
    name: &str,
    line_num: usize,
) -> Result<(), TrailerValidationFinding> {
    if name.starts_with('@') {
        return Err(TrailerValidationFinding::DisallowedHandleInName {
            line: line_num,
            handle: name.to_string(),
        });
    }

    for word in name.split_whitespace() {
        if word.starts_with('@') || word.contains('@') {
            return Err(TrailerValidationFinding::DisallowedHandleInName {
                line: line_num,
                handle: word.to_string(),
            });
        }
    }
    Ok(())
}

/// Validates that contributor email contains valid at and domain parts.
fn validate_contributor_email(
    email: &str,
    line_num: usize,
) -> Result<(), TrailerValidationFinding> {
    let at_idx = match email.find('@') {
        Some(idx) => idx,
        None => {
            return Err(TrailerValidationFinding::InvalidEmailFormat {
                line: line_num,
                email: email.to_string(),
            });
        }
    };

    let local_part = &email[..at_idx];
    let domain_part = &email[at_idx + 1..];

    if local_part.is_empty()
        || domain_part.is_empty()
        || !domain_part.contains('.')
        || domain_part.ends_with('.')
    {
        return Err(TrailerValidationFinding::InvalidEmailFormat {
            line: line_num,
            email: email.to_string(),
        });
    }
    Ok(())
}

/// Extracts contributor name and email enclosed within bracket pairs.
fn extract_name_and_email<'a>(
    payload: &'a str,
    raw_line: &str,
    line_num: usize,
) -> Result<(&'a str, &'a str), TrailerValidationFinding> {
    let open_idx = payload.find('<');
    let close_idx = payload.rfind('>');

    match (open_idx, close_idx) {
        (Some(open), Some(close)) if open < close => {
            let name = payload[..open].trim();
            let email = payload[open + 1..close].trim();
            let trailing = payload[close + 1..].trim();

            if name.is_empty() || !trailing.is_empty() {
                return Err(TrailerValidationFinding::MalformedSyntax {
                    line: line_num,
                    raw: raw_line.to_string(),
                });
            }
            Ok((name, email))
        }
        _ => Err(TrailerValidationFinding::MissingEmailBracket {
            line: line_num,
            raw: raw_line.to_string(),
        }),
    }
}

/// Parses an individual line to evaluate whether it is a valid `Co-authored-by:` trailer.
pub fn parse_coauthor_line(
    line: &str,
    line_num: usize,
) -> Result<Option<CoauthorTrailer>, TrailerValidationFinding> {
    let trimmed = line.trim();
    let lower = trimmed.to_ascii_lowercase();

    if lower.starts_with("co-authored-by") && !lower.starts_with("co-authored-by:") {
        return Err(TrailerValidationFinding::MalformedSyntax {
            line: line_num,
            raw: line.to_string(),
        });
    }

    if !lower.starts_with("co-authored-by:") {
        return Ok(None);
    }

    let payload = trimmed["co-authored-by:".len()..].trim();
    if payload.is_empty() {
        return Err(TrailerValidationFinding::MalformedSyntax {
            line: line_num,
            raw: line.to_string(),
        });
    }

    let (name, email) = extract_name_and_email(payload, line, line_num)?;
    validate_contributor_name(name, line_num)?;
    validate_contributor_email(email, line_num)?;

    Ok(Some(CoauthorTrailer {
        name: name.to_string(),
        email: email.to_string(),
        raw_line: line.to_string(),
    }))
}

/// Audits all commit message lines to evaluate and collect co-author trailers.
pub fn audit_commit_message_trailers(commit_message: &str) -> CoauthorAuditReport {
    let mut trailers = Vec::new();
    let mut findings = Vec::new();

    for (idx, line) in commit_message.lines().enumerate() {
        let line_num = idx + 1;
        match parse_coauthor_line(line, line_num) {
            Ok(Some(trailer)) => trailers.push(trailer),
            Ok(None) => {}
            Err(finding) => findings.push(finding),
        }
    }

    let is_valid = findings.is_empty();
    let total_trailers_found = trailers.len();

    CoauthorAuditReport {
        is_valid,
        trailers,
        findings,
        total_trailers_found,
    }
}
