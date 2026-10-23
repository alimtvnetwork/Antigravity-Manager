// Request identity and payload guards (split from common_utils.rs).
// Common utilities for request mapping across all protocols
// Provides unified grounding/networking logic

use serde_json::{json, Value};

/// Request configuration after grounding resolution

/// Check if two model strings are compatible (same family)
pub fn is_model_compatible(cached: &str, target: &str) -> bool {
    let c = cached.to_lowercase();
    let t = target.to_lowercase();

    if c == t {
        return true;
    }

    // All Claude series are universally compatible: All models belonging to the same Claude family will be directly judged to be signature compatible (for any future Claude 5/new models and variants)
    if c.contains("claude") && t.contains("claude") {
        return true;
    }

    // Gemini models: strict family match required for signatures
    if c.contains("gemini-1.5-pro") && t.contains("gemini-1.5-pro") {
        return true;
    }
    if c.contains("gemini-1.5-flash") && t.contains("gemini-1.5-flash") {
        return true;
    }
    if c.contains("gemini-2.0-flash") && t.contains("gemini-2.0-flash") {
        return true;
    }
    if c.contains("gemini-2.0-pro") && t.contains("gemini-2.0-pro") {
        return true;
    }
    if c.contains("gemini-3") && t.contains("gemini-3") {
        let c_flash = c.contains("flash");
        let t_flash = t.contains("flash");
        let c_pro = c.contains("pro");
        let t_pro = t.contains("pro");
        if c_flash == t_flash && c_pro == t_pro {
            return true;
        }
        if c_flash && t_flash {
            return true;
        }
        if c_pro && t_pro {
            return true;
        }
    }
    if c.contains("gemini-3.7") && t.contains("gemini-3.7") {
        return true;
    }

    false
}

/// Parse the official client fingerprint and return `(userAgent, ideType)`.
///
/// **Enterprise/GCP account** in official Go Worker (not `@gmail.com` / `@googlemail.com` email address)
/// Use `jetski` fingerprint and `antigravity` for the rest.
///
/// **Three adapters must share this function** - otherwise the same account will generate different fingerprints through different protocol entrances.
/// Historical defect: jetski simulation is only implemented in the Gemini path, and the Claude/OpenAI path is hard-coded
/// `"antigravity"`, causing fingerprint drift in corporate accounts.
pub fn resolve_official_fingerprint(
    token: Option<&crate::proxy::token_manager::ProxyToken>,
) -> (&'static str, &'static str) {
    let is_enterprise = token
        .map(|t| !t.email.ends_with("@gmail.com") && !t.email.ends_with("@googlemail.com"))
        .unwrap_or(false);
    if is_enterprise {
        ("jetski", "JETSKI")
    } else {
        ("antigravity", "ANTIGRAVITY")
    }
}

/// Construct the requestId of the official form: `agent/ {conversationId} / {unixMs} / {trajectoryId} / {step} `.
///
/// Official sample (3 documents verified verbatim), for example:
///
/// ```text
/// agent/a89a2006-72b4-470d-8282-3f1e4c88dc29/1790410048596/d3a2e3d2-21f9-411b-967f-53811898c2cd/14
/// ```
///
/// **Paragraph 2 `unixMs` Each request is different → Natural idempotent isolation** to avoid retrying the previous one
/// 429 / old cache. Historical defect: Claude’s path used `agent/antigravity/ {session[:8]} / {count} `,
/// **Excluding timestamp** - Retrying in the same session and the same round will get the exact same ID.
///
/// **Three adapters must share this function** - otherwise the same conversation will produce different ID forms through different entrances.
///
/// ## Relationship with `session_id` (important)
///
/// This function only reads `session_id`, never writes it**, and does not affect** its uniqueness:
/// The mechanism to prevent concurrent crosstalk between master and sub-agents is `thinking_store::derive_blended_session_id`
/// SHA256 multidimensional orthogonal hash (tenant + session semantic header + query sid + body sid + anchor),
/// It determines the key of thinking store / signature cache / prefix cache.
/// and outbound requestId are **two independent paths** (there is no code in the entire warehouse to reverse the session from requestId).
///
/// Session segments use a one-way hash derivation of `session_id` instead of the original text:
/// - The same session is stable (close to the official conversationId semantics);
/// - Do not expose the original text of the blended session_id inside the gateway to the upstream;
/// - The session_id of different agents/sessions is different → the derived value is different, and the isolation is maintained.
pub fn build_official_request_id(session_id: &str, step: u64) -> String {
    let ts = chrono::Utc::now().timestamp_millis();
    let sanitized = crate::proxy::thinking_store::sanitize_session_id(session_id);
    let conversation = if sanitized.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        let mut hasher = <sha2::Sha256 as sha2::Digest>::new();
        sha2::Digest::update(&mut hasher, sanitized.as_bytes());
        let digest = sha2::Digest::finalize(hasher);
        let hex = format!("{:x}", digest);
        hex[..16].to_string()
    };
    // The trajectory segment is unique per request, and together with unixMs ensures idempotent isolation
    let trajectory = &uuid::Uuid::new_v4().simple().to_string()[..8];
    format!("agent/{}/{}/{}/{}", conversation, ts, trajectory, step)
}

