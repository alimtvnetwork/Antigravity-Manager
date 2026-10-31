//! Single Problem Scope & Pull Request Cohesion Verifier.
//!
//! Enforces PR Scope & Grouping invariants per AGENTS.md:
//! - "Single Problem Scope: A PR represents a cohesive collection of fixes or features
//!    dedicated to a single problem class. Keep unrelated concerns (such as governance,
//!    release tooling, or documentation) in isolated PRs."
//! - "Self-Contained & Individually Revertable: A PR may contain multiple commits,
//!    but each commit must represent an independent, self-contained functional unit
//!    that is individually revertable, avoiding messy or tangled changesets."
//! - "Local Convergence & Final-State Commits: Commit freely during local debugging
//!    on development branches; however, before opening or merging a PR, audit and
//!    consolidate scattered iterative attempts into clean, high-quality units."

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Categories of changes across the repository to enforce concern isolation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangesetCategory {
    /// Core backend pipeline, proxy protocol adapters, engine, streaming, thinking block filtering.
    CorePipeline,
    /// Frontend user interface components, views, React stores, styling, Tailwind capsules.
    UIComponents,
    /// Governance policies, coding guidelines, AGENTS.md, specifications, AI memory.
    Governance,
    /// User and developer documentation, READMEs, architecture guides.
    Documentation,
    /// Release gate, version bump tooling, CI/CD automation workflows, build manifests.
    ReleaseTooling,
}

impl ChangesetCategory {
    /// Returns the canonical string representation of the category.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CorePipeline => "CorePipeline",
            Self::UIComponents => "UIComponents",
            Self::Governance => "Governance",
            Self::Documentation => "Documentation",
            Self::ReleaseTooling => "ReleaseTooling",
        }
    }

    /// Returns a descriptive explanation of the concerns encompassed by this category.
    pub fn description(&self) -> &'static str {
        match self {
            Self::CorePipeline => {
                "Core proxy backend, protocol adapters, engine, and streaming pipeline"
            }
            Self::UIComponents => "Frontend UI components, React stores, views, and styling",
            Self::Governance => "Governance rules, AGENTS.md, specifications, and AI memory",
            Self::Documentation => "Documentation, README files, guides, and user manuals",
            Self::ReleaseTooling => "Release gate validators, bump scripts, and CI/CD workflows",
        }
    }

    /// Returns true if AGENTS.md mandates keeping this concern in isolated PRs.
    pub fn is_isolated_concern(&self) -> bool {
        matches!(
            self,
            Self::Governance | Self::ReleaseTooling | Self::Documentation
        )
    }

    /// Determines whether this category can be mixed with another category in a single PR.
    pub fn can_mix_with(&self, other: ChangesetCategory) -> bool {
        if *self == other {
            return true;
        }

        // Functional code (CorePipeline and UIComponents) can co-exist in full-stack feature PRs,
        // although isolated single-tier PRs are preferred.
        if (*self == Self::CorePipeline && other == Self::UIComponents)
            || (*self == Self::UIComponents && other == Self::CorePipeline)
        {
            return true;
        }

        // Governance and ReleaseTooling MUST NEVER be mixed with functional application code.
        false
    }
}

impl fmt::Display for ChangesetCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents an audited file modification in a changeset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileChangeset {
    pub path: String,
    pub category: ChangesetCategory,
    pub lines_added: usize,
    pub lines_deleted: usize,
}

/// Metadata describing a commit under review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitMetadata {
    pub hash: String,
    pub message: String,
    pub files_changed: Vec<String>,
    pub author: Option<String>,
}

/// Revertability assessment for a single commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitRevertabilityReport {
    pub commit_hash: String,
    pub is_revertable: bool,
    pub touched_categories: Vec<ChangesetCategory>,
    pub violations: Vec<String>,
    pub rationale: String,
}

