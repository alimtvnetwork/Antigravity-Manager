// Tool argument sanitization helpers (split from response.rs)
use serde_json::Value;

pub fn normalize_and_sanitize_tool_args(tool_name: &str, args: &mut Value) {
    if let Some(obj) = args.as_object() {
        tracing::debug!(
            "[OpenAI] Tool Call (Passthrough): '{}' Args: {:?}",
            tool_name,
            obj
        );
    }
}

pub fn resolve_shell_tool_name(
    model_tool_name: &str,
    _client_tool_names: &std::collections::HashSet<String>,
) -> String {
    // Pure transparent transmission tool name without any rewriting
    model_tool_name.to_string()
}
