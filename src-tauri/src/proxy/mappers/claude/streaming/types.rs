// Streaming shared types (split from streaming.rs)
use crate::proxy::mappers::claude::models::*;
use serde_json::Value;

pub fn remap_function_call_args(name: &str, args: &mut Value) {
    // Purely transparent transmission of protocol tool parameters without any field renaming or interception rewriting
    if let Some(obj) = args.as_object() {
        tracing::debug!(
            "[Streaming] Tool Call (Passthrough): '{}' Args: {:?}",
            name,
            obj
        );
    }
}

/// block type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    None,
    Text,
    Thinking,
    Function,
}

/// signature manager
pub struct SignatureManager {
    pending: Option<String>,
}

impl SignatureManager {
    pub fn new() -> Self {
        Self { pending: None }
    }

    pub fn store(&mut self, signature: Option<String>) {
        if signature.is_some() {
            self.pending = signature;
        }
    }

    pub fn consume(&mut self) -> Option<String> {
        self.pending.take()
    }

    pub fn has_pending(&self) -> bool {
        self.pending.is_some()
    }
}
