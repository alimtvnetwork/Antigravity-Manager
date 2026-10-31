use super::*;

impl InboundThinkingPipeline {
    /// Unified and standardized management of tool call IDs (Pipeline First):
    /// Normalize the `ids` of all `functionCall` and `functionResponse` in contents,
    /// Uniformly align to the standard underlined form (`call_...`) to prevent signatures and reflection backfills from falling off due to client format drift.
    ///
    /// Returns the number of normalized IDs.
    pub fn normalize_tool_call_ids(contents: &mut [Value]) -> usize {
        let mut normalized_count = 0usize;
        for content in contents.iter_mut() {
            if let Some(parts) = content.get_mut("parts").and_then(|p| p.as_array_mut()) {
                for part in parts.iter_mut() {
                    if let Some(fc) = part.get_mut("functionCall") {
                        if let Some(id_val) = fc.get("id").and_then(|v| v.as_str()) {
                            let norm = crate::proxy::common::utils::normalize_tool_id(id_val);
                            if norm.as_ref() != id_val {
                                fc["id"] = serde_json::json!(norm.as_ref());
                                normalized_count += 1;
                            }
                        }
                    }
                    if let Some(fr) = part.get_mut("functionResponse") {
                        if let Some(id_val) = fr.get("id").and_then(|v| v.as_str()) {
                            let norm = crate::proxy::common::utils::normalize_tool_id(id_val);
                            if norm.as_ref() != id_val {
                                fr["id"] = serde_json::json!(norm.as_ref());
                                normalized_count += 1;
                            }
                        }
                    }
                }
            }
        }
        if normalized_count > 0 {
            tracing::debug!(
                "[InboundPipeline] Normalized {} tool call/response ID(s) to canonical 'call_...' format",
                normalized_count
            );
        }
        normalized_count
    }

    /// Normalize the role of the function response (`functionResponse`) wheel to the official Antigravity form.
    ///
    /// **Official form** (verified verbatim in 3 real samples): `functionResponse` is always located in `role: "model"`
    /// content, immediately after the `functionCall` content with the same role.
    /// The Gemini native protocol and the four major client protocols all put the tool receipt in `role: "user"`.
    ///
    /// **Actual measurement** (2026-09-26, `gemini-3.8-flash-tiered` @ daily, magic number receipt criterion):
    ///
    /// ```text
    /// fr @ role:user → 200, the receipt is consumed correctly
    /// fr @ role:model → 200, the receipt is consumed correctly
    /// fc + fr is the same as content @ model → 200, the receipt is consumed correctly
    /// ```
    ///
    /// Upstream is **fully tolerant** of both roles. Therefore, this normalization is "aligning the official form + stable prefix byte".
    /// Instead of "fixing 400"; at the same time, it is naturally **downward compatible** with two inbound forms: user / model.
    ///
    /// Behavior:
    ///
    /// 1. Pure receipt wheel (parts contains `functionResponse`, and has no non-receipt part) → changed to `role: "model"` in place;
    /// 2. Mixed round (receipt and visible text are in the same round) → cut into segments according to part type, and the receipt segment belongs to `model`,
    ///    The remaining segments maintain their original roles, and the relative order of part and content remain unchanged;
    /// 3. Receipts already in the `model` round are not touched (idempotent); rounds without receipts are not touched at all;
    /// 4. `inlineData` is regarded as "traveling media" - a group of people who follow its predecessor part.
    ///    Avoid mistaking `user [inlineData, text]` (the user posts a picture to ask a question) as a receipt wheel.
    ///
    /// Returns the number of rewritten content.
    pub fn normalize_function_response_roles(contents: &mut Vec<Value>) -> usize {
        /// Whether part carries the tool receipt itself.
        fn has_fr(p: &Value) -> bool {
            p.get("functionResponse").is_some()
        }
        /// Whether part is companion media only (no text, no receipt, no call).
        fn is_media(p: &Value) -> bool {
            (p.get("inlineData").is_some() || p.get("inline_data").is_some())
                && p.get("text").is_none()
                && !has_fr(p)
                && p.get("functionCall").is_none()
        }

        let mut rewritten = 0usize;
        let mut out: Vec<Value> = Vec::with_capacity(contents.len());

        for content in contents.drain(..) {
            let role = content
                .get("role")
                .and_then(|r| r.as_str())
                .unwrap_or("user")
                .to_string();

            let parts = match content.get("parts").and_then(|p| p.as_array()) {
                Some(p) if !p.is_empty() => p.clone(),
                _ => {
                    out.push(content);
                    continue;
                }
            };

            // Only processes "including receipt body" and the current round is not a model
            if !parts.iter().any(has_fr) || role == "model" {
                out.push(content);
                continue;
            }

            let all_response = parts.iter().all(|p| has_fr(p) || is_media(p));

            // Pure receipt wheel: only change the role, leaving the parts as they are.
            // Exception: the first content must not become a model (Gemini requires the conversation to start with user),
            // In fact, the receipt must follow the functionCall round, this is just for defense.
            if all_response {
                if out.is_empty() {
                    out.push(content);
                    continue;
                }
                let mut c = content;
                c["role"] = json!("model");
                out.push(c);
                rewritten += 1;
                continue;
            }

            // Mixed wheel: Cut into segments according to "receipt family/non-receipt family", keeping the relative order of parts
            let mut segments: Vec<(bool, Vec<Value>)> = Vec::new();
            for part in parts {
                let flag = if has_fr(&part) {
                    true
                } else if is_media(&part) {
                    // Follow-up media follows the pioneer group; without a pioneer, it is regarded as ordinary content
                    segments.last().map(|(f, _)| *f).unwrap_or(false)
                } else {
                    false
                };
                match segments.last_mut() {
                    Some((f, seg)) if *f == flag => seg.push(part),
                    _ => segments.push((flag, vec![part])),
                }
            }

            if segments.len() <= 1 {
                out.push(content);
                continue;
            }

            // If the first paragraph is a receipt family and there is no generated content before it, it will destroy "starting with user"——
            // At this time, change the paragraph to the original role (retaining the structure and order, just not aligning).
            let mut first = true;
            for (flag, seg) in segments {
                let seg_role = if flag {
                    if first && out.is_empty() {
                        role.as_str()
                    } else {
                        "model"
                    }
                } else {
                    role.as_str()
                };
                out.push(json!({ "role": seg_role, "parts": seg }));
                first = false;
            }
            rewritten += 1;
        }

        *contents = out;
        rewritten
    }
}
