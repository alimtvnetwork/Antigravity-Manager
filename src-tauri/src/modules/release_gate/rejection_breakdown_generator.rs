//! Rejection File-by-File Breakdown Generator (`Imp-624`).
//!
//! Enforces AGENTS.md Contributor Respect & Attribution guideline:
//! "Accompany every rejection with a concrete file-by-file accept/drop breakdown
//! and an actionable path forward."
//!
//! Features:
//! 1. Models file-level review decisions (`Accept`, `Drop`, `PartialAccept`).
//! 2. Validates that every rejection is accompanied by a concrete actionable path forward.
//! 3. Classifies git changesets against drop patterns and rationale providers.
//! 4. Renders standardized, clean GitHub Markdown tables with disposition badges.
//! 5. Preserves git `Co-authored-by:` trailers per contributor attribution guidelines.

use super::coauthor_attribution_verifier::format_coauthor_trailer;
use serde::{Deserialize, Serialize};

/// Disposition for an individual file changeset in a PR or patch review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileDisposition {
    /// File changes are approved and accepted.
    Accept,
    /// File changes are dropped/rejected.
    Drop,
    /// File changes are partially accepted with specific modifications or revisions needed.
    PartialAccept,
}

/// Backwards compatibility alias for `FileDisposition`.
pub type FileActionDisposition = FileDisposition;

impl FileDisposition {
    /// Returns the uppercase text label for this disposition.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Accept => "ACCEPT",
            Self::Drop => "DROP",
            Self::PartialAccept => "PARTIAL ACCEPT",
        }
    }

    /// Formats a GitHub Markdown badge with colored indicators.
    pub fn badge_markdown(&self) -> &'static str {
        match self {
            Self::Accept => "🟢 **Accept**",
            Self::Drop => "🔴 **Drop**",
            Self::PartialAccept => "🟡 **Partial Accept**",
        }
    }

    /// Whether this disposition represents an approved file.
    pub fn is_accepted(&self) -> bool {
        matches!(self, Self::Accept)
    }

    /// Whether this disposition represents a dropped file.
    pub fn is_dropped(&self) -> bool {
        matches!(self, Self::Drop)
    }

    /// Whether this disposition represents a partially accepted file.
    pub fn is_partial(&self) -> bool {
        matches!(self, Self::PartialAccept)
    }
}

impl std::fmt::Display for FileDisposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Accept => write!(f, "Accept"),
            Self::Drop => write!(f, "Drop"),
            Self::PartialAccept => write!(f, "Partial Accept"),
        }
    }
}

/// An evaluation entry for a single repository file in the pull request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileBreakdownItem {
    pub path: String,
    pub disposition: FileDisposition,
    pub rationale: String,
    pub suggested_action: Option<String>,
}

impl FileBreakdownItem {
    /// Constructs a new `FileBreakdownItem`.
    pub fn new(
        path: impl Into<String>,
        disposition: FileDisposition,
        rationale: impl Into<String>,
        suggested_action: Option<String>,
    ) -> Self {
        Self {
            path: path.into(),
            disposition,
            rationale: rationale.into(),
            suggested_action,
        }
    }

    /// Helper constructor for an accepted file.
    pub fn accept(path: impl Into<String>, rationale: impl Into<String>) -> Self {
        Self::new(path, FileDisposition::Accept, rationale, None)
    }

    /// Helper constructor for a dropped file.
    pub fn drop(
        path: impl Into<String>,
        rationale: impl Into<String>,
        suggested_action: impl Into<String>,
    ) -> Self {
        Self::new(
            path,
            FileDisposition::Drop,
            rationale,
            Some(suggested_action.into()),
        )
    }

    /// Helper constructor for a partially accepted file.
    pub fn partial(
        path: impl Into<String>,
        rationale: impl Into<String>,
        suggested_action: impl Into<String>,
    ) -> Self {
        Self::new(
            path,
            FileDisposition::PartialAccept,
            rationale,
            Some(suggested_action.into()),
        )
    }
}

