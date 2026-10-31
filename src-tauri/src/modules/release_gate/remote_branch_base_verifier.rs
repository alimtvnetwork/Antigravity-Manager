//! Remote Branch Base Verifier.
//!
//! Enforces remote base hygiene per AGENTS.md:
//! "Branch from remote bases (`origin/beta` for staged features, `origin/main` for direct hotfixes)
//! rather than local branches to prevent untracked ancestor commits."
//!
//! Invariants:
//! - Local branch bases (e.g. `main`, `beta`, `feature/local`, `HEAD`, `master`) are strictly rejected.
//! - Base ref MUST start with `origin/` or `upstream/` remote prefix (or `refs/remotes/origin/` / `refs/remotes/upstream/`).
//! - For `ReleaseChannel::StableMain`, base MUST be `origin/main` (or `upstream/main`).
//! - For `ReleaseChannel::PreviewBeta`, base MUST be `origin/beta` (or `upstream/beta`).

use serde::{Deserialize, Serialize};
use std::fmt;

/// Target release channel dictating the required remote branch base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReleaseChannel {
    /// Stable release channel deploying exclusively from `main`.
    StableMain,
    /// Preview/Beta release channel deploying exclusively from `beta`.
    PreviewBeta,
}

impl ReleaseChannel {
    /// Canonical remote base reference required for this channel.
    pub fn canonical_base(&self) -> &'static str {
        match self {
            Self::StableMain => "origin/main",
            Self::PreviewBeta => "origin/beta",
        }
    }

    /// Allowed base references (including primary origin and upstream remote).
    pub fn allowed_bases(&self) -> &'static [&'static str] {
        match self {
            Self::StableMain => &["origin/main", "upstream/main"],
            Self::PreviewBeta => &["origin/beta", "upstream/beta"],
        }
    }

    /// The targeted branch name on the remote.
    pub fn target_branch_name(&self) -> &'static str {
        match self {
            Self::StableMain => "main",
            Self::PreviewBeta => "beta",
        }
    }

    /// Human-readable channel name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::StableMain => "StableMain (main)",
            Self::PreviewBeta => "PreviewBeta (beta)",
        }
    }
}

/// Request parameters to verify a branch creation or git base reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchBaseVerification {
    pub branch_name: String,
    pub base_ref: String,
    pub channel: ReleaseChannel,
}

impl BranchBaseVerification {
    pub fn new(
        branch_name: impl Into<String>,
        base_ref: impl Into<String>,
        channel: ReleaseChannel,
    ) -> Self {
        Self {
            branch_name: branch_name.into(),
            base_ref: base_ref.into(),
            channel,
        }
    }
}

/// Verification report detailing branch base compliance against AGENTS.md rules.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BranchBaseReport {
    pub is_base_valid: bool,
    pub is_remote_tracking: bool,
    pub detected_remote: Option<String>,
    pub detected_branch: Option<String>,
    pub violations: Vec<String>,
}

/// Specific violations encountered when validating git branch bases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchBaseViolation {
    /// Branch name string is empty or contains only whitespace.
    EmptyBranchName,
    /// Base reference string is empty or contains only whitespace.
    EmptyBaseRef,
    /// Base reference is a local branch rather than a remote tracking branch.
    LocalBranchBaseForbidden { base_ref: String, guidance: String },
    /// Base reference does not have a recognized remote prefix (`origin/` or `upstream/`).
    MissingRemotePrefix { base_ref: String, guidance: String },
    /// Base reference does not match the mandatory base for the chosen release channel.
    ChannelBaseMismatch {
        channel: ReleaseChannel,
        base_ref: String,
        expected: String,
        guidance: String,
    },
    /// Branch name contains illegal git ref characters.
    InvalidBranchName { branch_name: String, reason: String },
}

