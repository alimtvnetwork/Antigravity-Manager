//! Data types for the UI interaction audit recorder.

use serde::{Deserialize, Serialize};

/// Who performed the recorded interaction.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InteractionActor {
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
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InteractionTrigger {
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

/// Input payload accepted by the `record_ui_interaction` Tauri command.
#[derive(Deserialize, Debug)]
pub struct RecordInteractionInput {
    pub actor: InteractionActor,
    pub method_name: String,
    pub route: String,
    pub component: Option<String>,
    pub element_id: Option<String>,
    pub trigger: InteractionTrigger,
    pub params: Option<serde_json::Value>,
    pub result: String,
    pub message: Option<String>,
}

/// One recorded UI interaction: the 8 audit fields plus the DB row id.
#[derive(Serialize, Deserialize, Clone, Debug)]
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