/// Details of an identified mixed concern violation across a PR changeset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MixedConcernViolation {
    pub isolated_category: ChangesetCategory,
    pub conflicting_category: ChangesetCategory,
    pub files: Vec<String>,
    pub message: String,
}

/// Comprehensive audit result for a pull request's scope cohesion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrScopeAudit {
    /// True if the PR adheres to the single problem scope invariant.
    pub is_single_problem_scope: bool,
    /// Primary dominant category of the PR changeset.
    pub primary_category: Option<ChangesetCategory>,
    /// Breakdown of files per category: `(category, file_count)`.
    pub category_distribution: Vec<(ChangesetCategory, usize)>,
    /// Cohesion score from 0.0 (completely fragmented) to 1.0 (perfectly cohesive).
    pub cohesion_score: f64,
    /// Any mixed concern violations detected.
    pub mixed_concerns: Vec<MixedConcernViolation>,
    /// Per-commit revertability verification reports.
    pub revertability_reports: Vec<CommitRevertabilityReport>,
    /// Actionable human-readable summary.
    pub summary: String,
}

/// Classifies a repository-relative file path into its appropriate ChangesetCategory.
pub fn classify_file_path(path: &str) -> ChangesetCategory {
    let clean = path.trim().replace('\\', "/");
    let clean = clean.strip_prefix("./").unwrap_or(&clean);
    let lower = clean.to_ascii_lowercase();

    // 1. Governance: AGENTS.md, specs, memory, prompts, guidelines, legal policies
    if lower == "agents.md"
        || lower.starts_with(".agents/")
        || lower.starts_with(".ai-memory/")
        || lower.starts_with("02-spec/")
        || lower.starts_with("01-prompts/")
        || lower == "license"
        || lower.starts_with("license.")
        || lower == "security.md"
        || lower == "contributing.md"
        || lower == "code_of_conduct.md"
        || lower.starts_with(".github/pull_request_template")
        || lower.starts_with(".github/issue_template/")
    {
        return ChangesetCategory::Governance;
    }

    // 2. ReleaseTooling: CI workflows, bump scripts, release gate, build configs
    if lower.starts_with(".github/workflows/")
        || lower.starts_with("scripts/")
        || lower.starts_with("src-tauri/src/modules/release_gate/")
        || lower == "package.json"
        || lower == "package-lock.json"
        || lower == "pnpm-lock.yaml"
        || lower == "yarn.lock"
        || lower == "cargo.toml"
        || lower == "cargo.lock"
        || lower == "dockerfile"
        || lower.starts_with("docker-compose")
        || lower.starts_with("bump")
        || lower.starts_with("release")
    {
        return ChangesetCategory::ReleaseTooling;
    }

    // 3. Documentation: root readmes, docs folder, changelog files
    if lower == "readme.md"
        || lower == "readme_en.md"
        || lower.starts_with("docs/")
        || lower.starts_with("doc/")
        || lower == "changelog.md"
        || lower == "changelog_en.md"
        || lower.ends_with(".md")
        || lower.ends_with(".txt")
    {
        return ChangesetCategory::Documentation;
    }

    // 4. UIComponents: React views, components, stores, styles, markup
    if lower.starts_with("src/components/")
        || lower.starts_with("src/views/")
        || lower.starts_with("src/stores/")
        || lower.starts_with("src/assets/")
        || lower.starts_with("src/styles/")
        || lower.ends_with(".tsx")
        || lower.ends_with(".jsx")
        || lower.ends_with(".css")
        || lower.ends_with(".scss")
        || lower.ends_with(".sass")
        || lower.ends_with(".less")
        || lower.ends_with(".html")
        || lower.ends_with(".vue")
        || lower.ends_with(".svelte")
        || lower.starts_with("src/")
    {
        return ChangesetCategory::UIComponents;
    }

    // 5. CorePipeline: backend proxy, protocol adapters, engine, and backend modules
    ChangesetCategory::CorePipeline
}

