//! Release Channel & Branch Separation Gate.
//! Enforces:
//! - Stable releases (vX.Y.Z) must deploy exclusively from the `main` branch.
//! - Preview/Beta releases (vX.Y.Z-beta.N) must deploy exclusively from the `beta` branch.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseChannel {
    Stable,
    Preview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelValidationResult {
    pub is_valid: bool,
    pub channel: ReleaseChannel,
    pub error_message: Option<String>,
}

/// Evaluates tag and branch compliance according to release guidelines.
pub fn validate_release_channel(tag: &str, branch: &str) -> ChannelValidationResult {
    let tag = tag.trim();
    let branch = branch.trim();

    let is_preview = tag.contains("-beta") || tag.contains("-alpha") || tag.contains("-rc");
    let channel = if is_preview {
        ReleaseChannel::Preview
    } else {
        ReleaseChannel::Stable
    };

    match channel {
        ReleaseChannel::Stable => {
            if branch == "main" || branch == "refs/heads/main" {
                ChannelValidationResult {
                    is_valid: true,
                    channel,
                    error_message: None,
                }
            } else {
                ChannelValidationResult {
                    is_valid: false,
                    channel,
                    error_message: Some(format!(
                        "Stable release tag '{tag}' is forbidden on branch '{branch}'. Must be tagged on 'main'."
                    )),
                }
            }
        }
        ReleaseChannel::Preview => {
            if branch == "beta" || branch == "refs/heads/beta" {
                ChannelValidationResult {
                    is_valid: true,
                    channel,
                    error_message: None,
                }
            } else {
                ChannelValidationResult {
                    is_valid: false,
                    channel,
                    error_message: Some(format!(
                        "Preview release tag '{tag}' is forbidden on branch '{branch}'. Must be tagged on 'beta'."
                    )),
                }
            }
        }
    }
}
