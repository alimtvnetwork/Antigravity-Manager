//! Maintainer Staging Protocol Guard.
//!
//! Enforces the Maintainer Staging Protocol per AGENTS.md:
//! "When introducing new changes (features, major refactors, non-trivial fixes),
//! prompt and confirm with maintainers whether to implement and test on `beta`
//! branch first. Validate stability on `beta` (with optional independent preview builds)
//! prior to merging into `main`."

use serde::{Deserialize, Serialize};

/// Verdict determining whether staging on `beta` branch is required, recommended, or unnecessary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StagingVerdict {
    /// High-impact changeset requiring mandatory `beta` branch staging.
    Required,
    /// Non-trivial changeset where `beta` branch staging is strongly recommended.
    Recommended,
    /// Low-risk, docs-only, or trivial one-line fix where direct base merge is permitted.
    Unnecessary,
}

impl StagingVerdict {
    /// Returns the string representation of the verdict.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Recommended => "recommended",
            Self::Unnecessary => "unnecessary",
        }
    }

    /// Whether staging on `beta` is mandatory.
    pub fn is_required(&self) -> bool {
        matches!(self, Self::Required)
    }

    /// Whether staging on `beta` is recommended.
    pub fn is_recommended(&self) -> bool {
        matches!(self, Self::Recommended)
    }

    /// Whether staging on `beta` is unnecessary.
    pub fn is_unnecessary(&self) -> bool {
        matches!(self, Self::Unnecessary)
    }
}

impl std::fmt::Display for StagingVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Deciding factors that influence the staging verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "factor_type", rename_all = "snake_case")]
pub enum StagingDecidingFactor {
    /// A database or configuration schema migration was detected.
    SchemaMigrationDetected,
    /// A public surface (CLI command, API route, IPC handler, or public trait) was modified.
    PublicSurfaceModified { surface_type: String },
    /// A new feature flag was introduced.
    FeatureFlagIntroduced { flag_name: String },
    /// Release machinery, CI/CD pipeline, or packaging scripts were modified.
    ReleaseMachineryModified,
    /// Changeset spans multiple functional modules.
    MultiModuleChangeset { modules_count: usize },
    /// Substantial line delta indicating a major codebase refactor.
    NonTrivialCodebaseRefactor { lines_changed: usize },
    /// Isolated trivial one-line fix.
    TrivialOneLineFix,
    /// Changeset consists exclusively of documentation updates.
    DocumentationOnly,
}

impl StagingDecidingFactor {
    /// Human-readable explanation of the factor.
    pub fn description(&self) -> String {
        match self {
            Self::SchemaMigrationDetected => {
                "Database or state schema migration detected".to_string()
            }
            Self::PublicSurfaceModified { surface_type } => {
                format!("Public surface modified: {}", surface_type)
            }
            Self::FeatureFlagIntroduced { flag_name } => {
                format!("New feature flag introduced: {}", flag_name)
            }
            Self::ReleaseMachineryModified => {
                "Release machinery or CI/CD workflow modified".to_string()
            }
            Self::MultiModuleChangeset { modules_count } => {
                format!("Multi-module changeset spanning {} files/modules", modules_count)
            }
            Self::NonTrivialCodebaseRefactor { lines_changed } => {
                format!("Non-trivial refactor with {} lines changed", lines_changed)
            }
            Self::TrivialOneLineFix => "Trivial single-line bug fix".to_string(),
            Self::DocumentationOnly => "Documentation-only updates".to_string(),
        }
    }
}

/// Summary metrics of a changeset evaluated against the staging protocol.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ChangeSetSummary {
    pub files_changed_count: usize,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub has_schema_migration: bool,
    pub has_public_surface_change: bool,
    pub has_feature_flag: bool,
    pub has_release_machinery_change: bool,
    pub is_docs_only: bool,
    pub is_one_line_fix: bool,
    pub surface_type: Option<String>,
    pub flag_name: Option<String>,
}

impl ChangeSetSummary {
    /// Creates a basic summary with file counts and line deltas.
    pub fn new(files_changed_count: usize, lines_added: usize, lines_removed: usize) -> Self {
        Self {
            files_changed_count,
            lines_added,
            lines_removed,
            ..Default::default()
        }
    }

    /// Calculates total lines changed across additions and deletions.
    pub fn total_lines_changed(&self) -> usize {
        self.lines_added.saturating_add(self.lines_removed)
    }

    /// Sets whether schema migration is present.
    pub fn with_schema_migration(mut self, has_migration: bool) -> Self {
        self.has_schema_migration = has_migration;
        self
    }

    /// Sets public surface modification metadata.
    pub fn with_public_surface(mut self, surface_type: impl Into<String>) -> Self {
        self.has_public_surface_change = true;
        self.surface_type = Some(surface_type.into());
        self
    }

    /// Sets feature flag metadata.
    pub fn with_feature_flag(mut self, flag_name: impl Into<String>) -> Self {
        self.has_feature_flag = true;
        self.flag_name = Some(flag_name.into());
        self
    }

    /// Sets whether release machinery was changed.
    pub fn with_release_machinery(mut self, has_change: bool) -> Self {
        self.has_release_machinery_change = has_change;
        self
    }

    /// Sets documentation-only status.
    pub fn with_docs_only(mut self, is_docs: bool) -> Self {
        self.is_docs_only = is_docs;
        self
    }

    /// Sets trivial one-line fix status.
    pub fn with_one_line_fix(mut self, is_one_line: bool) -> Self {
        self.is_one_line_fix = is_one_line;
        self
    }
}

