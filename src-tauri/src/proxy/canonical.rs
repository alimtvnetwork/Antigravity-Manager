//! Canonical Intermediate Representation (IR) Envelope for all inbound protocols.
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CanonicalProtocol {
    OpenAI,
    Claude,
    Gemini,
    Codex,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_results: Option<Value>,
}

/// Imp-19: Canonical Request Envelope
/// Protocol-agnostic representation of an inbound LLM prompt payload before Gemini API adaptation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalRequestEnvelope {
    pub protocol: CanonicalProtocol,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    pub messages: Vec<CanonicalMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_budget: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Value>,
    #[serde(default)]
    pub metadata: serde_json::Map<String, Value>,
}

impl CanonicalRequestEnvelope {
    pub fn new(protocol: CanonicalProtocol, model: &str) -> Self {
        Self {
            protocol,
            model: model.to_string(),
            system_prompt: None,
            messages: Vec::new(),
            thinking_budget: None,
            temperature: None,
            top_p: None,
            stream: false,
            tools: None,
            metadata: serde_json::Map::new(),
        }
    }
}
