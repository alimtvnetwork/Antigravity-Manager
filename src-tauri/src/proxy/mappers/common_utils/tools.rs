// Tool detection and payload cleaning (split from common_utils.rs).
// Common utilities for request mapping across all protocols
// Provides unified grounding/networking logic

use serde_json::{json, Value};

/// Request configuration after grounding resolution
#[derive(Debug, Clone)]

/// Inject current googleSearch tool and ensure no duplicate legacy search tools.
/// When client-defined function tools are present, skips googleSearch to avoid client-side empty/unknown tool dispatch errors.
pub fn inject_google_search_tool(body: &mut Value, _mapped_model: Option<&str>) {
    if let Some(obj) = body.as_object_mut() {
        let tools_entry = obj.entry("tools").or_insert_with(|| json!([]));
        if let Some(tools_arr) = tools_entry.as_array_mut() {
            let has_functions = tools_arr.iter().any(|t| {
                t.as_object().map_or(false, |o| {
                    o.contains_key("functionDeclarations")
                        || o.contains_key("function_declarations")
                })
            });

            // [STABILITY GUARD] 如果客户端自身已经定义了函数工具 (functionDeclarations / function_declarations)，
            // 不强行注入 googleSearch 工具。防止服务端接地调用导致客户端无法分发、空工具调用或报未知工具错误。
            if has_functions {
                tracing::debug!(
                    "Skipping googleSearch injection: functionDeclarations present, avoiding client tool dispatch conflicts"
                );
                return;
            }

            // 首先清理掉已存在的 googleSearch 或 googleSearchRetrieval，以防重复产生冲突
            tools_arr.retain(|t| {
                if let Some(o) = t.as_object() {
                    !(o.contains_key("googleSearch")
                        || o.contains_key("google_search")
                        || o.contains_key("googleSearchRetrieval"))
                } else {
                    true
                }
            });

            // 注入统一的 googleSearch (v1internal 规范)
            tools_arr.push(json!({
                "googleSearch": {}
            }));
        }
    }
}

/// 深度迭代清理客户端发送的 [undefined] 脏字符串，防止 Gemini 接口校验失败
pub fn deep_clean_undefined(value: &mut Value, depth: usize) {
    if depth > 10 {
        return;
    }
    match value {
        Value::Object(map) => {
            // 移除值为 "[undefined]" 的键
            map.retain(|_, v| {
                if let Some(s) = v.as_str() {
                    s != "[undefined]"
                } else {
                    true
                }
            });
            // 递归处理嵌套
            for v in map.values_mut() {
                deep_clean_undefined(v, depth + 1);
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                deep_clean_undefined(v, depth + 1);
            }
        }
        _ => {}
    }
}

/// Detects if the tool list contains a request for networking/web search.
/// Supported keywords: "web_search", "google_search", "web_search_20250305"
pub fn detects_networking_tool(tools: &Option<Vec<Value>>) -> bool {
    if let Some(list) = tools {
        for tool in list {
            // 1. 直发风格 (Claude/Simple OpenAI/Anthropic Builtin/Vertex): { "name": "..." } 或 { "type": "..." }
            if let Some(n) = tool.get("name").and_then(|v| v.as_str()) {
                if n == "web_search"
                    || n == "google_search"
                    || n == "web_search_20250305"
                    || n == "google_search_retrieval"
                    || n == "builtin_web_search"
                {
                    return true;
                }
            }

            if let Some(t) = tool.get("type").and_then(|v| v.as_str()) {
                if t == "web_search_20250305"
                    || t == "google_search"
                    || t == "web_search"
                    || t == "google_search_retrieval"
                    || t == "builtin_web_search"
                {
                    return true;
                }
            }

            // 2. OpenAI 嵌套风格: { "type": "function", "function": { "name": "..." } }
            if let Some(func) = tool.get("function") {
                if let Some(n) = func.get("name").and_then(|v| v.as_str()) {
                    let keywords = [
                        "web_search",
                        "google_search",
                        "web_search_20250305",
                        "google_search_retrieval",
                        "builtin_web_search",
                    ];
                    if keywords.contains(&n) {
                        return true;
                    }
                }
            }

            // 3. Gemini 原生风格: { "functionDeclarations": [ { "name": "..." } ] }
            if let Some(decls) = tool.get("functionDeclarations").and_then(|v| v.as_array()) {
                for decl in decls {
                    if let Some(n) = decl.get("name").and_then(|v| v.as_str()) {
                        if n == "web_search"
                            || n == "google_search"
                            || n == "google_search_retrieval"
                            || n == "builtin_web_search"
                        {
                            return true;
                        }
                    }
                }
            }

            // 4. Gemini googleSearch 声明 (含 googleSearchRetrieval 变体)
            if tool.get("googleSearch").is_some() || tool.get("googleSearchRetrieval").is_some() {
                return true;
            }
        }
    }
    false
}

/// 探测是否包含非联网相关的本地函数工具
pub fn contains_non_networking_tool(tools: &Option<Vec<Value>>) -> bool {
    if let Some(list) = tools {
        for tool in list {
            let mut is_networking = false;

            // 简单逻辑：如果它是一个函数声明且名字不是联网关键词，则视为非联网工具
            if let Some(n) = tool.get("name").and_then(|v| v.as_str()) {
                let keywords = [
                    "web_search",
                    "google_search",
                    "web_search_20250305",
                    "google_search_retrieval",
                    "builtin_web_search",
                ];
                if keywords.contains(&n) {
                    is_networking = true;
                }
            } else if let Some(func) = tool.get("function") {
                if let Some(n) = func.get("name").and_then(|v| v.as_str()) {
                    let keywords = [
                        "web_search",
                        "google_search",
                        "web_search_20250305",
                        "google_search_retrieval",
                        "builtin_web_search",
                    ];
                    if keywords.contains(&n) {
                        is_networking = true;
                    }
                }
            } else if tool.get("googleSearch").is_some()
                || tool.get("googleSearchRetrieval").is_some()
            {
                is_networking = true;
            } else if tool.get("functionDeclarations").is_some() {
                // 如果是 Gemini 风格的 functionDeclarations，进去看一眼
                if let Some(decls) = tool.get("functionDeclarations").and_then(|v| v.as_array()) {
                    for decl in decls {
                        if let Some(n) = decl.get("name").and_then(|v| v.as_str()) {
                            let keywords = [
                                "web_search",
                                "google_search",
                                "google_search_retrieval",
                                "builtin_web_search",
                            ];
                            if !keywords.contains(&n) {
                                return true; // 发现本地函数
                            }
                        }
                    }
                }
                is_networking = true; // 即使全是联网，外层也标记为联网
            }

            if !is_networking {
                return true;
            }
        }
    }
    false
}

/// 检测是否携带任何工具定义 (无论是本地函数还是联网工具)
pub fn has_any_tools(tools: &Option<Vec<Value>>) -> bool {
    if let Some(list) = tools {
        !list.is_empty()
    } else {
        false
    }
}

/// 检查 contents 中是否包含工具调用或工具返回结果 (表明处于多轮 Agent 会话中)
pub fn contents_has_tool_interactions(contents: &Value) -> bool {
    if let Some(arr) = contents.as_array() {
        for msg in arr {
            if let Some(parts) = msg.get("parts").and_then(|p| p.as_array()) {
                for part in parts {
                    if part.get("functionCall").is_some()
                        || part.get("functionResponse").is_some()
                        || part.get("tool_use").is_some()
                        || part.get("tool_result").is_some()
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}