/// Comprehensive staging evaluation report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagingReport {
    pub verdict: StagingVerdict,
    pub is_beta_staging_required: bool,
    pub is_beta_staging_recommended: bool,
    pub deciding_factors: Vec<StagingDecidingFactor>,
    pub guidance: String,
}

impl StagingReport {
    /// Checks if staging on `beta` is either required or recommended.
    pub fn is_staging_needed(&self) -> bool {
        self.is_beta_staging_required || self.is_beta_staging_recommended
    }

    /// Whether maintainer confirmation is needed prior to merging.
    pub fn requires_maintainer_confirmation(&self) -> bool {
        self.verdict.is_required() || self.verdict.is_recommended()
    }

    /// Recommended branch target for initial implementation/verification.
    pub fn recommended_branch_target(&self) -> &'static str {
        match self.verdict {
            StagingVerdict::Required | StagingVerdict::Recommended => "origin/beta",
            StagingVerdict::Unnecessary => "origin/main",
        }
    }
}

/// Evaluates whether a changeset requires or recommends staging on `beta` branch.
pub fn evaluate_staging_protocol(summary: &ChangeSetSummary) -> StagingReport {
    if is_exempt_from_staging(summary) {
        return build_exempt_report(summary);
    }

    let lines_changed = summary.total_lines_changed();
    if is_staging_required(summary, lines_changed) {
        build_required_report(summary, lines_changed)
    } else if is_staging_recommended(summary, lines_changed) {
        build_recommended_report(summary, lines_changed)
    } else {
        build_unnecessary_report()
    }
}

fn is_exempt_from_staging(summary: &ChangeSetSummary) -> bool {
    summary.is_docs_only || summary.is_one_line_fix
}

fn is_staging_required(summary: &ChangeSetSummary, lines_changed: usize) -> bool {
    summary.has_schema_migration
        || summary.has_public_surface_change
        || summary.has_feature_flag
        || summary.has_release_machinery_change
        || lines_changed > 500
}

fn is_staging_recommended(summary: &ChangeSetSummary, lines_changed: usize) -> bool {
    summary.files_changed_count > 3 || lines_changed > 100
}

fn build_exempt_report(summary: &ChangeSetSummary) -> StagingReport {
    let mut factors = Vec::new();
    if summary.is_docs_only {
        factors.push(StagingDecidingFactor::DocumentationOnly);
    }
    if summary.is_one_line_fix {
        factors.push(StagingDecidingFactor::TrivialOneLineFix);
    }

    StagingReport {
        verdict: StagingVerdict::Unnecessary,
        is_beta_staging_required: false,
        is_beta_staging_recommended: false,
        deciding_factors: factors,
        guidance: "Changeset qualifies as documentation-only or a trivial one-line fix. Direct base branch merge is permitted without beta staging.".to_string(),
    }
}

fn build_required_report(summary: &ChangeSetSummary, lines_changed: usize) -> StagingReport {
    let mut factors = Vec::new();

    if summary.has_schema_migration {
        factors.push(StagingDecidingFactor::SchemaMigrationDetected);
    }
    if summary.has_public_surface_change {
        let surface = summary.surface_type.clone().unwrap_or_else(|| "Public API/IPC".to_string());
        factors.push(StagingDecidingFactor::PublicSurfaceModified { surface_type: surface });
    }
    if summary.has_feature_flag {
        let flag = summary.flag_name.clone().unwrap_or_else(|| "feature_flag".to_string());
        factors.push(StagingDecidingFactor::FeatureFlagIntroduced { flag_name: flag });
    }
    if summary.has_release_machinery_change {
        factors.push(StagingDecidingFactor::ReleaseMachineryModified);
    }
    if lines_changed > 500 {
        factors.push(StagingDecidingFactor::NonTrivialCodebaseRefactor { lines_changed });
    }
    if summary.files_changed_count > 3 {
        factors.push(StagingDecidingFactor::MultiModuleChangeset {
            modules_count: summary.files_changed_count,
        });
    }

    StagingReport {
        verdict: StagingVerdict::Required,
        is_beta_staging_required: true,
        is_beta_staging_recommended: false,
        deciding_factors: factors,
        guidance: "High-impact changes detected. Maintainer Staging Protocol requires implementing and verifying on `beta` branch first before merging into `main`.".to_string(),
    }
}

fn build_recommended_report(summary: &ChangeSetSummary, lines_changed: usize) -> StagingReport {
    let mut factors = Vec::new();

    if summary.files_changed_count > 3 {
        factors.push(StagingDecidingFactor::MultiModuleChangeset {
            modules_count: summary.files_changed_count,
        });
    }
    if lines_changed > 100 {
        factors.push(StagingDecidingFactor::NonTrivialCodebaseRefactor { lines_changed });
    }

    StagingReport {
        verdict: StagingVerdict::Recommended,
        is_beta_staging_required: false,
        is_beta_staging_recommended: true,
        deciding_factors: factors,
        guidance: "Non-trivial changeset detected. Staging on `beta` branch is strongly recommended to validate stability prior to merging into `main`.".to_string(),
    }
}

fn build_unnecessary_report() -> StagingReport {
    StagingReport {
        verdict: StagingVerdict::Unnecessary,
        is_beta_staging_required: false,
        is_beta_staging_recommended: false,
        deciding_factors: Vec::new(),
        guidance: "Low-risk localized changeset. Direct merge without beta staging is permitted.".to_string(),
    }
}
