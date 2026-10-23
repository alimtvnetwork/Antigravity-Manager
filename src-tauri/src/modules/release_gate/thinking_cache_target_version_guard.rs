//! Thinking Cache Invalidation Target Version Guard (Imp-627).
//!
//! Enforces release gate compliance for thinking cache invalidation per AGENTS.md:
//! - Routine releases (no prompt): Keep `SUGGESTION_DELETE_THINKING_STORE = false`.
//! - Architecture / schema refactors (prompt users to clean once):
//!   1. Set `SUGGESTION_DELETE_THINKING_STORE = true`.
//!   2. Set `SUGGESTION_TARGET_VERSION = '<version>'` (e.g. `'4.8.2'`).
//!   3. On upgrade, users with existing cache get a one-time prompt;
//!      action state persists in `gui_config.json`.

use serde::{Deserialize, Serialize};

/// Type of release being evaluated against thinking cache policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseType {
    /// Routine release with no cache breaking schema or architecture alterations.
    RoutineRelease,
    /// Major or minor release involving breaking schema or architecture changes.
    SchemaOrArchitectureRefactor,
}

impl ReleaseType {
    /// Returns true if this is a routine release.
    pub fn is_routine(&self) -> bool {
        matches!(self, Self::RoutineRelease)
    }

    /// Returns true if this release involves schema or architecture refactoring.
    pub fn is_refactor(&self) -> bool {
        matches!(self, Self::SchemaOrArchitectureRefactor)
    }
}

/// Thinking cache configuration extracted from release sources.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThinkingCacheConfig {
    /// Indicates whether `SUGGESTION_DELETE_THINKING_STORE` is enabled.
    pub is_suggestion_delete_store_enabled: bool,
    /// Configured `SUGGESTION_TARGET_VERSION` string, if any.
    pub target_version: Option<String>,
    /// Target release package version (from package.json / Cargo.toml).
    pub package_version: String,
}

impl ThinkingCacheConfig {
    /// Creates a new thinking cache configuration.
    pub fn new(
        is_suggestion_delete_store_enabled: bool,
        target_version: Option<String>,
        package_version: impl Into<String>,
    ) -> Self {
        Self {
            is_suggestion_delete_store_enabled,
            target_version,
            package_version: package_version.into(),
        }
    }

    /// Creates a compliant routine release configuration.
    pub fn routine(package_version: impl Into<String>) -> Self {
        Self {
            is_suggestion_delete_store_enabled: false,
            target_version: None,
            package_version: package_version.into(),
        }
    }

    /// Creates a compliant architecture refactor release configuration.
    pub fn refactor(target_version: impl Into<String>, package_version: impl Into<String>) -> Self {
        let tv = target_version.into();
        Self {
            is_suggestion_delete_store_enabled: true,
            target_version: Some(tv),
            package_version: package_version.into(),
        }
    }

    /// Returns true if suggestion delete thinking store is enabled.
    pub fn is_prompt_enabled(&self) -> bool {
        self.is_suggestion_delete_store_enabled
    }
}

/// Specific violations detected during thinking cache invalidation audit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThinkingCacheGuardViolation {
    /// A routine release unexpectedly enables the one-time cache prompt.
    RoutineReleaseErroneouslyPrompts { target_version: Option<String> },
    /// An architecture/schema refactor release forgot to enable the prompt.
    ArchitectureRefactorMissingPrompt,
    /// Configured target version does not match the actual package release version.
    MismatchedTargetVersion {
        target_version: String,
        package_version: String,
    },
    /// The prompt is enabled, but the target version is empty or unspecified.
    EmptyTargetVersionOnEnabledPrompt,
}

impl ThinkingCacheGuardViolation {
    /// Produces a human-readable description of the violation.
    pub fn error_message(&self) -> String {
        match self {
            Self::RoutineReleaseErroneouslyPrompts { target_version } => format!(
                "Routine release has SUGGESTION_DELETE_THINKING_STORE = true (target_version: {:?}); routine releases must not prompt users.",
                target_version
            ),
            Self::ArchitectureRefactorMissingPrompt => {
                "Architecture/schema refactor release requires SUGGESTION_DELETE_THINKING_STORE = true with matching SUGGESTION_TARGET_VERSION.".to_string()
            }
            Self::MismatchedTargetVersion {
                target_version,
                package_version,
            } => format!(
                "SUGGESTION_TARGET_VERSION ('{}') does not match release package_version ('{}').",
                target_version, package_version
            ),
            Self::EmptyTargetVersionOnEnabledPrompt => {
                "SUGGESTION_DELETE_THINKING_STORE is true, but SUGGESTION_TARGET_VERSION is empty or missing.".to_string()
            }
        }
    }
}

