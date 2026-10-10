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
/// 判定某个 Part 是否为思考块。
///
/// **铁律：只认 `thought == true`。**
///
/// 绝不能退化为"带 `thoughtSignature` 且无 `functionCall`/`functionResponse`"——
/// 真实 Antigravity 报文会把签名挂在**纯正文 part** 上（官方不变量见
/// `.workbuddy/outputs/correct-assembly-spec.md`）。旧启发式会把可见回答误判为思考块，
/// 进而在回填阶段把正文改写成"内部思考"，并丢掉该轮唯一的签名——这正是
/// "签名剥离 → 跨请求死循环"事故的根因。
#[inline]
pub fn is_thought_part(part: &Value) -> bool {
    part.get("thought")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// 把签名归位到**该轮第一个非思考 part**，其余 part 一律删除签名字段。
///
/// 由官方报文归纳出的四条硬约束：
///
/// 1. 锚点 = 该轮第一个 `thought != true` 的 part，**不是**硬编码的 `parts[0]`
///    （该轮有思考块时，官方签名落在 `parts[1]`）；
/// 2. `thought: true` / `functionResponse` / 其余 part —— 字段必须**缺席**，而不是空串；
/// 3. 锚点自带真实签名时**原样保留**（覆盖"纯正文轮"与"思考块 + 正文轮"两种排列）；
/// 4. 锚点无签名且 `fallback_sig` 也为空时 —— **什么都不写**。官方在"在飞轮"上就是缺席的，
///    缺失签名是被容忍的，**绝不发明哨兵**。
///
/// 本函数**绝不重排、绝不插入** part —— 顺序即锚点语义。
///
/// 返回最终写入锚点的签名（若有）。
pub fn place_turn_signature(parts: &mut [Value], fallback_sig: Option<&str>) -> Option<String> {
    // 1. 锚点 = 第一个非思考 part；整轮皆思考则本轮无锚点
    let anchor = parts.iter().position(|p| !is_thought_part(p))?;

    // 2. 必须在清空之前取出锚点自带的签名
    let own_sig = parts[anchor]
        .get("thoughtSignature")
        .or_else(|| parts[anchor].get("thought_signature"))
        .and_then(|s| s.as_str())
        .filter(|s| is_real_signature(s))
        .map(str::to_string);

    // 3. 全量清空，保证非锚点 part 的签名字段确实"缺席"
    for part in parts.iter_mut() {
        if let Some(obj) = part.as_object_mut() {
            obj.remove("thoughtSignature");
            obj.remove("thought_signature");
        }
    }

    // 4. functionResponse 永不携带签名
    if parts[anchor].get("functionResponse").is_some() {
        return None;
    }

    // 5. 锚点自带优先，缺失时才使用回填来源（跨协议路径）
    let sig = own_sig.or_else(|| {
        fallback_sig
            .filter(|s| is_real_signature(s))
            .map(str::to_string)
    })?;

    // 权威防裂化：如果签名是原始二进制 protobuf (首字节 0x12)，必须转为标准 Base64 编码后再发送给 Gemini！
    let final_sig = if sig.as_bytes().first() == Some(&0x12) {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(sig.as_bytes())
    } else {
        sig
    };
    parts[anchor]["thoughtSignature"] = json!(final_sig);

    // 反向入库优化：如果锚点是工具调用，反向更新/修补回签名缓存与 SQLite tool_signatures 库！
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
    // 拦截伪思考标签与客户端占位脏数据
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

/// 提取并切除文本中的 `<think>...</think>` 标签内容（用于跨模型自愈与正文思考文本重提升）
/// 支持任意大小写（<think> / <THINK>）、前后正文无缝缝合拼接、以及未闭合标签容错
/// 返回 `Some((thought, remaining_visible))`
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
            // 未闭合标签容错：截断到文本末尾
            let thought = text[after_start..].trim().to_string();
            let before = text[..start_pos].trim();
            return Some((thought, before.to_string()));
        }
    }
    None
}

pub(crate) fn is_capturable_thought(thought: &str, signature: Option<&str>) -> bool {
    // 占位符或纯空白思考绝不可捕获为新的持久化思考记录！
    if is_placeholder_thought(thought) || thought.trim().is_empty() {
        // [DECOUPLE 2026-09-26] 官方新规：任何 model 轮都可能返回签名（正文轮 / 工具轮 / 纯思考轮）。
        // 纯 fc 轮（无思考文本）只要携带真实签名，也必须入库——否则下一轮锚点回填时找不到签名。
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
