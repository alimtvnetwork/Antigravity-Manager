use super::super::caveman_cleaner::CavemanCleaner;
use super::super::claude::models::{
    ClaudeRequest, ContentBlock, Message, MessageContent, SystemPrompt,
};
use super::super::openai::models::{OpenAIMessage, OpenAIRequest};
use super::super::rtk_cleaner::RtkCleaner;
use super::estimate_inline_data_tokens;
use super::estimate_media_tokens_from_url;
use super::estimate_tokens_from_str;
use super::ContextManager;
use serde_json::{json, Value};
use tracing::{debug, info};

// Gemini-side context management (split from context_manager.rs).
impl ContextManager {
    /// Estimate token usage for a Gemini Request represented as serde_json::Value
    pub fn estimate_gemini_token_usage(body: &Value) -> u32 {
        let body = body.get("request").unwrap_or(body);
        let mut total = 0;

        // systemInstruction
        if let Some(sys_inst) = body.get("systemInstruction") {
            if let Some(parts) = sys_inst.get("parts").and_then(|p| p.as_array()) {
                for part in parts {
                    if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                        total += estimate_tokens_from_str(text);
                    }
                }
            }
        }

        // contents
        if let Some(contents) = body.get("contents").and_then(|c| c.as_array()) {
            for msg in contents {
                total += 4; // msg overhead
                if let Some(parts) = msg.get("parts").and_then(|p| p.as_array()) {
                    for part in parts {
                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                            total += estimate_tokens_from_str(text);
                        }
                        if let Some(thought) = part.get("thought").and_then(|t| t.as_bool()) {
                            if thought {
                                total += 100; // thinking overhead
                            }
                        }
                        // inlineData: base64-encoded images/audio embedded in the request
                        if let Some(inline_data) = part.get("inlineData") {
                            let mime = inline_data
                                .get("mimeType")
                                .and_then(|m| m.as_str())
                                .unwrap_or("");
                            let data_len = inline_data
                                .get("data")
                                .and_then(|d| d.as_str())
                                .map(|s| s.len())
                                .unwrap_or(0);
                            total += estimate_inline_data_tokens(mime, data_len);
                        }
                        if let Some(fc) = part.get("functionCall") {
                            total += 20;
                            if let Some(name) = fc.get("name").and_then(|n| n.as_str()) {
                                total += estimate_tokens_from_str(name);
                            }
                            if let Some(args) = fc.get("args") {
                                if let Ok(json_str) = serde_json::to_string(args) {
                                    total += estimate_tokens_from_str(&json_str);
                                }
                            }
                        }
                        if let Some(fr) = part.get("functionResponse") {
                            total += 10;
                            if let Some(name) = fr.get("name").and_then(|n| n.as_str()) {
                                total += estimate_tokens_from_str(name);
                            }
                            if let Some(resp) = fr.get("response") {
                                if let Ok(json_str) = serde_json::to_string(resp) {
                                    total += estimate_tokens_from_str(&json_str);
                                }
                            }
                        }
                    }
                }
            }
        }

        // tools
        if let Some(tools) = body.get("tools").and_then(|t| t.as_array()) {
            for tool in tools {
                if let Ok(json_str) = serde_json::to_string(tool) {
                    total += estimate_tokens_from_str(&json_str);
                }
            }
        }

        total
    }

    /// Trim old tool messages in Gemini request body, keeping only the last N rounds

    pub fn trim_gemini_tool_messages(body: &mut Value, keep_last_n_rounds: usize) -> bool {
        if let Some(contents) = body.get_mut("contents").and_then(|c| c.as_array_mut()) {
            let mut tool_rounds = Vec::new();
            let mut current_round: Option<OpenAIToolRound> = None;

            for (i, msg) in contents.iter().enumerate() {
                let role = msg
                    .get("role")
                    .and_then(|r| r.as_str())
                    .unwrap_or("unknown");
                let has_function_call = msg
                    .get("parts")
                    .and_then(|p| p.as_array())
                    .map(|arr| arr.iter().any(|part| part.get("functionCall").is_some()))
                    .unwrap_or(false);
                let has_function_response = msg
                    .get("parts")
                    .and_then(|p| p.as_array())
                    .map(|arr| {
                        arr.iter()
                            .any(|part| part.get("functionResponse").is_some())
                    })
                    .unwrap_or(false);

                if role == "model" && has_function_call {
                    if let Some(round) = current_round.take() {
                        tool_rounds.push(round);
                    }
                    current_round = Some(OpenAIToolRound {
                        _assistant_index: i,
                        _tool_indices: Vec::new(),
                        indices: vec![i],
                    });
                } else if (role == "user" || role == "model") && has_function_response {
                    if let Some(ref mut round) = current_round {
                        round._tool_indices.push(i);
                        round.indices.push(i);
                    }
                } else if role == "user" {
                    if let Some(round) = current_round.take() {
                        tool_rounds.push(round);
                    }
                }
            }
            if let Some(round) = current_round {
                tool_rounds.push(round);
            }

            // 对保留下来的工具消息部分进行 RTK 日志降噪 (就地修改)
            for msg in contents.iter_mut() {
                if let Some(parts) = msg.get_mut("parts").and_then(|p| p.as_array_mut()) {
                    for part in parts {
                        if let Some(fr) = part.get_mut("functionResponse") {
                            if let Some(resp_obj) =
                                fr.get_mut("response").and_then(|r| r.as_object_mut())
                            {
                                for (_key, val) in resp_obj.iter_mut() {
                                    if let Some(s) = val.as_str() {
                                        let cleaned = RtkCleaner::clean(s, 48);
                                        if cleaned != s {
                                            *val = json!(cleaned);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if tool_rounds.len() <= keep_last_n_rounds {
                return false;
            }

            let rounds_to_remove = tool_rounds.len() - keep_last_n_rounds;
            let mut indices_to_remove = std::collections::HashSet::new();

            for round in tool_rounds.iter().take(rounds_to_remove) {
                for idx in &round.indices {
                    indices_to_remove.insert(*idx);
                }
            }

            let mut removed_count = 0;
            for idx in (0..contents.len()).rev() {
                if indices_to_remove.contains(&idx) {
                    contents.remove(idx);
                    removed_count += 1;
                }
            }

            if removed_count > 0 {
                info!(
                    "[ContextManager] [Gemini] Trimmed {} tool messages, kept last {} rounds",
                    removed_count, keep_last_n_rounds
                );
            }
            removed_count > 0
        } else {
            false
        }
    }

    /// Compress thinking in Gemini request body, keeping it in a lightweight representation

    pub fn compress_gemini_thinking_preserve_signature(
        body: &mut Value,
        protected_last_n: usize,
    ) -> bool {
        let contents = if body.get("contents").and_then(|c| c.as_array()).is_some() {
            body.get_mut("contents").and_then(|c| c.as_array_mut())
        } else {
            body.get_mut("request")
                .and_then(|r| r.get_mut("contents"))
                .and_then(|c| c.as_array_mut())
        };
        if let Some(contents) = contents {
            let total_turns = contents.len();
            if total_turns == 0 {
                return false;
            }

            let start_protection_idx = total_turns.saturating_sub(protected_last_n);
            let mut compressed_count = 0;

            for (i, msg) in contents.iter_mut().enumerate() {
                if i >= start_protection_idx {
                    continue;
                }

                let role = msg
                    .get("role")
                    .and_then(|r| r.as_str())
                    .unwrap_or("unknown");
                if role == "model" {
                    if let Some(parts) = msg.get_mut("parts").and_then(|p| p.as_array_mut()) {
                        for part in parts {
                            if let Some(obj) = part.as_object_mut() {
                                if let Some(thought) = obj.get("thought").and_then(|t| t.as_bool())
                                {
                                    if thought {
                                        if let Some(text) =
                                            obj.get_mut("text").and_then(|t| t.as_str())
                                        {
                                            if text.len() > 10 {
                                                obj.insert("text".to_string(), json!("..."));
                                                // [FIX] Remove thoughtSignature when compressing thought text
                                                // Signature is computed over original thought content; keeping it with "..." causes
                                                // Google API to return 400 INVALID_ARGUMENT: Invalid thought signature.
                                                obj.remove("thoughtSignature");
                                                compressed_count += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            compressed_count > 0
        } else {
            false
        }
    }

    /// Re-estimate (and optionally compress) AFTER mapping + thinking restore on the transit body.

    pub fn apply_post_transit_context_mgmt(body: &mut Value, mapped_model: &str) -> u32 {
        let estimated = Self::estimate_gemini_token_usage(body);
        let level = crate::proxy::config::get_global_compression_level();
        if level != "high" {
            return estimated;
        }
        let context_limit = if mapped_model.to_lowercase().contains("flash") {
            1_000_000u32
        } else {
            2_000_000u32
        };
        let ratio = estimated as f32 / context_limit as f32;
        if ratio > crate::proxy::config::get_global_threshold_l2() {
            if Self::compress_gemini_thinking_preserve_signature(body, 4) {
                return Self::estimate_gemini_token_usage(body);
            }
        }
        estimated
    }
}
#[cfg(test)]
mod tests {
    use super::OpenAIToolRound;
    use super::*;
    use crate::proxy::mappers::context_manager::OpenAIToolRound;

    // Helper to create a request since Default is not implemented
}
