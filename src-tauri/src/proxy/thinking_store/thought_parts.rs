use super::*;
use serde_json::{json, Value};

const PLACEHOLDER_THOUGHTS: &[&str] = &[
    "...",
    "·",
    ".",
    "···",
    "[undefined]",
    "Applying tool decisions and generating response...",
];

/// Server-side auto thinking. Clients usually omit thinking config;
/// if any of `claude` / `flash` / `pro` / `agent` appears in a model id
/// (requested or mapped), the proxy enables thoughts + signature restore.
/// Determine whether a Part is a thinking block.
///
/// **Iron rule: Only recognize `thought == true`. **
///
/// It must not degenerate into "with `thoughtSignature` and no `functionCall`/`functionResponse`" -
/// The real Antigravity message will hang the signature on the **plain text part** (see the official invariant
/// `.workbuddy/outputs/correct-assembly-spec.md`). The old heuristic misclassified visible responses as thought blocks,
/// Then during the backfilling stage, the text was rewritten as "Internal Thoughts" and the only signature of the round was discarded - this is exactly
/// The root cause of the "Signature Stripping → Cross-Request Infinite Loop" accident.
#[inline]
pub fn is_thought_part(part: &Value) -> bool {
    part.get("thought")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Return the signature to the first non-thinking part of the round, and delete the signature field in the remaining parts.
///
/// Four hard constraints summarized from official reports:
///
/// 1. Anchor = the first `thought != true` part of the round, **not** hard-coded `parts[0]`
///    (When there is a thinking block in this round, the official signature falls in `parts[1]`);
/// 2. `thought: true` / `functionResponse` / remaining part - the field must be **absent**, not an empty string;
/// 3. When the anchor comes with a real signature, it is **retained as it is** (covering the two arrangements of "pure text wheel" and "thinking block + text wheel");
/// 4. When the anchor has no signature and `fallback_sig` is also empty - **write nothing**. Officials are absent from "On the Flywheel".
///    Missing signatures are tolerated and sentinels are never invented.
///
/// This function **never rearranges or inserts** part - the order is the anchor semantics.
///
/// Returns the final signature written to the anchor (if any).
pub fn place_turn_signature(parts: &mut [Value], fallback_sig: Option<&str>) -> Option<String> {
    // 1. Anchor point = the first non-thinking part; if you think about it throughout the round, there will be no anchor point in this round.
    let anchor = parts.iter().position(|p| !is_thought_part(p))?;

    // 2. The signature that comes with the anchor must be removed before clearing it.
    let own_sig = parts[anchor]
        .get("thoughtSignature")
        .or_else(|| parts[anchor].get("thought_signature"))
        .and_then(|s| s.as_str())
        .filter(|s| is_real_signature(s))
        .map(str::to_string);

    // 3. Clear all of them to ensure that the signature field of the non-anchor part is indeed "absent"
    for part in parts.iter_mut() {
        if let Some(obj) = part.as_object_mut() {
            obj.remove("thoughtSignature");
            obj.remove("thought_signature");
        }
    }

    // 4. functionResponse never carries a signature
    if parts[anchor].get("functionResponse").is_some() {
        return None;
    }

    // 5. The anchor point comes with priority. If it is missing, the backfill source will be used (cross-protocol path)
    let sig = own_sig.or_else(|| {
        fallback_sig
            .filter(|s| is_real_signature(s))
            .map(str::to_string)
    })?;

    // Authoritative anti-cracking: If the signature is a raw binary protobuf (first byte 0x12), it must be converted to standard Base64 encoding before being sent to Gemini!
    let final_sig = if sig.as_bytes().first() == Some(&0x12) {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(sig.as_bytes())
    } else {
        sig
    };
    parts[anchor]["thoughtSignature"] = json!(final_sig);

    // Reverse warehousing optimization: If the anchor is a tool call, reversely update/patch back the signature cache and SQLite tool_signatures library!
    if let Some(fc) = parts[anchor].get("functionCall") {
        if let Some(id) = fc.get("id").and_then(|v| v.as_str()) {
            crate::proxy::SignatureCache::global().cache_tool_signature(id, final_sig.clone());
        }
    }

    Some(final_sig)
}

pub fn is_placeholder_thought(s: &str) -> bool {
    let t = s.trim();
    t.is_empty()
        || PLACEHOLDER_THOUGHTS.contains(&t)
        || t.chars().all(|c| c == '.' || c == '·' || c == '…')
}

pub fn is_meaningful_thought(thought: &str) -> bool {
    let t = thought.trim();
    if t.is_empty() || is_placeholder_thought(t) {
        return false;
    }
    // Intercept pseudo-thinking tags and client-side placeholder dirty data
    let stripped = t
        .trim_start_matches("<think>")
        .trim_end_matches("</think>")
        .trim_start_matches("Thinking Process:")
        .trim_start_matches("Thinking Process")
        .trim_start_matches("[Thinking]")
        .trim();
    if stripped.is_empty()
        || stripped.eq_ignore_ascii_case("none")
        || stripped.eq_ignore_ascii_case("null")
        || stripped.eq_ignore_ascii_case("undefined")
        || is_placeholder_thought(stripped)
    {
        return false;
    }
    true
}

/// Extract and cut off the `<think>...</think>` tag content in the text (used for cross-model self-healing and text thinking text re-improvement)
/// Supports any upper and lower case (<think> / <THINK>), seamless stitching of front and rear text, and tolerance for unclosed tags
/// Return `Some((thought, remaining_visible))`
pub fn extract_think_tags(text: &str) -> Option<(String, String)> {
    let lower = text.to_lowercase();
    let start_tag = "<think>";
    let end_tag = "</think>";

    if let Some(start_pos) = lower.find(start_tag) {
        let after_start = start_pos + start_tag.len();
        if let Some(end_rel) = lower[after_start..].find(end_tag) {
            let end_pos = after_start + end_rel;
            let thought = text[after_start..end_pos].trim().to_string();
            let before = text[..start_pos].trim();
            let after = text[end_pos + end_tag.len()..].trim();
            let visible = if before.is_empty() {
                after.to_string()
            } else if after.is_empty() {
                before.to_string()
            } else {
                format!("{}\n\n{}", before, after)
            };
            return Some((thought, visible));
        } else {
            // Unclosed tag tolerance: truncate to end of text
            let thought = text[after_start..].trim().to_string();
            let before = text[..start_pos].trim();
            return Some((thought, before.to_string()));
        }
    }
    None
}

pub(crate) fn is_capturable_thought(thought: &str, signature: Option<&str>) -> bool {
    // Placeholder or plain blank thoughts are never captured as new persistent thought records!
    if is_placeholder_thought(thought) || thought.trim().is_empty() {
        // [DECOUPLE 2026-09-26] Official new rule: Any model wheel may return a signature (text wheel / tool wheel / pure thinking wheel).
        // Pure fc rounds (no thinking text) must also be entered into the database as long as they carry real signatures - otherwise the signatures will not be found in the next round of anchor backfill.
        return signature.is_some_and(is_real_signature);
    }
    if signature.is_some_and(is_real_signature) {
        return true;
    }
    is_meaningful_thought(thought)
}
pub(crate) fn normalize_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut need_space = false;
    for word in s.split_whitespace() {
        if need_space {
            out.push(' ');
        }
        out.push_str(word);
        need_space = true;
    }
    out
}
pub(crate) fn turn_needs_restore(parts: &[Value], existing_thought: &str) -> bool {
    if is_placeholder_thought(existing_thought) {
        return true;
    }
    // Sentinel / missing thought signature still needs ThinkingStore or tool-sig alignment.
    let thought_sig_ok = parts.iter().any(|p| {
        p.get("thought").and_then(|v| v.as_bool()).unwrap_or(false)
            && p.get("thoughtSignature")
                .or_else(|| p.get("thought_signature"))
                .and_then(|s| s.as_str())
                .is_some_and(is_real_signature)
    });
    if !thought_sig_ok {
        return true;
    }
    let mut saw_function_call = false;
    for part in parts {
        if part.get("functionCall").is_some() {
            saw_function_call = true;
            if !part_has_signature(part) {
                return true;
            }
        }
    }
    if saw_function_call {
        return false;
    }
    !parts.iter().any(part_has_signature)
}
pub(crate) fn part_has_signature(part: &Value) -> bool {
    part.get("thoughtSignature")
        .or_else(|| part.get("thought_signature"))
        .and_then(|s| s.as_str())
        .is_some_and(is_real_signature)
}

/// Imp-80: Thinking Block Graceful Healing
/// Ensures empty or malformed thinking blocks in parts array are gracefully backfilled with placeholder content
/// to satisfy upstream Gemini structural constraints without losing conversation flow.
pub fn heal_empty_thought_parts(parts: &mut [Value]) -> usize {
    let mut healed = 0;
    for part in parts.iter_mut() {
        if is_thought_part(part) {
            let is_empty = part
                .get("text")
                .and_then(|t| t.as_str())
                .map(|s| s.trim().is_empty())
                .unwrap_or(true);
            if is_empty {
                if let Some(obj) = part.as_object_mut() {
                    obj.insert("text".to_string(), json!("..."));
                    healed += 1;
                }
            }
        }
    }
    healed
}