/// [JEIKCODE SYNTHETIC USER REMINDER]
/// In-place wrapping of system messages dynamically inserted mid-conversation into a `<system-reminder>` tag block.
/// The prompt words are in English, clearly informing the model: This content is a background reminder injected by the system layer, not user input in this round.
/// This ensures that the user's original query is completely transparently transmitted, while the global top-level systemInstruction remains absolutely frozen to stabilize the KV Cache.
pub fn wrap_in_system_reminder(content: &str) -> String {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.starts_with("<system-reminder>") && trimmed.ends_with("</system-reminder>") {
        return trimmed.to_string();
    }
    format!(
        "<system-reminder>\nBefore the user's request for this turn, the system provides the following reminder for your awareness. Please note that this is from prior system messages, not spoken by the user:\n{}\n</system-reminder>",
        trimmed
    )
}

/// [DEFENSE] General transfer message guaranteed text (gentle reminder to continue analysis to avoid triggering the Agent to mistakenly enter the modification stage)
pub const TRANSIT_DEFENSE_FALLBACK_TEXT: &str = "Please continue your analysis.";

/// [DEFENSE] Universal relay message protection node (protocol-independent)
/// Make sure that the last round of the message sent to Google Gemini strictly meets the specifications:
/// 1. Automatically compatible with tiled payloads or payloads containing "request" packaging;
/// 2. If contents is empty, append {"role": "user", "parts": [ {"text": TRANSIT_DEFENSE_FALLBACK_TEXT} ]};
/// 3. If the last round is "model" (missing user round), add {"role": "user", "parts": [ {"text": TRANSIT_DEFENSE_FALLBACK_TEXT} ]};
///    Exception: If the model wheel at the end carries functionCall / functionResponse (a tool wheel actively initiated by the model),
///    Considered as a legal intermediate state and not injected (otherwise it will be aligned with fr@model of normalize_function_response_roles
///    Fight, misjudge the official legal message as missing user turn, inject "Please continue your analysis." to cause an infinite loop in the tool);
/// 4. If the last round is "user" and its parts are empty, or contains only empty text / "(no content)" / "·" and no tools / pictures, the normalized filling is [ {"text": TRANSIT_DEFENSE_FALLBACK_TEXT} ];
/// 5. Fix the situation where parts is empty in the middle round to prevent Google from returning 400 "parts must not be empty".
pub fn ensure_gemini_payload_ends_with_user(body: &mut Value) -> bool {
    let contents = if let Some(contents) = body
        .get_mut("request")
        .and_then(|r| r.get_mut("contents"))
        .and_then(|c| c.as_array_mut())
    {
        contents
    } else if let Some(contents) = body.get_mut("contents").and_then(|c| c.as_array_mut()) {
        contents
    } else {
        return false;
    };

    let mut modified = false;

    // Defense 1: contents is entirely empty
    if contents.is_empty() {
        tracing::warn!("[Defense] Gemini contents array is empty, appending fallback user turn");
        contents.push(json!({
            "role": "user",
            "parts": [{ "text": TRANSIT_DEFENSE_FALLBACK_TEXT }]
        }));
        return true;
    }

    // Defense 2: Fix possible empty parts in history/intermediate rounds
    for turn in contents.iter_mut() {
        let is_model = turn
            .get("role")
            .and_then(|r| r.as_str())
            .map(|r| r == "model" || r == "assistant")
            .unwrap_or(false);
        if let Some(parts) = turn.get_mut("parts").and_then(|p| p.as_array_mut()) {
            if parts.is_empty() {
                modified = true;
                if is_model {
                    parts.push(json!({ "text": "..." }));
                } else {
                    parts.push(json!({ "text": TRANSIT_DEFENSE_FALLBACK_TEXT }));
                }
            }
        }
    }

    // Defense 3: Check the end of the turn.
    // Note: InboundThinkingPipeline::normalize_function_response_roles will align the pure receipt wheel to
    // role=model (the official Antigravity message convention is that fr is always in the model wheel). Therefore "the end is the model wheel"
    // It does not mean that the message is incomplete - if the round carries functionCall (the model actively initiates the tool round and waits for the receipt,
    // It is a legal intermediate state), and fake user words must not be injected, otherwise it will amplify into an infinite loop of the Agent tool.
    let need_append_user = if let Some(last_turn) = contents.last_mut() {
        let role = last_turn.get("role").and_then(|r| r.as_str()).unwrap_or("");
        if role == "model" || role == "assistant" {
            // The tool wheel is legal: it is not injected when the model wheel at the end contains functionCall or functionResponse
            let is_tool_turn = last_turn
                .get("parts")
                .and_then(|p| p.as_array())
                .map(|parts| {
                    parts.iter().any(|part| {
                        part.get("functionCall").is_some() || part.get("functionResponse").is_some()
                    })
                })
                .unwrap_or(false);
            !is_tool_turn
        } else {
            if let Some(parts) = last_turn.get_mut("parts").and_then(|p| p.as_array_mut()) {
                let has_substantive_part = parts.iter().any(|part| {
                    if part.get("functionCall").is_some()
                        || part.get("functionResponse").is_some()
                        || part.get("inlineData").is_some()
                        || part.get("fileData").is_some()
                    {
                        return true;
                    }
                    if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                        let t = text.trim();
                        !t.is_empty() && t != "(no content)" && t != "·"
                    } else {
                        false
                    }
                });

                if !has_substantive_part {
                    tracing::warn!(
                        "[Defense] Last user turn has no substantive content, normalizing to '{}'",
                        TRANSIT_DEFENSE_FALLBACK_TEXT
                    );
                    *parts = vec![json!({ "text": TRANSIT_DEFENSE_FALLBACK_TEXT })];
                    modified = true;
                }
            }
            false
        }
    } else {
        false
    };

    if need_append_user {
        tracing::warn!(
            "[Defense] Gemini payload ended with model turn, appending user turn with '{}'",
            TRANSIT_DEFENSE_FALLBACK_TEXT
        );
        contents.push(json!({
            "role": "user",
            "parts": [{ "text": TRANSIT_DEFENSE_FALLBACK_TEXT }]
        }));
        modified = true;
    }

    modified
}

/// Safely truncate string slices by the maximum number of bytes, ensuring that slice boundaries are strictly aligned on UTF-8 character boundaries.
/// If max_bytes happens to fall in the middle of multi-byte characters, it will automatically fall back to the left to the nearest legal character boundary.
pub fn safe_truncate_str(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Safely truncate a string slice by the maximum number of characters (Unicode scalar value).
/// If the total number of characters exceeds max_chars, valid slices corresponding to the first max_chars characters are intercepted.
pub fn safe_truncate_chars(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((byte_idx, _)) => &s[..byte_idx],
        None => s,
    }
}
