//! Instance session trace UUID generator.
//! Produces structured, instance-tagged trace identifiers (`agm-inst-<id>-<uuid>`)
//! to correlate log events, proxy requests, and UI interactions to specific instance sessions.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSessionTrace {
    pub raw: String,
    pub instance_tag: String,
    pub timestamp_epoch_ms: u64,
    pub random_token: String,
}

pub struct SessionTraceUuid;

impl SessionTraceUuid {
    /// Generates a new unique trace token tagged with the instance identifier.
    pub fn generate(instance_tag: &str) -> String {
        let clean_tag = if instance_tag.trim().is_empty() {
            "default"
        } else {
            instance_tag.trim()
        };

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let random_part = format!("{:08x}{:08x}", rand_u32(), rand_u32());

        format!("agm-inst-{}-{}-{}", clean_tag, now_ms, random_part)
    }

    /// Parses an existing session trace string into its components.
    pub fn parse(trace_id: &str) -> Option<ParsedSessionTrace> {
        let parts: Vec<&str> = trace_id.split('-').collect();
        // Format: ["agm", "inst", "<tag>", "<timestamp>", "<random>"]
        if parts.len() >= 5 && parts[0] == "agm" && parts[1] == "inst" {
            let timestamp: u64 = parts[3].parse().ok()?;
            Some(ParsedSessionTrace {
                raw: trace_id.to_string(),
                instance_tag: parts[2].to_string(),
                timestamp_epoch_ms: timestamp,
                random_token: parts[4].to_string(),
            })
        } else {
            None
        }
    }
}

fn rand_u32() -> u32 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(42);
    nanos.wrapping_mul(1664525).wrapping_add(1013904223)
}
