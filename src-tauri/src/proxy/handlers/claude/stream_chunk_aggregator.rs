//! Claude Stream Chunk Aggregator.
//! Buffers and reassembles partial Claude JSON SSE event fragments into complete
//! `content_block_delta` and `message_delta` event objects.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// An aggregated Claude SSE stream event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ClaudeStreamEvent {
    #[serde(rename = "message_start")]
    MessageStart { message: Value },
    #[serde(rename = "content_block_start")]
    ContentBlockStart { index: usize, content_block: Value },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta { index: usize, delta: Value },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop { index: usize },
    #[serde(rename = "message_delta")]
    MessageDelta { delta: Value, usage: Option<Value> },
    #[serde(rename = "message_stop")]
    MessageStop,
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "error")]
    Error { error: Value },
}

/// Accumulator for partial streaming events emitted by Claude endpoints.
#[derive(Debug, Default)]
pub struct ClaudeStreamChunkAggregator {
    current_text_buffer: String,
    current_thought_buffer: String,
    total_tokens_seen: u64,
}

impl ClaudeStreamChunkAggregator {
    /// Creates a new empty chunk aggregator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ingests a raw SSE line (`data: {...}`) and parses it into a ClaudeStreamEvent if valid.
    pub fn parse_line(&mut self, line: &str) -> Option<ClaudeStreamEvent> {
        let trimmed = line.trim();
        if !trimmed.starts_with("data:") {
            return None;
        }

        let json_part = trimmed.strip_prefix("data:")?.trim();
        if json_part == "[DONE]" {
            return Some(ClaudeStreamEvent::MessageStop);
        }

        let event: ClaudeStreamEvent = serde_json::from_str(json_part).ok()?;
        self.track_event(&event);
        Some(event)
    }

    /// Internal accumulator tracking tokens and text content across deltas.
    fn track_event(&mut self, event: &ClaudeStreamEvent) {
        if let ClaudeStreamEvent::ContentBlockDelta { delta, .. } = event {
            if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                self.current_text_buffer.push_str(text);
            }
            if let Some(thinking) = delta.get("thinking").and_then(|v| v.as_str()) {
                self.current_thought_buffer.push_str(thinking);
            }
        } else if let ClaudeStreamEvent::MessageDelta { usage, .. } = event {
            if let Some(u) = usage {
                if let Some(tokens) = u.get("output_tokens").and_then(|v| v.as_u64()) {
                    self.total_tokens_seen = tokens;
                }
            }
        }
    }

    /// Returns the accumulated full text string generated so far.
    pub fn full_text(&self) -> &str {
        &self.current_text_buffer
    }

    /// Returns the accumulated thinking block content so far.
    pub fn full_thinking(&self) -> &str {
        &self.current_thought_buffer
    }

    /// Total output tokens recorded from message delta usage objects.
    pub fn total_tokens(&self) -> u64 {
        self.total_tokens_seen
    }
}
