//! Release Gate & Compliance Architecture.
//! Enforces version manifest synchronization, release channel separation,
//! author attribution compliance, repository-relative Git path hygiene,
//! Docker latest tag sync discipline, pre-flight gate execution,
//! SemVer monotonic bump progression, changelog structure, rollback ref safety,
//! caller IDE self-preservation, transient test sandboxing, thinking cache invalidation,
//! co-author trailers, PR template alignment, profile duplication parity,
//! settings deep-merge synchronization, job breakaway fallback, batch file execution guards,
//! and deep transcript inspection.

pub mod attribution_verifier;
pub mod batch_file_spawn_guard;
pub mod caller_ide_preservation_guard;
pub mod channel_gate;
pub mod changelog_structure_validator;
pub mod coauthor_attribution_verifier;
pub mod docker_sync_verifier;
pub mod job_breakaway_fallback_ladder;
pub mod manifest_sync;
pub mod path_hygiene;
pub mod preflight_gate_runner;
pub mod profile_duplication_parity_guard;
pub mod pr_template_compliance_linter;
pub mod rollback_ref_guard;
pub mod semver_monotonic_validator;
pub mod settings_deep_merge_synchronizer;
pub mod test_sandbox_lifecycle_verifier;
pub mod thinking_cache_invalidation_guard;
pub mod transcript_deep_inspection_scanner;

#[cfg(test)]
pub mod tests_author_attribution;
#[cfg(test)]
pub mod tests_batch_file_spawn_guard;
#[cfg(test)]
pub mod tests_caller_ide_guard;
#[cfg(test)]
pub mod tests_changelog_structure;
#[cfg(test)]
pub mod tests_coauthor_attribution;
#[cfg(test)]
pub mod tests_docker_tag_sync;
#[cfg(test)]
pub mod tests_job_breakaway_fallback;
#[cfg(test)]
pub mod tests_preflight_gate;
#[cfg(test)]
pub mod tests_profile_duplication_parity;
#[cfg(test)]
pub mod tests_pr_template_linter;
#[cfg(test)]
pub mod tests_release_channel_gate;
#[cfg(test)]
pub mod tests_relative_path_linter;
#[cfg(test)]
pub mod tests_rollback_ref_guard;
#[cfg(test)]
pub mod tests_sandbox_lifecycle;
#[cfg(test)]
pub mod tests_semver_monotonic;
#[cfg(test)]
pub mod tests_settings_deep_merge;
#[cfg(test)]
pub mod tests_thinking_cache_guard;
#[cfg(test)]
pub mod tests_transcript_deep_inspection;
#[cfg(test)]
pub mod tests_version_manifest_sync;

pub use attribution_verifier::*;
pub use batch_file_spawn_guard::*;
pub use caller_ide_preservation_guard::*;
pub use channel_gate::*;
pub use changelog_structure_validator::*;
pub use coauthor_attribution_verifier::*;
pub use docker_sync_verifier::*;
pub use job_breakaway_fallback_ladder::*;
pub use manifest_sync::*;
pub use path_hygiene::*;
pub use preflight_gate_runner::*;
pub use profile_duplication_parity_guard::*;
pub use pr_template_compliance_linter::*;
pub use rollback_ref_guard::*;
pub use semver_monotonic_validator::*;
pub use settings_deep_merge_synchronizer::*;
pub use test_sandbox_lifecycle_verifier::*;
pub use thinking_cache_invalidation_guard::*;
pub use transcript_deep_inspection_scanner::*;
