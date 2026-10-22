//! Claude streaming tool use frame assembler.
//! Aggregates fragmented `content_block_start` and `content_block_delta` SSE chunks
//! into unified tool execution requests with validated argument JSON.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledClaudeToolUse {
    pub index: usize,
    pub id: String,
    pub name: String,
    pub accumulated_json: String,
    pub is_complete: bool,
}

#[derive(Debug, Default)]
pub struct ClaudeToolUseStreamAssembler {
    active_blocks: HashMap<usize, AssembledClaudeToolUse>,
}

impl ClaudeToolUseStreamAssembler {
    pub fn new() -> Self {
        Self {
            active_blocks: HashMap::new(),
        }
    }

    /// Handles a `content_block_start` event.
    pub fn handle_start(&mut self, index: usize, id: String, name: String) {
        self.active_blocks.insert(
            index,
            AssembledClaudeToolUse {
                index,
                id,
                name,
                accumulated_json: String::new(),
                is_complete: false,
            },
        );
    }

    /// Appends a partial JSON delta from a `content_block_delta` event.
    pub fn handle_delta(&mut self, index: usize, partial_json: &str) {
        if let Some(tool) = self.active_blocks.get_mut(&index) {
            tool.accumulated_json.push_str(partial_json);
        }
    }

    /// Finalizes the tool use block on `content_block_stop`.
    /// Returns the completed tool call with parsed arguments if valid JSON.
    pub fn handle_stop(&mut self, index: usize) -> Option<Value> {
        if let Some(mut tool) = self.active_blocks.remove(&index) {
            tool.is_complete = true;
            let parsed_input: Value = serde_json::from_str(&tool.accumulated_json)
                .unwrap_or_else(|_| json!({ "raw": tool.accumulated_json }));

            Some(json!({
                "type": "tool_use",
                "id": tool.id,
                "name": tool.name,
                "input": parsed_input,
            }))
        } else {
            None
        }
    }
}
