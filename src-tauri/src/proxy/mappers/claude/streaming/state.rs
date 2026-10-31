// StreamingState (split from streaming.rs)
use super::super::utils::to_claude_usage;
use super::types::{BlockType, SignatureManager};
use crate::proxy::common::client_adapter::{ClientAdapter, SignatureBufferStrategy};
use crate::proxy::mappers::claude::models::*;
use crate::proxy::mappers::estimation_calibrator::get_calibrator;
use crate::proxy::SignatureCache;
use bytes::Bytes;
use serde_json::{json, Value};

/// Streaming state machine

pub struct StreamingState {
    block_type: BlockType,
    pub block_index: usize,
    pub message_start_sent: bool,
    pub message_stop_sent: bool,
    pub(crate) used_tool: bool,
    signatures: SignatureManager,
    pub(crate) trailing_signature: Option<String>,
    pub web_search_query: Option<String>,
    pub grounding_chunks: Option<Vec<serde_json::Value>>,
    // [IMPROVED] Error recovery status tracking (prepared for future use)
    #[allow(dead_code)]
    parse_error_count: usize,
    #[allow(dead_code)]
    last_valid_state: Option<BlockType>,
    // [NEW] Model tracking for signature cache
    pub model_name: Option<String>,
    // [NEW v3.3.17] Session ID for session-based signature caching
    pub session_id: Option<String>,
    // [NEW] Flag for context usage scaling
    pub scaling_enabled: bool,
    // [NEW] Context limit for smart threshold recovery (default to 1M)
    pub context_limit: u32,
    // [NEW] MCP XML Bridge Buffer
    pub mcp_xml_buffer: String,
    pub in_mcp_xml: bool,
    // [FIX] Estimated prompt tokens for calibrator learning
    pub estimated_prompt_tokens: Option<u32>,
    // [FIX #859] Post-thinking interruption tracking
    pub has_thinking: bool,
    pub has_content: bool,
    pub message_count: usize, // [NEW v4.0.0] Message count for rewind detection
    pub client_adapter: Option<std::sync::Arc<dyn ClientAdapter>>, // [FIX] Remove Box, use Arc<dyn> directly
    // [FIX #MCP] Registered tool names for fuzzy matching
    pub registered_tool_names: Vec<String>,
    // [FIX #3379] Track whether any text_delta was emitted this turn (guard G7)
    pub text_delta_emitted_this_turn: bool,
    pub thinking_acc: crate::proxy::thinking_store::TurnAccumulator,
}

impl StreamingState {
    pub fn new() -> Self {
        Self {
            block_type: BlockType::None,
            block_index: 0,
            message_start_sent: false,
            message_stop_sent: false,
            used_tool: false,
            signatures: SignatureManager::new(),
            trailing_signature: None,
            web_search_query: None,
            grounding_chunks: None,
            // [IMPROVED] Initialize error recovery field
            parse_error_count: 0,
            last_valid_state: None,
            model_name: None,
            session_id: None,
            scaling_enabled: false,
            context_limit: 1_048_576, // Default to 1M
            mcp_xml_buffer: String::new(),
            in_mcp_xml: false,
            estimated_prompt_tokens: None,
            has_thinking: false,
            has_content: false,
            message_count: 0,
            client_adapter: None,
            registered_tool_names: Vec::new(),
            text_delta_emitted_this_turn: false,
            thinking_acc: crate::proxy::thinking_store::TurnAccumulator::new(),
        }
    }

    // [NEW] Set client adapter
    pub fn set_client_adapter(&mut self, adapter: Option<std::sync::Arc<dyn ClientAdapter>>) {
        self.client_adapter = adapter;
    }

    // [FIX #MCP] Set registered tool names for fuzzy matching
    pub fn set_registered_tool_names(&mut self, names: Vec<String>) {
        self.registered_tool_names = names;
    }

    /// Send SSE events
    pub fn emit(&self, event_type: &str, data: serde_json::Value) -> Bytes {
        let sse = format!(
            "event: {}\ndata: {}\n\n",
            event_type,
            serde_json::to_string(&data).unwrap_or_default()
        );
        Bytes::from(sse)
    }

    /// Send message_start event
    pub fn emit_message_start(&mut self, raw_json: &serde_json::Value) -> Bytes {
        if self.message_start_sent {
            return Bytes::new();
        }

        let usage = raw_json
            .get("usageMetadata")
            .and_then(|u| serde_json::from_value::<UsageMetadata>(u.clone()).ok())
            .map(|u| to_claude_usage(&u, self.scaling_enabled, self.context_limit));

        let mut message = json!({
            "id": raw_json.get("responseId")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| "msg_unknown"),
            "type": "message",
            "role": "assistant",
            "content": [],
            "model": raw_json.get("modelVersion")
                .and_then(|v| v.as_str())
                .unwrap_or(""),
            "stop_reason": null,
            "stop_sequence": null,
        });

