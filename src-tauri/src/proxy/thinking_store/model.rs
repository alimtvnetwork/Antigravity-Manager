pub const SENTINEL_SIGNATURE: &str = "skip_thought_signature_validator";

pub fn model_forces_server_thinking(model: &str) -> bool {
    if !crate::proxy::config::is_thinking_store_enabled() {
        return false;
    }
    let m = model.to_lowercase();
    if m.is_empty() {
        return false;
    }
    if m.contains("image")
        || m.contains("imagen")
        || m.contains("embed")
        || m.contains("lite")
        || m.contains("preview")
    {
        return false;
    }
    m.contains("claude")
        || m.contains("flash")
        || m.contains("pro")
        || m.contains("agent")
        || m.contains("gemini")
        || m.contains("thinking")
        || m.contains("o1")
        || m.contains("o3")
        || m.contains("deepseek")
}

pub fn any_model_forces_server_thinking(models: &[&str]) -> bool {
    models.iter().copied().any(model_forces_server_thinking)
}
#[derive(Debug, Clone)]
pub struct ThinkingRecord {
    pub fingerprint: String,
    pub thought: String,
    pub signature: Option<String>,
    pub tool_ids: Vec<String>,
    #[allow(dead_code)]
    pub tool_names: Vec<String>,
    pub visible: String,
}
