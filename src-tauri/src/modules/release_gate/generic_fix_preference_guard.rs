//! Generic-Fix Preference & Future-Proofing Guard.
//!
//! Enforces code quality and future-proofing invariants per AGENTS.md:
//! - "Code Quality: Prioritize root-cause, future-proof fixes rather than hardcoded logic,
//!   dead code, or speculative changes. Prioritize generalized solutions that cover entire
//!   classes of problems rather than one-off patches."
//! - "Model Routing Example: Prioritize wildcard patterns (e.g., `gemini-*-flash-*`)
//!   to anticipate future model releases rather than exact string matches."
//! - "Prompt Sanitization Example: For agent-client prompt sanitization, prioritize
//!   regex-based pattern matching over static keyword replacement, ensuring full coverage
//!   without stripping pipeline system prompts or user queries."

use serde::{Deserialize, Serialize};
use std::fmt;

/// Minimum branch threshold where an exact equality chain must be generalized.
pub const EXACT_CHAIN_THRESHOLD: usize = 3;

/// Minimum entry threshold where a literal lookup table must be generalized.
pub const LITERAL_TABLE_THRESHOLD: usize = 4;

/// Classification of a code pattern site performing matching, routing, or filtering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchSiteKind {
    /// Chained equality comparisons (e.g. `x == "a" || x == "b" || x == "c"`).
    ExactEqualityChain { branch_count: usize },
    /// Static lookup table / dictionary mapping literals to values.
    LiteralLookupTable { entry_count: usize },
    /// Future-proof wildcard glob pattern (e.g. `gemini-*-flash-*`, `claude-3-5-*`).
    WildcardPattern,
    /// Future-proof regex pattern (e.g. `^gemini-.*-flash-.*$`).
    RegexPattern,
    /// Extensible dynamic registry lookup table.
    RegistryLookup,
    /// Single exact equality comparison (e.g. `x == "model-name"`).
    SingleExactMatch,
}

impl MatchSiteKind {
    /// Returns true if the pattern kind is inherently future-proof.
    pub fn is_future_proof(&self) -> bool {
        matches!(
            self,
            Self::WildcardPattern | Self::RegexPattern | Self::RegistryLookup
        )
    }

    /// Returns true if the pattern uses exact string comparisons.
    pub fn is_exact_match(&self) -> bool {
        matches!(
            self,
            Self::ExactEqualityChain { .. } | Self::SingleExactMatch
        )
    }

    /// Returns true if the site is a literal lookup table.
    pub fn is_literal_table(&self) -> bool {
        matches!(self, Self::LiteralLookupTable { .. })
    }

    /// Returns a human-readable description of the match site kind.
    pub fn description(&self) -> String {
        match self {
            Self::ExactEqualityChain { branch_count } => {
                format!("Exact equality chain with {branch_count} branches")
            }
            Self::LiteralLookupTable { entry_count } => {
                format!("Literal lookup table with {entry_count} entries")
            }
            Self::WildcardPattern => "Wildcard pattern glob".to_string(),
            Self::RegexPattern => "Regular expression engine".to_string(),
            Self::RegistryLookup => "Dynamic registry lookup".to_string(),
            Self::SingleExactMatch => "Single exact match comparison".to_string(),
        }
    }
}

impl fmt::Display for MatchSiteKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// A monitored pattern matching or dispatch site in the codebase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchSite {
    /// Unique identifier or symbol name for the match site.
    pub site_id: String,
    /// Structural kind of pattern matching employed.
    pub kind: MatchSiteKind,
    /// Sample literals or candidates evaluated at this site.
    pub literal_samples: Vec<String>,
    /// Whether an architectural exemption justification is recorded.
    pub has_justification: bool,
    /// Detailed rationale explaining why exact matching was strictly necessary.
    pub justification: Option<String>,
}

impl MatchSite {
    /// Constructs a new match site with the specified kind.
    pub fn new(site_id: impl Into<String>, kind: MatchSiteKind) -> Self {
        Self {
            site_id: site_id.into(),
            kind,
            literal_samples: Vec::new(),
            has_justification: false,
            justification: None,
        }
    }

