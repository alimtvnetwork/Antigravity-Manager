use base64::prelude::*;

use super::*;

/// Process a parsed inbound email action and dispatch bidirectional 2-phase receipts
/// Outcome of executing an inbound action: shared mutable state extracted from the match arms.
#[derive(Debug, Clone)]
pub(crate) struct ActionOutcome {
    pub action_str: String,
    pub status: String,
    pub result_summary: String,
    pub output_text: String,
    pub exit_code: i32,
}

impl Default for ActionOutcome {
    fn default() -> Self {
        Self {
            action_str: "unknown".to_string(),
            status: "success".to_string(),
            result_summary: String::new(),
            output_text: String::new(),
            exit_code: 0,
        }
    }
}
