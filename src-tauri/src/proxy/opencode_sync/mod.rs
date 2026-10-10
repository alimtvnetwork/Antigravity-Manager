//! OpenCode configuration sync.
//!
//! Split from the former monolithic `opencode_sync.rs` into focused modules.
//! This file is a thin facade: it declares the submodules and re-exports the
//! public API so `crate::proxy::opencode_sync::X` keeps resolving.

mod binary;
mod catalog;
mod commands;
mod config_paths;
mod dtos;
pub(crate) mod lock;
mod models;
mod sync;
mod sync_apply;
mod sync_helpers;

#[cfg(test)]
mod canonical_family_tests;
#[cfg(test)]
mod tests_a_catalog;
#[cfg(test)]
mod tests_b_sync;
#[cfg(test)]
mod tests_c_providers;
#[cfg(test)]
mod tests_d_config;

pub use binary::check_opencode_installed;
pub use commands::{
    apply_remove_provider, execute_opencode_clear, execute_opencode_openai_sync,
    execute_opencode_remove_provider, execute_opencode_restore, execute_opencode_sync,
    extract_providers_from_config, get_canonical_families, get_opencode_config_content,
    get_opencode_providers, get_opencode_sync_status, read_opencode_config_content,
    read_opencode_providers, remove_opencode_provider, GetOpencodeConfigRequest,
    OpencodeProviderSummary,
};
pub use dtos::{CanonicalFamilyDto, OpencodeStatus};
pub use models::ModelInput;
pub use sync::{is_provider_validation_error, sync_opencode_config, sync_opencode_openai_provider};
pub use sync_apply::restore_opencode_config;
pub use sync_helpers::get_sync_status;

// Private items exercised by the test modules above. Re-exported crate-wide
// only for tests so the split test files can keep using `use super::*;`.
#[cfg(test)]
pub(crate) use catalog::{build_model_catalog, normalize_opencode_base_url, ModelDef, VariantType};
#[cfg(test)]
pub(crate) use commands::base_url_matches;
#[cfg(test)]
pub(crate) use config_paths::{
    extract_version, parse_config_file, resolve_active_config_file_name, strip_jsonc_comments,
    strip_jsonc_trailing_commas,
};
#[cfg(test)]
pub(crate) use lock::{
    atomically_write_config, ANTIGRAVITY_PROVIDER_ID, APIKEY_FUN_PROVIDER_ID, BACKUP_SUFFIX,
    OPENAI_COMPATIBLE_NPM, OPENCODE_CONFIG_FILE, OPENCODE_CONFIG_FILE_JSONC,
};
#[cfg(test)]
pub(crate) use models::{build_fallback_model_json, humanize_model_id, merge_catalog_models};
#[cfg(test)]
pub(crate) use sync::{sync_openai_provider_to_path, validate_provider_id};
#[cfg(test)]
pub(crate) use sync_apply::{
    apply_clear_to_config, apply_openai_compatible_provider_sync, apply_sync_to_config,
};
#[cfg(test)]
pub(crate) use sync_helpers::{build_model_json, build_variants_object};
