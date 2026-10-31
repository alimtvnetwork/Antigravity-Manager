// Tool detection and payload cleaning (split from common_utils.rs).
// Common utilities for request mapping across all protocols
// Provides unified grounding/networking logic

use serde_json::{json, Value};

/// Request configuration after grounding resolution

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

            // [STABILITY GUARD] If the client itself has defined function facilities (functionDeclarations / function_declarations),
            // Do not forcefully inject the googleSearch tool. Prevent server-side grounding calls from causing the client to fail to distribute, call empty tools, or report unknown tool errors.
            if has_functions {
                tracing::debug!(
                    "Skipping googleSearch injection: functionDeclarations present, avoiding client tool dispatch conflicts"
                );
                return;
            }

            // First clean up the existing googleSearch or googleSearchRetrieval to prevent repeated conflicts
            tools_arr.retain(|t| {
                if let Some(o) = t.as_object() {
                    !(o.contains_key("googleSearch")
                        || o.contains_key("google_search")
                        || o.contains_key("googleSearchRetrieval"))
                } else {
                    true
                }
            });

            // Inject unified googleSearch (v1internal specification)
            tools_arr.push(json!({
                "googleSearch": {}
            }));
        }
    }
}

/// In-depth iteration cleans up [undefined] dirty strings sent by the client to prevent Gemini interface verification failure
pub fn deep_clean_undefined(value: &mut Value, depth: usize) {
    if depth > 10 {
        return;
    }
    match value {
        Value::Object(map) => {
            // Remove keys with value "[undefined]"
            map.retain(|_, v| {
                if let Some(s) = v.as_str() {
                    s != "[undefined]"
                } else {
                    true
                }
            });
            // Handle nesting recursively
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
            // 1. Straight hair style (Claude/Simple OpenAI/Anthropic Builtin/Vertex): { "name": "..." } or { "type": "..." }
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

            // 2. OpenAI nested style: { "type": "function", "function": { "name": "..." } }
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

            // 3. Gemini native style: { "functionDeclarations": [ { "name": "..." } ] }
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

            // 4. Gemini googleSearch statement (including googleSearchRetrieval variant)
            if tool.get("googleSearch").is_some() || tool.get("googleSearchRetrieval").is_some() {
                return true;
            }
        }
    }
    false
}

/// Detect whether non-networking related local function tools are included
pub fn contains_non_networking_tool(tools: &Option<Vec<Value>>) -> bool {
    if let Some(list) = tools {
        for tool in list {
            let mut is_networking = false;

            // Simple logic: if it is a function declaration and the name is not a networking keyword, it is considered a non-networking tool
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
                // If it is Gemini style functionDeclarations, go in and take a look.
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
                                return true; // discover local functions
                            }
                        }
                    }
                }
                is_networking = true; // Even though it is all networked, the outer layer is marked as networked
            }

            if !is_networking {
                return true;
            }
        }
    }
    false
}

/// Check whether any tool definitions are carried (either local functions or network tools)
pub fn has_any_tools(tools: &Option<Vec<Value>>) -> bool {
    if let Some(list) = tools {
        !list.is_empty()
    } else {
        false
    }
}

/// Check whether contents contains tool calls or tool return results (indicating multiple Agent sessions)
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
