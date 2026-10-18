//! Field-mapping adapters converting legacy snake_case row objects into target PascalCase entities without loss.

use crate::error::AppError;
use crate::modules::db_orm::{
    ActivePrompt, BackupBatch, ConversationSequence, ErrorLog, FailedCommand, GreenProject,
    ProjectSequence, PromptBackup, RunningProject, TaskHistory,
};
use serde_json::Value;

/// Extracts a string from a JSON Value trying multiple candidate field names.
/// Handles both string literals and numeric values cleanly.
pub fn get_str(val: &Value, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(v) = val.get(*k) {
            if let Some(s) = v.as_str() {
                return Some(s.to_string());
            } else if let Some(n) = v.as_i64() {
                return Some(n.to_string());
            } else if let Some(b) = v.as_bool() {
                return Some(b.to_string());
            }
        }
    }
    None
}

/// Extracts an optional string, filtering out explicit JSON nulls.
pub fn get_opt_str(val: &Value, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(v) = val.get(*k) {
            if v.is_null() {
                return None;
            }
            if let Some(s) = v.as_str() {
                return Some(s.to_string());
            } else if let Some(n) = v.as_i64() {
                return Some(n.to_string());
            }
        }
    }
    None
}

/// Extracts an i64 from a JSON Value trying multiple candidate field names.
/// Handles numeric integers and numeric strings.
pub fn get_i64(val: &Value, keys: &[&str]) -> Option<i64> {
    for k in keys {
        if let Some(v) = val.get(*k) {
            if let Some(n) = v.as_i64() {
                return Some(n);
            } else if let Some(s) = v.as_str() {
                if let Ok(n) = s.parse::<i64>() {
                    return Some(n);
                }
            }
        }
    }
    None
}

/// Extracts an optional i64, filtering out nulls.
pub fn get_opt_i64(val: &Value, keys: &[&str]) -> Option<i64> {
    for k in keys {
        if let Some(v) = val.get(*k) {
            if v.is_null() {
                return None;
            }
            if let Some(n) = v.as_i64() {
                return Some(n);
            } else if let Some(s) = v.as_str() {
                if let Ok(n) = s.parse::<i64>() {
                    return Some(n);
                }
            }
        }
    }
    None
}

/// Extracts an affirmative boolean from a JSON Value.
/// Supports boolean values, numeric 1/0, and truthy strings ("true", "1", "yes").
pub fn get_bool(val: &Value, keys: &[&str]) -> bool {
    for k in keys {
        if let Some(v) = val.get(*k) {
            if let Some(b) = v.as_bool() {
                return b;
            }
            if let Some(n) = v.as_i64() {
                return n != 0;
            }
            if let Some(s) = v.as_str() {
                let lower = s.trim().to_lowercase();
                if lower == "1" || lower == "true" || lower == "yes" {
                    return true;
                }
            }
        }
    }
    false
}

/// Adapts a legacy or target row object to an `ErrorLog` entity.
pub fn adapt_error_log(row: &Value) -> Result<ErrorLog, AppError> {
    Ok(ErrorLog {
        error_log_id: get_str(row, &["ErrorLogId", "id", "error_log_id"])
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        error_code: get_str(row, &["ErrorCode", "error_code"])
            .unwrap_or_else(|| "E1001".to_string()),
        severity: get_str(row, &["Severity", "severity"]).unwrap_or_else(|| "error".to_string()),
        message: get_str(row, &["Message", "message"]).unwrap_or_default(),
        stack_trace: get_opt_str(row, &["StackTrace", "stack_trace"]),
        source: get_opt_str(row, &["Source", "source"]),
        trigger_action: get_opt_str(row, &["TriggerAction", "trigger_action"]),
        metadata_json: get_opt_str(row, &["MetadataJson", "metadata_json"]),
        created_at: get_i64(row, &["CreatedAt", "created_at"])
            .unwrap_or_else(|| chrono::Utc::now().timestamp()),
    })
}

