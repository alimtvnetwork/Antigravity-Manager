//! Git Co-Author Attribution Trailer Verifier.
//!
//! Enforces contributor attribution discipline per AGENTS.md:
//! Preserves authorship by validating and formatting `Co-authored-by: Name <email>`
//! trailers on merge commits, proxy PRs, and squash commits.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoAuthor {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AttributionTrailerReport {
    pub has_coauthors: bool,
    pub coauthors: Vec<CoAuthor>,
    pub invalid_trailers: Vec<String>,
}

/// Formats a standardized git `Co-authored-by:` trailer.
pub fn format_coauthor_trailer(name: &str, email: &str) -> Result<String, &'static str> {
    let clean_name = name.trim();
    let clean_email = email.trim().trim_start_matches('<').trim_end_matches('>');

    if clean_name.is_empty() {
        return Err("Contributor name cannot be empty");
    }
    if clean_email.is_empty() || !clean_email.contains('@') || !clean_email.contains('.') {
        return Err("Invalid contributor email address format");
    }

    Ok(format!("Co-authored-by: {} <{}>", clean_name, clean_email))
}

/// Parses and validates all `Co-authored-by:` trailers in a commit message.
pub fn parse_coauthor_trailers(commit_message: &str) -> AttributionTrailerReport {
    let mut coauthors = Vec::new();
    let mut invalid_trailers = Vec::new();

    for line in commit_message.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Co-authored-by:") {
            let content = trimmed["Co-authored-by:".len()..].trim();
            if let Some(open_idx) = content.find('<') {
                if let Some(close_idx) = content.find('>') {
                    if open_idx < close_idx {
                        let name = content[..open_idx].trim().to_string();
                        let email = content[open_idx + 1..close_idx].trim().to_string();
                        if !name.is_empty() && email.contains('@') && email.contains('.') {
                            coauthors.push(CoAuthor { name, email });
                            continue;
                        }
                    }
                }
            }
            invalid_trailers.push(trimmed.to_string());
        }
    }

    let has_coauthors = !coauthors.is_empty();

    AttributionTrailerReport {
        has_coauthors,
        coauthors,
        invalid_trailers,
    }
}
