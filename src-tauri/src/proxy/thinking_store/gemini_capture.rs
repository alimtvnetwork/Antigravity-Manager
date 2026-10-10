use super::*;
use serde_json::{json, Value};

pub fn capture_gemini_contents(store_key: &str, contents: &[Value]) {
    if !crate::proxy::config::is_thinking_store_enabled() || store_key.is_empty() {
        return;
    }
    for (c_idx, content) in contents.iter().enumerate() {
        let role = content.get("role").and_then(|v| v.as_str()).unwrap_or("");
        if role != "model" && role != "assistant" {
            continue;
        }
        if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
            let preceding_turn = if c_idx > 0 {
                contents.get(c_idx - 1)
            } else {
                None
            };
            let anchor = compute_causal_anchor(preceding_turn);
            capture_gemini_parts_with_anchor(store_key, parts, &anchor);
        }
    }
}

/// Capture client-supplied thinking, restore missing blocks, then prune compressed-away history.
///
/// Client histories usually have no real thinking (Claude/OpenAI). After a session is
/// warm in memory, this path is RAM-only: no SQLite open, no placeholder ingest, no prune
/// rewrite. JSON fill still copies stored thought text into the freshly built request.
pub fn hydrate_gemini_contents(store_key: &str, contents: &mut Vec<Value>) -> usize {
    hydrate_gemini_contents_with_model(store_key, contents, None)
}

pub fn hydrate_gemini_contents_with_model(
    store_key: &str,
    contents: &mut Vec<Value>,
    target_model: Option<&str>,
) -> usize {
    if store_key.is_empty() {
        return 0;
    }
    let store = ThinkingStore::global();
    store.touch_session(store_key);
    // 优先执行拓扑还原，保证历史已有记录对齐为真实真签名
    let restored = store.restore_gemini_contents_with_model(store_key, contents, target_model);
    // 只有在完成还原后，若仍有客户端自带的合法实质思考块，才安全吸纳进库
    if contents_have_capturable_thought(contents) {
        store.ingest_from_contents(store_key, contents);
    }
    store.prune_orphaned_records(store_key, contents);
    restored
}

pub(crate) fn is_model_or_assistant(content: &Value) -> bool {
    matches!(
        content.get("role").and_then(|v| v.as_str()),
        Some("model") | Some("assistant")
    )
}

pub(crate) fn contents_have_capturable_thought(contents: &[Value]) -> bool {
    for content in contents {
        if !is_model_or_assistant(content) {
            continue;
        }
        let Some(parts) = content.get("parts").and_then(|p| p.as_array()) else {
            continue;
        };
        for part in parts {
            let is_thought = part
                .get("thought")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !is_thought {
                continue;
            }
            let text = part.get("text").and_then(|t| t.as_str()).unwrap_or("");
            let sig = part
                .get("thoughtSignature")
                .or_else(|| part.get("thought_signature"))
                .and_then(|s| s.as_str());
            if is_capturable_thought(text, sig) {
                return true;
            }
        }
    }
    false
}

pub fn capture_gemini_parts(store_key: &str, parts: &[Value]) {
    capture_gemini_parts_with_anchor(store_key, parts, "root");
}

pub fn capture_gemini_parts_with_anchor(store_key: &str, parts: &[Value], anchor: &str) {
    let mut acc = TurnAccumulator::with_anchor(anchor);
    for part in parts {
        acc.ingest_part(part);
    }
    acc.commit(store_key);
}

pub fn capture_gemini_response(store_key: &str, response: &Value) {
    capture_gemini_response_with_preceding(store_key, response, None);
}

pub fn capture_gemini_response_with_preceding(
    store_key: &str,
    response: &Value,
    preceding: Option<&Value>,
) {
    let raw = response.get("response").unwrap_or(response);
    if let Some(parts) = raw
        .get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|p| p.as_array())
    {
        let anchor = compute_causal_anchor(preceding);
        capture_gemini_parts_with_anchor(store_key, parts, &anchor);
    }
}

/// 四大协议统一思考补齐管线：确保所有 Gemini contents 中的 model 轮次在开启思考时，必须具备合法的思考块与签名
pub fn finalize_gemini_contents_thinking(contents: &mut [Value], is_thinking_enabled: bool) {
    finalize_gemini_contents_thinking_with_model(contents, is_thinking_enabled, None);
}