/// Adapts a legacy or target row object to a `RunningProject` entity.
pub fn adapt_running_project(row: &Value) -> Result<RunningProject, AppError> {
    Ok(RunningProject {
        running_project_id: get_str(row, &["RunningProjectId", "id", "running_project_id"])
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        instance_id: get_str(row, &["InstanceId", "instance_id"])
            .unwrap_or_else(|| "default".to_string()),
        repo_name: get_str(row, &["RepoName", "repo_name"]).unwrap_or_default(),
        repo_path: get_str(row, &["RepoPath", "repo_path"]).unwrap_or_default(),
        workspace_storage_path: get_opt_str(
            row,
            &["WorkspaceStoragePath", "workspace_storage_path"],
        ),
        is_running: get_bool(row, &["IsRunning", "is_running"]),
        last_detected_at: get_i64(row, &["LastDetectedAt", "last_detected_at"]).unwrap_or(0),
        updated_at: get_i64(row, &["UpdatedAt", "updated_at"]).unwrap_or(0),
    })
}

/// Adapts a legacy or target row object to an `ActivePrompt` entity.
pub fn adapt_active_prompt(row: &Value) -> Result<ActivePrompt, AppError> {
    Ok(ActivePrompt {
        active_prompt_id: get_str(row, &["ActivePromptId", "id", "active_prompt_id"])
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        running_project_id: get_str(
            row,
            &["RunningProjectId", "project_id", "running_project_id"],
        )
        .unwrap_or_default(),
        instance_id: get_str(row, &["InstanceId", "instance_id"])
            .unwrap_or_else(|| "default".to_string()),
        repo_path: get_str(row, &["RepoPath", "repo_path"]).unwrap_or_default(),
        prompt_content: get_str(row, &["PromptContent", "prompt_content"]).unwrap_or_default(),
        model: get_opt_str(row, &["Model", "model"]),
        session_id: get_opt_str(row, &["SessionId", "session_id"]),
        status: get_str(row, &["Status", "status"]).unwrap_or_else(|| "queued".to_string()),
        created_at: get_i64(row, &["CreatedAt", "created_at"]).unwrap_or(0),
        updated_at: get_i64(row, &["UpdatedAt", "updated_at"]).unwrap_or(0),
        image_payload: get_opt_str(row, &["ImagePayload", "image_payload"]),
    })
}

/// Adapts a legacy or target row object to a `ProjectSequence` entity.
pub fn adapt_project_sequence(row: &Value) -> Result<ProjectSequence, AppError> {
    Ok(ProjectSequence {
        project_sequence_id: get_str(row, &["ProjectSequenceId", "id", "project_sequence_id"])
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        sequence_id: get_i64(row, &["SequenceId", "sequence_id", "seq_id"]).unwrap_or(0),
        running_project_id: get_str(
            row,
            &["RunningProjectId", "running_project_id", "project_id"],
        )
        .unwrap_or_default(),
        repo_name: get_str(row, &["RepoName", "repo_name"]).unwrap_or_default(),
        repo_path: get_str(row, &["RepoPath", "repo_path"]).unwrap_or_default(),
        instance_id: get_str(row, &["InstanceId", "instance_id"])
            .unwrap_or_else(|| "default".to_string()),
        updated_at: get_i64(row, &["UpdatedAt", "updated_at"]).unwrap_or(0),
    })
}

/// Adapts a legacy or target row object to a `ConversationSequence` entity.
pub fn adapt_conversation_sequence(row: &Value) -> Result<ConversationSequence, AppError> {
    Ok(ConversationSequence {
        conversation_sequence_id: get_str(
            row,
            &["ConversationSequenceId", "id", "conversation_sequence_id"],
        )
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        sequence_id: get_i64(row, &["SequenceId", "sequence_id", "seq_id"]).unwrap_or(0),
        project_key: get_str(row, &["ProjectKey", "project_key"]).unwrap_or_default(),
        title: get_str(row, &["Title", "title"]).unwrap_or_default(),
        instance_id: get_str(row, &["InstanceId", "instance_id"])
            .unwrap_or_else(|| "default".to_string()),
        updated_at: get_i64(row, &["UpdatedAt", "updated_at"]).unwrap_or(0),
    })
}

