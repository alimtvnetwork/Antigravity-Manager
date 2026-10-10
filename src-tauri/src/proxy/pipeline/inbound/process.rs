use super::*;

impl InboundThinkingPipeline {
    pub fn process_contents(
        contents: &mut Vec<Value>,
        protocol: ProxyProtocol,
        target_model: &str,
        is_thinking_enabled: bool,
        session_id: Option<&str>,
        is_retry: bool,
    ) {
        let trusts_signature = protocol.trusts_client_signature();
        let is_claude = target_model.to_lowercase().contains("claude");

        // 0. 工具调用 ID 统一归一化治理（Pipeline First）：
        // 彻底解决不同客户端在 tool call ID 格式上不一致（如缺少下划线 `call573077` vs `call_573077`）的问题，
        // 将 `functionCall.id` 与 `functionResponse.id` 统一对齐为带 `_` 的标准形态 (`call_...`)。
        Self::normalize_tool_call_ids(contents);

        // 0.1 统一上下文结构对齐（Pipeline First 统一治理）：
        // 将连续的 user 消息直到下一个 model，统一合并为一个 user 轮次的多个 block (parts)，保持严格顺序。
        // 这彻底消除了 Adapter 层各自为政导致的轮次错位，使得四大协议进入流水线后结构 100% 同构！
        // 连续 user 轮**保持独立**（对齐官方形态）。
        //
        // 历史实现会把连续 user 轮的 parts 合并进前一个 content，理由是
        // "Gemini 强制要求 user/model 交替"。但官方 Antigravity 报文里连续 user 轮
        // 是常态（`f81eae5c` 的 contents[11] 用户消息紧接 contents[12] SYSTEM_MESSAGE，
        // 两者独立），v1internal 上游并不要求严格交替。
        //
        // 实测（2026-09-26，`gemini-3.8-flash-tiered` @ daily）：
        //   两个独立 user content     → 200，上下文理解正确
        //   合并为单个 user content   → 200，上下文理解正确
        // 两者都成立，故按官方形态保留独立，以获得跨协议一致的前缀字节。

        // 预先计算每一轮的前置因果锚点 (causal anchor)，以便无 ID 的 Gemini 原生工具调用也能无损合成确定性 ID
        let anchors: Vec<String> = (0..contents.len())
            .map(|i| {
                let preceding = if i > 0 { contents.get(i - 1) } else { None };
                crate::proxy::thinking_store::compute_causal_anchor(preceding)
            })
            .collect();

        // 1. 协议策略清洗与位置规范化
        for (msg_idx, content) in contents.iter_mut().enumerate() {
            let _anchor = &anchors[msg_idx];
            let is_model = matches!(
                content.get("role").and_then(|r| r.as_str()),
                Some("model") | Some("assistant")
            );

            if let Some(parts) = content.get_mut("parts").and_then(|p| p.as_array_mut()) {
                if is_model {
                    let mut thinking_part = None;
                    let mut extra_thinking_parts = Vec::new();
                    let mut other_parts = Vec::new();
                    // [2026-09-27] 占位思考块被丢弃时，若其携带真实签名则暂存于此，
                    // 组装阶段转挂到该轮第一个非思考 part（锚点）。
                    let mut placeholder_sig: Option<String> = None;

                    for mut part in parts.drain(..) {
                        // 铁律：只认 thought: true。真机报文的正文 part 同样携带签名，
                        // 绝不能凭"有签名"判定思考块 —— 否则可见回答会被改写成内部思考，
                        // 并静默丢弃该轮唯一的签名。
                        let is_thought = crate::proxy::thinking_store::is_thought_part(&part);

                        if is_thought {
                            let text = part.get("text").and_then(|v| v.as_str()).unwrap_or("");
                            let is_placeholder =
                                crate::proxy::thinking_store::is_placeholder_thought(text);

                            // 校验客户端签名有效性与模型兼容性
                            let mut effective_sig = None;
                            if let Some(sig) = part
                                .get("thoughtSignature")
                                .or_else(|| part.get("thought_signature"))
                                .or_else(|| part.get("signature"))
                                .and_then(|s| s.as_str())
                            {
                                if sig == crate::proxy::thinking_store::SENTINEL_SIGNATURE {
                                    // Claude 模型绝不接受 Gemini 哨兵签名，避免触发 400 Invalid signature
                                    if !is_claude {
                                        effective_sig = Some(sig.to_string());
                                    }
                                } else if trusts_signature && sig.len() >= 50 {
                                    let cached_family = crate::proxy::SignatureCache::global()
                                        .get_signature_family(sig);
                                    let compatible = match cached_family {
                                        Some(family) => {
                                            crate::proxy::mappers::common_utils::is_model_compatible(
                                                &family,
                                                target_model,
                                            ) || (is_claude
                                                && family.to_lowercase().contains("claude"))
                                        }
                                        None => {
                                            if target_model.to_lowercase().contains("gemini") {
                                                crate::proxy::thinking_store::is_likely_gemini_signature(sig)
                                            } else if is_claude {
                                                crate::proxy::thinking_store::is_claude_signature(
                                                    sig,
                                                )
                                            } else {
                                                true
                                            }
                                        }
                                    };
                                    if compatible {
                                        let final_sig = if is_claude {
                                            crate::proxy::thinking_store::ensure_google_claude_thought_signature(sig)
                                        } else {
                                            sig.to_string()
                                        };
                                        effective_sig = Some(final_sig);
                                    } else if target_model.to_lowercase().contains("gemini") {
                                        tracing::warn!(
                                            "[InboundPipeline] Stripping foreign signature (len: {}) from thought block for Gemini model {}",
                                            sig.len(), target_model
                                        );
                                        effective_sig = None;
                                    } else if is_claude {
                                        tracing::warn!(
                                            "[InboundPipeline] Stripping foreign signature (len: {}) from thought block for Claude model {}",
                                            sig.len(), target_model
                                        );
                                        effective_sig = None;
                                    }
                                }
                            }

                            // [2026-09-27] 占位思考块直接丢弃（不写思考块），签名照常落锚点。
                            //
                            // 官方样本（baogao.txt 24 轮）：9 轮无思考块但首非思考 part 带签名，
                            // 「无思考块 + 锚点带签名」是官方标准形态。占位思考块（空 /
                            // "." / "..." / 空格 / "·" 等）无信息量，不该出现在出站报文中：
                            // 它曾是网关"防御性占位"的产物（FIX #3382 / Layer-2 压缩已移除）。
                            //
                            // 若占位块本身携带真实签名（客户端把签名误挂在思考块上），
                            // 签名不随思考块丢弃 —— 摘出后挂到第一个非思考 part（锚点），
                            // finalize 的 place_turn_signature 会按锚点规则最终归位。
                            let text_trimmed = text.trim();
                            if is_placeholder || text_trimmed.is_empty() {
                                if let Some(ref sig) = effective_sig {
                                    // 占位思考块携带的签名：转移给该轮锚点（首个非思考 part）
                                    placeholder_sig = Some(sig.clone());
                                    tracing::debug!(
                                        "[InboundPipeline] Placeholder thought block dropped; its signature (len: {}) kept for anchor backfill.",
                                        sig.len(),
                                    );
                                } else {
                                    tracing::debug!(
                                        "[InboundPipeline] Placeholder thought block dropped (text={:?}).",
                                        text_trimmed,
                                    );
                                }
                                continue;
                            }

                            let final_thought_text = text;

                            let mut thought_obj = json!({
                                "text": final_thought_text,
                                "thought": true,
                            });
                            if let Some(sig) = effective_sig {
                                // 铁律 I4：Gemini 目标的思考块**绝不**携带签名。
                                // 客户端在思考块上携带签名属于**矛盾组合** ——
                                // 官方 33 个 model 轮里 `thought:true` 与签名共现 0 次；
                                // 文本为占位符（`...` / `·` / `[undefined]` 等）时更属
                                // 典型的客户端占位脏数据污染。
                                //
                                // 只剥离 Gemini 目标的思考块签名 —— Claude 目标按
                                // Anthropic 规范**必须**在思考块上携带签名。
                                // 也不因此关闭思考：那会反向改写历史轮语义。
                                if is_claude {
                                    thought_obj["thoughtSignature"] = json!(sig);
                                } else {
                                    tracing::debug!(
                                        "[InboundPipeline] Stripped signature from thought block for Gemini target (len: {}). I4: thought parts never carry signatures.",
                                        sig.len(),
                                    );
                                }
                            }

                            if thinking_part.is_none() {
                                thinking_part = Some(thought_obj);
                            } else {
                                // 多个思考块时，非首位的多余思考块降级为普通文本
                                if !final_thought_text.is_empty() {
                                    extra_thinking_parts
                                        .push(json!({ "text": final_thought_text }));
                                }
                            }
                        } else {
                            if target_model.to_lowercase().contains("gemini") {
                                // Gemini 原生：客户端自带的真实签名**原样透传**，绝不发明哨兵。
                                // 依据（3 份官方报文 / 23 处签名）：哨兵出现 0 次，它不属于
                                // Antigravity 协议；而签名缺失是被上游容忍的（在飞轮即缺席）。
                                // 归位（含清除非锚点签名）统一交给终审 place_turn_signature。
                                if let Some(obj) = part.as_object_mut() {
                                    obj.remove("thought_signature");
                                }
                            } else if is_claude {
                                if let Some(obj) = part.as_object_mut() {
                                    obj.remove("thoughtSignature");
                                    obj.remove("thought_signature");
                                }
                            }
                            // 非思考部件：可能是普通正文/过程进度说明（commentary），也可能是 functionCall 等
                            let is_plain_text = part.get("text").is_some()
                                && part.get("functionCall").is_none()
                                && part.get("functionResponse").is_none();

                            if is_plain_text {
                                let raw_text =
                                    part.get("text").and_then(|v| v.as_str()).unwrap_or("");
                                if raw_text.trim().is_empty() {
                                    // 丢弃纯空白文本部件，避免触发 Gemini 400 校验或破坏前缀缓存哈希稳定性
                                    continue;
                                }

                                // 跨家族协议自愈：检查是否夹带 <think> 标签包裹的思考内容（如从 Claude 跨切回 Gemini）
                                if let Some((extracted_thought, clean_visible)) =
                                    crate::proxy::thinking_store::extract_think_tags(raw_text)
                                {
                                    if thinking_part.is_none() && is_thinking_enabled {
                                        if crate::proxy::thinking_store::is_meaningful_thought(
                                            &extracted_thought,
                                        ) {
                                            // 铁律 I4：思考块绝不携带签名
                                            // （官方报文里 thought:true 的 part 没有 thoughtSignature）
                                            let t_obj = json!({
                                                "text": extracted_thought.as_str(),
                                                "thought": true,
                                            });
                                            thinking_part = Some(t_obj);
                                        }
                                    }
                                    if !clean_visible.is_empty() {
                                        other_parts.push(json!({ "text": clean_visible }));
                                    }
                                    continue;
                                }

                                // 协议无关自愈：检查是否夹带旧版遗留思考前缀 (如 **Thinking**)
                                if raw_text.trim_start().starts_with("**Thinking**") {
                                    let clean_thought = Self::strip_thinking_prefix(raw_text);
                                    if thinking_part.is_none() && is_thinking_enabled {
                                        if crate::proxy::thinking_store::is_meaningful_thought(
                                            &clean_thought,
                                        ) {
                                            // 铁律 I4：思考块绝不携带签名
                                            let t_obj = json!({
                                                "text": clean_thought,
                                                "thought": true,
                                            });
                                            thinking_part = Some(t_obj);
                                        }
                                    }
                                    // 若已有思考块，该遗留思考块作为陈旧副本剥离，防止二次污染正文
                                    continue;
                                }

                                other_parts.push(part);
                            } else {
                                other_parts.push(part);
                            }
                        }
                    }

                    // 核心前缀保序：首位强制存在且仅存在一个 thinking_part，其余正文与工具调用紧随其后
                    let mut new_parts = Vec::with_capacity(parts.len() + 1);
                    if let Some(tp) = thinking_part {
                        new_parts.push(tp);
                    }
                    new_parts.extend(extra_thinking_parts);
                    new_parts.extend(other_parts);

                    // [2026-09-27] 占位思考块被丢弃时的签名转移：
                    // 若该轮思考块全部为占位/空（无 thinking_part），或占位块带真实签名，
                    // 把签名挂到第一个非思考 part（锚点）——该锚点可能是正文 text 也可能
                    // 是 functionCall。finalize 的 place_turn_signature 会按「锚点=首个
                    // 非思考 part」规则最终归位；若此处已挂，place 也会摘用。
                    //
                    // ⚠️ 必须在 other_parts 合并之后查找锚点：占位思考丢弃后，若该轮
                    // 首个非思考 part 是正文 text（含 extra_thinking 无、thinking_part 无时），
                    // 锚点只存在于 other_parts —— 过早查找会漏挂签名。
                    if let Some(sig) = placeholder_sig.take() {
                        if let Some(anchor) = new_parts
                            .iter_mut()
                            .find(|p| !crate::proxy::thinking_store::is_thought_part(p))
                        {
                            if let Some(obj) = anchor.as_object_mut() {
                                obj["thoughtSignature"] = json!(sig);
                            }
                        }
                    }

                    *parts = new_parts;
                } else {
                    // role == "user" 的通用进站治理：多模态工具响应 (functionResponse) 深度解构
                    // 确保全协议 (OpenAI / Claude / Gemini / Responses) 的工具结果中夹带的图片均被提升为独立的 inlineData 视觉感知输入
                    let mut extra_inline_parts = Vec::new();
                    for part in parts.iter_mut() {
                        if let Some(fr) = part.get_mut("functionResponse") {
                            if let Some(resp) = fr.get_mut("response") {
                                for key in ["result", "output"] {
                                    if let Some(v) = resp.get_mut(key) {
                                        if let Some(s) = v.as_str() {
                                            if s.contains("data:image/") {
                                                let clean_s = crate::proxy::mappers::common_utils::extract_multimodal_from_tool_text(s, &mut extra_inline_parts);
                                                *v = json!(clean_s);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    parts.extend(extra_inline_parts);
                }
            }
        }

        // 2. 状态机无损复活 (Hydration)
        // 开思考时全局复活；关思考时若包含工具调用，亦执行复活以取回历史工具防伪签名
        let has_function_call = contents.iter().any(|c| {
            c.get("parts")
                .and_then(|p| p.as_array())
                .map_or(false, |parts| {
                    parts.iter().any(|p| p.get("functionCall").is_some())
                })
        });
        if (is_thinking_enabled || has_function_call) && !is_retry {
            if let Some(s_id) = session_id {
                crate::proxy::thinking_store::hydrate_gemini_contents_with_model(
                    s_id,
                    contents,
                    Some(target_model),
                );
            }
        }

        // 3. 终审把关与脱敏规范化 (Finalize)
        crate::proxy::thinking_store::finalize_gemini_contents_thinking_with_model(
            contents,
            is_thinking_enabled,
            Some(target_model),
        );

        // 4. 工具回执 role 归一化（对齐官方 Antigravity 形态）。
        //    必须放在**末位**：若前置，回执轮会进入上面的 `is_model` 分支，
        //    从而跳过 `role == "user"` 分支里的多模态解构（图片提升为 inlineData）。
        Self::normalize_function_response_roles(contents);
    }
}