/// Structured report for a rejection or changeset breakdown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RejectionBreakdownReport {
    pub pr_number: Option<u64>,
    pub title: String,
    pub contributor: String,
    pub summary: String,
    pub file_breakdowns: Vec<FileBreakdownItem>,
    pub actionable_path_forward: Vec<String>,
    pub co_authored_by_trailer: Option<String>,
}

impl RejectionBreakdownReport {
    /// Constructs a basic report with title and contributor handle.
    pub fn new(title: impl Into<String>, contributor: impl Into<String>) -> Self {
        Self {
            pr_number: None,
            title: title.into(),
            contributor: contributor.into(),
            summary: String::new(),
            file_breakdowns: Vec::new(),
            actionable_path_forward: Vec::new(),
            co_authored_by_trailer: None,
        }
    }

    /// Builder method to set PR number.
    pub fn with_pr_number(mut self, pr_number: u64) -> Self {
        self.pr_number = Some(pr_number);
        self
    }

    /// Builder method to set summary description.
    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = summary.into();
        self
    }

    /// Builder method to add a single file breakdown item.
    pub fn with_file_breakdown(mut self, item: FileBreakdownItem) -> Self {
        self.file_breakdowns.push(item);
        self
    }

    /// Builder method to set actionable path forward steps.
    pub fn with_actionable_path(mut self, steps: Vec<String>) -> Self {
        self.actionable_path_forward = steps;
        self
    }

    /// Builder method to append an actionable step.
    pub fn with_actionable_step(mut self, step: impl Into<String>) -> Self {
        self.actionable_path_forward.push(step.into());
        self
    }

    /// Builder method to set an explicit `Co-authored-by:` trailer.
    pub fn with_co_author_trailer(mut self, trailer: impl Into<String>) -> Self {
        self.co_authored_by_trailer = Some(trailer.into());
        self
    }

    /// Builder method to format and attach contributor attribution.
    pub fn with_co_author(mut self, name: &str, email: &str) -> Self {
        self.set_co_author(name, email);
        self
    }

    /// Formats and sets contributor `Co-authored-by:` trailer.
    pub fn set_co_author(&mut self, name: &str, email: &str) {
        if let Ok(trailer) = format_coauthor_trailer(name, email) {
            self.co_authored_by_trailer = Some(trailer);
        } else {
            self.co_authored_by_trailer = Some(format!(
                "Co-authored-by: {} <{}>",
                name.trim(),
                email.trim()
            ));
        }
    }

    /// Checks if the breakdown contains any rejections (`Drop` or `PartialAccept`).
    pub fn has_rejections(&self) -> bool {
        self.file_breakdowns
            .iter()
            .any(|item| item.disposition != FileDisposition::Accept)
    }

    /// Total count of evaluated files.
    pub fn total_files(&self) -> usize {
        self.file_breakdowns.len()
    }

    /// Count of accepted files.
    pub fn accepted_count(&self) -> usize {
        self.file_breakdowns
            .iter()
            .filter(|i| i.disposition == FileDisposition::Accept)
            .count()
    }

    /// Count of dropped files.
    pub fn dropped_count(&self) -> usize {
        self.file_breakdowns
            .iter()
            .filter(|i| i.disposition == FileDisposition::Drop)
            .count()
    }

    /// Count of partially accepted files.
    pub fn partial_count(&self) -> usize {
        self.file_breakdowns
            .iter()
            .filter(|i| i.disposition == FileDisposition::PartialAccept)
            .count()
    }
}

/// Validates whether a file path adheres to repository-relative Git path hygiene.
pub fn is_relative_git_path(path: &str) -> bool {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return false;
    }
    // Reject Windows drive letters
    if trimmed.len() >= 2
        && trimmed
            .chars()
            .next()
            .map_or(false, |c| c.is_ascii_alphabetic())
        && trimmed.as_bytes()[1] == b':'
    {
        return false;
    }
    // Reject root-anchored paths
    if trimmed.starts_with('/') || trimmed.starts_with('\\') {
        return false;
    }
    // Reject tilde home paths
    if trimmed.starts_with('~') {
        return false;
    }
    // Reject parent traversal that escapes the repo root
    if trimmed.starts_with("../") || trimmed.starts_with("..\\") || trimmed == ".." {
        return false;
    }
    true
}

