use super::*;

/// 精简报文字段的**展示顺序**：按用户 / 开发者关注度从高到低排列。
/// 未列入的字段按其原有相对顺序追加在后面（依赖 `serde_json` 的 `preserve_order`）。
/// 目标：模型 → 思考块 → 上下文 → 用量/缓存 → 工具调用 → 其余，四协议行为一致。
const CORE_FIELD_ORDER: [&str; 22] = [
    // 1. 模型标识
    "model",
    // 2. 思考块：开关 / 强度 / 内容 / 签名
    "thinking",
    "reasoning_effort",
    "reasoning",
    "reasoning_content",
    "generationConfig",
    "generation_config",
    "thinking_signature",
    "thought_signature",
    "signature",
    "thoughtSignature",
    "_session_thinking_id",
    // 3. 上下文：系统提示词与对话
    "system",
    "systemInstruction",
    "instructions",
    "messages",
    "contents",
    "input",
    // 4. 用量与缓存
    "usage",
    "usageMetadata",
    // 5. 工具调用（关注度次之）
    "tool_calls",
    "tools",
];

/// 按关注度重排顶层字段 —— **只调整顺序，不删减任何字段**。
/// 四协议与调试日志共用，保证展示顺序一致；`request` 包装层递归重排并保持在前。
pub fn reorder_payload_fields(value: &Value) -> Value {
    let Value::Object(map) = value else {
        return value.clone();
    };
    let mut ordered = Map::new();
    let mut rest: Map<String, Value> = Map::new();
    for (k, v) in map {
        if k == "request" {
            ordered.insert(k.clone(), reorder_payload_fields(v));
        } else {
            rest.insert(k.clone(), v.clone());
        }
    }
    for key in CORE_FIELD_ORDER {
        if let Some(v) = rest.remove(key) {
            ordered.insert(key.to_string(), v);
        }
    }
    for (k, v) in rest {
        ordered.insert(k, v);
    }
    Value::Object(ordered)
}
