use super::*;
use serde_json::Value;
use sha2::{Digest, Sha256};

pub(crate) fn record_bytes(rec: &ThinkingRecord) -> usize {
    rec.thought.len() + rec.signature.as_ref().map(|s| s.len()).unwrap_or(0) + rec.visible.len()
}
fn hash_normalized_ws(hasher: &mut impl Digest, s: &str) {
    let mut need_space = false;
    for word in s.split_whitespace() {
        if need_space {
            hasher.update(b" ");
        }
        hasher.update(word.as_bytes());
        need_space = true;
    }
}
fn write_canonical_json(val: &Value, out: &mut Vec<u8>) {
    match val {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(b) => out.extend_from_slice(if *b { b"true" } else { b"false" }),
        Value::Number(n) => out.extend_from_slice(n.to_string().as_bytes()),
        Value::String(s) => {
            if let Ok(json_str) = serde_json::to_string(s) {
                out.extend_from_slice(json_str.as_bytes());
            } else {
                out.extend_from_slice(s.as_bytes());
            }
        }
        Value::Array(arr) => {
            out.push(b'[');
            for (i, elem) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                write_canonical_json(elem, out);
            }
            out.push(b']');
        }
        Value::Object(map) => {
            out.push(b'{');
            let mut sorted_keys: Vec<&String> = map.keys().collect();
            sorted_keys.sort();
            for (i, &key) in sorted_keys.iter().enumerate() {
                if i > 0 {
                    out.push(b',');
                }
                if let Ok(json_key) = serde_json::to_string(key) {
                    out.extend_from_slice(json_key.as_bytes());
                } else {
                    out.extend_from_slice(key.as_bytes());
                }
                out.push(b':');
                if let Some(v) = map.get(key) {
                    write_canonical_json(v, out);
                }
            }
            out.push(b'}');
        }
    }
}
pub fn canonical_json_hash(val: Option<&Value>) -> String {
    let mut out = Vec::with_capacity(128);
    match val {
        Some(v) => write_canonical_json(v, &mut out),
        None => out.extend_from_slice(b"{}"),
    }
    let mut hasher = Sha256::new();
    hasher.update(&out);
    let hex = format!("{:x}", hasher.finalize());
    hex[..12].to_string()
}

pub fn compute_causal_anchor(turn: Option<&Value>) -> String {
    let Some(content) = turn else {
        return "root".to_string();
    };

    let mut hasher = Sha256::new();
    let role = content.get("role").and_then(|r| r.as_str()).unwrap_or("");
    hasher.update(role.as_bytes());
    hasher.update([0xff]);

    if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
        for part in parts {
            if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                hasher.update(b"txt:");
                hasher.update(text.len().to_string().as_bytes());
                hasher.update([0xfe]);
                let prefix_len = text.len().min(48);
                hasher.update(&text.as_bytes()[..prefix_len]);
                hasher.update([0xfd]);
                let suffix_start = text.len().saturating_sub(48);
                hasher.update(&text.as_bytes()[suffix_start..]);
                hasher.update([0xfc]);
            } else if let Some(fr) = part.get("functionResponse") {
                hasher.update(b"fr:");
                let fr_name = fr.get("name").and_then(|n| n.as_str()).unwrap_or("");
                hasher.update(fr_name.as_bytes());
                hasher.update([0xfe]);
                if let Some(resp) = fr.get("response") {
                    let mut resp_bytes = Vec::new();
                    write_canonical_json(resp, &mut resp_bytes);
                    let resp_hash = Sha256::digest(&resp_bytes);
                    hasher.update(&resp_hash[..8]);
                }
                hasher.update([0xfd]);
            } else if let Some(fc) = part.get("functionCall") {
                hasher.update(b"fc:");
                let fc_name = fc.get("name").and_then(|n| n.as_str()).unwrap_or("");
                hasher.update(fc_name.as_bytes());
                hasher.update([0xfe]);
                let args_hash = canonical_json_hash(fc.get("args"));
                hasher.update(args_hash.as_bytes());
                hasher.update([0xfd]);
            }
        }
    }

    let hex = format!("{:x}", hasher.finalize());
    hex[..12].to_string()
}

pub fn synthesize_tool_id(
    tool_name: &str,
    args: Option<&Value>,
    causal_anchor: &str,
    call_index_in_turn: usize,
) -> String {
    let args_hash = canonical_json_hash(args);
    let anchor_clean = if causal_anchor.is_empty() {
        "root"
    } else {
        causal_anchor
    };
    format!(
        "call_{}_{}_{}_{}",
        tool_name, anchor_clean, args_hash, call_index_in_turn
    )
}

fn inspect_parts(parts: &[Value]) -> (String, Vec<String>, Vec<String>, String) {
    inspect_parts_with_anchor(parts, "root")
}
pub(crate) fn inspect_parts_with_anchor(
    parts: &[Value],
    anchor: &str,
) -> (String, Vec<String>, Vec<String>, String) {
    let mut visible = String::new();
    let mut thought = String::new();
    let mut tool_ids = Vec::new();
    let mut tool_names = Vec::new();
    let mut function_call_count = 0usize;
    for part in parts {
        let is_thought = part
            .get("thought")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
            if is_thought {
                thought.push_str(text);
            } else {
                visible.push_str(text);
            }
        }
        if let Some(fc) = part.get("functionCall") {
            let name = fc
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let explicit_id = fc
                .get("id")
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
                .map(|s| crate::proxy::common::utils::normalize_tool_id(s).into_owned());

            let synthetic = synthesize_tool_id(&name, fc.get("args"), anchor, function_call_count);
            function_call_count += 1;

            // 核心演进：全面转战因果伪哈希 ID！首位强制存入确定性 synthetic ID
            if !tool_ids.iter().any(|x| x == &synthetic) {
                tool_ids.push(synthetic);
            }
            if let Some(real_id) = explicit_id {
                if !tool_ids.iter().any(|x| x == &real_id) {
                    tool_ids.push(real_id);
                }
            }
            tool_names.push(name);
        }
    }
    (visible, tool_ids, tool_names, thought)
}

pub fn fingerprint(visible: &str, tool_ids: &[String], tool_names: &[String]) -> String {
    let mut hasher = Sha256::new();
    hash_normalized_ws(&mut hasher, visible);
    hasher.update([0xff]);
    for id in tool_ids {
        hasher.update(id.as_bytes());
        hasher.update([0xfe]);
    }
    hasher.update([0xfd]);
    for name in tool_names {
        hasher.update(name.as_bytes());
        hasher.update([0xfc]);
    }
    let hex = format!("{:x}", hasher.finalize());
    hex[..16].to_string()
}
