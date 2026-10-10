// Request helper functions (split from request.rs).
// OpenAI → Gemini 请求转换
use super::super::models::*;
use crate::proxy::model_specs;
use crate::proxy::token_manager::ProxyToken;

use serde_json::{json, Value};

pub(crate) fn is_tiered_flash_model(model: &str) -> bool {
    let model_id = model
        .rsplit('/')
        .next()
        .unwrap_or(model)
        .to_ascii_lowercase();
    model_id
        .strip_prefix("gemini-")
        .and_then(|rest| rest.strip_suffix("-flash-tiered"))
        .is_some_and(|version| !version.is_empty())
}

/// Collect system/developer text without joining. A string content is one block;
/// a content array contributes one block per text part.
fn collect_system_instruction_blocks(request: &OpenAIRequest) -> Vec<String> {
    let mut blocks = Vec::new();

    if let Some(inst) = &request.instructions {
        if !inst.trim().is_empty() {
            blocks.push(inst.clone());
        }
    }

    // [JEIKCODE FROZEN SYSTEM PRINCIPLE]
    // 严格遵循系统指令绝对冻结法则：仅收集最开头的连续 system / developer 消息。
    // 一旦遇到首个非 system/developer 消息（即对话已进入多轮状态），立即停止收集！
    // 对话中途出现的任何 system/developer 消息一律保留在 contents 中作为 synthetic user 处理，
    // 绝对严禁提取并追加至 systemInstruction，杜绝顶层系统前缀突变破坏 KV Cache！
    for msg in &request.messages {
        if msg.role != "system" && msg.role != "developer" {
            break;
        }
        match &msg.content {
            Some(OpenAIContent::String(text)) => {
                if !text.trim().is_empty() {
                    blocks.push(text.clone());
                }
            }
            Some(OpenAIContent::Array(items)) => {
                for item in items {
                    if let OpenAIContentBlock::Text { text } = item {
                        if !text.trim().is_empty() {
                            blocks.push(text.clone());
                        }
                    }
                }
            }
            None => {}
        }
    }

    blocks
}

fn is_apply_patch_tool_name(name: &str) -> bool {
    name == "apply_patch" || name == "apply_patch_v2"
}

#[allow(dead_code)]
fn should_preserve_tool_output(tool_name: &str, output: &str) -> bool {
    is_apply_patch_tool_name(tool_name)
        || output.contains("apply_patch verification failed")
        || output.contains("Failed to find expected lines")
        || output.contains("Failed to find context")
        || output.contains("Expected update hunk")
}

fn qualify_namespace_tool_name(namespace_name: &str, child_name: &str) -> String {
    let child = child_name.trim();
    let ns = namespace_name.trim();
    if child.is_empty() || ns.is_empty() || child.starts_with("mcp__") {
        return child.to_string();
    }
    if child.starts_with(ns) {
        return child.to_string();
    }
    if ns.ends_with("__") {
        return format!("{}{}", ns, child);
    }
    format!("{}__{}", ns, child)
}

pub(crate) fn flatten_tools(tools: &[Value]) -> Vec<Value> {
    let mut flat = Vec::new();
    for tool in tools {
        let t = tool.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if t == "namespace" {
            let namespace_name = tool.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if let Some(sub_tools) = tool.get("tools").and_then(|v| v.as_array()) {
                let sub_flat = flatten_tools(sub_tools);
                for mut sub_tool in sub_flat {
                    if let Some(obj) = sub_tool.as_object_mut() {
                        let mut name = String::new();
                        if let Some(n) = obj.get("name").and_then(|v| v.as_str()) {
                            name = n.to_string();
                        } else if let Some(func) = obj.get("function") {
                            if let Some(n) = func.get("name").and_then(|v| v.as_str()) {
                                name = n.to_string();
                            }
                        }
                        if !name.is_empty() {
                            let qualified = qualify_namespace_tool_name(namespace_name, &name);
                            if obj.contains_key("name") {
                                obj.insert("name".to_string(), json!(qualified));
                            }
                            if let Some(func) = obj.get_mut("function") {
                                if let Some(func_obj) = func.as_object_mut() {
                                    func_obj.insert("name".to_string(), json!(qualified));
                                }
                            }
                        }
                    }
                    flat.push(sub_tool);
                }
            }
        } else {
            flat.push(tool.clone());
        }
    }
    flat
}

pub fn extract_client_tool_names(tools: &Option<Vec<Value>>) -> std::collections::HashSet<String> {
    let mut names = std::collections::HashSet::new();
    if let Some(tools_list) = tools {
        let flat_tools = flatten_tools(tools_list);
        for tool in flat_tools {
            let name_opt = tool
                .get("function")
                .and_then(|f| f.get("name"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    tool.get("name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .or_else(|| {
                    tool.get("type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                });
            if let Some(name) = name_opt {
                names.insert(name);
            }
        }
    }
    names
}
