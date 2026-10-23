// System instructions phase (split from request.rs).
// OpenAI → Gemini request conversion
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

    // Follow the principle of pure transparent transmission: do not replace any dynamic fields such as date, path, UUID, etc., and completely retain the real environment perception of the client and Agent.
    // Identity claim normalization (Codex `based on GPT-x` / vendor `created by …` / Claude Agent SDK, etc.) has been **unified and converged**
    // Go to the protocol-independent prompt word cleaning pipeline node: `PromptSanitizer::normalize_system_identity`, in the system prompt word header
    // Do broad spectrum matching within the window. According to AGENTS.md "Pipeline First / Fix Strategy" - the adaptation layer only performs parameter normalization and protocol conversion.
    // Cleaning will be handled uniformly by the pipeline to avoid writing a special case in each of the four protocols for the same type of WAF risks.
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

    // [New] Pre-built mapping of tool names to original Schema for subsequent parameter type correction
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
