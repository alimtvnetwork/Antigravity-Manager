// Parts/payload-level sanitization methods (split from prompt_sanitizer.rs)
use super::patterns::{
    RE_CODE_BLOCK, RE_MULTI_NEWLINE, SELF_IDENTITY_BLOCK, SYSTEM_PROMPT_END_MARKERS,
};
use super::PromptSanitizer;
use serde_json::Value;

impl PromptSanitizer {
    fn sanitize_parts_core(parts: &mut Vec<Value>, system_scope: bool) -> usize {
        let mut cleaned_count = 0;

        // 0. Physically prune client-injected billing metadata parts (issue #3452)
        //    Model-agnostic: removed completely if matched.
        let before = parts.len();
        parts.retain(|part| {
            let is_thought = part
                .get("thought")
                .and_then(Value::as_bool)
                .unwrap_or(false)
                || part.get("thoughtSignature").is_some()
                || part.get("thought_signature").is_some();
            if is_thought {
                return true;
            }
            match part.get("text").and_then(Value::as_str) {
                Some(t) => !Self::is_billing_metadata(t),
                None => true,
            }
        });
        cleaned_count += before - parts.len();

        for part in parts.iter_mut() {
            if let Some(obj) = part.as_object_mut() {
                // Thought blocks are strictly protected by thoughtSignature and must remain byte-level immutable
                if obj.get("thought").and_then(Value::as_bool).unwrap_or(false)
                    || obj.contains_key("thoughtSignature")
                {
                    continue;
                }

                if let Some(text_val) = obj.get("text").and_then(Value::as_str) {
                    // Client identity mapping (exact match) - system prompt only, user turn text preserved
                    let mapped = if system_scope {
                        Self::normalize_client_identity(text_val)
                    } else {
                        text_val
                    };
                    // Header track (applies to all text): billing pseudo-headers and risky client fingerprint declarations
                    let cleaned = Self::clean_text(mapped);
                    // System track (system prompt only): strip boundary markers / official identity + normalize attribution
                    let cleaned = if system_scope {
                        Self::normalize_system_identity(&Self::strip_pipeline_markers(&cleaned))
                    } else {
                        cleaned
                    };
                    if cleaned != text_val {
                        obj.insert("text".to_string(), Value::String(cleaned));
                        cleaned_count += 1;
                    }
                }
            }
        }

        // Physically prune empty text parts generated after sanitization
        // Note: thought blocks and non-text parts (inlineData, functionCall, etc.) must remain intact
        parts.retain(|part| {
            let is_thought = part
                .get("thought")
                .and_then(Value::as_bool)
                .unwrap_or(false)
                || part.get("thoughtSignature").is_some()
                || part.get("thought_signature").is_some();
            if is_thought {
                return true;
            }
            if let Some(text) = part.get("text").and_then(Value::as_str) {
                !text.trim().is_empty()
            } else {
                true
            }
        });

        // Rule: ensure thought block is strictly at index 0 if present in current turn
        Self::ensure_thought_block_first(parts);

        cleaned_count
    }

    /// Content track sanitization: Header track only, NO identity normalization.
    /// Used for `contents` (user / model turns) - strictly preserves user queries.
    pub fn sanitize_parts(parts: &mut Vec<Value>) -> usize {
        Self::sanitize_parts_core(parts, false)
    }

    /// System track sanitization: Header track + System track, used only for `systemInstruction`.
    pub fn sanitize_system_parts(parts: &mut Vec<Value>) -> usize {
        Self::sanitize_parts_core(parts, true)
    }

    /// Ensures thought block is strictly at index 0 of parts array
    pub fn ensure_thought_block_first(parts: &mut Vec<Value>) {
        if parts.len() <= 1 {
            return;
        }

        // 铁律：只认 thought: true。若沿用"有签名即思考块"的启发式，会把带签名的正文
        // 误判为思考块并前移 —— 既改写 part 顺序（= 签名锚点语义），也让真正的前缀错位。
        let thought_idx = parts
            .iter()
            .position(|p| crate::proxy::thinking_store::is_thought_part(p));

        if let Some(idx) = thought_idx {
            if idx != 0 {
                let thought_part = parts.remove(idx);
                parts.insert(0, thought_part);
            }
        }
    }

    /// Pipeline node: Sanitizes systemInstruction and contents within proxy payload
    /// Supports top-level Gemini body as well as payload wrapped under `request` key.
    pub fn sanitize_gemini_payload(body: &mut Value) -> usize {
        let mut total_cleaned = 0;

        // 0. Deep clean '[undefined]' placeholder strings (injected by clients like Cherry Studio)
        //    Executed uniformly across all protocols.
        crate::proxy::mappers::common_utils::deep_clean_undefined(body, 0);

        // Compatibility: sanitize inner object if wrapped under 'request'
        if let Some(inner) = body.get_mut("request").and_then(Value::as_object_mut) {
            let mut inner_val = Value::Object(inner.clone());
            let count = Self::sanitize_gemini_payload_inner(&mut inner_val);
            if let Value::Object(new_inner) = inner_val {
                *inner = new_inner;
            }
            total_cleaned += count;
        }

        // Sanitize current level
        total_cleaned += Self::sanitize_gemini_payload_inner(body);
        total_cleaned
    }

    fn sanitize_gemini_payload_inner(body: &mut Value) -> usize {
        let mut cleaned_count = 0;

        // 1. Sanitize system prompt (systemInstruction)
        //    Executes System track: Header track + identity normalization
        if let Some(sys) = body
            .get_mut("systemInstruction")
            .and_then(Value::as_object_mut)
        {
            if let Some(parts) = sys.get_mut("parts").and_then(Value::as_array_mut) {
                cleaned_count += Self::sanitize_system_parts(parts);
            }

            // If parts is empty after sanitization, remove systemInstruction to avoid sending empty structure to Google
            let is_parts_empty = sys
                .get("parts")
                .and_then(Value::as_array)
                .map(|p| p.is_empty())
                .unwrap_or(true);

            if is_parts_empty {
                body.as_object_mut().map(|b| b.remove("systemInstruction"));
            }
        }

        // 2. Sanitize all dialogue turns (contents, including user and model turns)
        if let Some(contents) = body.get_mut("contents").and_then(Value::as_array_mut) {
            for turn in contents.iter_mut() {
                if let Some(parts) = turn.get_mut("parts").and_then(Value::as_array_mut) {
                    cleaned_count += Self::sanitize_parts(parts);
                }
            }

            // Turn protection: prune turns where parts became empty to prevent malformed payload
            contents.retain(|turn| {
                turn.get("parts")
                    .and_then(Value::as_array)
                    .map(|p| !p.is_empty())
                    .unwrap_or(true)
            });
        }

        cleaned_count
    }
}