    /// Constructs an exact equality chain match site.
    pub fn exact_chain(site_id: impl Into<String>, branch_count: usize) -> Self {
        Self::new(site_id, MatchSiteKind::ExactEqualityChain { branch_count })
    }

    /// Constructs a literal lookup table match site.
    pub fn literal_table(site_id: impl Into<String>, entry_count: usize) -> Self {
        Self::new(site_id, MatchSiteKind::LiteralLookupTable { entry_count })
    }

    /// Constructs a wildcard pattern match site.
    pub fn wildcard(site_id: impl Into<String>) -> Self {
        Self::new(site_id, MatchSiteKind::WildcardPattern)
    }

    /// Constructs a regex pattern match site.
    pub fn regex(site_id: impl Into<String>) -> Self {
        Self::new(site_id, MatchSiteKind::RegexPattern)
    }

    /// Constructs a dynamic registry match site.
    pub fn registry(site_id: impl Into<String>) -> Self {
        Self::new(site_id, MatchSiteKind::RegistryLookup)
    }

    /// Constructs a single exact match site.
    pub fn single_exact(site_id: impl Into<String>) -> Self {
        Self::new(site_id, MatchSiteKind::SingleExactMatch)
    }

    /// Sets sample literals observed at this match site.
    pub fn with_samples(mut self, samples: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.literal_samples = samples.into_iter().map(Into::into).collect();
        self
    }

    /// Adds a single sample literal to this site.
    pub fn with_sample(mut self, sample: impl Into<String>) -> Self {
        self.literal_samples.push(sample.into());
        self
    }

    /// Attaches an architectural justification for an exemption.
    pub fn with_justification(mut self, reason: impl Into<String>) -> Self {
        let text = reason.into();
        let trimmed = text.trim();
        self.has_justification = !trimmed.is_empty();
        self.justification = if self.has_justification {
            Some(trimmed.to_string())
        } else {
            None
        };
        self
    }

    /// Clears any existing justification on this site.
    pub fn without_justification(mut self) -> Self {
        self.has_justification = false;
        self.justification = None;
        self
    }
}

/// Recommended generalization strategy for a non-future-proof match site.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeneralizationSuggestion {
    /// Refactor exact checks into a wildcard glob pattern.
    UseWildcardGlob { recommended_pattern: String },
    /// Refactor exact checks into a regular expression pattern.
    UseRegexEngine { recommended_pattern: String },
    /// Refactor static table into an extensible dynamic registry.
    UseDynamicRegistry,
    /// Explicit justification provided and accepted as valid exemption.
    JustifiedExemptionAcknowledged,
}

impl GeneralizationSuggestion {
    /// Returns true if this suggestion represents an acknowledged exemption.
    pub fn is_acknowledged(&self) -> bool {
        matches!(self, Self::JustifiedExemptionAcknowledged)
    }

    /// Returns a concise human-readable summary of the recommendation.
    pub fn summary(&self) -> String {
        match self {
            Self::UseWildcardGlob {
                recommended_pattern,
            } => {
                format!("Use wildcard glob: `{recommended_pattern}`")
            }
            Self::UseRegexEngine {
                recommended_pattern,
            } => {
                format!("Use regex engine: `{recommended_pattern}`")
            }
            Self::UseDynamicRegistry => "Use dynamic registry lookup".to_string(),
            Self::JustifiedExemptionAcknowledged => "Justified exemption acknowledged".to_string(),
        }
    }
}

impl fmt::Display for GeneralizationSuggestion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())
    }
}

/// Evaluation result for a single match site.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneralizationFinding {
    /// Identifier of the evaluated match site.
    pub site_id: String,
    /// Positive boolean indicating whether refactoring to a generalized pattern is required.
    pub is_generalization_required: bool,
    /// Explanatory diagnostic message.
    pub explanation: String,
    /// Concrete generalization suggestion or exemption acknowledgment.
    pub suggested_generalization: GeneralizationSuggestion,
}

impl GeneralizationFinding {
    /// Returns true if the finding indicates full compliance with generic-fix preference.
    pub fn is_compliant(&self) -> bool {
        !self.is_generalization_required
    }
}

