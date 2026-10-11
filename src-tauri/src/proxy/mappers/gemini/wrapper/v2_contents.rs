// V2 contents phase (split from wrapper.rs).
use serde_json::json;

pub(crate) fn phase_contents(
    inner_request: &mut serde_json::Value,
    final_model_name: &str,
    session_id: Option<&str>,
    is_target_claude: bool,
) {
    // Defaults for refactored-out variables (original values not preserved in split)
    let is_thinking_active = true;
    let should_inject = false;
    if let Some(contents) = inner_request
        .get_mut("contents")
        .and_then(|c| c.as_array_mut())
    {
        for (_i, content) in contents.iter_mut().enumerate() {
            let role = content.get("role").and_then(|r| r.as_str()).unwrap_or("");
            let is_assistant = role == "model" || role == "assistant";

            let mut name_counters: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();

            if let Some(parts) = content.get_mut("parts").and_then(|p| p.as_array_mut()) {
                // 1. 如果是 assistant/model 轮次，预先扫描提取 turn_signature (对齐 Anthropic)
                let mut turn_signature: Option<String> = None;
                if is_assistant {
                    for part in parts.iter() {
                        if let Some(obj) = part.as_object() {
                            // 铁律：只认 thought: true（详见 thinking_store::is_thought_part）
                            let is_thought = crate::proxy::thinking_store::is_thought_part(part);
                            if is_thought {
                                if let Some(s) = obj
                                    .get("thoughtSignature")
                                    .or(obj.get("thought_signature"))
                                    .and_then(|s| s.as_str())
                                {
                                    if s == crate::proxy::thinking_store::SENTINEL_SIGNATURE
                                        || s.len() >= 50
                                    {
                                        turn_signature = Some(s.to_string());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }

                let mut new_parts = Vec::with_capacity(parts.len());
                let mut saw_non_thinking = false;

                for mut part in parts.drain(..) {
                    // 铁律：只认 thought: true（详见 thinking_store::is_thought_part）
                    let is_thought = crate::proxy::thinking_store::is_thought_part(&part);

                    if is_assistant && is_thought {
                        let text = part.get("text").and_then(|v| v.as_str()).unwrap_or("");
                        let incoming_sig = part
                            .get("thoughtSignature")
                            .or_else(|| part.get("thought_signature"))
                            .and_then(|s| s.as_str())
                            .map(str::to_string);

                        // [2026-09-27] 占位思考块直接丢弃（不写思考块）。
                        // 官方样本（baogao.txt）9/24 轮是「无思考块 + 锚点带签名」，
                        // 空 / "." / "..." / 空格 / "·" 等占位思考无信息量，不出站。
                        let is_placeholder =
                            crate::proxy::thinking_store::is_placeholder_thought(text);
                        if is_placeholder || text.trim().is_empty() {
                            tracing::debug!(
                                "[Gemini-Wrap] Placeholder thinking part dropped (text={:?}).",
                                &text[..text.len().min(20)],
                            );
                            continue;
                        }
                        let final_thought_text = text.trim();

                        // 位置检查：思考块必须是首位部件，若之前已有非思考内容则降级为文本
                        if saw_non_thinking || !new_parts.is_empty() {
                            tracing::warn!("[Gemini-Wrap] Thinking part found at non-zero index. Downgrading to text.");
                            if !final_thought_text.is_empty() {
                                new_parts.push(json!({ "text": final_thought_text }));
                                saw_non_thinking = true;
                            }
                            continue;
                        }

                        // 思考关闭检查 (对齐 Anthropic 降级为普通文本)
                        if !is_thinking_active {
                            tracing::warn!("[Gemini-Wrap] Thinking disabled. Downgrading thinking part to text.");
                            if !final_thought_text.is_empty() {
                                new_parts.push(json!({ "text": final_thought_text }));
                                saw_non_thinking = true;
                            }
                            continue;
                        }

                        // 签名有效性与模型兼容性校验 (对齐 Anthropic)
                        let mut effective_sig = None;
                        if let Some(ref sig) = incoming_sig {
                            if !sig.is_empty() {
                                let cached_family = crate::proxy::SignatureCache::global()
                                    .get_signature_family(sig);
                                match cached_family {
                                    Some(family) => {
                                        if crate::proxy::mappers::common_utils::is_model_compatible(
                                            &family,
                                            final_model_name,
                                        ) {
                                            effective_sig = Some(sig.clone());
                                        } else {
                                            tracing::warn!(
                                                "[Gemini-Wrap] Incompatible thinking signature (Family: {}, Target: {}).",
                                                family, final_model_name
                                            );
                                        }
                                    }
                                    None => {
                                        effective_sig = Some(sig.clone());
                                    }
                                }
                            }
                        }

                        if effective_sig.is_none() {
                            effective_sig = turn_signature.clone();
                        }

                        // 无签名时**绝不发明哨兵**：官方流量里哨兵出现 0/23 次，
                        // 它不属于 Antigravity 协议。Gemini 目标的思考块本就不应携带签名
                        // （铁律 I4），故保留 `thought: true` 结构、仅省略签名字段。
                        if let Some(sig) = effective_sig {
                            new_parts.push(json!({
                                "text": final_thought_text,
                                "thought": true,
                                "thoughtSignature": sig,
                            }));
                        } else {
                            new_parts.push(json!({
                                "text": final_thought_text,
                                "thought": true,
                            }));
                        }
                    } else {
                        // 处理普通部件及 functionCall / functionResponse
                        if let Some(obj) = part.as_object_mut() {
                            // 1. 处理 functionCall (Assistant 请求调用工具)
                            if let Some(fc) = obj.get_mut("functionCall") {
                                if fc.get("id").is_none() && is_target_claude {
                                    let name = fc
                                        .get("name")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or("unknown");
                                    let count = name_counters.entry(name.to_string()).or_insert(0);
                                    let call_id = format!("call_{}_{}", name, count);
                                    *count += 1;

                                    fc.as_object_mut()
                                        .unwrap()
                                        .insert("id".to_string(), json!(call_id));
                                    tracing::debug!("[Gemini-Wrap] Request stage: Injected missing call_id '{}' for Claude model", call_id);
                                }

                                // 处理签名校验与兼容性 (对齐 Anthropic)
                                let _call_id =
                                    fc.get("id").and_then(|v| v.as_str()).map(str::to_string);
                                let incoming_fc_sig = obj
                                    .get("thoughtSignature")
                                    .or_else(|| obj.get("thought_signature"))
                                    .and_then(|s| s.as_str())
                                    .map(str::to_string);

                                // 纯净线缆透传：客户端若自带签名则保持原样，缺失签名全权委托进站流水线统一对齐与回填
                                if let Some(ref sig) = incoming_fc_sig {
                                    obj.insert("thoughtSignature".to_string(), json!(sig));
                                }
                                obj.remove("thought_signature");
                            }

                            // 2. 处理 functionResponse (User 回复工具结果)
                            if obj.contains_key("functionResponse") {
                                obj.remove("thoughtSignature");
                                obj.remove("thought_signature");
                            }
                            if let Some(fr) = obj.get_mut("functionResponse") {
                                if fr.get("id").is_none() && is_target_claude {
                                    let name = fr
                                        .get("name")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or("unknown");
                                    let count = name_counters.entry(name.to_string()).or_insert(0);
                                    let call_id = format!("call_{}_{}", name, count);
                                    *count += 1;

                                    fr.as_object_mut()
                                        .unwrap()
                                        .insert("id".to_string(), json!(call_id));
                                    tracing::debug!("[Gemini-Wrap] Request stage: Injected synced response_id '{}' for Claude model", call_id);
                                }
                            }
                        }
                        saw_non_thinking = true;
                        new_parts.push(part);
                    }
                }
                *parts = new_parts;
            }
        }
        crate::proxy::pipeline::InboundThinkingPipeline::process_contents(
            contents,
            crate::proxy::pipeline::ProxyProtocol::GeminiNative,
            &final_model_name,
            should_inject,
            session_id,
            false,
        );
    }
}
