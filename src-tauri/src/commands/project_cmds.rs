//! Project & conversation management commands for Tauri IPC

use crate::error::AppError;
use crate::modules::repo_db::sequences;
use crate::modules::repo_db::sqlite_export::{
    export_prompt_tree_to_bytes, export_prompt_tree_to_file, import_prompt_tree_from_source,
    validate_import_source, ExportOptions, ExportResult, ExportScope, ImportOptions, ImportResult,
    ValidationResult,
};
use std::path::PathBuf;

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

/// Helper to parse flexible scope input from frontend
fn resolve_export_scope(scope_val: serde_json::Value, id: Option<String>) -> ExportScope {
    if let Ok(mut scope) = serde_json::from_value::<ExportScope>(scope_val.clone()) {
        if scope.project_id.is_none() && scope.target_type == "project" {
            scope.project_id = id.clone();
        }
        if scope.conversation_id.is_none() && scope.target_type == "conversation" {
            scope.conversation_id = id.clone();
        }
        scope
    } else if let Some(scope_str) = scope_val.as_str() {
        match scope_str {
            "project" => ExportScope {
                target_type: "project".to_string(),
                project_id: id,
                project_name: None,
                conversation_id: None,
                prompt_ids: None,
            },
            "conversation" => ExportScope {
                target_type: "conversation".to_string(),
                project_id: None,
                project_name: None,
                conversation_id: id,
                prompt_ids: None,
            },
            "prompt_subtree" => ExportScope {
                target_type: "prompt_subtree".to_string(),
                project_id: None,
                project_name: None,
                conversation_id: None,
                prompt_ids: id.map(|i| vec![i]),
            },
            _ => ExportScope {
                target_type: "full_tree".to_string(),
                project_id: None,
                project_name: None,
                conversation_id: None,
                prompt_ids: None,
            },
        }
    } else {
        ExportScope::default()
    }
}

/// Export a section of the prompt tree (JSON, SQLite DB, or YAML) to disk.
#[tauri::command]
pub async fn export_prompt_tree_section(
    scope: serde_json::Value,
    id: Option<String>,
    format: Option<String>,
    file_path: Option<String>,
    options: Option<ExportOptions>,
    destination_path: Option<String>,
) -> Result<ExportResult, AppError> {
    let resolved_scope = resolve_export_scope(scope, id);
    let mut resolved_options = options.unwrap_or_default();
    if let Some(fmt) = format {
        resolved_options.format = fmt;
    }

    let target_path_str = destination_path
        .or(file_path)
        .ok_or_else(|| AppError::Config("Destination path is required for section export".to_string()))?;
    let path = PathBuf::from(target_path_str);

    tokio::task::spawn_blocking(move || {
        export_prompt_tree_to_file(&resolved_scope, &resolved_options, &path)
    })
    .await
    .map_err(|e| AppError::Process(format!("Task execution failed: {e}")))?
}

/// Web-compatible section export returning in-memory Base64 data.
#[tauri::command]
pub async fn export_prompt_tree_section_bytes(
    scope: serde_json::Value,
    id: Option<String>,
    format: Option<String>,
    options: Option<ExportOptions>,
) -> Result<(String, String, ExportResult), AppError> {
    let resolved_scope = resolve_export_scope(scope, id);
    let mut resolved_options = options.unwrap_or_default();
    if let Some(fmt) = format {
        resolved_options.format = fmt;
    }

    tokio::task::spawn_blocking(move || {
        export_prompt_tree_to_bytes(&resolved_scope, &resolved_options)
    })
    .await
    .map_err(|e| AppError::Process(format!("Task execution failed: {e}")))?
}

/// Validates an import source file or payload and returns statistics preview.
#[tauri::command]
pub async fn validate_prompt_tree_import(
    file_path: Option<String>,
    source_path: Option<String>,
    data_b64: Option<String>,
) -> Result<ValidationResult, AppError> {
    let path_str = source_path.or(file_path);
    let path_buf = path_str.map(PathBuf::from);

    tokio::task::spawn_blocking(move || {
        validate_import_source(path_buf.as_deref(), data_b64.as_deref())
    })
    .await
    .map_err(|e| AppError::Process(format!("Task execution failed: {e}")))?
}

/// Imports a prompt tree section from a file or Base64 payload, materializing images.
#[tauri::command]
pub async fn import_prompt_tree_section(
    file_path: Option<String>,
    source_path: Option<String>,
    data_b64: Option<String>,
    target_instance: Option<String>,
    options: Option<ImportOptions>,
) -> Result<ImportResult, AppError> {
    let path_str = source_path.or(file_path);
    let path_buf = path_str.map(PathBuf::from);
    let opts = options.unwrap_or_default();
    let inst_id = target_instance.or(opts.target_instance_id.clone());

    tokio::task::spawn_blocking(move || {
        import_prompt_tree_from_source(
            path_buf.as_deref(),
            data_b64.as_deref(),
            inst_id.as_deref(),
            &opts.conflict_mode,
            opts.materialize_images,
        )
    })
    .await
    .map_err(|e| AppError::Process(format!("Task execution failed: {e}")))?
}