/// Adapts a legacy or target row object to a `FailedCommand` entity.
pub fn adapt_failed_command(row: &Value) -> Result<FailedCommand, AppError> {
    Ok(FailedCommand {
        failed_command_id: get_i64(row, &["FailedCommandId", "id", "failed_command_id"])
            .unwrap_or(0),
        command: get_str(row, &["Command", "command"]).unwrap_or_default(),
        full_args: get_str(row, &["FullArgs", "full_args"]).unwrap_or_default(),
        domain: get_str(row, &["Domain", "domain"]).unwrap_or_else(|| "root".to_string()),
        error_code: get_str(row, &["ErrorCode", "error_code"])
            .unwrap_or_else(|| "E1001".to_string()),
        message: get_str(row, &["Message", "message"]).unwrap_or_default(),
        suggestions: get_str(row, &["Suggestions", "suggestions"]).unwrap_or_default(),
        hit_count: get_i64(row, &["HitCount", "hit_count"]).unwrap_or(1),
        working_dir: get_str(row, &["WorkingDir", "working_dir"]).unwrap_or_default(),
        agm_version: get_str(row, &["AgmVersion", "agm_version"]).unwrap_or_default(),
        is_resolved: get_bool(row, &["IsResolved", "is_resolved"]),
        created_at: get_str(row, &["CreatedAt", "created_at"]).unwrap_or_default(),
        last_seen_at: get_str(row, &["LastSeenAt", "last_seen_at"]).unwrap_or_default(),
    })
}

/// Adapts a legacy or target row object to a `BackupBatch` entity.
pub fn adapt_backup_batch(row: &Value) -> Result<BackupBatch, AppError> {
    Ok(BackupBatch {
        backup_batch_id: get_str(row, &["BackupBatchId", "id", "backup_batch_id"])
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        created_at: get_i64(row, &["CreatedAt", "created_at"]).unwrap_or(0),
        prompts_count: get_i64(row, &["PromptsCount", "prompts_count"]).unwrap_or(0),
        file_path: get_str(row, &["FilePath", "file_path"]).unwrap_or_default(),
        retention_days: get_i64(row, &["RetentionDays", "retention_days"]).unwrap_or(1),
        is_fully_restored: get_bool(row, &["IsFullyRestored", "is_fully_restored"]),
    })
}

/// Adapts a legacy or target row object to a `PromptBackup` entity.
pub fn adapt_prompt_backup(row: &Value) -> Result<PromptBackup, AppError> {
    Ok(PromptBackup {
        prompt_backup_id: get_str(row, &["PromptBackupId", "id", "prompt_backup_id"])
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        backup_batch_id: get_str(row, &["BackupBatchId", "backup_batch_id"]).unwrap_or_default(),
        original_prompt_id: get_str(
            row,
            &["OriginalPromptId", "prompt_id", "original_prompt_id"],
        )
        .unwrap_or_default(),
        project_name: get_str(row, &["ProjectName", "project_name"]).unwrap_or_default(),
        project_path: get_str(row, &["ProjectPath", "project_path"]).unwrap_or_default(),
        running_project_id: get_str(
            row,
            &["RunningProjectId", "project_id", "running_project_id"],
        )
        .unwrap_or_default(),
        conversation_id: get_str(row, &["ConversationId", "conversation_id"]).unwrap_or_default(),
        conversation_name: get_opt_str(row, &["ConversationName", "conversation_name"]),
        sequence_id: get_i64(row, &["SequenceId", "sequence_id"]).unwrap_or(0),
        prompt_text: get_str(row, &["PromptText", "prompt_text"]).unwrap_or_default(),
        has_images: get_bool(row, &["HasImages", "has_images"]),
        images_payload: get_opt_str(row, &["ImagesPayload", "images_payload"]),
        status: get_str(row, &["Status", "status"]).unwrap_or_else(|| "queued".to_string()),
        created_at: get_i64(row, &["CreatedAt", "created_at"]).unwrap_or(0),
        is_restored: get_bool(row, &["IsRestored", "is_restored"]),
        restored_at: get_opt_i64(row, &["RestoredAt", "restored_at"]),
        instance_id: get_str(row, &["InstanceId", "instance_id"])
            .unwrap_or_else(|| "default".to_string()),
    })
}

