//! Tool call response formatter.
//! Normalizes tool call execution outputs into unified Canonical IR and
//! protocol-specific structures across Claude, OpenAI, and Gemini adapters.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Structured representation of a completed tool execution outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionOutcome {
    pub tool_call_id: String,
    pub tool_name: String,
    pub output: Value,
    pub is_error: bool,
}

impl ToolExecutionOutcome {
    /// Constructs a success tool outcome.
    pub fn success(
        tool_call_id: impl Into<String>,
        tool_name: impl Into<String>,
        output: Value,
    ) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            tool_name: tool_name.into(),
            output,
            is_error: false,
        }
    }

    /// Constructs an error tool outcome.
    pub fn error(
        tool_call_id: impl Into<String>,
        tool_name: impl Into<String>,
        error_message: impl Into<String>,
    ) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            tool_name: tool_name.into(),
            output: json!({ "error": error_message.into() }),
            is_error: true,
        }
    }

    /// Formats as an OpenAI chat completion tool message (`role: "tool"`).
    pub fn to_openai_message(&self) -> Value {
        let content_str = match &self.output {
            Value::String(s) => s.clone(),
            other => serde_json::to_string(other).unwrap_or_else(|_| "{}".to_string()),
        };

        json!({
            "role": "tool",
            "tool_call_id": self.tool_call_id,
            "name": self.tool_name,
            "content": content_str,
        })
    }

    /// Formats as a Claude `tool_result` content block.
    pub fn to_claude_content_block(&self) -> Value {
        let content_str = match &self.output {
            Value::String(s) => s.clone(),
            other => serde_json::to_string(other).unwrap_or_else(|_| "{}".to_string()),
        };

        json!({
            "type": "tool_result",
            "tool_use_id": self.tool_call_id,
            "content": content_str,
            "is_error": self.is_error,
        })
    }

    /// Formats as a Gemini `functionResponse` part.
    pub fn to_gemini_part(&self) -> Value {
        let response_obj = match &self.output {
            Value::Object(map) => Value::Object(map.clone()),
            Value::String(s) => json!({ "output": s }),
            other => json!({ "response": other }),
        };

        json!({
            "functionResponse": {
                "name": self.tool_name,
                "response": response_obj,
            }
        })
    }
}
