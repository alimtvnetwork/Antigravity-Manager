// PartProcessor: tool-call handling (split from streaming.rs)
use super::processor::PartProcessor;
use super::state::StreamingState;
use super::types::remap_function_call_args;
use super::types::BlockType;
use crate::proxy::mappers::claude::models::*;
use crate::proxy::signature_cache::cacheentry::SignatureCache;
use bytes::Bytes;
use serde_json::{json, Value};

impl<'a> PartProcessor<'a> {
    // -------------------------------------------------------------------------
    // [FIX #3379] Helpers: call:default_api:* leakage detection & recovery
    // -------------------------------------------------------------------------

    /// Attempt to extract `(tool_name, args_str)` from a raw `call:default_api:*` text.
    /// Returns None if the text does not strictly match the expected prefix format.
    fn try_parse_call_default_api(text: &str) -> Option<(String, String)> {
        const PREFIX: &str = "call:default_api:";
        let trimmed = text.trim();
        if !trimmed.starts_with(PREFIX) {
            return None;
        }
        let rest = &trimmed[PREFIX.len()..];
        // Tool name ends at the first `{` or `(` delimiter
        let tool_end = rest.find(|c| c == '{' || c == '(').unwrap_or(rest.len());
        let tool_name = rest[..tool_end].trim().to_string();
        if tool_name.is_empty() {
            return None;
        }
        let args_str = rest[tool_end..].trim().to_string();
        Some((tool_name, args_str))
    }

    /// Two-phase JSON argument parser for Gemini's loose call:default_api format.
    ///
    /// Phase 1: standard `serde_json` parse (handles well-formed JSON).
    /// Phase 2: lenient key-quoting heuristic for unquoted keys (e.g. `{file_path:foo}`).
    /// Returns `None` (guard G6) if both phases fail.
    pub(crate) fn parse_loose_json_args(args_str: &str) -> Option<serde_json::Value> {
        if args_str.is_empty() {
            // No-arg tool call → valid empty object
            return Some(serde_json::json!({}));
        }

        // Phase 1: strict JSON
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(args_str) {
            if v.is_object() {
                return Some(v);
            }
        }

        // Phase 2: lenient key-quoting for Gemini's internal pseudo-JSON
        // Only attempt if the string looks like a {…} block.
        let s = args_str.trim();
        if !s.starts_with('{') || !s.ends_with('}') {
            return None;
        }
        let inner = &s[1..s.len() - 1];

        // Naïve key-quoting: add double-quotes around bare identifier keys.
        // This covers the most common Gemini output patterns.
        let mut result = String::from("{");
        let mut in_value = false;
        let mut i = 0;
        let chars: Vec<char> = inner.chars().collect();
        while i < chars.len() {
            let c = chars[i];
            match c {
                ',' if !in_value => {
                    result.push(',');
                    i += 1;
                }
                ':' => {
                    result.push(':');
                    in_value = true;
                    i += 1;
                }
                '"' if in_value => {
                    // Quoted value — pass through until closing quote
                    result.push('"');
                    i += 1;
                    while i < chars.len() && chars[i] != '"' {
                        if chars[i] == '\\' {
                            result.push('\\');
                            i += 1;
                        }
                        if i < chars.len() {
                            result.push(chars[i]);
                            i += 1;
                        }
                    }
                    if i < chars.len() {
                        result.push('"');
                        i += 1;
                    }
                    in_value = false;
                }
                _ if !in_value => {
                    // Bare key — wrap with quotes
                    let key_start = i;
                    while i < chars.len() && chars[i] != ':' && chars[i] != ',' {
                        i += 1;
                    }
                    let key: String = chars[key_start..i].iter().collect();
                    let key_trimmed = key.trim();
                    result.push('"');
                    result.push_str(key_trimmed);
                    result.push('"');
                }
                _ => {
                    // Bare value — collect until `,` or end
                    let val_start = i;
                    while i < chars.len() && chars[i] != ',' {
                        result.push(chars[i]);
                        i += 1;
                    }
                    let _ = val_start; // consumed inline
                    in_value = false;
                }
            }
        }
        result.push('}');

        serde_json::from_str::<serde_json::Value>(&result)
            .ok()
            .filter(|v| v.is_object())
    }

