use super::*;
use sha2::{Digest, Sha256};

pub(crate) fn rec(thought: &str, visible: &str, tool_id: Option<&str>) -> ThinkingRecord {
    let tool_ids = tool_id.map(|id| vec![id.to_string()]).unwrap_or_default();
    let tool_names = if tool_id.is_some() {
        vec!["shell".to_string()]
    } else {
        Vec::new()
    };
    let fp = fingerprint(visible, &tool_ids, &tool_names);
    let mut hasher = Sha256::new();
    hasher.update(thought.as_bytes());
    hasher.update(visible.as_bytes());
    if let Some(t_id) = tool_id {
        hasher.update(t_id.as_bytes());
    }
    let hash_hex = format!("{:x}", hasher.finalize());
    let sig = format!("sig_{:0>56}", &hash_hex[..40]);
    ThinkingRecord {
        fingerprint: fp,
        thought: thought.to_string(),
        signature: Some(sig),
        tool_ids,
        tool_names,
        visible: visible.to_string(),
    }
}
