//! Single Problem Scope & PR Problem Classification Linter.
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
//! - "Review & Template Alignment: Route every PR through peer review and complete
//!    .github/pull_request_template.md (problem classification, behavior alterations,
//!    unverified paths, and rollback strategy)."

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Functional architecture domains across the repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FunctionalDomain {
    /// Proxy server, protocol conversion pipelines, and core handlers.
    GatewayProxy,
    /// Release gate validators, preflight checks, CI/CD, and bump scripts.
    ReleaseTooling,
    /// Multi-instance management, lifecycle daemons, and process dispatch.
    InstanceManagement,
    /// SQLite split databases, Diesel schemas, and ORM migrations.
    DatabaseOrm,
    /// Frontend React components, Tailwind styling, capsules, and views.
    UiComponents,
    /// Developer documentation, architecture specs, and Markdown guides.
    Documentation,
}

impl FunctionalDomain {
    /// Returns the canonical string identifier for this domain.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GatewayProxy => "GatewayProxy",
            Self::ReleaseTooling => "ReleaseTooling",
            Self::InstanceManagement => "InstanceManagement",
            Self::DatabaseOrm => "DatabaseOrm",
            Self::UiComponents => "UiComponents",
            Self::Documentation => "Documentation",
        }
    }

    /// Returns true if this domain represents documentation or specs.
    pub fn is_documentation(&self) -> bool {
        matches!(self, Self::Documentation)
    }
}

impl fmt::Display for FunctionalDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Violations identified during PR problem classification and scope auditing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeEntanglementViolation {
    /// PR entangles more than two distinct non-documentation domains.
    UnrelatedDomainsMixed {
        domains: Vec<FunctionalDomain>,
        file_count: usize,
    },
    /// PR body lacks the required problem classification section header.
    MissingProblemClassificationHeader,
}

impl fmt::Display for ScopeEntanglementViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnrelatedDomainsMixed { domains, file_count } => {
                let domain_list = domains
                    .iter()
                    .map(|d| d.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(
                    f,
                    "PR modifies {} unrelated domains across {} file(s): [{}]",
                    domains.len(),
                    file_count,
                    domain_list
                )
            }
            Self::MissingProblemClassificationHeader => {
                write!(
                    f,
                    "PR description is missing mandatory 'Problem Classification' header"
                )
            }
        }
    }
}

impl std::error::Error for ScopeEntanglementViolation {}

/// Comprehensive audit report detailing PR scope cohesion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeLinterReport {
    pub is_cohesive: bool,
    pub primary_domain: Option<FunctionalDomain>,
    pub touched_domains: Vec<FunctionalDomain>,
    pub violations: Vec<ScopeEntanglementViolation>,
}

impl ScopeLinterReport {
    /// Returns true if any scope violations were found.
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Generates a human-readable summary of the PR scope audit.
    pub fn summary(&self) -> String {
        if self.is_cohesive {
            let primary = self
                .primary_domain
                .map(|d| d.as_str())
                .unwrap_or("None");
            format!(
                "PR scope cohesive: primary domain '{}', {} touched domain(s)",
                primary,
                self.touched_domains.len()
            )
        } else {
            format!(
                "PR scope entangled: {} violation(s) identified",
                self.violations.len()
            )
        }
    }
}

impl fmt::Display for ScopeLinterReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())?;
        for v in &self.violations {
            write!(f, "\n  - {}", v)?;
        }
        Ok(())
    }
}

