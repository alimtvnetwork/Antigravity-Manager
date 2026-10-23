// PartProcessor: text part handling (split from streaming.rs)
use super::state::StreamingState;
use super::types::BlockType;
use super::PartProcessor;
use crate::proxy::mappers::claude::models::*;
use bytes::Bytes;
use serde_json::{json, Value};

impl<'a> PartProcessor<'a> {
    pub(crate) fn process_text(&mut self, text: &str, signature: Option<String>) -> Vec<Bytes> {
        let mut chunks = Vec::new();

        // Empty text with signature - temporary storage
        if text.is_empty() {
            if signature.is_some() {
                self.state.set_trailing_signature(signature);
            }
            return chunks;
        }

        // [FIX #859] Mark that we have received actual content (text)
        self.state.has_content = true;

        // Processing previous trailingSignature
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

        // Non-empty text with signature - processed immediately
        if signature.is_some() {
            // [FIX] To protect the signature, the Text block where the signature is located is sent directly
            // Note: You must not open the thinking block here, as there may be non-thinking content before it.
            // In this case, we just need to make sure that the signature is cached in the state.
            self.state.store_signature(signature);

            chunks.extend(
                self.state
                    .start_block(BlockType::Text, json!({ "type": "text", "text": "" })),
            );
            chunks.push(self.state.emit_delta("text_delta", json!({ "text": text })));
            chunks.extend(self.state.end_block());

            return chunks;
        }

        // Ordinary text (without signature)

        // [NEW] MCP XML Bridge: Intercept and parse <mcp__...> tags
        if text.contains("<mcp__") || self.state.in_mcp_xml {
            self.state.in_mcp_xml = true;
            self.state.mcp_xml_buffer.push_str(text);

            // Check if we have a complete tag in the buffer
            if self.state.mcp_xml_buffer.contains("</mcp__")
                && self.state.mcp_xml_buffer.contains('>')
            {
                let buffer = self.state.mcp_xml_buffer.clone();
                if let Some(start_idx) = buffer.find("<mcp__") {
                    if let Some(tag_end_idx) = buffer[start_idx..].find('>') {
                        let actual_tag_end = start_idx + tag_end_idx;
                        let tool_name = &buffer[start_idx + 1..actual_tag_end];
                        let end_tag = format!("</{}>", tool_name);

                        if let Some(close_idx) = buffer.find(&end_tag) {
                            let input_str = &buffer[actual_tag_end + 1..close_idx];
                            let input_json: serde_json::Value =
                                serde_json::from_str(input_str.trim())
                                    .unwrap_or_else(|_| json!({ "input": input_str.trim() }));

                            // Construct and send tool_use
                            let fc = FunctionCall {
                                name: tool_name.to_string(),
                                args: Some(input_json),
                                id: Some(format!("{}-xml", tool_name)),
                            };

                            let tool_chunks = self.process_function_call(&fc, None);

                            // Clean buffer and reset state
                            self.state.mcp_xml_buffer.clear();
                            self.state.in_mcp_xml = false;

                            // Handle non-XML text that may exist before the tag
                            if start_idx > 0 {
                                let prefix_text = &buffer[..start_idx];
                                // No recursion here. Directly emit the previous text block.
                                if self.state.current_block_type() != BlockType::Text {
                                    chunks.extend(self.state.start_block(
                                        BlockType::Text,
                                        json!({ "type": "text", "text": "" }),
                                    ));
                                }
                                chunks.push(
                                    self.state
                                        .emit_delta("text_delta", json!({ "text": prefix_text })),
                                );
                            }

                            chunks.extend(tool_chunks);

                            // Handle non-XML text that may exist after tags
                            let suffix = &buffer[close_idx + end_tag.len()..];
                            if !suffix.is_empty() {
                                // Process suffix content recursively
                                chunks.extend(self.process_text(suffix, None));
                            }

                            return chunks;
                        }
                    }
                }
            }
            // While in XML, don't emit text deltas
            return vec![];
        }

        // [FIX #3379] call:default_api:* leakage recovery bridge
        // Attempt to recover leaked tool-call text as a proper tool_use block.
        // Strict Fail-Closed: any unmet guard falls through to normal text_delta.
        if let Some(recovery_chunks) = self.try_recover_call_default_api_text(text) {
            return recovery_chunks;
        }

        if self.state.current_block_type() != BlockType::Text {
            chunks.extend(
                self.state
                    .start_block(BlockType::Text, json!({ "type": "text", "text": "" })),
            );
        }

        self.state.text_delta_emitted_this_turn = true;
        chunks.push(self.state.emit_delta("text_delta", json!({ "text": text })));

        chunks
    }
}
