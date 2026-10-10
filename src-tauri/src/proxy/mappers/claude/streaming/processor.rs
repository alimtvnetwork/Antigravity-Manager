// PartProcessor: dispatch + thinking (split from streaming.rs)
use super::state::StreamingState;
use super::types::BlockType;
use crate::proxy::mappers::claude::models::*;
use bytes::Bytes;
use serde_json::{json, Value};

/// Part 处理器
pub struct PartProcessor<'a> {
    state: &'a mut StreamingState,
}

impl<'a> PartProcessor<'a> {
    pub fn new(state: &'a mut StreamingState) -> Self {
        Self { state }
    }

    /// 处理单个 part
    pub fn process(&mut self, part: &GeminiPart) -> Vec<Bytes> {
        let mut chunks = Vec::new();
        // Gemini thought_signature is a base64 protobuf string (e.g. EiY... or EtY...)
        // DO NOT decode it to raw UTF-8 bytes: that corrupts ASCII-range protobufs (like UUID tags 0x12, 0x26, 0x0a, 0x24)
        // into control characters, shrinks length below MIN_SIGNATURE_LENGTH, and breaks Gemini signature validation.
        let signature = part.thought_signature.clone();

        // 1. FunctionCall 处理
        if let Some(fc) = &part.function_call {
            // 先处理 trailingSignature (B4/C3 场景)
            if self.state.has_trailing_signature() {
                chunks.extend(self.state.end_block());
                if let Some(trailing_sig) = self.state.trailing_signature.take() {
                    chunks.push(self.state.emit(
                        "content_block_start",
                        json!({
                            "type": "content_block_start",
                            "index": self.state.current_block_index(),
                            "content_block": { "type": "thinking", "thinking": "" }
                        }),
                    ));
                    chunks.push(
                        self.state
                            .emit_delta("thinking_delta", json!({ "thinking": "" })),
                    );
                    chunks.push(
                        self.state
                            .emit_delta("signature_delta", json!({ "signature": trailing_sig })),
                    );
                    chunks.extend(self.state.end_block());
                }
            }

            chunks.extend(self.process_function_call(fc, signature));
            // [FIX #859] Mark that we have received actual content (tool use)
            self.state.has_content = true;
            return chunks;
        }

        // 2. Text 处理
        if let Some(text) = &part.text {
            if part.thought.unwrap_or(false) {
                // Thinking
                chunks.extend(self.process_thinking(text, signature));
            } else {
                // 普通 Text
                chunks.extend(self.process_text(text, signature));
            }
        }

        // 3. InlineData (Image) 处理
        if let Some(img) = &part.inline_data {
            let mime_type = &img.mime_type;
            let data = &img.data;
            if !data.is_empty() {
                let markdown_img = format!("![image](data:{};base64,{})", mime_type, data);
                chunks.extend(self.process_text(&markdown_img, None));
            }
        }

        chunks
    }

    /// 处理 Thinking
    fn process_thinking(&mut self, text: &str, signature: Option<String>) -> Vec<Bytes> {
        let mut chunks = Vec::new();

        // 处理之前的 trailingSignature
        if self.state.has_trailing_signature() {
            chunks.extend(self.state.end_block());
            if let Some(trailing_sig) = self.state.trailing_signature.take() {
                chunks.push(self.state.emit(
                    "content_block_start",
                    json!({
                        "type": "content_block_start",
                        "index": self.state.current_block_index(),
                        "content_block": { "type": "thinking", "thinking": "" }
                    }),
                ));
                chunks.push(
                    self.state
                        .emit_delta("thinking_delta", json!({ "thinking": "" })),
                );
                chunks.push(
                    self.state
                        .emit_delta("signature_delta", json!({ "signature": trailing_sig })),
                );
                chunks.extend(self.state.end_block());
            }
        }

        // 开始或继续 thinking 块
        if self.state.current_block_type() != BlockType::Thinking {
            chunks.extend(self.state.start_block(
                BlockType::Thinking,
                json!({ "type": "thinking", "thinking": "" }),
            ));
        }

        // [FIX #859] Mark that we have received thinking content
        self.state.has_thinking = true;

        if !text.is_empty() {
            chunks.push(
                self.state
                    .emit_delta("thinking_delta", json!({ "thinking": text })),
            );
        }

        // [NEW] Apply Client Adapter Strategy
        let use_fifo = self
            .state
            .client_adapter
            .as_ref()
            .map(|a| a.signature_buffer_strategy() == SignatureBufferStrategy::Fifo)
            .unwrap_or(false);

        // [IMPROVED] Store signature to global cache
        if let Some(ref sig) = signature {
            // 1. Cache family if we know the model
            if let Some(model) = &self.state.model_name {
                SignatureCache::global().cache_thinking_family(sig.clone(), model.clone());
            }

            // 2. [NEW v3.3.17] Cache to session-based storage for tool loop recovery
            if let Some(session_id) = &self.state.session_id {
                // If FIFO strategy is enabled, use a unique index for each signature (e.g. timestamp or counter)
                // However, our cache implementation currently keys by session_id.
                // For FIFO, we might just rely on the fact that we are processing in order.
                // But specifically for opencode, it might be calling tools in parallel or sequence.

                SignatureCache::global().cache_session_signature(
                    session_id,
                    sig.clone(),
                    self.state.message_count,
                );
                tracing::debug!(
                    "[Claude-SSE] Cached signature to session {} (length: {}) [FIFO: {}]",
                    session_id,
                    sig.len(),
                    use_fifo
                );
            }

            tracing::debug!(
                "[Claude-SSE] Captured thought_signature from thinking block (length: {})",
                sig.len()
            );
        }

        // 暂存签名 (for local block handling)
        // If FIFO, we strictly follow the sequence. The default logic is effectively LIFO for a single turn
        // (store latest, consume at end).
        // For opencode, we just want to ensure we capture IT.
        self.state.store_signature(signature);

        chunks
    }
}
