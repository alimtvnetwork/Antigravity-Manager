//! Monotonic Semantic Version Bump Validator.
//!
//! Validates atomic SemVer progression (patch, minor, major, beta monotonic increments)
//! preventing version downgrades, duplicate releases, or tag format corruptions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ParsedSemVer {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub beta: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BumpType {
    Major,
    Minor,
    Patch,
    BetaIncrement,
    PrereleaseToStable,
    Identical,
    Downgrade,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonotonicEvaluation {
    pub is_valid: bool,
    pub bump_type: BumpType,
    pub current_version: String,
    pub next_version: String,
    pub reason: String,
}

impl ParsedSemVer {
    pub fn parse(raw: &str) -> Option<Self> {
        let s = raw.trim().trim_start_matches('v');
        let (core, beta_part) = if let Some(idx) = s.find("-beta.") {
            let (c, b) = s.split_at(idx);
            let num_str = b.trim_start_matches("-beta.");
            let num = num_str.parse::<u64>().ok()?;
            (c, Some(num))
        } else {
            (s, None)
        };

        let parts: Vec<&str> = core.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let major = parts[0].parse::<u64>().ok()?;
        let minor = parts[1].parse::<u64>().ok()?;
        let patch = parts[2].parse::<u64>().ok()?;

        Some(Self {
            major,
            minor,
            patch,
            beta: beta_part,
        })
    }
}

/// Evaluates the version transition from `current_raw` to `next_raw`.
pub fn validate_semver_transition(current_raw: &str, next_raw: &str) -> MonotonicEvaluation {
    let curr = match ParsedSemVer::parse(current_raw) {
        Some(v) => v,
        None => {
            return MonotonicEvaluation {
                is_valid: false,
                bump_type: BumpType::Invalid,
                current_version: current_raw.to_string(),
                next_version: next_raw.to_string(),
                reason: format!("Failed to parse current version '{}'", current_raw),
            }
        }
    };

    let next = match ParsedSemVer::parse(next_raw) {
        Some(v) => v,
        None => {
            return MonotonicEvaluation {
                is_valid: false,
                bump_type: BumpType::Invalid,
                current_version: current_raw.to_string(),
                next_version: next_raw.to_string(),
                reason: format!("Failed to parse next version '{}'", next_raw),
            }
        }
    };

    if curr == next {
        return MonotonicEvaluation {
            is_valid: false,
            bump_type: BumpType::Identical,
            current_version: current_raw.to_string(),
            next_version: next_raw.to_string(),
            reason: "Next version is identical to current version".to_string(),
        };
    }

    // Downgrade detection
    if next.major < curr.major
        || (next.major == curr.major && next.minor < curr.minor)
        || (next.major == curr.major && next.minor == curr.minor && next.patch < curr.patch)
    {
        return MonotonicEvaluation {
            is_valid: false,
            bump_type: BumpType::Downgrade,
            current_version: current_raw.to_string(),
            next_version: next_raw.to_string(),
            reason: format!("Version downgrade from {} to {}", current_raw, next_raw),
        };
    }

    // Beta transitions
    if curr.beta.is_some() && next.beta.is_none() {
        if next.major == curr.major && next.minor == curr.minor && next.patch == curr.patch {
            return MonotonicEvaluation {
                is_valid: true,
                bump_type: BumpType::PrereleaseToStable,
                current_version: current_raw.to_string(),
                next_version: next_raw.to_string(),
                reason: "Promoting prerelease beta to stable release".to_string(),
            };
        }
    }

    if let (Some(b_curr), Some(b_next)) = (curr.beta, next.beta) {
        if next.major == curr.major && next.minor == curr.minor && next.patch == curr.patch {
            if b_next > b_curr {
                return MonotonicEvaluation {
                    is_valid: true,
                    bump_type: BumpType::BetaIncrement,
                    current_version: current_raw.to_string(),
                    next_version: next_raw.to_string(),
                    reason: "Monotonic beta increment".to_string(),
                };
            } else {
                return MonotonicEvaluation {
                    is_valid: false,
                    bump_type: BumpType::Downgrade,
                    current_version: current_raw.to_string(),
                    next_version: next_raw.to_string(),
                    reason: "Beta version must increase monotonically".to_string(),
                };
            }
        }
    }

    if next.major > curr.major {
        return MonotonicEvaluation {
            is_valid: true,
            bump_type: BumpType::Major,
            current_version: current_raw.to_string(),
            next_version: next_raw.to_string(),
            reason: "Major version bump".to_string(),
        };
    }

    if next.minor > curr.minor {
        return MonotonicEvaluation {
            is_valid: true,
            bump_type: BumpType::Minor,
            current_version: current_raw.to_string(),
            next_version: next_raw.to_string(),
            reason: "Minor version bump".to_string(),
        };
    }

    if next.patch > curr.patch {
        return MonotonicEvaluation {
            is_valid: true,
            bump_type: BumpType::Patch,
            current_version: current_raw.to_string(),
            next_version: next_raw.to_string(),
            reason: "Patch version bump".to_string(),
        };
    }

    MonotonicEvaluation {
        is_valid: false,
        bump_type: BumpType::Downgrade,
        current_version: current_raw.to_string(),
        next_version: next_raw.to_string(),
        reason: "Version does not advance monotonically".to_string(),
    }
}
