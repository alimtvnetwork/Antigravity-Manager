// StreamingState (split from streaming.rs)
use super::types::{BlockType, SignatureManager};
use super::utils::to_claude_usage;
use crate::proxy::common::client_adapter::{ClientAdapter, SignatureBufferStrategy};
use crate::proxy::mappers::claude::models::*;
use crate::proxy::mappers::estimation_calibrator::get_calibrator;
use crate::proxy::SignatureCache;
use bytes::Bytes;
use serde_json::{json, Value};

/// 流式状态机

pub struct StreamingState {
    block_type: BlockType,
    pub block_index: usize,
    pub message_start_sent: bool,
    pub message_stop_sent: bool,
    used_tool: bool,
    signatures: SignatureManager,
    pub(crate) trailing_signature: Option<String>,
    pub web_search_query: Option<String>,
    pub grounding_chunks: Option<Vec<serde_json::Value>>,
    // [IMPROVED] Error recovery 状态追踪 (prepared for future use)
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
    // [NEW] MCP XML Bridge 缓冲区
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
            // [IMPROVED] 初始化 error recovery 字段
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

    /// 发送 SSE 事件
    pub fn emit(&self, event_type: &str, data: serde_json::Value) -> Bytes {
        let sse = format!(
            "event: {}\ndata: {}\n\n",
            event_type,
            serde_json::to_string(&data).unwrap_or_default()
        );
        Bytes::from(sse)
    }

    /// 发送 message_start 事件
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

    /// 开始新的内容块
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

    /// 结束当前内容块
    pub fn end_block(&mut self) -> Vec<Bytes> {
        if self.block_type == BlockType::None {
            return vec![];
        }

        let mut chunks = Vec::new();

        // Thinking 块结束时发送暂存的签名（上游未下发时回退到会话签名）。
        // **绝不发明哨兵** —— 哨兵是"跳过校验"开关而非假合法签名，
        // 下发给客户端只会污染其历史；且官方流量里出现 0/23 次，不属于 Antigravity 协议。
        // 真签名的恢复由网关侧 SQL 状态机（`hydrate`）与终审 `place_turn_signature` 承担。
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

    /// 发送 delta 事件
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

    /// 发送结束事件
    pub fn emit_finish(
        &mut self,
        finish_reason: Option<&str>,
        usage_metadata: Option<&UsageMetadata>,
    ) -> Vec<Bytes> {
        let mut chunks = Vec::new();

        // 关闭最后一个块
        chunks.extend(self.end_block());

        // 处理 trailingSignature (B4/C3 场景)
        // [FIX] 只有当还没有发送过任何块时, 才能以 thinking 块结束(作为消息的开头)
        // 实际上, 对于 Claude 协议, 如果已经发送过 Text, 就不能在此追加 Thinking。
        // 这里的解决方案是: 只存储签名, 不再发送非法的末尾 Thinking 块。
        // 签名会通过 SignatureCache 在下一轮请求中自动恢复。
        if let Some(signature) = self.trailing_signature.take() {
            tracing::info!(
                "[Streaming] Captured trailing signature (len: {}), caching for session.",
                signature.len()
            );
            self.signatures.store(Some(signature));
            // 不再追加 chunks.push(self.emit("content_block_start", ...))
        }

        // 处理 grounding(web search) -> 转换为 Markdown 文本块
        if self.web_search_query.is_some() || self.grounding_chunks.is_some() {
            let mut grounding_text = String::new();

            // 1. 处理搜索词
            if let Some(query) = &self.web_search_query {
                if !query.is_empty() {
                    grounding_text.push_str("\n\n---\n**🔍 已为您搜索：** ");
                    grounding_text.push_str(query);
                }
            }

            // 2. 处理来源链接
            if let Some(chunks) = &self.grounding_chunks {
                let mut links = Vec::new();
                for (i, chunk) in chunks.iter().enumerate() {
                    if let Some(web) = chunk.get("web") {
                        let title = web
                            .get("title")
                            .and_then(|v| v.as_str())
                            .unwrap_or("网页来源");
                        let uri = web.get("uri").and_then(|v| v.as_str()).unwrap_or("#");
                        links.push(format!("[{}] [{}]({})", i + 1, title, uri));
                    }
                }

                if !links.is_empty() {
                    grounding_text.push_str("\n\n**🌐 来源引文：**\n");
                    grounding_text.push_str(&links.join("\n"));
                }
            }

            let trimmed_grounding = grounding_text.trim();
            if !trimmed_grounding.is_empty() {
                // 发送一个新的 text 块
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

        // 确定 stop_reason
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

    /// 标记使用了工具
    pub fn mark_tool_used(&mut self) {
        self.used_tool = true;
    }

    /// 获取当前块类型
    pub fn current_block_type(&self) -> BlockType {
        self.block_type
    }

    /// 获取当前块索引
    pub fn current_block_index(&self) -> usize {
        self.block_index
    }

    /// 存储签名
    pub fn store_signature(&mut self, signature: Option<String>) {
        self.signatures.store(signature);
    }

    /// 设置 trailing signature
    pub fn set_trailing_signature(&mut self, signature: Option<String>) {
        self.trailing_signature = signature;
    }

    /// 获取 trailing signature (仅用于检查)
    pub fn has_trailing_signature(&self) -> bool {
        self.trailing_signature.is_some()
    }

    /// 处理 SSE 解析错误，实现优雅降级
    ///
    /// 当 SSE stream 中发生解析错误时:
    /// 1. 安全关闭当前 block
    /// 2. 递增错误计数器
    /// 3. 在 debug 模式下输出错误信息
    #[allow(dead_code)] // Prepared for future error recovery implementation
    pub fn handle_parse_error(&mut self, raw_data: &str) -> Vec<Bytes> {
        let mut chunks = Vec::new();

        self.parse_error_count += 1;

        tracing::warn!(
            "[SSE-Parser] Parse error #{} occurred. Raw data length: {} bytes",
            self.parse_error_count,
            raw_data.len()
        );

        // 安全关闭当前 block
        if self.block_type != BlockType::None {
            self.last_valid_state = Some(self.block_type);
            chunks.extend(self.end_block());
        }

        // Debug 模式下输出详细错误信息
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

        // 错误率过高时发出警告并尝试发送错误信号
        if self.parse_error_count > 3 {
            // 降低阈值,更早通知用户
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
                        "message": "网络连接不稳定，请检查您的网络或代理设置。",
                    }
                }),
            ));
        }

        chunks
    }

    /// 重置错误状态 (recovery 后调用)
    #[allow(dead_code)]
    pub fn reset_error_state(&mut self) {
        self.parse_error_count = 0;
        self.last_valid_state = None;
    }

    /// 获取错误计数 (用于监控)
    #[allow(dead_code)]
    pub fn get_error_count(&self) -> usize {
        self.parse_error_count
    }
}
