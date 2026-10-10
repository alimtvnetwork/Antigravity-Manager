use super::caveman_cleaner::CavemanCleaner;
use super::claude::models::{ClaudeRequest, ContentBlock, Message, MessageContent, SystemPrompt};
use super::openai::models::{OpenAIMessage, OpenAIRequest};
use super::rtk_cleaner::RtkCleaner;
use serde_json::{json, Value};
use tracing::{debug, info};

// OpenAI-side context management (split from context_manager.rs).
impl ContextManager {
    pub fn clean_openai_tool_message(msg: &mut OpenAIMessage) -> bool {
        if msg.role == "tool" {
            if let Some(ref mut content) = msg.content {
                match content {
                    crate::proxy::mappers::openai::models::OpenAIContent::String(s) => {
                        let cleaned = RtkCleaner::clean(s, 48);
                        if cleaned != *s {
                            *s = cleaned;
                            return true;
                        }
                    }
                    crate::proxy::mappers::openai::models::OpenAIContent::Array(blocks) => {
                        let mut modified = false;
                        for block in blocks {
                            if let crate::proxy::mappers::openai::models::OpenAIContentBlock::Text { text } = block {
                                let cleaned = RtkCleaner::clean(text, 48);
                                if cleaned != *text {
                                    *text = cleaned;
                                    modified = true;
                                }
                            }
                        }
                        return modified;
                    }
                }
            }
        }
        false
    }

    /// Internal helper to strip thinking blocks from messages outside the protected range

    pub fn extract_last_openai_valid_signature(session_id: &str) -> Option<String> {
        crate::proxy::signature_cache::SignatureCache::global().get_session_signature(session_id)
    }

    // ===== [Layer 1] Tool Message Intelligent Trimming =====
    // Borrowed from Practical-Guide-to-Context-Engineering
    // This layer removes old tool call/result pairs while preserving recent ones
    // Advantage: Does NOT break Prompt Cache (only removes messages, doesn't modify content)

    /// Trim old tool messages, keeping only the last N rounds
    ///
    /// A "tool round" consists of:
    /// - An assistant message with tool_use
    /// - One or more user messages with tool_result
    ///
    /// Returns true if any messages were removed

    /// Restore reasoning text for assistant messages from cache in OpenAI format
    pub fn restore_openai_reasoning_content(messages: &mut Vec<OpenAIMessage>, session_id: &str) {
        let mut assistant_turn_count = 0;
        for msg in messages.iter_mut() {
            if msg.role == "assistant" {
                let is_missing = msg
                    .reasoning_content
                    .as_ref()
                    .map(|s| s.is_empty() || s == "[undefined]")
                    .unwrap_or(true);

                if is_missing {
                    if let Some(cached_reasoning) = crate::proxy::SignatureCache::global()
                        .get_session_reasoning(session_id, assistant_turn_count)
                    {
                        tracing::debug!(
                            "[OpenAI-Reasoning] Restored reasoning for assistant turn {} (len: {})",
                            assistant_turn_count,
                            cached_reasoning.len()
                        );
                        msg.reasoning_content = Some(cached_reasoning);
                    }
                }
                assistant_turn_count += 1;
            }
        }
    }

    /// Purify reasoning text in OpenAI history to save tokens