impl fmt::Display for BranchBaseViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBranchName => write!(f, "Branch name cannot be empty"),
            Self::EmptyBaseRef => write!(f, "Base reference cannot be empty"),
            Self::LocalBranchBaseForbidden { base_ref, guidance } => {
                write!(
                    f,
                    "Local branch base '{base_ref}' is strictly forbidden to prevent untracked ancestor commits. {guidance}"
                )
            }
            Self::MissingRemotePrefix { base_ref, guidance } => {
                write!(
                    f,
                    "Base reference '{base_ref}' lacks a valid remote tracking prefix (expected 'origin/' or 'upstream/'). {guidance}"
                )
            }
            Self::ChannelBaseMismatch {
                channel,
                base_ref,
                expected,
                guidance,
            } => {
                write!(
                    f,
                    "Base ref '{base_ref}' violates channel {} rules. Expected base '{expected}'. {guidance}",
                    channel.display_name()
                )
            }
            Self::InvalidBranchName {
                branch_name,
                reason,
            } => {
                write!(f, "Invalid branch name '{branch_name}': {reason}")
            }
        }
    }
}

impl std::error::Error for BranchBaseViolation {}

/// Normalizes a git reference by stripping `refs/remotes/` or `refs/heads/`.
pub fn normalize_git_ref(raw_ref: &str) -> &str {
    let trimmed = raw_ref.trim();
    if let Some(stripped) = trimmed.strip_prefix("refs/remotes/") {
        stripped
    } else if let Some(stripped) = trimmed.strip_prefix("refs/heads/") {
        stripped
    } else {
        trimmed
    }
}

/// Parses the remote name and branch name from a normalized ref.
/// Recognizes `origin/<branch>` and `upstream/<branch>`.
pub fn parse_remote_and_branch(clean_ref: &str) -> (Option<String>, Option<String>, bool) {
    if let Some(branch) = clean_ref.strip_prefix("origin/") {
        (Some("origin".to_string()), Some(branch.to_string()), true)
    } else if let Some(branch) = clean_ref.strip_prefix("upstream/") {
        (Some("upstream".to_string()), Some(branch.to_string()), true)
    } else {
        let branch = clean_ref.rsplit('/').next().unwrap_or(clean_ref);
        (None, Some(branch.to_string()), false)
    }
}

/// Evaluates a [`BranchBaseVerification`] request and returns a [`BranchBaseReport`].
pub fn evaluate_branch_base(verification: &BranchBaseVerification) -> BranchBaseReport {
    let branch_name = verification.branch_name.trim();
    let base_ref = verification.base_ref.trim();
    let mut violations = Vec::new();

    if branch_name.is_empty() {
        violations.push("Branch name cannot be empty".to_string());
    } else if branch_name.contains(' ')
        || branch_name.contains("..")
        || branch_name.starts_with('-')
    {
        violations.push(format!(
            "Branch name '{branch_name}' contains illegal characters or formatting"
        ));
    }

    if base_ref.is_empty() {
        violations.push("Base reference cannot be empty".to_string());
        return BranchBaseReport {
            is_base_valid: false,
            is_remote_tracking: false,
            detected_remote: None,
            detected_branch: None,
            violations,
        };
    }

    let clean_ref = normalize_git_ref(base_ref);
    let (remote, branch, is_remote) = parse_remote_and_branch(clean_ref);

    let is_explicit_local = verification.base_ref.trim().starts_with("refs/heads/")
        || clean_ref == "main"
        || clean_ref == "beta"
        || clean_ref == "master"
        || clean_ref == "HEAD"
        || clean_ref.starts_with("feature/")
        || clean_ref.starts_with("feat/")
        || clean_ref.starts_with("hotfix/")
        || clean_ref.starts_with("fix/")
        || clean_ref.starts_with("patch/")
        || clean_ref.starts_with("dev")
        || !is_remote;

    if !is_remote || is_explicit_local {
        violations.push(format!(
            "Forbidden local branch base '{base_ref}'. Must branch from remote bases ('{}') to prevent untracked ancestor commits.",
            verification.channel.canonical_base()
        ));
    }

    if is_remote {
        let allowed = verification.channel.allowed_bases();
        if !allowed.contains(&clean_ref) {
            let guidance = match verification.channel {
                ReleaseChannel::StableMain => {
                    "Direct hotfixes for stable releases must base from origin/main (or upstream/main), not origin/beta."
                }
                ReleaseChannel::PreviewBeta => {
                    "Staged features for preview releases must base from origin/beta (or upstream/beta), not origin/main."
                }
            };
            violations.push(format!(
                "Channel base mismatch: base ref '{clean_ref}' is invalid for {}. Expected one of {:?}. {}",
                verification.channel.display_name(),
                allowed,
                guidance
            ));
        }
    }

    let is_base_valid = violations.is_empty();

    BranchBaseReport {
        is_base_valid,
        is_remote_tracking: is_remote,
        detected_remote: remote,
        detected_branch: branch,
        violations,
    }
}

