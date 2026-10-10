use crate::proxy::mappers::claude::models::{ClaudeRequest, MessageContent};
use crate::proxy::mappers::openai::models::{OpenAIContent, OpenAIRequest};
use serde_json::Value;
use sha2::{Digest, Sha256};

mod sanitize_user_text_for_fingerprint;
mod tests;

pub use sanitize_user_text_for_fingerprint::sanitize_user_text_for_fingerprint;
pub use sanitize_user_text_for_fingerprint::SessionManager;
