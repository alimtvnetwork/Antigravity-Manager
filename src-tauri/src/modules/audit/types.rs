//! Data types for the UI interaction audit recorder.

use serde::{Deserialize, Serialize};

/// Who performed the recorded interaction.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum InteractionActor {
    #[default]
    User,
    Cli,
    Telegram,
    System,
}

impl InteractionActor {
    pub fn as_str(&self) -> &'static str {
        match self {
            InteractionActor::User => "user",
            InteractionActor::Cli => "cli",
            InteractionActor::Telegram => "telegram",
            InteractionActor::System => "system",
        }
    }
}

/// What triggered the recorded interaction.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum InteractionTrigger {
    #[default]
    Click,
    Keyboard,
    Programmatic,
    Navigation,
    Input,
    Scroll,
}

impl InteractionTrigger {
    pub fn as_str(&self) -> &'static str {
        match self {
            InteractionTrigger::Click => "click",
            InteractionTrigger::Keyboard => "keyboard",
            InteractionTrigger::Programmatic => "programmatic",
            InteractionTrigger::Navigation => "navigation",
            InteractionTrigger::Input => "input",
            InteractionTrigger::Scroll => "scroll",
        }
    }
}

fn default_route() -> String {
    "/".to_string()
}

fn default_result() -> String {
    "ok".to_string()
}

/// Input payload accepted by the `record_ui_interaction` Tauri command.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecordInteractionInput {
    #[serde(default)]
    pub actor: InteractionActor,
    #[serde(default, alias = "method_name")]
    pub method_name: String,
    #[serde(default = "default_route")]
    pub route: String,
    pub component: Option<String>,
    #[serde(alias = "element_id")]
    pub element_id: Option<String>,
    #[serde(default)]
    pub trigger: InteractionTrigger,
    pub params: Option<serde_json::Value>,
    #[serde(default = "default_result")]
    pub result: String,
    pub message: Option<String>,
}

/// One recorded UI interaction: the 8 audit fields plus the DB row id.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InteractionEntry {
    pub id: i64,
    pub timestamp: i64,
    pub actor: String,
    pub method_name: String,
    pub ui_target: String,
    pub trigger: String,
    pub params: String,
    pub result: String,
    pub message: String,
    /// Compact JSON array of `{function, location}` frames captured at record
    /// time. `None` for rows written before the trace column existed.
    pub stack_trace: Option<String>,
}

/// Outcome returned by `record_ui_interaction`.
#[derive(Serialize, Debug)]
pub struct RecordInteractionOutcome {
    pub recorded: bool,
    pub id: Option<i64>,
}

/// Compose the `ui_target` field from route + component + element id,
/// skipping parts that are missing or blank.
pub fn format_ui_target(route: &str, component: Option<&str>, element_id: Option<&str>) -> String {
    let mut parts = vec![route.trim()];
    for part in [component, element_id].into_iter().flatten() {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }
    parts.join(" > ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_interaction_input_defaults() {
        let json = r#"{
            "methodName": "verifyClick"
        }"#;
        let input: RecordInteractionInput =
            serde_json::from_str(json).expect("should deserialize with defaults");
        assert_eq!(input.method_name, "verifyClick");
        assert_eq!(input.actor, InteractionActor::User);
        assert_eq!(input.trigger, InteractionTrigger::Click);
        assert_eq!(input.route, "/");
        assert_eq!(input.result, "ok");
        assert!(input.component.is_none());
        assert!(input.element_id.is_none());
        assert!(input.params.is_none());
        assert!(input.message.is_none());
    }

    #[test]
    fn test_record_interaction_input_full_and_aliases() {
        let json = r#"{
            "actor": "cli",
            "method_name": "runDiag",
            "route": "/diagnostics",
            "component": "DiagCard",
            "element_id": "btn-repair",
            "trigger": "keyboard",
            "params": {"key": "Enter"},
            "result": "error",
            "message": "failed to repair"
        }"#;
        let input: RecordInteractionInput =
            serde_json::from_str(json).expect("should deserialize full payload with aliases");
        assert_eq!(input.actor, InteractionActor::Cli);
        assert_eq!(input.method_name, "runDiag");
        assert_eq!(input.route, "/diagnostics");
        assert_eq!(input.component.as_deref(), Some("DiagCard"));
        assert_eq!(input.element_id.as_deref(), Some("btn-repair"));
        assert_eq!(input.trigger, InteractionTrigger::Keyboard);
        assert_eq!(input.result, "error");
        assert_eq!(input.message.as_deref(), Some("failed to repair"));
    }
}