/// Classifies a repository-relative file path into its functional domain.
pub fn classify_file_domain(path: &str) -> FunctionalDomain {
    let normalized = path.trim().replace('\\', "/");
    let lower = normalized.to_ascii_lowercase();

    if lower.starts_with("docs/") || lower.contains("/docs/") || lower.ends_with(".md") {
        FunctionalDomain::Documentation
    } else if lower.starts_with("src-tauri/src/proxy/")
        || lower.contains("/proxy/")
        || lower.starts_with("proxy/")
    {
        FunctionalDomain::GatewayProxy
    } else if lower.starts_with("src-tauri/src/modules/release_gate/")
        || lower.contains("/release_gate/")
        || lower.starts_with(".github/")
        || lower.contains("/.github/")
    {
        FunctionalDomain::ReleaseTooling
    } else if lower.starts_with("src-tauri/src/modules/instance/")
        || lower.contains("/instance/")
        || lower.starts_with("instance/")
    {
        FunctionalDomain::InstanceManagement
    } else if lower.starts_with("src-tauri/src/modules/db_orm/")
        || lower.contains("/db_orm/")
        || lower.starts_with("db_orm/")
    {
        FunctionalDomain::DatabaseOrm
    } else if lower.starts_with("src/")
        || lower.contains("/components/")
        || lower.starts_with("components/")
        || lower.ends_with(".tsx")
        || lower.ends_with(".jsx")
        || lower.ends_with(".css")
    {
        FunctionalDomain::UiComponents
    } else {
        FunctionalDomain::GatewayProxy
    }
}

/// Collects unique domains preserving discovery order.
fn collect_unique_domains(file_paths: &[String]) -> Vec<FunctionalDomain> {
    let mut domains = Vec::new();
    for path in file_paths {
        let domain = classify_file_domain(path);
        if !domains.contains(&domain) {
            domains.push(domain);
        }
    }
    domains
}

/// Determines the primary dominant domain from the provided file paths.
fn determine_primary_domain(file_paths: &[String]) -> Option<FunctionalDomain> {
    if file_paths.is_empty() {
        return None;
    }
    let mut counts: HashMap<FunctionalDomain, usize> = HashMap::new();
    for path in file_paths {
        let domain = classify_file_domain(path);
        *counts.entry(domain).or_insert(0) += 1;
    }

    let non_doc_max = counts
        .iter()
        .filter(|(domain, _)| !domain.is_documentation())
        .max_by_key(|(_, count)| *count)
        .map(|(domain, _)| *domain);

    non_doc_max.or_else(|| {
        counts
            .into_iter()
            .max_by_key(|(_, count)| count)
            .map(|(domain, _)| domain)
    })
}

/// Checks if touched domains violate single problem scope cohesion.
fn check_unrelated_domains(
    touched_domains: &[FunctionalDomain],
    file_count: usize,
) -> Option<ScopeEntanglementViolation> {
    let non_doc_domains: Vec<FunctionalDomain> = touched_domains
        .iter()
        .copied()
        .filter(|d| !d.is_documentation())
        .collect();

    if non_doc_domains.len() > 2 {
        Some(ScopeEntanglementViolation::UnrelatedDomainsMixed {
            domains: non_doc_domains,
            file_count,
        })
    } else {
        None
    }
}

/// Checks for mandatory Problem Classification header in the PR description.
fn check_classification_header(pr_body: &str) -> Option<ScopeEntanglementViolation> {
    let has_header = pr_body.contains("Problem Classification")
        || pr_body.contains("problem classification");
    if !has_header {
        Some(ScopeEntanglementViolation::MissingProblemClassificationHeader)
    } else {
        None
    }
}

/// Audits a PR changeset and description for single problem scope compliance.
pub fn audit_pr_scope(file_paths: &[String], pr_body: &str) -> ScopeLinterReport {
    let touched_domains = collect_unique_domains(file_paths);
    let primary_domain = determine_primary_domain(file_paths);
    let mut violations = Vec::new();

    if let Some(violation) = check_unrelated_domains(&touched_domains, file_paths.len()) {
        violations.push(violation);
    }
    if let Some(violation) = check_classification_header(pr_body) {
        violations.push(violation);
    }

    let is_cohesive = violations.is_empty();
    ScopeLinterReport {
        is_cohesive,
        primary_domain,
        touched_domains,
        violations,
    }
}
