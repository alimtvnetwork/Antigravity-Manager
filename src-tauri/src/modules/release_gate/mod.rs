//! Release Gate & Compliance Architecture.
//! Enforces version manifest synchronization, release channel separation,
//! author attribution compliance, repository-relative Git path hygiene,
//! Docker latest tag sync discipline, pre-flight gate execution,
//! SemVer monotonic bump progression, changelog structure, rollback ref safety,
//! caller IDE self-preservation, transient test sandboxing, thinking cache invalidation,
//! co-author trailers, PR template alignment, profile duplication parity,
//! settings deep-merge synchronization, job breakaway fallback, batch file execution guards,
//! deep transcript inspection, FIFO queue query discipline, capsule styling,
//! origin tier discrimination, truncation callout parsing, no-silent-errors rules,
//! bounded matcher and timeout guards, commit history convergence, remote branch base precedence,
//! risky path replacement fallback discipline, side effect ordering around the point of no return,
//! wildcard model routing, regex prompt sanitization, pipeline protocol decoupling,
//! single problem scope cohesion, and contributor cost disclosure.

pub mod agent_prompt_sanitizer_guard;
pub mod attribution_verifier;
pub mod batch_file_spawn_guard;
pub mod bounded_matcher_timeout_guard;
pub mod bounded_matcher_timeout_verifier;
pub mod branch_release_channel_gate;
pub mod caller_ide_preservation_guard;
pub mod capsule_dropdown_styling_guard;
pub mod channel_gate;
pub mod changelog_structure_validator;
pub mod changelog_tag_monotonic_gate;
pub mod coauthor_attribution_verifier;
pub mod coauthor_trailer_alignment_guard;
pub mod commit_convergence_auditor;
pub mod contributor_cost_disclosure_linter;
pub mod cross_platform_compat_guard;
pub mod docker_sync_verifier;
pub mod fifo_queue_ordering_guard;
pub mod generic_fix_preference_guard;
pub mod individually_revertable_commit_guard;
pub mod headless_cli_parity_guard;
pub mod job_breakaway_fallback_ladder;
pub mod manifest_sync;
pub mod no_silent_errors_linter;
pub mod no_silent_errors_justification_auditor;
pub mod origin_tier_hierarchy_guard;
pub mod path_hygiene;
pub mod pipeline_protocol_decoupling_guard;
pub mod preflight_checks_gate;
pub mod preflight_clippy_gate;
pub mod preflight_gate_runner;
pub mod profile_duplication_parity_guard;
pub mod pr_problem_classification_linter;
pub mod pr_scope_cohesion_verifier;
pub mod pr_template_compliance_guard;
pub mod pr_template_compliance_linter;
pub mod pr_unverified_paths_linter;
pub mod published_tip_rewrite_guard;
pub mod readme_changelog_sync_gate;
pub mod readme_changelog_synchronizer;
pub mod rejection_breakdown_generator;
pub mod release_notes_relative_paths_linter;
pub mod remote_branch_base_verifier;
pub mod risky_path_fallback_guard;
pub mod rollback_ref_guard;
pub mod rollback_reference_drift_verifier;
pub mod semver_monotonic_validator;
pub mod settings_deep_merge_synchronizer;
pub mod side_effect_ordering_guard;
pub mod sole_contributor_attribution_enforcer;
pub mod sole_contributor_attribution_guard;
pub mod staging_protocol_guard;
pub mod strictly_relative_git_paths_guard;
pub mod tag_changelog_heading_matcher;
pub mod tag_changelog_heading_matcher_gate;
pub mod test_sandbox_lifecycle_verifier;
pub mod thinking_cache_invalidation_controller;
pub mod thinking_cache_invalidation_guard;
pub mod thinking_cache_target_version_guard;
pub mod transcript_deep_inspection_scanner;
pub mod transcript_full_log_reverse_scanner;
pub mod truncation_callout_parser_guard;
pub mod unverified_paths_disclosure_guard;
pub mod verified_fallback_path_linter;
pub mod wildcard_model_routing_guard;