    pub fn purify_openai_history(
        messages: &mut Vec<OpenAIMessage>,
        strategy: PurificationStrategy,
    ) -> bool {
        let protected_last_n = match strategy {
            PurificationStrategy::Soft => 4, // Keep thinking for recent 4 messages (approx 2 turns)
            PurificationStrategy::Aggressive => 0,
        };
        let total_msgs = messages.len();
        if total_msgs == 0 {
            return false;
        }
        let start_protection_idx = total_msgs.saturating_sub(protected_last_n);
        let mut modified = false;

        for (i, msg) in messages.iter_mut().enumerate() {
            if i >= start_protection_idx {
                continue;
            }
            if msg.role == "assistant" && msg.reasoning_content.is_some() {
                let has_tool_calls = msg
                    .tool_calls
                    .as_ref()
                    .map(|tc| !tc.is_empty())
                    .unwrap_or(false);
                if !has_tool_calls {
                    tracing::debug!(
                        "[ContextManager] Purifying reasoning_content of message {} (len: {})",
                        i,
                        msg.reasoning_content.as_ref().unwrap().len()
                    );
                    msg.reasoning_content = None;
                    modified = true;
                }
                // [REMOVED 2026-09-27] 带 tool_calls 的 reasoning 不再压缩为 "..."，保持原样透传。
            }
            if msg.role == "user" || msg.role == "assistant" {
                if let Some(ref mut content) = msg.content {
                    match content {
                        crate::proxy::mappers::openai::models::OpenAIContent::String(s) => {
                            let cleaned = CavemanCleaner::clean(s);
                            if cleaned != *s {
                                *s = cleaned;
                                modified = true;
                            }
                        }
                        crate::proxy::mappers::openai::models::OpenAIContent::Array(blocks) => {
                            for block in blocks {
                                if let crate::proxy::mappers::openai::models::OpenAIContentBlock::Text { text } = block {
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
        }
        modified
    }

    /// Trim old tool messages in OpenAI format, keeping only the last N rounds

    pub fn trim_openai_tool_messages(
        messages: &mut Vec<OpenAIMessage>,
        keep_last_n_rounds: usize,
    ) -> bool {
        let tool_rounds = identify_openai_tool_rounds(messages);
        let mut modified = false;

        // Clean retained tool messages using RtkCleaner
        for msg in messages.iter_mut() {
            if Self::clean_openai_tool_message(msg) {
                modified = true;
            }
        }

        if tool_rounds.len() <= keep_last_n_rounds {
            return modified;
        }

        let rounds_to_remove = tool_rounds.len() - keep_last_n_rounds;
        let mut indices_to_remove = std::collections::HashSet::new();

        for round in tool_rounds.iter().take(rounds_to_remove) {
            for idx in &round.indices {
                indices_to_remove.insert(*idx);
            }
        }

        let mut removed_count = 0;
        for idx in (0..messages.len()).rev() {
            if indices_to_remove.contains(&idx) {
                messages.remove(idx);
                removed_count += 1;
            }
        }

        if removed_count > 0 {
            info!(
                "[ContextManager] [OpenAI] Trimmed {} tool messages, kept last {} rounds",
                removed_count, keep_last_n_rounds
            );
        }
        removed_count > 0
    }
}

/// Represents a tool call round (assistant tool_use + user tool_result(s))
#[derive(Debug)]
struct ToolRound {
    _assistant_index: usize,
    tool_result_indices: Vec<usize>,
    indices: Vec<usize>, // All indices in this round
}

/// Identify tool call rounds in the message history
fn identify_tool_rounds(messages: &[Message]) -> Vec<ToolRound> {
    let mut rounds = Vec::new();
    let mut current_round: Option<ToolRound> = None;

    for (i, msg) in messages.iter().enumerate() {
        match msg.role.as_str() {
            "assistant" => {
                if has_tool_use(&msg.content) {
                    // Save previous round if exists
                    if let Some(round) = current_round.take() {
                        rounds.push(round);
                    }
                    // Start new round
                    current_round = Some(ToolRound {
                        _assistant_index: i,
                        tool_result_indices: Vec::new(),
                        indices: vec![i],
                    });
                }
            }
            "user" => {
                if let Some(ref mut round) = current_round {
                    if has_tool_result(&msg.content) {
                        round.tool_result_indices.push(i);
                        round.indices.push(i);
                    } else {
                        // Normal user message ends the current round
                        rounds.push(current_round.take().unwrap());
                    }
                }
            }
            _ => {}
        }
    }

    // Save last round if exists
    if let Some(round) = current_round {
        rounds.push(round);
    }

    debug!(
        "[ContextManager] Identified {} tool rounds in {} messages",
        rounds.len(),
        messages.len()
    );

    rounds
}

struct OpenAIToolRound {
    _assistant_index: usize,
    _tool_indices: Vec<usize>,
    indices: Vec<usize>,
}

fn identify_openai_tool_rounds(messages: &[OpenAIMessage]) -> Vec<OpenAIToolRound> {
    let mut rounds = Vec::new();
    let mut current_round: Option<OpenAIToolRound> = None;

    for (i, msg) in messages.iter().enumerate() {
        if msg.role == "assistant"
            && msg.tool_calls.is_some()
            && !msg.tool_calls.as_ref().unwrap().is_empty()
        {
            if let Some(round) = current_round.take() {
                rounds.push(round);
            }
            current_round = Some(OpenAIToolRound {
                _assistant_index: i,
                _tool_indices: Vec::new(),
                indices: vec![i],
            });
        } else if msg.role == "tool" || msg.role == "function" || msg.tool_call_id.is_some() {
            if let Some(ref mut round) = current_round {
                round._tool_indices.push(i);
                round.indices.push(i);
            }
        } else if msg.role == "user" {
            if let Some(round) = current_round.take() {
                rounds.push(round);
            }
        }
    }
    if let Some(round) = current_round {
        rounds.push(round);
    }
    rounds
}

/// Check if message content contains tool_use
fn has_tool_use(content: &MessageContent) -> bool {
    if let MessageContent::Array(blocks) = content {
        blocks
            .iter()
            .any(|b| matches!(b, ContentBlock::ToolUse { .. }))
    } else {
        false
    }
}

/// Check if message content contains tool_result
fn has_tool_result(content: &MessageContent) -> bool {
    if let MessageContent::Array(blocks) = content {
        blocks
            .iter()
            .any(|b| matches!(b, ContentBlock::ToolResult { .. }))
    } else {
        false
    }
}
impl ContextManager {
    /// Estimate token usage for an OpenAI Request

    pub fn estimate_openai_token_usage(request: &OpenAIRequest) -> u32 {
        let mut total = 0;

        // System or developer messages, tools definitions, prompt
        if let Some(prompt) = &request.prompt {
            total += estimate_tokens_from_str(prompt);
        }
        if let Some(instructions) = &request.instructions {
            total += estimate_tokens_from_str(instructions);
        }

        for msg in &request.messages {
            total += 4; // msg overhead

            if let Some(ref content) = msg.content {
                match content {
                    crate::proxy::mappers::openai::models::OpenAIContent::String(s) => {
                        total += estimate_tokens_from_str(s);
                    }
                    crate::proxy::mappers::openai::models::OpenAIContent::Array(blocks) => {
                        for block in blocks {
                            match block {
                                crate::proxy::mappers::openai::models::OpenAIContentBlock::Text { text } => {
                                    total += estimate_tokens_from_str(text);
                                }
                                crate::proxy::mappers::openai::models::OpenAIContentBlock::ImageUrl { image_url } => {
                                    // Gemini counts images at ~258 tokens for standard resolution.
                                    // For base64 data URLs, estimate higher based on payload size
                                    // since very large images can consume significantly more tokens.
                                    total += estimate_image_tokens_from_url(&image_url.url);
                                }
                                crate::proxy::mappers::openai::models::OpenAIContentBlock::AudioUrl { audio_url } => {
                                    // Audio is tokenized at ~32 tokens per second (~25 bytes/token from base64)
                                    total += estimate_media_tokens_from_url(&audio_url.url);
                                }
                                crate::proxy::mappers::openai::models::OpenAIContentBlock::InputAudio { input_audio } => {
                                    // input_audio 携带裸 base64，直接按 inlineData 估算
                                    total += estimate_inline_data_tokens(
                                        &input_audio.mime_type(),
                                        input_audio.data.len(),
                                    );
                                }
                                crate::proxy::mappers::openai::models::OpenAIContentBlock::VideoUrl { video_url } => {
                                    // Video token estimation based on media payload size
                                    total += estimate_media_tokens_from_url(&video_url.url);
                                }
                            }
                        }
                    }
                }
            }

            if let Some(ref reasoning) = msg.reasoning_content {
                total += estimate_tokens_from_str(reasoning);
                total += 100; // signature/thinking overhead
            }

            if let Some(ref tool_calls) = msg.tool_calls {
                for tc in tool_calls {
                    total += 20;
                    if let Some(ref func) = tc.function {
                        total += estimate_tokens_from_str(&func.name);
                        total += estimate_tokens_from_str(&func.arguments);
                    }
                }
            }

            if let Some(ref name) = msg.name {
                total += estimate_tokens_from_str(name);
            }
        }

        if let Some(tools) = &request.tools {
            for tool in tools {
                if let Ok(json_str) = serde_json::to_string(tool) {
                    total += estimate_tokens_from_str(&json_str);
                }
            }
        }

        if let Some(thinking) = &request.thinking {
            if let Some(budget) = thinking.budget_tokens {
                total += budget;
            }
        }

        total
    }
}