    /// Fail-Closed recovery: attempts to convert a `call:default_api:*` text leak
    /// into a proper `tool_use` block by passing 7 strict guards.
    ///
    /// Returns `Some(chunks)` when recovery succeeds, or `None` to let the caller
    /// fall through to the normal `text_delta` path.
    pub(crate) fn try_recover_call_default_api_text(
        &mut self,
        text: &str,
    ) -> Option<Vec<bytes::Bytes>> {
        // G1: Tools must have been registered for this request
        if self.state.registered_tool_names.is_empty() {
            return None;
        }

        // G3: Strict prefix check before doing any expensive work
        if !text.trim().starts_with("call:default_api:") {
            return None;
        }

        let (tool_name, args_str) = Self::try_parse_call_default_api(text)?;

        // G5: The entire text (trimmed) must consist solely of this one expression.
        // This prevents documentation strings like "Use call:default_api:Read{...}" from
        // being mistakenly executed.
        let full_expr = format!(
            "call:default_api:{}{}",
            tool_name,
            if args_str.is_empty() {
                String::new()
            } else {
                format!(" {}", args_str)
            }
        );
        // Allow minor whitespace variance but reject any surrounding prose
        let trimmed_text = text.trim();
        // The text must start with call:default_api:<tool_name> and end right after the args block
        if !trimmed_text.starts_with(&format!("call:default_api:{}", tool_name)) {
            return None;
        }
        // After the tool name and args there must be nothing else (ignore trailing whitespace)
        let after_tool_name = &trimmed_text["call:default_api:".len() + tool_name.len()..].trim();
        // after_tool_name is either empty (no args) or is the args string itself
        if !after_tool_name.is_empty()
            && !after_tool_name.starts_with('{')
            && !after_tool_name.starts_with('(')
        {
            // There is non-arg text after the tool name — reject
            return None;
        }
        let _ = full_expr; // suppress unused warning

        // G4: Tool name must be in the registered whitelist (exact match, case-insensitive)
        let matched_name = self
            .state
            .registered_tool_names
            .iter()
            .find(|n| n.eq_ignore_ascii_case(&tool_name))
            .cloned()?;

        // G2: Current turn must NOT have already produced a structured functionCall
        if self.state.used_tool {
            tracing::warn!(
                "[#3379] Detected call:default_api:{} leakage but used_tool=true; skipping recovery",
                tool_name
            );
            return None;
        }

        // G7: No text_delta must have been emitted this turn
        if self.state.text_delta_emitted_this_turn {
            tracing::warn!(
                "[#3379] Detected call:default_api:{} leakage but text_delta already emitted this turn; skipping recovery",
                tool_name
            );
            return None;
        }

        // G6: Arguments must parse as a valid JSON object
        let parsed_args = Self::parse_loose_json_args(&args_str)?;

        // All guards passed — perform recovery
        tracing::warn!(
            "[#3379-RECOVERY] Recovering call:default_api:{} as tool_use (args_len={})",
            matched_name,
            args_str.len()
        );

        let fc = FunctionCall {
            name: matched_name,
            args: Some(parsed_args),
            id: None,
        };

        Some(self.process_function_call(&fc, None))
    }

    /// Process FunctionCall and capture signature for global storage
    pub(crate) fn process_function_call(
        &mut self,
        fc: &FunctionCall,
        signature: Option<String>,
    ) -> Vec<Bytes> {
        let mut chunks = Vec::new();

        self.state.mark_tool_used();

        let tool_id = fc.id.clone().unwrap_or_else(|| {
            format!(
                "{}-{}",
                fc.name,
                crate::proxy::common::utils::generate_random_id()
            )
        });

        let tool_name = fc.name.clone();

        // Record real tool_id into TurnAccumulator for precise session/fingerprint recovery
        self.state.thinking_acc.record_tool_id(&tool_name, &tool_id);

        // 1. 发送 content_block_start (input 为空对象)
        let mut tool_use = json!({
            "type": "tool_use",
            "id": tool_id,
            "name": tool_name,
            "input": {} // 必须为空，参数通过 delta 发送
        });

        if let Some(ref sig) = signature {
            tool_use["signature"] = json!(sig);

            // 2. Cache tool signature (Layer 1 recovery)
            SignatureCache::global().cache_tool_signature(&tool_id, sig.clone());

            // 3. [NEW v3.3.17] Cache to session-based storage
            if let Some(session_id) = &self.state.session_id {
                SignatureCache::global().cache_session_signature(
                    session_id,
                    sig.clone(),
                    self.state.message_count,
                );
            }

            tracing::debug!(
                "[Claude-SSE] Captured thought_signature for function call (length: {})",
                sig.len()
            );
        }

        chunks.extend(self.state.start_block(BlockType::Function, tool_use));

        // 2. 发送 input_json_delta (完整的参数 JSON 字符串)
        // [FIX #Bug2/#Bug4] ALWAYS emit input_json_delta, even for empty/null args.
        // Claude protocol requires this delta before content_block_stop.
        // Skipping it causes clients (Claude Code) to misinterpret tool calls as text.
        {
            let json_str = if let Some(args) = &fc.args {
                let mut remapped_args = args.clone();
                remap_function_call_args(&fc.name, &mut remapped_args);
                serde_json::to_string(&remapped_args).unwrap_or_else(|_| "{}".to_string())
            } else {
                // [FIX #Bug4] No args provided (e.g. EnterPlanMode): emit empty JSON object
                tracing::debug!(
                    "[Streaming] Tool '{}' has no args, emitting empty input_json_delta",
                    fc.name
                );
                "{}".to_string()
            };

            chunks.push(
                self.state
                    .emit_delta("input_json_delta", json!({ "partial_json": json_str })),
            );
        }

        // 3. 结束块
        chunks.extend(self.state.end_block());

        chunks
    }
}