        // Capture model name for signature cache
        if let Some(m) = raw_json.get("modelVersion").and_then(|v| v.as_str()) {
            self.model_name = Some(m.to_string());
        }

        if let Some(u) = usage {
            message["usage"] = json!(u);
        } else {
            message["usage"] = json!({
                "input_tokens": 0,
                "output_tokens": 0
            });
        }

        let result = self.emit(
            "message_start",
            json!({
                "type": "message_start",
                "message": message
            }),
        );

        self.message_start_sent = true;
        result
    }

    /// Start a new content block
    pub fn start_block(
        &mut self,
        block_type: BlockType,
        content_block: serde_json::Value,
    ) -> Vec<Bytes> {
        let mut chunks = Vec::new();
        if self.block_type != BlockType::None {
            chunks.extend(self.end_block());
        }

        chunks.push(self.emit(
            "content_block_start",
            json!({
                "type": "content_block_start",
                "index": self.block_index,
                "content_block": content_block
            }),
        ));

        self.block_type = block_type;
        chunks
    }

    /// End current content block
    pub fn end_block(&mut self) -> Vec<Bytes> {
        if self.block_type == BlockType::None {
            return vec![];
        }

        let mut chunks = Vec::new();

        // The temporary signature is sent at the end of the Thinking block (fallback to the session signature when the upstream has not issued it).
        // **Never invent sentry** - Sentinel is a "skip verification" switch rather than a fake legal signature.
        // Distributing it to the client will only pollute its history; and it appears 0/23 times in official traffic, it does not belong to the Antigravity protocol.
        // The recovery of the true signature is undertaken by the gateway-side SQL state machine (`hydrate`) and the final review `place_turn_signature`.
        if self.block_type == BlockType::Thinking {
            let signature = if self.signatures.has_pending() {
                self.signatures.consume()
            } else {
                self.session_id.as_deref().and_then(|sid| {
                    crate::proxy::SignatureCache::global().get_session_signature(sid)
                })
            };

            if let Some(sig) = signature {
                chunks.push(self.emit_delta("signature_delta", json!({ "signature": sig })));
            }
        }

        chunks.push(self.emit(
            "content_block_stop",
            json!({
                "type": "content_block_stop",
                "index": self.block_index
            }),
        ));

        self.block_index += 1;
        self.block_type = BlockType::None;

        chunks
    }

    /// Send delta event
    pub fn emit_delta(&self, delta_type: &str, delta_content: serde_json::Value) -> Bytes {
        let mut delta = json!({ "type": delta_type });
        if let serde_json::Value::Object(map) = delta_content {
            for (k, v) in map {
                delta[k] = v;
            }
        }

        self.emit(
            "content_block_delta",
            json!({
                "type": "content_block_delta",
                "index": self.block_index,
                "delta": delta
            }),
        )
    }

    /// Send end event
    pub fn emit_finish(
        &mut self,
        finish_reason: Option<&str>,
        usage_metadata: Option<&UsageMetadata>,
    ) -> Vec<Bytes> {
        let mut chunks = Vec::new();

        // Close last block
        chunks.extend(self.end_block());

        // Handle trailingSignature (B4/C3 scenario)
        // [FIX] Ending with thinking block (as start of message) only when no block has been sent yet
        // In fact, for the Claude protocol, if Text has already been sent, Thinking cannot be appended here.
        // The solution here is: just store the signature and no longer send the illegal last Thinking block.
        // The signature is automatically restored on the next round of requests via SignatureCache.
        if let Some(signature) = self.trailing_signature.take() {
            tracing::info!(
                "[Streaming] Captured trailing signature (len: {}), caching for session.",
                signature.len()
            );
            self.signatures.store(Some(signature));
            // No more appending chunks.push(self.emit("content_block_start", ...))
        }

        // Process grounding(web search) -> Convert to Markdown text block
        if self.web_search_query.is_some() || self.grounding_chunks.is_some() {
            let mut grounding_text = String::new();

            // 1. Process search terms
            if let Some(query) = &self.web_search_query {
                if !query.is_empty() {
                    grounding_text.push_str("\n\n---\n**🔍 Searched for you:**");
                    grounding_text.push_str(query);
                }
            }

            // 2. Process source links
            if let Some(chunks) = &self.grounding_chunks {
                let mut links = Vec::new();
                for (i, chunk) in chunks.iter().enumerate() {
                    if let Some(web) = chunk.get("web") {
                        let title = web
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Web source");
                        let uri = web.get("uri").and_then(|v| v.as_str()).unwrap_or("#");
                        links.push(format!("[{}] [{}]({})", i + 1, title, uri));
                    }
                }

                if !links.is_empty() {
                    grounding_text.push_str("\n\n**🌐 Source citation:**\n");
                    grounding_text.push_str(&links.join("\n"));
                }
            }

            let trimmed_grounding = grounding_text.trim();
            if !trimmed_grounding.is_empty() {
                // Send a new text block
                chunks.push(self.emit(
                    "content_block_start",
                    json!({
                        "type": "content_block_start",
                        "index": self.block_index,
                        "content_block": { "type": "text", "text": "" }
                    }),
                ));
                chunks.push(self.emit_delta("text_delta", json!({ "text": trimmed_grounding })));
                chunks.push(self.emit(
                    "content_block_stop",
                    json!({ "type": "content_block_stop", "index": self.block_index }),
                ));
                self.block_index += 1;
            }
        }

        // OK stop_reason
        let stop_reason = if self.used_tool {
            "tool_use"
        } else if finish_reason == Some("MAX_TOKENS") {
            "max_tokens"
        } else {
            "end_turn"
        };

        let usage = usage_metadata
            .map(|u| {
                // [FIX] Record actual token usage for calibrator learning
                // Now properly pairs estimated tokens from request with actual tokens from response
                if let (Some(estimated), Some(actual)) =
                    (self.estimated_prompt_tokens, u.prompt_token_count)
                {
                    if estimated > 0 && actual > 0 {
                        get_calibrator().record(estimated, actual);
                        tracing::debug!(
                            "[Calibrator] Recorded: estimated={}, actual={}, ratio={:.2}x",
                            estimated,
                            actual,
                            actual as f64 / estimated as f64
                        );
                    }
                }
                to_claude_usage(u, self.scaling_enabled, self.context_limit)
            })
            .unwrap_or(Usage {
                input_tokens: 0,
                output_tokens: 0,
                cache_read_input_tokens: None,
                cache_creation_input_tokens: None,
                server_tool_use: None,
            });

        chunks.push(self.emit(
            "message_delta",
            json!({
                "type": "message_delta",
                "delta": { "stop_reason": stop_reason, "stop_sequence": null },
                "usage": usage
            }),
        ));

        if !self.message_stop_sent {
            chunks.push(Bytes::from(
                "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n",
            ));
            self.message_stop_sent = true;
        }

        chunks
    }

    /// Mark tools used
    pub fn mark_tool_used(&mut self) {
        self.used_tool = true;
    }

    /// Get the current block type
    pub fn current_block_type(&self) -> BlockType {
        self.block_type
    }

    /// Get the current block index
    pub fn current_block_index(&self) -> usize {
        self.block_index
    }

    /// Store signature
    pub fn store_signature(&mut self, signature: Option<String>) {
        self.signatures.store(signature);
    }

    /// Set trailing signature
    pub fn set_trailing_signature(&mut self, signature: Option<String>) {
        self.trailing_signature = signature;
    }

    /// Get trailing signature (for checking only)
    pub fn has_trailing_signature(&self) -> bool {
        self.trailing_signature.is_some()
    }

    /// Handle SSE parsing errors to achieve graceful degradation
    ///
    /// When a parsing error occurs in an SSE stream:
    /// 1. Safely close the current block
    /// 2. Increment error counter
    /// 3. Output error information in debug mode
    #[allow(dead_code)] // Prepared for future error recovery implementation
    pub fn handle_parse_error(&mut self, raw_data: &str) -> Vec<Bytes> {
        let mut chunks = Vec::new();

        self.parse_error_count += 1;

        tracing::warn!(
            "[SSE-Parser] Parse error #{} occurred. Raw data length: {} bytes",
            self.parse_error_count,
            raw_data.len()
        );

        // Safely close the current block
        if self.block_type != BlockType::None {
            self.last_valid_state = Some(self.block_type);
            chunks.extend(self.end_block());
        }

        // Output detailed error information in Debug mode
        #[cfg(debug_assertions)]
        {
            let preview = if raw_data.len() > 100 {
                format!(
                    "{}...",
                    crate::proxy::mappers::common_utils::safe_truncate_str(raw_data, 100)
                )
            } else {
                raw_data.to_string()
            };
            tracing::debug!("[SSE-Parser] Failed chunk preview: {}", preview);
        }

        // Warn when the error rate is too high and try to send an error signal
        if self.parse_error_count > 3 {
            // Lower the threshold to notify users earlier
            tracing::error!(
                "[SSE-Parser] High error rate detected ({} errors). Stream may be corrupted.",
                self.parse_error_count
            );

            // [FIX] Explicitly signal error to client to prevent UI freeze
            // using standard SSE error event format
            // data: {"type": "error", "error": {...}}
            chunks.push(self.emit(
                "error",
                json!({
                    "type": "error",
                    "error": {
                        "type": "overloaded_error", // Use standard type
                        "message": "The network connection is unstable, please check your network or proxy settings.",
                    }
                }),
            ));
        }

        chunks
    }

    /// Reset error status (called after recovery)
    #[allow(dead_code)]
    pub fn reset_error_state(&mut self) {
        self.parse_error_count = 0;
        self.last_valid_state = None;
    }

    /// Get error count (for monitoring)
    #[allow(dead_code)]
    pub fn get_error_count(&self) -> usize {
        self.parse_error_count
    }
}
