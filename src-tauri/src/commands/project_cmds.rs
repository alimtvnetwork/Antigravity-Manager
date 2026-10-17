//! Project & conversation management commands for Tauri IPC

use crate::error::AppError;
use crate::modules::repo_db::sequences;

#[tauri::command]
pub async fn rename_conversation(
    instance_id: Option<String>,
    conversation_id: String,
    new_title: String,
) -> Result<(), AppError> {
    let clean_title = new_title.trim();
    if clean_title.is_empty() {
        return Err(AppError::Config(
            "Conversation title cannot be empty".to_string(),
        ));
    }
    let clean_cid = conversation_id.trim();
    if clean_cid.is_empty() {
        return Err(AppError::Config(
            "Conversation ID cannot be empty".to_string(),
        ));
    }

    let inst = instance_id.filter(|s| !s.is_empty());
    let title_owned = clean_title.to_string();
    let cid_owned = clean_cid.to_string();

    tokio::task::spawn_blocking(move || {
        sequences::update_conversation_title(inst.as_deref(), &cid_owned, &title_owned)
    })
    .await
    .map_err(|e| AppError::Process(format!("Task execution failed: {e}")))?
}
