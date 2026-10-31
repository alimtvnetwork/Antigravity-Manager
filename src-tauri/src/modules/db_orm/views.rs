//! Pre-defined SQLite view DDLs and strongly typed view models.
//!
//! Enforces non-negotiable database rules:
//! 1. View names start with `Vw` and are singular PascalCase.
//! 2. Column names are PascalCase.
//! 3. FIFO ordering is strictly preserved in queue and backup views (`CreatedAt ASC, Id ASC`).

use crate::error::AppError;
use crate::modules::db_orm::traits::{FromRow, ViewEntity};
use rusqlite::{Connection, Row};
use serde::{Deserialize, Serialize};

pub const DB_ERRORS: &str = "errors.db";
pub const DB_REPO_PROMPTS: &str = "repo-prompts.db";
pub const DB_PROMPT_BACKUPS: &str = "prompt-backups.db";
pub const DB_TASK_HISTORY: &str = "task-history.db";

/// View schema metadata holding view name, associated split database file, and DDL string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewSchema {
    pub view_name: &'static str,
    pub db_file: &'static str,
    pub ddl: &'static str,
}

// ============================================================================
// 1. errors.db Views
// ============================================================================

/// Readonly view model for `VwRecentErrors`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct VwRecentErrors {
    pub error_log_id: String,
    pub error_code: String,
    pub severity: String,
    pub message: String,
    pub source: Option<String>,
    pub trigger_action: Option<String>,
    pub created_at: i64,
}

impl ViewEntity for VwRecentErrors {
    const VIEW_NAME: &'static str = "VwRecentErrors";
    const DB_FILE: &'static str = DB_ERRORS;
    const DDL: &'static str = "CREATE VIEW IF NOT EXISTS VwRecentErrors AS SELECT \
        ErrorLogId, \
        ErrorCode, \
        Severity, \
        Message, \
        Source, \
        TriggerAction, \
        CreatedAt \
    FROM ErrorLog \
    ORDER BY CreatedAt DESC \
    LIMIT 100;";
}

impl FromRow for VwRecentErrors {
    fn from_row(row: &Row) -> Result<Self, AppError> {
        Ok(Self {
            error_log_id: row.get("ErrorLogId")?,
            error_code: row.get("ErrorCode")?,
            severity: row.get("Severity")?,
            message: row.get("Message")?,
            source: row.get("Source")?,
            trigger_action: row.get("TriggerAction")?,
            created_at: row.get("CreatedAt")?,
        })
    }
}

/// Readonly view model for `VwErrorSummaryByCode`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct VwErrorSummaryByCode {
    pub error_code: String,
    pub severity: String,
    pub occurrences_count: i64,
    pub first_seen_at: i64,
    pub last_seen_at: i64,
}

impl ViewEntity for VwErrorSummaryByCode {
    const VIEW_NAME: &'static str = "VwErrorSummaryByCode";
    const DB_FILE: &'static str = DB_ERRORS;
    const DDL: &'static str = "CREATE VIEW IF NOT EXISTS VwErrorSummaryByCode AS SELECT \
        ErrorCode, \
        Severity, \
        COUNT(*) AS OccurrencesCount, \
        MIN(CreatedAt) AS FirstSeenAt, \
        MAX(CreatedAt) AS LastSeenAt \
    FROM ErrorLog \
    GROUP BY ErrorCode, Severity \
    ORDER BY OccurrencesCount DESC;";
}

impl FromRow for VwErrorSummaryByCode {
    fn from_row(row: &Row) -> Result<Self, AppError> {
        Ok(Self {
            error_code: row.get("ErrorCode")?,
            severity: row.get("Severity")?,
            occurrences_count: row.get("OccurrencesCount")?,
            first_seen_at: row.get("FirstSeenAt")?,
            last_seen_at: row.get("LastSeenAt")?,
        })
    }
}

// ============================================================================
// 2. repo-prompts.db Views
// ============================================================================

/// Readonly view model for `VwActivePromptsWithProjects`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct VwActivePromptsWithProjects {
    pub active_prompt_id: String,
    pub running_project_id: String,
    pub instance_id: String,
    pub repo_name: Option<String>,
    pub repo_path: String,
    pub prompt_content: String,
    pub model: Option<String>,
    pub session_id: Option<String>,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub image_payload: Option<String>,
    pub is_project_running: Option<bool>,
}

