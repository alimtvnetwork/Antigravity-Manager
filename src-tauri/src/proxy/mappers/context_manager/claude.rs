use super::super::caveman_cleaner::CavemanCleaner;
use super::super::claude::models::{
    ClaudeRequest, ContentBlock, Message, MessageContent, SystemPrompt,
};
use super::super::openai::models::{OpenAIMessage, OpenAIRequest};
use super::super::rtk_cleaner::RtkCleaner;
use super::estimate_tokens_from_str;
use super::openai::identify_tool_rounds;
use super::ContextManager;
use super::PurificationStrategy;
use serde_json::{json, Value};
use tracing::{debug, info};

// Claude-side context management (split from context_manager.rs).
impl ContextManager {
    pub fn purify_history(messages: &mut Vec<Message>, strategy: PurificationStrategy) -> bool {
        let protected_last_n = match strategy {
            PurificationStrategy::Soft => 4, // Protect last ~2 turns (User-AI-User-AI)
            PurificationStrategy::Aggressive => 0, // No protection
        };

        let mut modified = Self::strip_thinking_blocks(messages, protected_last_n);

        // Apply Caveman cleaning to older message contents to save tokens
        let total_msgs = messages.len();
        let start_protection_idx = total_msgs.saturating_sub(protected_last_n);
        for (i, msg) in messages.iter_mut().enumerate() {
            if i >= start_protection_idx {
                continue;
            }
            if msg.role == "user" || msg.role == "assistant" {
                match &mut msg.content {
                    MessageContent::String(s) => {
                        let cleaned = CavemanCleaner::clean(s);
                        if cleaned != *s {
                            *s = cleaned;
                            modified = true;
                        }
                    }
                    MessageContent::Array(blocks) => {
                        for block in blocks {
                            if let ContentBlock::Text { text } = block {
                                let cleaned = CavemanCleaner::clean(text);
                                if cleaned != *text {
                                    *text = cleaned;
                                    modified = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        modified
    }

    /// Clean tool result messages using RtkCleaner

    pub fn clean_tool_message(msg: &mut Message) -> bool {
        let mut modified = false;
        if let MessageContent::Array(blocks) = &mut msg.content {
            for block in blocks {
                if let ContentBlock::ToolResult { content, .. } = block {
                    if let Some(s) = content.as_str() {
                        let cleaned = RtkCleaner::clean(s, 48);
                        if cleaned != s {
                            *content = serde_json::Value::String(cleaned);
                            modified = true;
                        }
                    }
                }
            }
        }
        modified
    }

    /// Clean OpenAI tool message using RtkCleaner

    fn strip_thinking_blocks(messages: &mut Vec<Message>, protected_last_n: usize) -> bool {
        let total_msgs = messages.len();
        if total_msgs == 0 {
            return false;
        }

        let start_protection_idx = total_msgs.saturating_sub(protected_last_n);
        let mut modified = false;

        for (i, msg) in messages.iter_mut().enumerate() {
            // Skip protected messages
            if i >= start_protection_idx {
                continue;
            }

            if msg.role == "assistant" {
                if let MessageContent::Array(blocks) = &mut msg.content {
                    let original_len = blocks.len();
                    // Retain only non-Thinking blocks
                    blocks.retain(|b| !matches!(b, ContentBlock::Thinking { .. }));

                    if blocks.len() != original_len {
                        modified = true;
                        debug!(
                            "[ContextManager] Stripped {} thinking blocks from message {}",
                            original_len - blocks.len(),
                            i
                        );
                    }
                }
            }
        }

        modified
    }
}

impl ContextManager {
    /// Estimate token usage for a Claude Request
    ///
    /// This is a lightweight estimation, not a precise count.
    /// It iterates through all messages and blocks to sum up estimated tokens.

    pub fn estimate_token_usage(request: &ClaudeRequest) -> u32 {
        let mut total = 0;

        // System prompt
        if let Some(sys) = &request.system {
            match sys {
                SystemPrompt::String(s) => total += estimate_tokens_from_str(s),
                SystemPrompt::Array(blocks) => {
                    for block in blocks {
                        total += estimate_tokens_from_str(&block.text);
                    }
                }
            }
        }

        // Messages
        for msg in &request.messages {
            // Message overhead
            total += 4;

            match &msg.content {
                MessageContent::String(s) => {
                    total += estimate_tokens_from_str(s);
                }
                MessageContent::Array(blocks) => {
                    for block in blocks {
                        match block {
                            ContentBlock::Text { text } => {
                                total += estimate_tokens_from_str(text);
                            }
                            ContentBlock::Thinking { thinking, .. } => {
                                total += estimate_tokens_from_str(thinking);
                                // Signature overhead
                                total += 100;
                            }
                            ContentBlock::RedactedThinking { data } => {
                                total += estimate_tokens_from_str(data);
                            }
                            ContentBlock::ToolUse { name, input, .. } => {
                                total += 20; // Function call overhead
                                total += estimate_tokens_from_str(name);
                                if let Ok(json_str) = serde_json::to_string(input) {
                                    total += estimate_tokens_from_str(&json_str);
                                }
                            }
                            ContentBlock::ToolResult { content, .. } => {
                                total += 10; // Result overhead
                                             // content is serde_json::Value
                                if let Some(s) = content.as_str() {
                                    total += estimate_tokens_from_str(s);
                                } else if let Some(arr) = content.as_array() {
                                    for item in arr {
                                        if let Some(text) =
                                            item.get("text").and_then(|t| t.as_str())
                                        {
                                            total += estimate_tokens_from_str(text);
                                        }
                                    }
                                } else {
                                    // Fallback for objects or other types
                                    if let Ok(s) = serde_json::to_string(content) {
                                        total += estimate_tokens_from_str(&s);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Tools definition overhead (rough estimate)
        if let Some(tools) = &request.tools {
            for tool in tools {
                if let Ok(json_str) = serde_json::to_string(tool) {
                    total += estimate_tokens_from_str(&json_str);
                }
            }
        }

        // Thinking budget overhead if enabled
        if let Some(thinking) = &request.thinking {
            if let Some(budget) = thinking.budget_tokens {
                // Reserve budget in estimation
                total += budget;
            }
        }

        total
    }

    // ===== [Layer 3 Helper] Extract Last Valid Signature =====
    // Used by Layer 3 to preserve signature when generating XML summary

    /// Extract the last valid thinking signature from message history
    ///
    /// This is critical for Layer 3 (Fork + Summary) to preserve the signature chain.
    /// The signature will be embedded in the XML summary and restored after fork.
    ///
    /// Returns None if no valid signature found (length >= 50)

    pub fn extract_last_valid_signature(messages: &[Message]) -> Option<String> {
        // Iterate in reverse to find the most recent signature
        for msg in messages.iter().rev() {
            if msg.role == "assistant" {
                if let MessageContent::Array(blocks) = &msg.content {
                    for block in blocks {
                        if let ContentBlock::Thinking {
                            signature: Some(sig),
                            ..
                        } = block
                        {
                            // Minimum signature length check (same as SignatureCache)
                            if sig.len() >= 50 {
                                debug!(
                                    "[ContextManager] [Layer-3] Extracted last valid signature (len: {})",
                                    sig.len()
                                );
                                return Some(sig.clone());
                            }
                        }
                    }
                }
            }
        }

        debug!("[ContextManager] [Layer-3] No valid signature found in history");
        None
    }

    /// Extract last valid signature for OpenAI using SignatureCache

    pub fn trim_tool_messages(messages: &mut Vec<Message>, keep_last_n_rounds: usize) -> bool {
        let tool_rounds = identify_tool_rounds(messages);
        let mut modified = false;

        // Clean retained tool messages using RtkCleaner
        for msg in messages.iter_mut() {
            if Self::clean_tool_message(msg) {
                modified = true;
            }
        }

        if tool_rounds.len() <= keep_last_n_rounds {
            return modified; // No trimming needed, but might have cleaned some messages
        }

        // Identify indices to remove (older rounds)
        let rounds_to_remove = tool_rounds.len() - keep_last_n_rounds;
        let mut indices_to_remove = std::collections::HashSet::new();

        for round in tool_rounds.iter().take(rounds_to_remove) {
            for idx in &round.indices {
                indices_to_remove.insert(*idx);
            }
        }

        // Remove in reverse order to avoid index shifting
        let mut removed_count = 0;
        for idx in (0..messages.len()).rev() {
            if indices_to_remove.contains(&idx) {
                messages.remove(idx);
                removed_count += 1;
            }
        }

        if removed_count > 0 {
            info!(
                "[ContextManager] [Layer-1] Trimmed {} tool messages, kept last {} rounds",
                removed_count, keep_last_n_rounds
            );
        }

        removed_count > 0
    }
}