/// Classifies a list of repository file paths into FileChangeset items.
pub fn classify_changeset(files: &[&str]) -> Vec<FileChangeset> {
    files
        .iter()
        .map(|path| FileChangeset {
            path: path.to_string(),
            category: classify_file_path(path),
            lines_added: 0,
            lines_deleted: 0,
        })
        .collect()
}

/// Validates whether an individual commit represents a self-contained, individually revertable unit.
pub fn validate_commit_revertability(commit: &CommitMetadata) -> CommitRevertabilityReport {
    let mut violations = Vec::new();

    if commit.files_changed.is_empty() {
        violations.push("Commit modifies zero files".to_string());
        return CommitRevertabilityReport {
            commit_hash: commit.hash.clone(),
            is_revertable: false,
            touched_categories: Vec::new(),
            violations,
            rationale: "Empty commit cannot be safely reverted as an independent unit".to_string(),
        };
    }

    // Check commit message for lack of rationale or noisy trial-and-error markers
    let msg_lower = commit.message.trim().to_ascii_lowercase();
    let is_noisy_message = msg_lower.is_empty()
        || msg_lower == "wip"
        || msg_lower == "wip2"
        || msg_lower == "fix"
        || msg_lower == "temp"
        || msg_lower == "checkpoint"
        || msg_lower == "test"
        || msg_lower == "fixed typo"
        || msg_lower.starts_with("try fix");

    if is_noisy_message {
        violations.push(format!(
            "Commit message '{}' lacks self-contained rationale for independent reversion",
            commit.message.trim()
        ));
    }

    // Inspect touched categories within this single commit
    let mut categories_set = HashSet::new();
    for file in &commit.files_changed {
        categories_set.insert(classify_file_path(file));
    }

    let touched_categories: Vec<ChangesetCategory> = categories_set.into_iter().collect();

    // Check for tangled concerns inside a single commit
    let has_governance = touched_categories.contains(&ChangesetCategory::Governance);
    let has_release_tooling = touched_categories.contains(&ChangesetCategory::ReleaseTooling);
    let has_pipeline = touched_categories.contains(&ChangesetCategory::CorePipeline);
    let has_ui = touched_categories.contains(&ChangesetCategory::UIComponents);

    if has_governance && (has_pipeline || has_ui || has_release_tooling) {
        violations.push(
            "Tangled changeset: Commit mixes Governance with application code or tooling, preventing independent reversion"
                .to_string(),
        );
    }

    if has_release_tooling && (has_pipeline || has_ui) {
        violations.push(
            "Tangled changeset: Commit mixes ReleaseTooling with application code, preventing independent reversion"
                .to_string(),
        );
    }

    if touched_categories.len() >= 3 {
        violations.push(format!(
            "Tangled changeset: Commit spans {} disjoint categories simultaneously",
            touched_categories.len()
        ));
    }

    let is_revertable = violations.is_empty();
    let rationale = if is_revertable {
        "Commit represents an independent, self-contained functional unit that is individually revertable".to_string()
    } else {
        format!(
            "Commit fails revertability check: {}",
            violations.join("; ")
        )
    };

    CommitRevertabilityReport {
        commit_hash: commit.hash.clone(),
        is_revertable,
        touched_categories,
        violations,
        rationale,
    }
}

/// Identifies mixed concern violations across an entire changeset.
pub fn check_mixed_concerns(files: &[FileChangeset]) -> Vec<MixedConcernViolation> {
    let mut violations = Vec::new();
    let mut category_files: HashMap<ChangesetCategory, Vec<String>> = HashMap::new();

    for file in files {
        category_files
            .entry(file.category)
            .or_default()
            .push(file.path.clone());
    }

    let categories: Vec<ChangesetCategory> = category_files.keys().copied().collect();

    for i in 0..categories.len() {
        for j in (i + 1)..categories.len() {
            let cat_a = categories[i];
            let cat_b = categories[j];

            if !cat_a.can_mix_with(cat_b) {
                let (isolated, conflicting) = if cat_a.is_isolated_concern() {
                    (cat_a, cat_b)
                } else {
                    (cat_b, cat_a)
                };

                let files_involved = category_files.get(&isolated).cloned().unwrap_or_default();

                violations.push(MixedConcernViolation {
                    isolated_category: isolated,
                    conflicting_category: conflicting,
                    files: files_involved,
                    message: format!(
                        "AGENTS.md Violation: {} concerns must remain in isolated PRs, not mixed with {} changes",
                        isolated.as_str(),
                        conflicting.as_str()
                    ),
                });
            }
        }
    }

    violations
}