impl ViewEntity for VwActivePromptsWithProjects {
    const VIEW_NAME: &'static str = "VwActivePromptsWithProjects";
    const DB_FILE: &'static str = DB_REPO_PROMPTS;
    const DDL: &'static str = "CREATE VIEW IF NOT EXISTS VwActivePromptsWithProjects AS SELECT \
        p.ActivePromptId, \
        p.RunningProjectId, \
        p.InstanceId, \
        r.RepoName, \
        p.RepoPath, \
        p.PromptContent, \
        p.Model, \
        p.SessionId, \
        p.Status, \
        p.CreatedAt, \
        p.UpdatedAt, \
        p.ImagePayload, \
        r.IsRunning AS IsProjectRunning \
    FROM ActivePrompt p \
    LEFT JOIN RunningProject r ON p.RunningProjectId = r.RunningProjectId;";
}

impl FromRow for VwActivePromptsWithProjects {
    fn from_row(row: &Row) -> Result<Self, AppError> {
        Ok(Self {
            active_prompt_id: row.get("ActivePromptId")?,
            running_project_id: row.get("RunningProjectId")?,
            instance_id: row.get("InstanceId")?,
            repo_name: row.get("RepoName")?,
            repo_path: row.get("RepoPath")?,
            prompt_content: row.get("PromptContent")?,
            model: row.get("Model")?,
            session_id: row.get("SessionId")?,
            status: row.get("Status")?,
            created_at: row.get("CreatedAt")?,
            updated_at: row.get("UpdatedAt")?,
            image_payload: row.get("ImagePayload")?,
            is_project_running: row
                .get::<_, Option<i64>>("IsProjectRunning")
                .map(|opt| opt.map(|v| v != 0))?,
        })
    }
}

/// Readonly view model for `VwQueuedPromptsFifo`.
///
/// Enforces strict First-In-First-Out ordering via `ORDER BY p.CreatedAt ASC, p.ActivePromptId ASC`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct VwQueuedPromptsFifo {
    pub active_prompt_id: String,
    pub running_project_id: String,
    pub instance_id: String,
    pub repo_name: Option<String>,
    pub repo_path: String,
    pub prompt_content: String,
    pub model: Option<String>,
    pub session_id: Option<String>,
    pub status: String,
    pub created_at: i64,
}

impl ViewEntity for VwQueuedPromptsFifo {
    const VIEW_NAME: &'static str = "VwQueuedPromptsFifo";
    const DB_FILE: &'static str = DB_REPO_PROMPTS;
    const DDL: &'static str = "CREATE VIEW IF NOT EXISTS VwQueuedPromptsFifo AS SELECT \
        p.ActivePromptId, \
        p.RunningProjectId, \
        p.InstanceId, \
        r.RepoName, \
        p.RepoPath, \
        p.PromptContent, \
        p.Model, \
        p.SessionId, \
        p.Status, \
        p.CreatedAt \
    FROM ActivePrompt p \
    LEFT JOIN RunningProject r ON p.RunningProjectId = r.RunningProjectId \
    WHERE p.Status = 'queued' \
    ORDER BY p.CreatedAt ASC, p.ActivePromptId ASC;";
}

impl FromRow for VwQueuedPromptsFifo {
    fn from_row(row: &Row) -> Result<Self, AppError> {
        Ok(Self {
            active_prompt_id: row.get("ActivePromptId")?,
            running_project_id: row.get("RunningProjectId")?,
            instance_id: row.get("InstanceId")?,
            repo_name: row.get("RepoName")?,
            repo_path: row.get("RepoPath")?,
            prompt_content: row.get("PromptContent")?,
            model: row.get("Model")?,
            session_id: row.get("SessionId")?,
            status: row.get("Status")?,
            created_at: row.get("CreatedAt")?,
        })
    }
}

// ============================================================================
// 3. prompt-backups.db Views
// ============================================================================

