// System instructions phase (split from request.rs).
// OpenAI → Gemini 请求转换
use super::super::models::*;
use crate::proxy::model_specs;
use crate::proxy::token_manager::ProxyToken;

use super::helpers::collect_system_instruction_blocks;
use super::helpers::flatten_tools;
use serde_json::{json, Value};

pub(crate) struct SystemState {
    pub system_instructions: Vec<String>,
    pub tool_id_to_name: std::collections::HashMap<String, String>,
    pub tool_name_to_schema: std::collections::HashMap<String, serde_json::Value>,
}

pub(crate) fn phase_system(request: &super::super::models::OpenAIRequest) -> SystemState {
    let mut system_instructions: Vec<String> = collect_system_instruction_blocks(request);

    // 遵循纯透传原则：不替换日期、路径、UUID 等任何动态字段，完整保留客户端与 Agent 的真实环境感知。
    // 身份声明归一化（Codex `based on GPT-x` / 厂商 `created by …` / Claude Agent SDK 等）已**统一收敛**
    // 到协议无关的提示词清洗流水线节点：`PromptSanitizer::normalize_system_identity`，在系统提示词头部
    // 窗口内做广谱匹配。依据 AGENTS.md「Pipeline First / Fix Strategy」——适配层只做参数归一化与协议转换，
    // 清洗一律由流水线统一处理，避免同一类 WAF 风险在四个协议里各写一份特例。
    let mut seen_system_instruction_keys = std::collections::HashSet::new();
    system_instructions.retain(|inst| {
        let key = inst.trim();
        !key.is_empty() && seen_system_instruction_keys.insert(key.to_string())
    });

    // Pre-scan to map tool_call_id to function name (for Codex)
    let mut tool_id_to_name = std::collections::HashMap::new();
    for msg in &request.messages {
        if let Some(tool_calls) = &msg.tool_calls {
            for call in tool_calls {
                let name = if let Some(func) = &call.function {
                    func.name.clone()
                } else if call.operation.is_some() || call.r#type == "apply_patch_call" {
                    "apply_patch".to_string()
                } else {
                    continue;
                };
                let final_name = if name == "local_shell_call" {
                    "shell"
                } else {
                    &name
                };
                tool_id_to_name.insert(call.id.clone(), final_name.to_string());
            }
        }
    }

    // [New] 预先构建工具名称到原始 Schema 的映射，用于后续参数类型修正
    let mut tool_name_to_schema = std::collections::HashMap::new();
    if let Some(tools) = &request.tools {
        let flat_tools = flatten_tools(tools);
        for tool in &flat_tools {
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

            let params_opt = tool
                .get("function")
                .and_then(|f| f.get("parameters"))
                .or_else(|| tool.get("parameters"));

            if let (Some(name), Some(params)) = (name_opt, params_opt) {
                tool_name_to_schema.insert(name, params.clone());
            }
        }
    }

    SystemState {
        system_instructions,
        tool_id_to_name,
        tool_name_to_schema,
    }
}