/// Calculates the cohesion score of a changeset from 0.0 to 1.0.
pub fn calculate_cohesion_score(files: &[FileChangeset]) -> (Option<ChangesetCategory>, f64) {
    if files.is_empty() {
        return (None, 1.0);
    }

    let mut counts: HashMap<ChangesetCategory, usize> = HashMap::new();
    for file in files {
        *counts.entry(file.category).or_insert(0) += 1;
    }

    let total_files = files.len() as f64;
    let mut primary_category = None;
    let mut max_count = 0;

    for (cat, count) in &counts {
        if *count > max_count {
            max_count = *count;
            primary_category = Some(*cat);
        }
    }

    let base_score = (max_count as f64) / total_files;

    // Apply strict penalties for mixed isolated concerns per AGENTS.md
    let mixed_violations = check_mixed_concerns(files);
    let penalty_factor = 0.5_f64.powi(mixed_violations.len() as i32);
    let final_score = (base_score * penalty_factor).clamp(0.0, 1.0);

    (primary_category, final_score)
}

/// Performs a comprehensive audit of a PR's scope cohesion and commit revertability.
pub fn audit_pr_scope(commits: &[CommitMetadata], files: &[&str]) -> PrScopeAudit {
    let file_changesets = classify_changeset(files);
    let (primary_category, cohesion_score) = calculate_cohesion_score(&file_changesets);
    let mixed_concerns = check_mixed_concerns(&file_changesets);

    let mut distribution_map: HashMap<ChangesetCategory, usize> = HashMap::new();
    for fc in &file_changesets {
        *distribution_map.entry(fc.category).or_insert(0) += 1;
    }

    let mut category_distribution: Vec<(ChangesetCategory, usize)> =
        distribution_map.into_iter().collect();
    category_distribution.sort_by(|a, b| b.1.cmp(&a.1));

    let revertability_reports: Vec<CommitRevertabilityReport> =
        commits.iter().map(validate_commit_revertability).collect();

    let all_commits_revertable =
        revertability_reports.is_empty() || revertability_reports.iter().all(|r| r.is_revertable);

    let is_single_problem_scope =
        cohesion_score >= 0.75 && mixed_concerns.is_empty() && all_commits_revertable;

    let summary = if is_single_problem_scope {
        format!(
            "PR adheres to Single Problem Scope (cohesion score {:.2}, dominant concern: {}). All commits are individually revertable.",
            cohesion_score,
            primary_category.map(|c| c.as_str()).unwrap_or("None")
        )
    } else {
        let mut reasons = Vec::new();
        if !mixed_concerns.is_empty() {
            reasons.push(format!(
                "contains {} mixed concern violation(s)",
                mixed_concerns.len()
            ));
        }
        if cohesion_score < 0.75 {
            reasons.push(format!(
                "cohesion score {:.2} below threshold 0.75",
                cohesion_score
            ));
        }
        if !all_commits_revertable {
            reasons.push("one or more commits are not individually revertable".to_string());
        }

        format!(
            "PR violates AGENTS.md Single Problem Scope ({}): split unrelated concerns into isolated PRs.",
            reasons.join(", ")
        )
    };

    PrScopeAudit {
        is_single_problem_scope,
        primary_category,
        category_distribution,
        cohesion_score,
        mixed_concerns,
        revertability_reports,
        summary,
    }
}