/// Readonly view model for `VwUnrestoredBackups`.
///
/// Enforces strict First-In-First-Out ordering via `ORDER BY pb.CreatedAt ASC, pb.PromptBackupId ASC`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct VwUnrestoredBackups {
    pub prompt_backup_id: String,
    pub backup_batch_id: String,
    pub original_prompt_id: String,
    pub project_name: String,
    pub project_path: String,
    pub conversation_id: String,
    pub sequence_id: i64,
    pub prompt_text: String,
    pub has_images: bool,
    pub status: String,
    pub created_at: i64,
    pub instance_id: String,
    pub batch_file_path: String,
}

impl ViewEntity for VwUnrestoredBackups {
    const VIEW_NAME: &'static str = "VwUnrestoredBackups";
    const DB_FILE: &'static str = DB_PROMPT_BACKUPS;
    const DDL: &'static str = "CREATE VIEW IF NOT EXISTS VwUnrestoredBackups AS SELECT \
        pb.PromptBackupId, \
        pb.BackupBatchId, \
        pb.OriginalPromptId, \
        pb.ProjectName, \
        pb.ProjectPath, \
        pb.ConversationId, \
        pb.SequenceId, \
        pb.PromptText, \
        pb.HasImages, \
        pb.Status, \
        pb.CreatedAt, \
        pb.InstanceId, \
        bb.FilePath AS BatchFilePath \
    FROM PromptBackup pb \
    JOIN BackupBatch bb ON pb.BackupBatchId = bb.BackupBatchId \
    WHERE pb.IsRestored = 0 \
    ORDER BY pb.CreatedAt ASC, pb.PromptBackupId ASC;";
}

impl FromRow for VwUnrestoredBackups {
    fn from_row(row: &Row) -> Result<Self, AppError> {
        Ok(Self {
            prompt_backup_id: row.get("PromptBackupId")?,
            backup_batch_id: row.get("BackupBatchId")?,
            original_prompt_id: row.get("OriginalPromptId")?,
            project_name: row.get("ProjectName")?,
            project_path: row.get("ProjectPath")?,
            conversation_id: row.get("ConversationId")?,
            sequence_id: row.get("SequenceId")?,
            prompt_text: row.get("PromptText")?,
            has_images: row.get::<_, i64>("HasImages").map(|v| v != 0)?,
            status: row.get("Status")?,
            created_at: row.get("CreatedAt")?,
            instance_id: row.get("InstanceId")?,
            batch_file_path: row.get("BatchFilePath")?,
        })
    }
}

/// Readonly view model for `VwLatestBackupBatches`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct VwLatestBackupBatches {
    pub backup_batch_id: String,
    pub created_at: i64,
    pub prompts_count: i64,
    pub file_path: String,
    pub retention_days: i64,
    pub is_fully_restored: bool,
    pub total_backups_count: i64,
    pub restored_count: i64,
}

impl ViewEntity for VwLatestBackupBatches {
    const VIEW_NAME: &'static str = "VwLatestBackupBatches";
    const DB_FILE: &'static str = DB_PROMPT_BACKUPS;
    const DDL: &'static str = "CREATE VIEW IF NOT EXISTS VwLatestBackupBatches AS SELECT \
        bb.BackupBatchId, \
        bb.CreatedAt, \
        bb.PromptsCount, \
        bb.FilePath, \
        bb.RetentionDays, \
        bb.IsFullyRestored, \
        COUNT(pb.PromptBackupId) AS TotalBackupsCount, \
        COALESCE(SUM(CASE WHEN pb.IsRestored = 1 THEN 1 ELSE 0 END), 0) AS RestoredCount \
    FROM BackupBatch bb \
    LEFT JOIN PromptBackup pb ON bb.BackupBatchId = pb.BackupBatchId \
    GROUP BY bb.BackupBatchId \
    ORDER BY bb.CreatedAt DESC;";
}

impl FromRow for VwLatestBackupBatches {
    fn from_row(row: &Row) -> Result<Self, AppError> {
        Ok(Self {
            backup_batch_id: row.get("BackupBatchId")?,
            created_at: row.get("CreatedAt")?,
            prompts_count: row.get("PromptsCount")?,
            file_path: row.get("FilePath")?,
            retention_days: row.get("RetentionDays")?,
            is_fully_restored: row.get::<_, i64>("IsFullyRestored").map(|v| v != 0)?,
            total_backups_count: row.get("TotalBackupsCount")?,
            restored_count: row.get("RestoredCount")?,
        })
    }
}

// ============================================================================
// 4. task-history.db Views
// ============================================================================