/// Comprehensive report of the thinking cache configuration audit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThinkingCacheReport {
    /// Whether the configuration satisfies all release gate invariants.
    pub is_compliant: bool,
    /// List of violations encountered.
    pub violations: Vec<ThinkingCacheGuardViolation>,
    /// Actionable guidance for developers to remediate non-compliance.
    pub guidance: String,
}

impl ThinkingCacheReport {
    /// Returns true if no violations were detected.
    pub fn is_clean(&self) -> bool {
        self.is_compliant
    }
}

/// Normalizes version string by trimming whitespace and optional 'v' prefix.
fn normalize_version(raw_version: &str) -> &str {
    raw_version.trim().trim_start_matches('v')
}

/// Checks whether target version matches package version.
fn is_version_matching(target_version: &str, package_version: &str) -> bool {
    let clean_target = normalize_version(target_version);
    let clean_package = normalize_version(package_version);
    target_version.trim() == package_version.trim() || clean_target == clean_package
}

/// Validates target version field when cache prompt is enabled.
fn validate_enabled_target_version(
    config: &ThinkingCacheConfig,
    violations: &mut Vec<ThinkingCacheGuardViolation>,
) {
    match &config.target_version {
        None => {
            violations.push(ThinkingCacheGuardViolation::EmptyTargetVersionOnEnabledPrompt);
        }
        Some(tv) if tv.trim().is_empty() => {
            violations.push(ThinkingCacheGuardViolation::EmptyTargetVersionOnEnabledPrompt);
        }
        Some(tv) => {
            if !is_version_matching(tv, &config.package_version) {
                violations.push(ThinkingCacheGuardViolation::MismatchedTargetVersion {
                    target_version: tv.clone(),
                    package_version: config.package_version.clone(),
                });
            }
        }
    }
}

/// Generates actionable guidance from violations list.
fn generate_guidance(
    violations: &[ThinkingCacheGuardViolation],
    release_type: &ReleaseType,
) -> String {
    if violations.is_empty() {
        return "Thinking cache invalidation configuration is compliant with AGENTS.md release policy.".to_string();
    }

    let mut notes = Vec::new();
    for violation in violations {
        match violation {
            ThinkingCacheGuardViolation::RoutineReleaseErroneouslyPrompts { .. } => {
                notes.push("Set SUGGESTION_DELETE_THINKING_STORE = false in SuggestionDeleteThinkingModal.tsx for routine releases.");
            }
            ThinkingCacheGuardViolation::ArchitectureRefactorMissingPrompt => {
                notes.push("Set SUGGESTION_DELETE_THINKING_STORE = true and SUGGESTION_TARGET_VERSION to the new version in SuggestionDeleteThinkingModal.tsx.");
            }
            ThinkingCacheGuardViolation::MismatchedTargetVersion {
                target_version,
                package_version,
            } => {
                notes.push("Align SUGGESTION_TARGET_VERSION with package version.");
            }
            ThinkingCacheGuardViolation::EmptyTargetVersionOnEnabledPrompt => {
                notes.push("Specify non-empty SUGGESTION_TARGET_VERSION matching the current package version.");
            }
        }
    }
    notes.join(" ")
}

/// Evaluates thinking cache configuration against release gate policy.
pub fn evaluate_thinking_cache_configuration(
    config: &ThinkingCacheConfig,
    release_type: &ReleaseType,
) -> ThinkingCacheReport {
    let mut violations = Vec::new();

    match release_type {
        ReleaseType::RoutineRelease => {
            if config.is_suggestion_delete_store_enabled {
                violations.push(
                    ThinkingCacheGuardViolation::RoutineReleaseErroneouslyPrompts {
                        target_version: config.target_version.clone(),
                    },
                );
            }
        }
        ReleaseType::SchemaOrArchitectureRefactor => {
            if !config.is_suggestion_delete_store_enabled {
                violations.push(ThinkingCacheGuardViolation::ArchitectureRefactorMissingPrompt);
            }
        }
    }

    if config.is_suggestion_delete_store_enabled {
        validate_enabled_target_version(config, &mut violations);
    }

    let is_compliant = violations.is_empty();
    let guidance = generate_guidance(&violations, release_type);

    ThinkingCacheReport {
        is_compliant,
        violations,
        guidance,
    }
}