/// Aggregated report evaluating multiple pattern matching sites across the codebase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeneralizationReport {
    /// Positive boolean indicating whether all evaluated sites comply without violations.
    pub is_clean: bool,
    /// Total count of match sites analyzed.
    pub total_sites_evaluated: usize,
    /// Count of match sites requiring generalization refactoring.
    pub generalization_required_count: usize,
    /// List of individual findings for each evaluated site.
    pub findings: Vec<GeneralizationFinding>,
}

impl GeneralizationReport {
    /// Constructs an empty report with zero evaluations.
    pub fn empty() -> Self {
        Self {
            is_clean: true,
            total_sites_evaluated: 0,
            generalization_required_count: 0,
            findings: Vec::new(),
        }
    }

    /// Returns all findings where generalization is required.
    pub fn required_findings(&self) -> Vec<&GeneralizationFinding> {
        self.findings
            .iter()
            .filter(|f| f.is_generalization_required)
            .collect()
    }

    /// Returns all findings that comply or are acknowledged exemptions.
    pub fn compliant_findings(&self) -> Vec<&GeneralizationFinding> {
        self.findings
            .iter()
            .filter(|f| !f.is_generalization_required)
            .collect()
    }

    /// Formats an actionable GitHub Markdown summary table of the report.
    pub fn format_markdown_summary(&self) -> String {
        let mut out = String::new();
        out.push_str("### Generic-Fix Preference & Future-Proofing Report\n\n");
        if self.is_clean {
            out.push_str("🟢 **Status**: PASS - All evaluated match sites comply with generic-fix preference.\n\n");
        } else {
            out.push_str(&format!(
                "🔴 **Status**: ACTION REQUIRED - {} site(s) require generalization.\n\n",
                self.generalization_required_count
            ));
        }
        out.push_str(&format!(
            "- **Total Sites Evaluated**: {}\n- **Generalization Required**: {}\n\n",
            self.total_sites_evaluated, self.generalization_required_count
        ));
        out.push_str("| Site ID | Required | Suggested Action | Details |\n");
        out.push_str("| :--- | :---: | :--- | :--- |\n");
        for f in &self.findings {
            let status = if f.is_generalization_required {
                "⚠️ Yes"
            } else {
                "✅ No"
            };
            out.push_str(&format!(
                "| `{}` | {} | {} | {} |\n",
                f.site_id,
                status,
                f.suggested_generalization.summary(),
                f.explanation
            ));
        }
        out
    }
}

/// Evaluates a collection of match sites and produces an aggregated generalization report.
pub fn evaluate_match_sites(sites: &[MatchSite]) -> GeneralizationReport {
    let findings: Vec<GeneralizationFinding> = sites.iter().map(evaluate_single_site).collect();
    let generalization_required_count = findings
        .iter()
        .filter(|f| f.is_generalization_required)
        .count();
    let is_clean = generalization_required_count == 0;
    GeneralizationReport {
        is_clean,
        total_sites_evaluated: sites.len(),
        generalization_required_count,
        findings,
    }
}

/// Evaluates an individual match site against future-proofing requirements.
pub fn evaluate_single_site(site: &MatchSite) -> GeneralizationFinding {
    match &site.kind {
        MatchSiteKind::ExactEqualityChain { branch_count } => {
            evaluate_exact_chain(site, *branch_count)
        }
        MatchSiteKind::LiteralLookupTable { entry_count } => {
            evaluate_literal_table(site, *entry_count)
        }
        MatchSiteKind::WildcardPattern => evaluate_wildcard_pattern(site),
        MatchSiteKind::RegexPattern => evaluate_regex_pattern(site),
        MatchSiteKind::RegistryLookup => evaluate_registry_lookup(site),
        MatchSiteKind::SingleExactMatch => evaluate_single_exact_match(site),
    }
}