/// Readonly view model for `VwRecentTaskHistory`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct VwRecentTaskHistory {
    pub task_history_id: String,
    pub action_code: Option<i32>,
    pub action: String,
    pub status: String,
    pub subject: String,
    pub detail: String,
    pub instance_id: String,
    pub payload_json: String,
    pub from_email: String,
    pub to_email: String,
    pub created_at: i64,
    pub finished_at: Option<i64>,
    pub is_completed: bool,
}

impl ViewEntity for VwRecentTaskHistory {
    const VIEW_NAME: &'static str = "VwRecentTaskHistory";
    const DB_FILE: &'static str = DB_TASK_HISTORY;
    const DDL: &'static str = "CREATE VIEW IF NOT EXISTS VwRecentTaskHistory AS SELECT \
        TaskHistoryId, \
        ActionCode, \
        Action, \
        Status, \
        Subject, \
        Detail, \
        InstanceId, \
        PayloadJson, \
        FromEmail, \
        ToEmail, \
        CreatedAt, \
        FinishedAt, \
        IsCompleted \
    FROM TaskHistory \
    ORDER BY CreatedAt DESC \
    LIMIT 200;";
}

impl FromRow for VwRecentTaskHistory {
    fn from_row(row: &Row) -> Result<Self, AppError> {
        Ok(Self {
            task_history_id: row.get("TaskHistoryId")?,
            action_code: row.get("ActionCode")?,
            action: row.get("Action")?,
            status: row.get("Status")?,
            subject: row.get("Subject")?,
            detail: row.get("Detail")?,
            instance_id: row.get("InstanceId")?,
            payload_json: row.get("PayloadJson")?,
            from_email: row.get("FromEmail")?,
            to_email: row.get("ToEmail")?,
            created_at: row.get("CreatedAt")?,
            finished_at: row.get("FinishedAt")?,
            is_completed: row.get::<_, i64>("IsCompleted").map(|v| v != 0)?,
        })
    }
}

// ============================================================================
// Registry of All Pre-Defined Views
// ============================================================================

pub const ALL_VIEWS: &[ViewSchema] = &[
    ViewSchema {
        view_name: VwRecentErrors::VIEW_NAME,
        db_file: VwRecentErrors::DB_FILE,
        ddl: VwRecentErrors::DDL,
    },
    ViewSchema {
        view_name: VwErrorSummaryByCode::VIEW_NAME,
        db_file: VwErrorSummaryByCode::DB_FILE,
        ddl: VwErrorSummaryByCode::DDL,
    },
    ViewSchema {
        view_name: VwActivePromptsWithProjects::VIEW_NAME,
        db_file: VwActivePromptsWithProjects::DB_FILE,
        ddl: VwActivePromptsWithProjects::DDL,
    },
    ViewSchema {
        view_name: VwQueuedPromptsFifo::VIEW_NAME,
        db_file: VwQueuedPromptsFifo::DB_FILE,
        ddl: VwQueuedPromptsFifo::DDL,
    },
    ViewSchema {
        view_name: VwUnrestoredBackups::VIEW_NAME,
        db_file: VwUnrestoredBackups::DB_FILE,
        ddl: VwUnrestoredBackups::DDL,
    },
    ViewSchema {
        view_name: VwLatestBackupBatches::VIEW_NAME,
        db_file: VwLatestBackupBatches::DB_FILE,
        ddl: VwLatestBackupBatches::DDL,
    },
    ViewSchema {
        view_name: VwRecentTaskHistory::VIEW_NAME,
        db_file: VwRecentTaskHistory::DB_FILE,
        ddl: VwRecentTaskHistory::DDL,
    },
];

/// Returns all view schemas defined for a given Split-DB filename.
pub fn get_views_for_db(db_file: &str) -> Vec<ViewSchema> {
    ALL_VIEWS
        .iter()
        .copied()
        .filter(|v| v.db_file == db_file)
        .collect()
}

/// Executes all view DDL statements for a given Split-DB file against the provided connection.
pub fn init_views_for_db(conn: &Connection, db_file: &str) -> Result<(), AppError> {
    for schema in ALL_VIEWS {
        if schema.db_file == db_file {
            conn.execute_batch(schema.ddl).map_err(AppError::Database)?;
        }
    }
    Ok(())
}