#[cfg(test)]
pub mod tests_agent_prompt_sanitizer;
#[cfg(test)]
pub mod tests_author_attribution;
#[cfg(test)]
pub mod tests_batch_file_spawn_guard;
#[cfg(test)]
pub mod tests_bounded_matcher_timeout;
#[cfg(test)]
pub mod tests_bounded_matcher_timeout_verifier;
#[cfg(test)]
pub mod tests_branch_release_channel_gate;
#[cfg(test)]
pub mod tests_caller_ide_guard;
#[cfg(test)]
pub mod tests_capsule_dropdown_styling;
#[cfg(test)]
pub mod tests_changelog_structure;
#[cfg(test)]
pub mod tests_changelog_tag_monotonic_gate;
#[cfg(test)]
pub mod tests_coauthor_attribution;
#[cfg(test)]
pub mod tests_coauthor_trailer_alignment_guard;
#[cfg(test)]
pub mod tests_commit_convergence;
#[cfg(test)]
pub mod tests_contributor_cost_disclosure;
#[cfg(test)]
pub mod tests_cross_platform_compat;
#[cfg(test)]
pub mod tests_docker_tag_sync;
#[cfg(test)]
pub mod tests_fifo_queue_ordering;
#[cfg(test)]
pub mod tests_generic_fix_preference;
#[cfg(test)]
pub mod tests_headless_cli_parity;
#[cfg(test)]
pub mod tests_individually_revertable_commit_guard;
#[cfg(test)]
pub mod tests_job_breakaway_fallback;
#[cfg(test)]
pub mod tests_no_silent_errors_linter;
#[cfg(test)]
pub mod tests_no_silent_errors_justification_auditor;
#[cfg(test)]
pub mod tests_origin_tier_hierarchy;
#[cfg(test)]
pub mod tests_pipeline_protocol_decoupling;
#[cfg(test)]
pub mod tests_preflight_checks_gate;
#[cfg(test)]
pub mod tests_preflight_clippy_gate;
#[cfg(test)]
pub mod tests_preflight_gate;
#[cfg(test)]
pub mod tests_profile_duplication_parity;
#[cfg(test)]
pub mod tests_pr_problem_classification_linter;
#[cfg(test)]
pub mod tests_pr_scope_cohesion;
#[cfg(test)]
pub mod tests_pr_template_compliance_guard;
#[cfg(test)]
pub mod tests_pr_template_linter;
#[cfg(test)]
pub mod tests_pr_unverified_paths_linter;
#[cfg(test)]
pub mod tests_published_tip_rewrite;
#[cfg(test)]
pub mod tests_readme_changelog_sync_gate;
#[cfg(test)]
pub mod tests_readme_changelog_synchronizer;
#[cfg(test)]
pub mod tests_rejection_breakdown_generator;
#[cfg(test)]
pub mod tests_release_channel_gate;
#[cfg(test)]
pub mod tests_release_notes_relative_paths_linter;
#[cfg(test)]
pub mod tests_relative_path_linter;
#[cfg(test)]
pub mod tests_remote_branch_base;
#[cfg(test)]
pub mod tests_risky_path_fallback;
#[cfg(test)]
pub mod tests_rollback_ref_guard;
#[cfg(test)]
pub mod tests_rollback_reference_drift_verifier;
#[cfg(test)]
pub mod tests_sandbox_lifecycle;
#[cfg(test)]
pub mod tests_semver_monotonic;
#[cfg(test)]
pub mod tests_settings_deep_merge;
#[cfg(test)]
pub mod tests_side_effect_ordering;
#[cfg(test)]
pub mod tests_sole_contributor_attribution_enforcer;
#[cfg(test)]
pub mod tests_sole_contributor_attribution_guard;
#[cfg(test)]
pub mod tests_staging_protocol;
#[cfg(test)]
pub mod tests_strictly_relative_git_paths_guard;
#[cfg(test)]
pub mod tests_tag_changelog_heading_matcher;
#[cfg(test)]
pub mod tests_tag_changelog_heading_matcher_gate;
#[cfg(test)]
pub mod tests_thinking_cache_guard;
#[cfg(test)]
pub mod tests_thinking_cache_invalidation_controller;
#[cfg(test)]
pub mod tests_thinking_cache_target_version_guard;
#[cfg(test)]
pub mod tests_transcript_deep_inspection;
#[cfg(test)]
pub mod tests_transcript_full_log_reverse_scanner;
#[cfg(test)]
pub mod tests_truncation_callout_parser;
#[cfg(test)]
pub mod tests_unverified_paths_disclosure_guard;
#[cfg(test)]
pub mod tests_verified_fallback_path_linter;
#[cfg(test)]
pub mod tests_version_manifest_sync;
#[cfg(test)]
pub mod tests_wildcard_model_routing;