fn evaluate_exact_chain(site: &MatchSite, branch_count: usize) -> GeneralizationFinding {
    if site.has_justification {
        let reason = site.justification.as_deref().unwrap_or("Exempted");
        let explanation = format!(
            "Exact equality chain ({branch_count} branches) exempted by justification: {reason}"
        );
        return GeneralizationFinding {
            site_id: site.site_id.clone(),
            is_generalization_required: false,
            explanation,
            suggested_generalization: GeneralizationSuggestion::JustifiedExemptionAcknowledged,
        };
    }
    let recommended_pattern = derive_recommended_wildcard(&site.literal_samples);
    let is_required = branch_count >= EXACT_CHAIN_THRESHOLD;
    let explanation = if is_required {
        format!(
            "Exact equality chain ({branch_count} branches) exceeds threshold (>= {EXACT_CHAIN_THRESHOLD}). \
             Prioritize wildcard patterns (e.g. `gemini-*-flash-*`) or regex per AGENTS.md."
        )
    } else {
        format!(
            "Exact equality chain ({branch_count} branches) is below threshold (< {EXACT_CHAIN_THRESHOLD})."
        )
    };
    GeneralizationFinding {
        site_id: site.site_id.clone(),
        is_generalization_required: is_required,
        explanation,
        suggested_generalization: GeneralizationSuggestion::UseWildcardGlob {
            recommended_pattern,
        },
    }
}

fn evaluate_literal_table(site: &MatchSite, entry_count: usize) -> GeneralizationFinding {
    if site.has_justification {
        let reason = site.justification.as_deref().unwrap_or("Exempted");
        let explanation = format!(
            "Literal lookup table ({entry_count} entries) exempted by justification: {reason}"
        );
        return GeneralizationFinding {
            site_id: site.site_id.clone(),
            is_generalization_required: false,
            explanation,
            suggested_generalization: GeneralizationSuggestion::JustifiedExemptionAcknowledged,
        };
    }
    let is_required = entry_count >= LITERAL_TABLE_THRESHOLD;
    let explanation = if is_required {
        format!(
            "Literal lookup table ({entry_count} entries) exceeds threshold (>= {LITERAL_TABLE_THRESHOLD}). \
             Prioritize dynamic registry lookup or future-proof rule dispatch per AGENTS.md."
        )
    } else {
        format!(
            "Literal lookup table ({entry_count} entries) is below threshold (< {LITERAL_TABLE_THRESHOLD})."
        )
    };
    GeneralizationFinding {
        site_id: site.site_id.clone(),
        is_generalization_required: is_required,
        explanation,
        suggested_generalization: GeneralizationSuggestion::UseDynamicRegistry,
    }
}

fn evaluate_wildcard_pattern(site: &MatchSite) -> GeneralizationFinding {
    let (explanation, suggested_generalization) = if site.has_justification {
        (
            format!(
                "Wildcard pattern matcher with justification: {}",
                site.justification.as_deref().unwrap_or("Justified")
            ),
            GeneralizationSuggestion::JustifiedExemptionAcknowledged,
        )
    } else {
        let pattern = derive_recommended_wildcard(&site.literal_samples);
        (
            "Wildcard pattern matcher complies with generic-fix preference (future-proof)."
                .to_string(),
            GeneralizationSuggestion::UseWildcardGlob {
                recommended_pattern: pattern,
            },
        )
    };
    GeneralizationFinding {
        site_id: site.site_id.clone(),
        is_generalization_required: false,
        explanation,
        suggested_generalization,
    }
}

fn evaluate_regex_pattern(site: &MatchSite) -> GeneralizationFinding {
    let (explanation, suggested_generalization) = if site.has_justification {
        (
            format!(
                "Regex pattern matcher with justification: {}",
                site.justification.as_deref().unwrap_or("Justified")
            ),
            GeneralizationSuggestion::JustifiedExemptionAcknowledged,
        )
    } else {
        let pattern = derive_recommended_regex(&site.literal_samples);
        (
            "Regex pattern matcher complies with generic-fix preference (future-proof)."
                .to_string(),
            GeneralizationSuggestion::UseRegexEngine {
                recommended_pattern: pattern,
            },
        )
    };
    GeneralizationFinding {
        site_id: site.site_id.clone(),
        is_generalization_required: false,
        explanation,
        suggested_generalization,
    }
}

