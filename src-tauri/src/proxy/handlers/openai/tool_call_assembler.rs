//! OpenAI Tool Call Delta Assembler.
//! Reassembles streaming partial `tool_calls` delta fragments into complete,
//! fully-formed JSON tool call specifications.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Partial or complete tool call structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssembledToolCall {
    pub id: String,
    pub call_type: String,
    pub function_name: String,
    pub arguments_buffer: String,
}

/// In-flight builder for aggregating streaming function deltas.
#[derive(Debug, Default)]
pub struct ToolCallAssembler {
    calls_by_index: HashMap<usize, AssembledToolCall>,
}

impl ToolCallAssembler {
    /// Creates a new assembler instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ingests a `tool_calls` delta JSON array from a streaming chunk choice.
    pub fn feed_delta(&mut self, tool_calls_delta: &Value) {
        let array = match tool_calls_delta.as_array() {
            Some(a) => a,
            None => return,
        };

        for item in array {
            let index = item.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            let call = self
                .calls_by_index
                .entry(index)
                .or_insert_with(|| AssembledToolCall {
                    id: String::new(),
                    call_type: "function".to_string(),
                    function_name: String::new(),
                    arguments_buffer: String::new(),
                });

            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                call.id = id.to_string();
            }

            if let Some(call_type) = item.get("type").and_then(|v| v.as_str()) {
                call.call_type = call_type.to_string();
            }

            if let Some(func) = item.get("function") {
                if let Some(name) = func.get("name").and_then(|v| v.as_str()) {
                    call.function_name.push_str(name);
                }
                if let Some(args) = func.get("arguments").and_then(|v| v.as_str()) {
                    call.arguments_buffer.push_str(args);
                }
            }
        }
    }

    /// Finalizes and exports all assembled tool calls ordered by index.
    pub fn finalize(&self) -> Vec<Value> {
        let mut sorted_keys: Vec<usize> = self.calls_by_index.keys().copied().collect();
        sorted_keys.sort_unstable();

        let mut out = Vec::with_capacity(sorted_keys.len());
        for key in sorted_keys {
            if let Some(c) = self.calls_by_index.get(&key) {
                out.push(json!({
                    "id": c.id,
                    "type": c.call_type,
                    "function": {
                        "name": c.function_name,
                        "arguments": c.arguments_buffer
                    }
                }));
            }
        }

        out
    }

    /// Returns true if any tool calls have been assembled.
    pub fn has_calls(&self) -> bool {
        !self.calls_by_index.is_empty()
    }
}