pub use agent_prompt_sanitizer_guard::*;
pub use attribution_verifier::*;
pub use batch_file_spawn_guard::*;
pub use bounded_matcher_timeout_guard::*;
pub use bounded_matcher_timeout_verifier::*;
pub use branch_release_channel_gate::*;
pub use caller_ide_preservation_guard::*;
pub use capsule_dropdown_styling_guard::*;
pub use channel_gate::*;
pub use changelog_structure_validator::*;
pub use changelog_tag_monotonic_gate::*;
pub use coauthor_attribution_verifier::*;
pub use coauthor_trailer_alignment_guard::*;
pub use commit_convergence_auditor::*;
pub use contributor_cost_disclosure_linter::*;
pub use cross_platform_compat_guard::*;
pub use docker_sync_verifier::*;
pub use fifo_queue_ordering_guard::*;
pub use generic_fix_preference_guard::*;
pub use individually_revertable_commit_guard::*;
pub use headless_cli_parity_guard::*;
pub use job_breakaway_fallback_ladder::*;
pub use manifest_sync::*;
pub use no_silent_errors_linter::*;
pub use no_silent_errors_justification_auditor::*;
pub use origin_tier_hierarchy_guard::*;
pub use path_hygiene::*;
pub use pipeline_protocol_decoupling_guard::*;
pub use preflight_checks_gate::*;
pub use preflight_clippy_gate::*;
pub use preflight_gate_runner::*;
pub use profile_duplication_parity_guard::*;
pub use pr_problem_classification_linter::*;
pub use pr_scope_cohesion_verifier::*;
pub use pr_template_compliance_guard::*;
pub use pr_template_compliance_linter::*;
pub use pr_unverified_paths_linter::*;
pub use published_tip_rewrite_guard::*;
pub use readme_changelog_sync_gate::*;
pub use readme_changelog_synchronizer::*;
pub use rejection_breakdown_generator::*;
pub use release_notes_relative_paths_linter::*;
pub use remote_branch_base_verifier::*;
pub use risky_path_fallback_guard::*;
pub use rollback_ref_guard::*;
pub use rollback_reference_drift_verifier::*;
pub use semver_monotonic_validator::*;
pub use settings_deep_merge_synchronizer::*;
pub use side_effect_ordering_guard::*;
pub use sole_contributor_attribution_enforcer::*;
pub use sole_contributor_attribution_guard::*;
pub use staging_protocol_guard::*;
pub use strictly_relative_git_paths_guard::*;
pub use tag_changelog_heading_matcher::*;
pub use tag_changelog_heading_matcher_gate::*;
pub use test_sandbox_lifecycle_verifier::*;
pub use thinking_cache_invalidation_controller::*;
pub use thinking_cache_invalidation_guard::*;
pub use thinking_cache_target_version_guard::*;
pub use transcript_deep_inspection_scanner::*;
pub use transcript_full_log_reverse_scanner::*;
pub use truncation_callout_parser_guard::*;
pub use unverified_paths_disclosure_guard::*;
pub use verified_fallback_path_linter::*;
pub use wildcard_model_routing_guard::*;
