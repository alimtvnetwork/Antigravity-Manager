//! Model name mapping (Claude <-> Gemini, route resolution).
//! Facade: implementation lives in submodules, each <= 500 lines.

pub mod claude_gemini;
pub mod forwarding;
pub mod routing;

pub use claude_gemini::{get_supported_models, map_claude_model_to_gemini};
pub use forwarding::{update_dynamic_forwarding_rules, DYNAMIC_MODEL_FORWARDING_RULES};
pub use routing::{normalize_to_standard_id, resolve_model_route};