fn evaluate_registry_lookup(site: &MatchSite) -> GeneralizationFinding {
    let (explanation, suggested_generalization) = if site.has_justification {
        (
            format!(
                "Dynamic registry lookup with justification: {}",
                site.justification.as_deref().unwrap_or("Justified")
            ),
            GeneralizationSuggestion::JustifiedExemptionAcknowledged,
        )
    } else {
        (
            "Dynamic registry lookup complies with generic-fix preference (extensible)."
                .to_string(),
            GeneralizationSuggestion::UseDynamicRegistry,
        )
    };
    GeneralizationFinding {
        site_id: site.site_id.clone(),
        is_generalization_required: false,
        explanation,
        suggested_generalization,
    }
}

fn evaluate_single_exact_match(site: &MatchSite) -> GeneralizationFinding {
    if site.has_justification {
        let reason = site.justification.as_deref().unwrap_or("Justified");
        let explanation = format!("Single exact match exempted by justification: {reason}");
        return GeneralizationFinding {
            site_id: site.site_id.clone(),
            is_generalization_required: false,
            explanation,
            suggested_generalization: GeneralizationSuggestion::JustifiedExemptionAcknowledged,
        };
    }
    let recommended_pattern = derive_recommended_wildcard(&site.literal_samples);
    let explanation = "Single exact match without justification is hardcoded; \
                       prioritize wildcard patterns (e.g. `gemini-*-flash-*`) or regex per AGENTS.md."
        .to_string();
    GeneralizationFinding {
        site_id: site.site_id.clone(),
        is_generalization_required: true,
        explanation,
        suggested_generalization: GeneralizationSuggestion::UseWildcardGlob {
            recommended_pattern,
        },
    }
}

/// Derives a future-proof wildcard glob pattern from sample literals.
pub fn derive_recommended_wildcard(samples: &[String]) -> String {
    if samples.is_empty() {
        return "*".to_string();
    }
    if samples.len() == 1 {
        return derive_single_sample_wildcard(&samples[0]);
    }
    let prefix = find_common_prefix(samples);
    let suffix = find_common_suffix(samples);
    if !prefix.is_empty() && !suffix.is_empty() && prefix.len() + suffix.len() < samples[0].len() {
        format!("{prefix}*{suffix}")
    } else if !prefix.is_empty() {
        format!("{prefix}*")
    } else if !suffix.is_empty() {
        format!("*{suffix}")
    } else {
        format!("*{}*", samples[0])
    }
}

fn derive_single_sample_wildcard(sample: &str) -> String {
    let parts: Vec<&str> = sample.split('-').collect();
    if parts.len() >= 3 {
        format!("{}-*-{}*", parts[0], parts[parts.len() - 1])
    } else if parts.len() == 2 {
        format!("{}-*", parts[0])
    } else {
        format!("{sample}*")
    }
}

/// Derives a recommended regular expression pattern from sample literals.
pub fn derive_recommended_regex(samples: &[String]) -> String {
    if samples.is_empty() {
        return r"^.*$".to_string();
    }
    let prefix = find_common_prefix(samples);
    if !prefix.is_empty() {
        format!(r"^{prefix}.*$")
    } else {
        format!(r"^.*{}.*$", samples[0])
    }
}

fn find_common_prefix(items: &[String]) -> String {
    if items.is_empty() {
        return String::new();
    }
    let first = &items[0];
    let mut prefix = String::new();
    for (i, ch) in first.chars().enumerate() {
        if items[1..].iter().all(|it| it.chars().nth(i) == Some(ch)) {
            prefix.push(ch);
        } else {
            break;
        }
    }
    prefix
}

fn find_common_suffix(items: &[String]) -> String {
    if items.is_empty() {
        return String::new();
    }
    let first = &items[0];
    let mut suffix_chars = Vec::new();
    for (i, ch) in first.chars().rev().enumerate() {
        if items[1..]
            .iter()
            .all(|it| it.chars().rev().nth(i) == Some(ch))
        {
            suffix_chars.push(ch);
        } else {
            break;
        }
    }
    suffix_chars.into_iter().rev().collect()
}
