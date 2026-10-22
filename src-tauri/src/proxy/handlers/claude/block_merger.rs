//! Claude Content Block Merger.
//! Merges fragmented Claude streaming block deltas into unified thinking and content segments.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClaudeMergedBlocks {
    pub thinking_text: String,
    pub content_text: String,
    pub stop_reason: Option<String>,
}

/// Accumulates streaming Claude events (content_block_delta, content_block_start, message_delta).
pub fn merge_claude_stream_deltas(events: &[Value]) -> ClaudeMergedBlocks {
    let mut merged = ClaudeMergedBlocks::default();

    for ev in events {
        let ev_type = ev.get("type").and_then(|t| t.as_str()).unwrap_or_default();

        match ev_type {
            "content_block_delta" => {
                if let Some(delta) = ev.get("delta") {
                    let delta_type = delta
                        .get("type")
                        .and_then(|t| t.as_str())
                        .unwrap_or_default();

                    if delta_type == "thinking_delta" {
                        if let Some(th) = delta.get("thinking").and_then(|t| t.as_str()) {
                            merged.thinking_text.push_str(th);
                        }
                    } else if delta_type == "text_delta" {
                        if let Some(txt) = delta.get("text").and_then(|t| t.as_str()) {
                            merged.content_text.push_str(txt);
                        }
                    }
                }
            }
            "message_delta" => {
                if let Some(stop) = ev
                    .get("delta")
                    .and_then(|d| d.get("stop_reason"))
                    .and_then(|s| s.as_str())
                {
                    merged.stop_reason = Some(stop.to_string());
                }
            }
            _ => {}
        }
    }

    merged
}
