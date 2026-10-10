use super::*;

impl InboundThinkingPipeline {
    /// 工具调用 ID 统一规范化治理（Pipeline First）：
    /// 规范化 contents 中所有的 `functionCall` 与 `functionResponse` 的 `id`，
    /// 统一对齐为带下划线的标准形态 (`call_...`)，杜绝客户端格式漂移导致的签名与思考回填脱落。
    ///
    /// 返回被规范化的 ID 数量。
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

    /// 把工具回执（`functionResponse`）轮的 role 归一化为官方 Antigravity 形态。
    ///
    /// **官方形态**（3 份实样本逐字核对）：`functionResponse` 恒位于 `role: "model"` 的
    /// content 中，紧跟同 role 的 `functionCall` content 之后。
    /// 而 Gemini 原生协议与四大客户端协议都把工具回执放在 `role: "user"`。
    ///
    /// **实测**（2026-09-26，`gemini-3.8-flash-tiered` @ daily，魔数回执判据）：
    ///
    /// ```text
    /// fr @ role:user             → 200，回执被正确消费
    /// fr @ role:model            → 200，回执被正确消费
    /// fc + fr 同 content @ model → 200，回执被正确消费
    /// ```
    ///
    /// 上游对两种 role **完全宽容**。故本归一化是「对齐官方形态 + 稳定前缀字节」，
    /// 而不是「修复 400」；同时天然**向下兼容** user / model 两种入站形态。
    ///
    /// 行为：
    ///
    /// 1. 纯回执轮（parts 含 `functionResponse`，且无非回执 part）→ 就地改为 `role: "model"`；
    /// 2. 混合轮（回执与可见文本同处一轮）→ 按 part 类型切段，回执段归 `model`、
    ///    其余段保持原 role，**part 相对顺序与 content 相对顺序原样保持**；
    /// 3. 已在 `model` 轮内的回执不动（幂等）；无回执的轮次完全不碰；
    /// 4. `inlineData` 视为「随行媒体」——跟随其前驱 part 的族群，
    ///    避免把 `user [inlineData, text]`（用户发图提问）误判为回执轮。
    ///
    /// 返回被改写的 content 数。
    pub fn normalize_function_response_roles(contents: &mut Vec<Value>) -> usize {
        /// part 是否携带工具回执本体。
        fn has_fr(p: &Value) -> bool {
            p.get("functionResponse").is_some()
        }
        /// part 是否仅为随行媒体（无文本、无回执、无调用）。
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

            // 只处理「含回执本体」且当前不是 model 的轮次
            if !parts.iter().any(has_fr) || role == "model" {
                out.push(content);
                continue;
            }

            let all_response = parts.iter().all(|p| has_fr(p) || is_media(p));

            // 纯回执轮：只改 role，parts 原样。
            // 例外：首条 content 不得变成 model（Gemini 要求对话以 user 开头），
            // 实际上回执必然跟在 functionCall 轮之后，此处仅作防御。
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

            // 混合轮：按「回执族 / 非回执族」切段，保持 parts 相对顺序
            let mut segments: Vec<(bool, Vec<Value>)> = Vec::new();
            for part in parts {
                let flag = if has_fr(&part) {
                    true
                } else if is_media(&part) {
                    // 随行媒体跟随前驱族群；无前驱则视为普通内容
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

            // 首段若为回执族且前面没有已产出的 content，会破坏「以 user 开头」——
            // 此时把该段改为原 role（保留结构与顺序，仅不做对齐）。
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