/// Matches a file path against a pattern (exact, prefix `*`, suffix `*`, or substring).
pub fn pattern_matches(path: &str, pattern: &str) -> bool {
    let pat = pattern.trim();
    if pat.is_empty() {
        return false;
    }
    if pat == "*" || pat == path {
        return true;
    }
    if let Some(suffix) = pat.strip_prefix('*') {
        return path.ends_with(suffix);
    }
    if let Some(prefix) = pat.strip_suffix('*') {
        return path.starts_with(prefix);
    }
    if let Some(idx) = pat.find('*') {
        let prefix = &pat[..idx];
        let suffix = &pat[idx + 1..];
        return path.starts_with(prefix) && path.ends_with(suffix);
    }
    path.contains(pat)
}

/// Escapes special Markdown table characters (`|`, linebreaks).
fn escape_markdown_table_cell(text: &str) -> String {
    text.replace('\r', "")
        .replace('\n', " ")
        .replace('|', "\\|")
}

/// Formats a complete Markdown report with file table, disposition badges, rationales,
/// and an explicit "Actionable Path Forward" checklist per AGENTS.md.
pub fn generate_markdown_rejection_breakdown(report: &RejectionBreakdownReport) -> String {
    let mut out = String::new();

    // 1. Header
    match report.pr_number {
        Some(num) => {
            if report.title.trim().is_empty() {
                out.push_str(&format!("### Review Feedback for PR #{}\n\n", num));
            } else {
                out.push_str(&format!(
                    "### Review Feedback for PR #{} (`{}`)\n\n",
                    num,
                    report.title.trim()
                ));
            }
        }
        None => {
            if report.title.trim().is_empty() {
                out.push_str("### Review Feedback: File-by-File Breakdown\n\n");
            } else {
                out.push_str(&format!(
                    "### Review Feedback for `{}`\n\n",
                    report.title.trim()
                ));
            }
        }
    }

    // 2. Contributor recognition
    let clean_contributor = report.contributor.trim();
    if !clean_contributor.is_empty() {
        let handle = clean_contributor.trim_start_matches('@');
        out.push_str(&format!(
            "Thank you @{} for your contribution to Antigravity Manager!\n\n",
            handle
        ));
    }
    out.push_str(
        "Per our contributor respect guidelines, every rejection is accompanied by a concrete \
        file-by-file accept/drop breakdown and an actionable path forward.\n\n",
    );

    // 3. Summary
    if !report.summary.trim().is_empty() {
        out.push_str("#### Summary\n\n");
        out.push_str(report.summary.trim());
        out.push_str("\n\n");
    }

    // 4. File-by-File Table
    out.push_str("#### File-by-File Disposition Breakdown\n\n");
    out.push_str("| File | Disposition | Rationale | Suggested Action |\n");
    out.push_str("| :--- | :---: | :--- | :--- |\n");

    for item in &report.file_breakdowns {
        let action = item.suggested_action.as_deref().unwrap_or("-");
        out.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            item.path,
            item.disposition.badge_markdown(),
            escape_markdown_table_cell(&item.rationale),
            escape_markdown_table_cell(action),
        ));
    }
    out.push('\n');

    // 5. Actionable Path Forward Checklist
    out.push_str("#### 🧭 Actionable Path Forward\n\n");
    if report.actionable_path_forward.is_empty() {
        out.push_str("- [ ] No further revisions required; changeset is ready for merge.\n");
    } else {
        for step in &report.actionable_path_forward {
            let trimmed = step.trim();
            if trimmed.starts_with("- [ ] ") || trimmed.starts_with("- [x] ") {
                out.push_str(trimmed);
                out.push('\n');
            } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                out.push_str(&format!("- [ ] {}\n", &trimmed[2..]));
            } else {
                out.push_str(&format!("- [ ] {}\n", trimmed));
            }
        }
    }

    // 6. Contributor Attribution
    if let Some(trailer) = &report.co_authored_by_trailer {
        let trimmed_trailer = trailer.trim();
        if !trimmed_trailer.is_empty() {
            out.push_str("\n#### Contributor Attribution\n\n");
            out.push_str("```git\n");
            out.push_str(trimmed_trailer);
            out.push_str("\n```\n");
        }
    }

    out
}

