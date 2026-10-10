use crate::proxy::token_manager::ProxyToken;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod modelspec;
mod resolve_custom_budget;
mod tests;

pub use modelspec::get_max_output_tokens;
pub use modelspec::get_thinking_budget;
pub use modelspec::is_bare_gemini_v36_or_above_flash;
pub use modelspec::is_explicit_heuristic_tier_model;
pub use modelspec::is_gemini_under_v3;
pub use modelspec::is_gemini_v3_or_above;
pub use modelspec::is_thinking_model;
pub use modelspec::is_tiered_flash_model;
pub use modelspec::normalize_client_thinking_level;
pub use modelspec::resolve_alias;
pub use modelspec::resolve_authoritative_thinking_budget;
pub use modelspec::resolve_gemini_3x_flash_tiered;
pub use modelspec::ModelSpec;
pub(crate) use modelspec::SpecsConfig;
pub use resolve_custom_budget::resolve_custom_budget;
pub(crate) use tests::tests;