/// Verifies branch base compliance per AGENTS.md, returning `Ok(report)` or `Err(violation)`.
pub fn verify_branch_base(
    verification: &BranchBaseVerification,
) -> Result<BranchBaseReport, BranchBaseViolation> {
    let branch_name = verification.branch_name.trim();
    let base_ref = verification.base_ref.trim();

    if branch_name.is_empty() {
        return Err(BranchBaseViolation::EmptyBranchName);
    }

    if branch_name.contains(' ') || branch_name.contains("..") || branch_name.starts_with('-') {
        return Err(BranchBaseViolation::InvalidBranchName {
            branch_name: branch_name.to_string(),
            reason: "Contains whitespace, double-dots, or starts with hyphen".to_string(),
        });
    }

    if base_ref.is_empty() {
        return Err(BranchBaseViolation::EmptyBaseRef);
    }

    let clean_ref = normalize_git_ref(base_ref);
    let (remote, branch, is_remote) = parse_remote_and_branch(clean_ref);

    if !is_remote {
        let is_known_local_name = clean_ref == "main"
            || clean_ref == "beta"
            || clean_ref == "master"
            || clean_ref == "HEAD"
            || clean_ref.starts_with("feature/")
            || clean_ref.starts_with("feat/")
            || clean_ref.starts_with("hotfix/")
            || clean_ref.starts_with("fix/")
            || verification.base_ref.trim().starts_with("refs/heads/");

        if is_known_local_name {
            return Err(BranchBaseViolation::LocalBranchBaseForbidden {
                base_ref: base_ref.to_string(),
                guidance: format!(
                    "Branch from remote tracking bases ('{}') rather than local branches to prevent untracked ancestor commits.",
                    verification.channel.canonical_base()
                ),
            });
        } else {
            return Err(BranchBaseViolation::MissingRemotePrefix {
                base_ref: base_ref.to_string(),
                guidance: format!(
                    "Base ref must start with 'origin/' or 'upstream/'. Canonical base: '{}'",
                    verification.channel.canonical_base()
                ),
            });
        }
    }

    let allowed = verification.channel.allowed_bases();
    if !allowed.contains(&clean_ref) {
        let guidance = match verification.channel {
            ReleaseChannel::StableMain => {
                "Direct hotfixes for stable releases must base from origin/main (or upstream/main), not origin/beta.".to_string()
            }
            ReleaseChannel::PreviewBeta => {
                "Staged features for preview releases must base from origin/beta (or upstream/beta), not origin/main.".to_string()
            }
        };

        return Err(BranchBaseViolation::ChannelBaseMismatch {
            channel: verification.channel,
            base_ref: base_ref.to_string(),
            expected: allowed.join(" or "),
            guidance,
        });
    }

    Ok(BranchBaseReport {
        is_base_valid: true,
        is_remote_tracking: true,
        detected_remote: remote,
        detected_branch: branch,
        violations: Vec::new(),
    })
}