/// Parses a list of files and classifies them against drop patterns and a custom rationale provider.
pub fn parse_and_classify_changeset(
    files: &[&str],
    drop_patterns: &[&str],
    rationale_provider: impl Fn(&str) -> (FileDisposition, String, Option<String>),
) -> RejectionBreakdownReport {
    let mut file_breakdowns = Vec::with_capacity(files.len());
    let mut actionable_path_forward = Vec::new();

    let mut accepted_count = 0;
    let mut dropped_count = 0;
    let mut partial_count = 0;

    for &file in files {
        let (mut disposition, mut rationale, mut suggested_action) = rationale_provider(file);

        // If file matches any drop pattern, enforce Drop
        let matched_drop_pattern = drop_patterns
            .iter()
            .find(|&&pat| pattern_matches(file, pat));
        if let Some(&pat) = matched_drop_pattern {
            disposition = FileDisposition::Drop;
            if rationale.trim().is_empty() {
                rationale = format!(
                    "Matches drop pattern '{}'; file should be dropped from review changeset.",
                    pat
                );
            }
            if suggested_action.is_none() {
                suggested_action = Some(format!("Exclude or remove '{}' from PR branch.", file));
            }
        }

        match disposition {
            FileDisposition::Accept => accepted_count += 1,
            FileDisposition::Drop => dropped_count += 1,
            FileDisposition::PartialAccept => partial_count += 1,
        }

        if let Some(action) = &suggested_action {
            let trimmed = action.trim();
            if !trimmed.is_empty() && !actionable_path_forward.contains(&trimmed.to_string()) {
                actionable_path_forward.push(trimmed.to_string());
            }
        }

        file_breakdowns.push(FileBreakdownItem {
            path: file.to_string(),
            disposition,
            rationale,
            suggested_action,
        });
    }

    let summary = format!(
        "Reviewed {} changeset file(s): {} accept, {} drop, {} partial accept.",
        files.len(),
        accepted_count,
        dropped_count,
        partial_count
    );

    RejectionBreakdownReport {
        pr_number: None,
        title: "PR Changeset Review".to_string(),
        contributor: String::new(),
        summary,
        file_breakdowns,
        actionable_path_forward,
        co_authored_by_trailer: None,
    }
}

/// Ensures no rejection is issued without a concrete actionable path forward and valid file rationales.
pub fn validate_rejection_has_actionable_path(
    report: &RejectionBreakdownReport,
) -> Result<(), String> {
    if report.file_breakdowns.is_empty() {
        return Err(
            "Rejection breakdown cannot be empty; at least one file must be evaluated.".to_string(),
        );
    }

    for item in &report.file_breakdowns {
        if !is_relative_git_path(&item.path) {
            return Err(format!(
                "File path '{}' is not a repository-relative Git path.",
                item.path
            ));
        }
        if item.rationale.trim().is_empty() {
            return Err(format!(
                "File '{}' has an empty review rationale; every review decision must disclose rationale.",
                item.path
            ));
        }
    }

    let rejected_count = report
        .file_breakdowns
        .iter()
        .filter(|item| item.disposition != FileDisposition::Accept)
        .count();

    if rejected_count > 0 {
        if report.actionable_path_forward.is_empty()
            || report
                .actionable_path_forward
                .iter()
                .all(|step| step.trim().is_empty())
        {
            return Err(format!(
                "Rejection includes {} dropped or partially accepted file(s), but lacks an actionable path forward.",
                rejected_count
            ));
        }
    }

    Ok(())
}