/// Adapts a legacy or target row object to a `GreenProject` entity.
pub fn adapt_green_project(row: &Value) -> Result<GreenProject, AppError> {
    Ok(GreenProject {
        green_project_id: get_str(row, &["GreenProjectId", "id", "green_project_id"])
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        project_identifier: get_str(
            row,
            &["ProjectIdentifier", "project_identifier", "identifier"],
        )
        .unwrap_or_default(),
        project_path: get_str(row, &["ProjectPath", "project_path", "path"]).unwrap_or_default(),
        added_at: get_i64(row, &["AddedAt", "added_at"]).unwrap_or(0),
        status: get_str(row, &["Status", "status"]).unwrap_or_else(|| "active".to_string()),
        last_checked_at: get_opt_i64(row, &["LastCheckedAt", "last_checked_at"]),
    })
}

/// Adapts a legacy or target row object to a `TaskHistory` entity.
pub fn adapt_task_history(row: &Value) -> Result<TaskHistory, AppError> {
    Ok(TaskHistory {
        task_history_id: get_str(row, &["TaskHistoryId", "id", "task_history_id"])
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        action: get_str(row, &["Action", "action"]).unwrap_or_default(),
        action_code: get_opt_i64(row, &["ActionCode", "action_code"]),
        status: get_str(row, &["Status", "status"]).unwrap_or_default(),
        subject: get_str(row, &["Subject", "subject"]).unwrap_or_default(),
        detail: get_str(row, &["Detail", "detail"]).unwrap_or_default(),
        instance_id: get_str(row, &["InstanceId", "instance_id"])
            .unwrap_or_else(|| "default".to_string()),
        split_path: get_str(row, &["SplitPath", "split_path"]).unwrap_or_default(),
        from_email: get_str(row, &["FromEmail", "from_email"]).unwrap_or_default(),
        to_email: get_str(row, &["ToEmail", "to_email"]).unwrap_or_default(),
        created_at: get_i64(row, &["CreatedAt", "created_at"]).unwrap_or(0),
        finished_at: get_opt_i64(row, &["FinishedAt", "finished_at"]),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapt_error_log_snake_and_pascal() {
        let legacy_json = serde_json::json!({
            "id": "err-001",
            "error_code": "E1001",
            "severity": "warn",
            "message": "Legacy failure",
            "stack_trace": "line 10",
            "source": "cli",
            "trigger_action": "start",
            "metadata_json": "{\"key\": \"val\"}",
            "created_at": 1740000000
        });

        let adapted = adapt_error_log(&legacy_json).unwrap();
        assert_eq!(adapted.error_log_id, "err-001");
        assert_eq!(adapted.error_code, "E1001");
        assert_eq!(adapted.severity, "warn");
        assert_eq!(adapted.message, "Legacy failure");
        assert_eq!(adapted.stack_trace, Some("line 10".to_string()));
        assert_eq!(adapted.source, Some("cli".to_string()));
        assert_eq!(adapted.trigger_action, Some("start".to_string()));
        assert_eq!(
            adapted.metadata_json,
            Some("{\"key\": \"val\"}".to_string())
        );
        assert_eq!(adapted.created_at, 1740000000);

        let pascal_json = serde_json::json!({
            "ErrorLogId": "err-002",
            "ErrorCode": "E2001",
            "Severity": "error",
            "Message": "Pascal failure",
            "CreatedAt": 1740000100
        });
        let adapted_p = adapt_error_log(&pascal_json).unwrap();
        assert_eq!(adapted_p.error_log_id, "err-002");
        assert_eq!(adapted_p.error_code, "E2001");
        assert_eq!(adapted_p.severity, "error");
        assert_eq!(adapted_p.message, "Pascal failure");
        assert_eq!(adapted_p.created_at, 1740000100);
    }

    #[test]
    fn test_adapt_running_project_booleans() {
        let legacy_json = serde_json::json!({
            "id": "proj-1",
            "instance_id": "inst-1",
            "repo_name": "test-repo",
            "repo_path": "/path/to/repo",
            "is_running": 1,
            "last_detected_at": 100,
            "updated_at": 200
        });

        let adapted = adapt_running_project(&legacy_json).unwrap();
        assert_eq!(adapted.running_project_id, "proj-1");
        assert!(adapted.is_running);

        let legacy_false = serde_json::json!({
            "id": "proj-2",
            "is_running": 0
        });
        let adapted_f = adapt_running_project(&legacy_false).unwrap();
        assert!(!adapted_f.is_running);
    }
}
