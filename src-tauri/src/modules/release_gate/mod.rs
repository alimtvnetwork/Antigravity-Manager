//! Release Gate & Compliance Architecture.
//! Enforces version manifest synchronization, release channel separation,
//! author attribution compliance, and repository-relative Git path hygiene.

pub mod attribution_verifier;
pub mod channel_gate;
pub mod manifest_sync;
pub mod path_hygiene;

#[cfg(test)]
pub mod tests_author_attribution;
#[cfg(test)]
pub mod tests_release_channel_gate;
#[cfg(test)]
pub mod tests_relative_path_linter;
#[cfg(test)]
pub mod tests_version_manifest_sync;

pub use attribution_verifier::*;
pub use channel_gate::*;
pub use manifest_sync::*;
pub use path_hygiene::*;
