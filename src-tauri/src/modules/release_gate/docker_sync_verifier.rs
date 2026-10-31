//! Production Docker & GitHub Tag Sync Discipline Verifier.
//!
//! Enforces release channel discipline per AGENTS.md:
//! - Stable Releases: Exclusively on `main`. Deploys official production packages,
//!   updates Docker/GitHub `latest` tags, and services automatic update channels.
//! - Preview Releases: Exclusively on `beta`. Independently builds and publishes
//!   pre-releases (`makeLatest: false`, `prerelease: true`) without touching production
//!   update channels.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockerSyncEvaluation {
    pub is_valid: bool,
    pub is_production_channel: bool,
    pub latest_tag_allowed: bool,
    pub reason: String,
}

/// Evaluates whether a release dispatch configuration satisfies channel isolation rules.
pub fn evaluate_docker_tag_sync(
    branch: &str,
    tag: &str,
    make_latest: bool,
    prerelease: bool,
) -> DockerSyncEvaluation {
    let clean_branch = branch.trim();
    let clean_tag = tag.trim();

    let is_main = clean_branch == "main";
    let is_beta = clean_branch == "beta";

    if is_main {
        if prerelease {
            return DockerSyncEvaluation {
                is_valid: false,
                is_production_channel: true,
                latest_tag_allowed: false,
                reason: "Stable release on 'main' cannot be marked as a prerelease".to_string(),
            };
        }
        if clean_tag.contains("-beta.") || clean_tag.contains("-rc.") {
            return DockerSyncEvaluation {
                is_valid: false,
                is_production_channel: true,
                latest_tag_allowed: false,
                reason: format!(
                    "Prerelease tag '{}' is strictly forbidden on 'main' branch",
                    clean_tag
                ),
            };
        }
        if !make_latest {
            return DockerSyncEvaluation {
                is_valid: false,
                is_production_channel: true,
                latest_tag_allowed: true,
                reason: "Production release on 'main' must update the 'latest' tag".to_string(),
            };
        }
        DockerSyncEvaluation {
            is_valid: true,
            is_production_channel: true,
            latest_tag_allowed: true,
            reason: "Valid production release satisfying Docker/GitHub latest tag sync".to_string(),
        }
    } else if is_beta {
        if make_latest {
            return DockerSyncEvaluation {
                is_valid: false,
                is_production_channel: false,
                latest_tag_allowed: false,
                reason: "Preview release on 'beta' must NEVER update production 'latest' tag (makeLatest must be false)".to_string(),
            };
        }
        if !prerelease {
            return DockerSyncEvaluation {
                is_valid: false,
                is_production_channel: false,
                latest_tag_allowed: false,
                reason: "Preview release on 'beta' must have prerelease: true".to_string(),
            };
        }
        if !clean_tag.contains("-beta.") && !clean_tag.contains("-rc.") {
            return DockerSyncEvaluation {
                is_valid: false,
                is_production_channel: false,
                latest_tag_allowed: false,
                reason: format!(
                    "Preview release on 'beta' requires a prerelease tag suffix, got '{}'",
                    clean_tag
                ),
            };
        }
        DockerSyncEvaluation {
            is_valid: true,
            is_production_channel: false,
            latest_tag_allowed: false,
            reason: "Valid preview release isolating production channels".to_string(),
        }
    } else {
        DockerSyncEvaluation {
            is_valid: false,
            is_production_channel: false,
            latest_tag_allowed: false,
            reason: format!(
                "Unauthorized release branch '{}'; releases must originate from 'main' or 'beta'",
                clean_branch
            ),
        }
    }
}
