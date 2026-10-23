//! Release Gate & Compliance Architecture.
//! Enforces version manifest synchronization, release channel separation,
//! author attribution compliance, repository-relative Git path hygiene,
//! Docker latest tag sync discipline, pre-flight gate execution,
//! SemVer monotonic bump progression, changelog structure, and rollback ref safety.

pub mod attribution_verifier;
pub mod channel_gate;
pub mod changelog_structure_validator;
pub mod docker_sync_verifier;
pub mod manifest_sync;
pub mod path_hygiene;
pub mod preflight_gate_runner;
pub mod rollback_ref_guard;
pub mod semver_monotonic_validator;

#[cfg(test)]
pub mod tests_author_attribution;
#[cfg(test)]
pub mod tests_changelog_structure;
#[cfg(test)]
pub mod tests_docker_tag_sync;
#[cfg(test)]
pub mod tests_preflight_gate;
#[cfg(test)]
pub mod tests_release_channel_gate;
#[cfg(test)]
pub mod tests_relative_path_linter;
#[cfg(test)]
pub mod tests_rollback_ref_guard;
#[cfg(test)]
pub mod tests_semver_monotonic;
#[cfg(test)]
pub mod tests_version_manifest_sync;

pub use attribution_verifier::*;
pub use channel_gate::*;
pub use changelog_structure_validator::*;
pub use docker_sync_verifier::*;
pub use manifest_sync::*;
pub use path_hygiene::*;
pub use preflight_gate_runner::*;
pub use rollback_ref_guard::*;
pub use semver_monotonic_validator::*;