pub fn finalize_gemini_contents_thinking_with_model(
    contents: &mut [Value],
    is_thinking_enabled: bool,
    target_model: Option<&str>,
) {
    // 预先计算每一轮的前置因果锚点 (causal anchor)，以便无 ID 的 Gemini 原生工具调用也能合成确定性 ID
    let anchors: Vec<String> = (0..contents.len())
        .map(|i| {
            let preceding = if i > 0 { contents.get(i - 1) } else { None };
            compute_causal_anchor(preceding)
        })
        .collect();

    for (msg_idx, msg) in contents.iter_mut().enumerate() {
        let _anchor = &anchors[msg_idx];
        let is_model = matches!(
            msg.get("role").and_then(|r| r.as_str()),
            Some("model") | Some("assistant")
        );

        if !is_model {
            // 非 model 轮次（如 user 轮次的 functionResponse）：清洗可能混入的误标签名
            if let Some(parts) = msg.get_mut("parts").and_then(|p| p.as_array_mut()) {
                for part in parts.iter_mut() {
                    if let Some(obj) = part.as_object_mut() {
                        obj.remove("thought_signature");
                        if obj.contains_key("functionResponse") {
                            obj.remove("thoughtSignature");
                        }
                    }
                }
            }
            continue;
        }

        if let Some(parts) = msg.get_mut("parts").and_then(|p| p.as_array_mut()) {
            let mut thinking_parts = Vec::new();
            let mut other_parts = Vec::new();

            for mut part in parts.drain(..) {
                if let Some(obj) = part.as_object_mut() {
                    // 统一清洗向 Google 发送的非标准蛇形字段
                    obj.remove("thought_signature");
                }
                // 铁律：只认 thought: true。functionCall 与纯正文都会携带 thoughtSignature，
                // 绝不能仅凭签名判定为思考块，否则会漏补首位 thought、关思考时误删工具。
                let is_thought = is_thought_part(&part);
                if is_thought {
                    thinking_parts.push(part);
                } else {
                    other_parts.push(part);
                }
            }

            let is_claude_turn = target_model
                .map(|m| m.to_lowercase().contains("claude"))
                .unwrap_or(false);

            // 1. 提取当前轮次已有合法的真实签名 (纯检查当前轮部件自带签名)
            // [DECOUPLE 2026-09-26] 不再假设「签名只在 functionCall 上」——官方新规：
            // 任何轮的第一个非思考 part（正文 text 或 functionCall）都可能携带签名。
            // 凡非思考 part 自带合法签名即为 turn_real_sig。
            let turn_real_sig: Option<String> = other_parts.iter().find_map(|p| {
                if p.get("functionCall").is_some() || p.get("text").is_some() {
                    let sig = p
                        .get("thoughtSignature")
                        .and_then(|s| s.as_str())
                        .filter(|s| is_real_signature(s))
                        .map(str::to_string);
                    sig.filter(|s| {
                        if is_claude_turn {
                            is_claude_signature(s)
                        } else {
                            is_likely_gemini_signature(s)
                        }
                    })
                } else {
                    None
                }
            });

            // 1.1 [FIX 2026-09-26] Gemini 目标下，纯 fc 轮（第一个非思考 part 是 functionCall）若自身
            //     未携带签名，必须从签名缓存按 tool_id 回填——官方/上游规则：
            //     「每轮第一个非思考 part 必须带 thought_signature，后续 part 可不带」。
            //     现场 400 铁证：`Function call is missing a thought_signature... call default_api:grep, position 4`，
            //     而该 tool_id 的签名其实已在 tool_signatures 缓存 (1384 B)，只是组装时从未按 id 回填。
            let cached_tool_sig: Option<String> = if !is_claude_turn && turn_real_sig.is_none() {
                other_parts.iter().find_map(|p| {
                    let fc = p.get("functionCall")?;
                    let id = fc.get("id").and_then(|v| v.as_str())?;
                    crate::proxy::SignatureCache::global()
                        .get_tool_signature(id)
                        .filter(|s| is_likely_gemini_signature(s))
                })
            } else {
                None
            };

            // 2. 签名归位（终审出站门禁 Gatekeeper）：
            // 核心铁律：签名只写在「该轮第一个非思考 part」上，其余 part 一律删除签名字段。
            // 依据 3 份官方报文 / 23 处签名：每轮至多 1 个签名、必落锚点；哨兵出现 0 次。
            if is_claude_turn {
                // Claude 模型：Anthropic 官方规范要求签名必须且只能在思考块上，工具调用绝不携带签名，更不塞假哨兵
                for part in other_parts.iter_mut() {
                    if let Some(obj) = part.as_object_mut() {
                        obj.remove("thoughtSignature");
                        obj.remove("thought_signature");
                    }
                }
            } else {
                // Gemini 原生：真签名优先原样保留，缺失才使用回填来源，都没有则留空。
                // 回填来源优先级：当前轮自带 turn_real_sig → 按 tool_id 查签名缓存 cached_tool_sig。
                // （客户端重传的历史 fc 轮不自带签名，但 tool_signatures 缓存里有，必须回填，
                //   否则「第一个非思考 part=fc 且无签名」会被上游 400 拒绝。）
                let fallback = turn_real_sig
                    .as_deref()
                    .filter(|s| is_likely_gemini_signature(s))
                    .or_else(|| {
                        cached_tool_sig
                            .as_deref()
                            .filter(|s| is_likely_gemini_signature(s))
                    });
                place_turn_signature(&mut other_parts, fallback);
            }

            // 3. 治理思考块 (thinking_parts)
            if is_thinking_enabled {
                if is_claude_turn {
                    // Claude 模型：Anthropic 引擎强制要求签名必须且只能在思考块上！
                    // 工具调用 (functionCall) 彻底剥离签名，绝不注入假哨兵
                    if thinking_parts.is_empty() {
                        if let Some(ref real_sig) = turn_real_sig {
                            if is_claude_signature(real_sig) {
                                let mut thought_obj = json!({
                                    "text": "...",
                                    "thought": true,
                                });
                                thought_obj["thoughtSignature"] =
                                    json!(ensure_google_claude_thought_signature(real_sig));
                                thinking_parts.push(thought_obj);
                            }
                        }
                    } else if let Some(ref real_sig) = turn_real_sig {
                        if is_claude_signature(real_sig) {
                            let wrapped = ensure_google_claude_thought_signature(real_sig);
                            for tp in thinking_parts.iter_mut() {
                                tp["thoughtSignature"] = json!(wrapped);
                            }
                        } else {
                            for tp in thinking_parts.iter_mut() {
                                if let Some(obj) = tp.as_object_mut() {
                                    obj.remove("thoughtSignature");
                                }
                            }
                        }
                    } else {
                        let mut valid_thinking = Vec::new();
                        for mut tp in thinking_parts.drain(..) {
                            let has_valid_sig = tp
                                .get("thoughtSignature")
                                .and_then(|s| s.as_str())
                                .map(|s| {
                                    s != SENTINEL_SIGNATURE
                                        && s.len() >= 50
                                        && is_claude_signature(s)
                                })
                                .unwrap_or(false);
                            if has_valid_sig {
                                if let Some(sig) =
                                    tp.get("thoughtSignature").and_then(|s| s.as_str())
                                {
                                    tp["thoughtSignature"] =
                                        json!(ensure_google_claude_thought_signature(sig));
                                }
                                valid_thinking.push(tp);
                            } else {
                                // 无合法签名的思考块（如 Gemini 历史思考块跨切至 Claude）：
                                // Anthropic 强制要求思考块签名必须合法。无合法 Claude 签名时，
                                // 将思考内容降级为带 <think> 标签的正文文本置于首位，既完整保留思考上下文，又彻底规避 Anthropic 400 校验报错！
                                if let Some(text) = tp.get("text").and_then(|t| t.as_str()) {
                                    if text != "..." && !text.trim().is_empty() {
                                        let wrapped = if text.trim_start().starts_with("<think>") {
                                            text.to_string()
                                        } else {
                                            format!("<think>\n{}\n</think>\n\n", text.trim())
                                        };
                                        other_parts.insert(0, json!({ "text": wrapped }));
                                    }
                                }
                            }
                        }
                        thinking_parts = valid_thinking;
                    }
                } else {
                    // Gemini 原生：思考块绝不携带签名（铁律 I4），且**绝不凭空注入**占位思考块。
                    //
                    // 官方报文里 functionCall 轮是"纯净"的 —— 只有 functionCall，没有任何
                    // thought part（33 个 model 轮里 thought × functionCall 共现 0 次）。
                    // 注入 {text:"...", thought:true} 会制造出官方从不产生的排列，
                    // 并把签名锚点从 parts[0] 挤到 parts[1]。
                    for tp in thinking_parts.iter_mut() {
                        if let Some(obj) = tp.as_object_mut() {
                            obj.remove("thoughtSignature");
                            obj.remove("thought_signature");
                        }
                    }
                }

                // 思考块始终强制排在最前面，其他部件紧随其后
                parts.extend(thinking_parts);
            } else {
                // 当思考模式为关时：
                // 1. 绝不主动注入任何占位思考块（如 "..."）；
                // 2. 若含有实质性思考内容的思考块，单次出站降级为普通文本以防丢失语义，摘除 thought: true 标记；
                // 3. 纯占位符则直接剔除，绝不上送 thought: true 结构
                for tp in thinking_parts {
                    let text = tp.get("text").and_then(|t| t.as_str()).unwrap_or("");
                    if is_meaningful_thought(text) {
                        let wrapped = if is_claude_turn {
                            if text.trim_start().starts_with("<think>") {
                                text.to_string()
                            } else {
                                format!("<think>\n{}\n</think>\n\n", text.trim())
                            }
                        } else {
                            text.to_string()
                        };
                        parts.push(json!({ "text": wrapped }));
                    }
                }
            }

            parts.extend(other_parts);
        }
    }
}
