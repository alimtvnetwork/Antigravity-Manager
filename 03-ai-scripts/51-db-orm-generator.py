#!/usr/bin/env python3
"""
51-db-orm-generator.py — Inspects and compiles declarative SQLite schema models
into type-safe Rust ORM structs, column constants, SQLite DDLs, and views
under src-tauri/src/modules/db_orm/generated/.

Enforces non-negotiable database rules:
1. Singular PascalCase table names (e.g. ErrorLog, ActivePrompt, RunningProject, TaskHistory).
2. PascalCase column names (e.g. ErrorLogId, PromptContent, CreatedAt).
3. Primary keys follow {TableName}Id format.
4. Affirmative booleans with Is* or Has* prefixes (IsActive, HasImages, IsRunning). Total ban on negative names.
5. Pre-defined Views for multi-table queries (VwActivePromptsWithProjects, VwRecentErrors, VwRecentTaskHistory).

Usage:
  python 03-ai-scripts/51-db-orm-generator.py
  python 03-ai-scripts/51-db-orm-generator.py --out-dir src-tauri/src/modules/db_orm/generated
  python 03-ai-scripts/51-db-orm-generator.py --dry-run
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from importlib import import_module
from pathlib import Path
import re
import sys

# Integrate shared core engine
sys.path.insert(0, str(Path(__file__).parent))
try:
    engine = import_module("02-shared-engine")
    ExitCodeType = engine.ExitCodeType
except ImportError:
    class ExitCodeType:
        SUCCESS = 0
        VIOLATIONS_FOUND = 1
        TOOL_ERROR = 2


@dataclass
class ColumnDef:
    name: str
    rust_type: str
    sql_type: str
    is_primary_key: bool = False
    is_nullable: bool = False
    is_autoincrement: bool = False
    default_value: str | None = None
    foreign_key: str | None = None

    def validate(self, table_name: str) -> None:
        # Enforce PascalCase column name
        if not re.match(r"^[A-Z][a-zA-Z0-9]*$", self.name):
            raise ValueError(f"Column '{self.name}' in table '{table_name}' must be PascalCase.")

        # Enforce affirmative boolean prefix and ban negative column names
        if self.rust_type in ("bool", "Option<bool>"):
            if not (self.name.startswith("Is") or self.name.startswith("Has")):
                raise ValueError(
                    f"Boolean column '{self.name}' in table '{table_name}' must start with 'Is' or 'Has'."
                )
            if self.name.startswith(("IsNot", "HasNo", "IsNon", "HasNone")):
                raise ValueError(
                    f"Negative boolean column '{self.name}' in table '{table_name}' is strictly forbidden."
                )

        # Enforce primary key convention: {TableName}Id
        if self.is_primary_key:
            expected_pk = f"{table_name}Id"
            if self.name != expected_pk:
                raise ValueError(
                    f"Primary key for table '{table_name}' must be named '{expected_pk}', got '{self.name}'."
                )


@dataclass
class IndexDef:
    name: str
    columns: list[str]
    is_unique: bool = False


@dataclass
class TableDef:
    name: str
    db_file: str
    columns: list[ColumnDef]
    indices: list[IndexDef]

    def validate(self) -> None:
        if not re.match(r"^[A-Z][a-zA-Z0-9]*$", self.name):
            raise ValueError(f"Table name '{self.name}' must be PascalCase.")
        if self.name.endswith("s") and not self.name.endswith("Status"):
            plural_exceptions = {"ConversationSequence", "ProjectSequence"}
            if self.name not in plural_exceptions and self.name.endswith("s"):
                raise ValueError(f"Table name '{self.name}' must be singular PascalCase (found trailing 's').")
        for col in self.columns:
            col.validate(self.name)


@dataclass
class ViewDef:
    name: str
    db_file: str
    columns: list[tuple[str, str]]  # (Name, RustType)
    sql_query: str

    def validate(self) -> None:
        if not self.name.startswith("Vw"):
            raise ValueError(f"View name '{self.name}' must start with 'Vw'.")
        if not re.match(r"^Vw[A-Z][a-zA-Z0-9]*$", self.name):
            raise ValueError(f"View name '{self.name}' must be PascalCase.")


# Declarative Schema Registry
SCHEMA_TABLES: list[TableDef] = [
    # -------------------------------------------------------------
    # 1. errors.db (System Diagnostics)
    # -------------------------------------------------------------
    TableDef(
        name="SchemaVersion",
        db_file="errors.db",
        columns=[
            ColumnDef("SchemaVersionId", "i64", "INTEGER", is_primary_key=True, is_autoincrement=True),
            ColumnDef("VersionNumber", "i64", "INTEGER"),
            ColumnDef("AppliedAt", "i64", "INTEGER"),
            ColumnDef("Description", "String", "TEXT"),
            ColumnDef("Checksum", "String", "TEXT"),
        ],
        indices=[],
    ),
    TableDef(
        name="ErrorLog",
        db_file="errors.db",
        columns=[
            ColumnDef("ErrorLogId", "String", "TEXT", is_primary_key=True),
            ColumnDef("ErrorCode", "String", "TEXT"),
            ColumnDef("Severity", "String", "TEXT"),
            ColumnDef("Message", "String", "TEXT"),
            ColumnDef("StackTrace", "Option<String>", "TEXT", is_nullable=True),
            ColumnDef("Source", "Option<String>", "TEXT", is_nullable=True),
            ColumnDef("TriggerAction", "Option<String>", "TEXT", is_nullable=True),
            ColumnDef("MetadataJson", "Option<String>", "TEXT", is_nullable=True),
            ColumnDef("CreatedAt", "i64", "INTEGER"),
        ],
        indices=[
            IndexDef("IdxErrorLogCreatedAt", ["CreatedAt DESC"]),
            IndexDef("IdxErrorLogErrorCode", ["ErrorCode"]),
            IndexDef("IdxErrorLogSeverity", ["Severity"]),
        ],
    ),

    # -------------------------------------------------------------
    # 2. repo-prompts.db (Running Projects & Active Prompts)
    # -------------------------------------------------------------
    TableDef(
        name="RunningProject",
        db_file="repo-prompts.db",
        columns=[
            ColumnDef("RunningProjectId", "String", "TEXT", is_primary_key=True),
            ColumnDef("InstanceId", "String", "TEXT"),
            ColumnDef("RepoName", "String", "TEXT"),
            ColumnDef("RepoPath", "String", "TEXT"),
            ColumnDef("WorkspaceStoragePath", "Option<String>", "TEXT", is_nullable=True),
            ColumnDef("IsRunning", "bool", "INTEGER", default_value="1"),
            ColumnDef("LastDetectedAt", "i64", "INTEGER"),
            ColumnDef("UpdatedAt", "i64", "INTEGER"),
        ],
        indices=[
            IndexDef("IdxRunningProjectInstanceRunning", ["InstanceId", "IsRunning"]),
        ],
    ),
    TableDef(
        name="ActivePrompt",
        db_file="repo-prompts.db",
        columns=[
            ColumnDef("ActivePromptId", "String", "TEXT", is_primary_key=True),
            ColumnDef("RunningProjectId", "String", "TEXT", foreign_key="RunningProject(RunningProjectId)"),
            ColumnDef("InstanceId", "String", "TEXT"),
            ColumnDef("RepoPath", "String", "TEXT"),
            ColumnDef("PromptContent", "String", "TEXT"),
            ColumnDef("Model", "Option<String>", "TEXT", is_nullable=True),
            ColumnDef("SessionId", "Option<String>", "TEXT", is_nullable=True),
            ColumnDef("Status", "String", "TEXT"),
            ColumnDef("CreatedAt", "i64", "INTEGER"),
            ColumnDef("UpdatedAt", "i64", "INTEGER"),
            ColumnDef("ImagePayload", "Option<String>", "TEXT", is_nullable=True),
        ],
        indices=[
            IndexDef("IdxActivePromptInstanceStatus", ["InstanceId", "Status"]),
            IndexDef("IdxActivePromptCreatedAt", ["CreatedAt ASC"]),
        ],
    ),
    TableDef(
        name="ProjectSequence",
        db_file="repo-prompts.db",
        columns=[
            ColumnDef("ProjectSequenceId", "String", "TEXT", is_primary_key=True),
            ColumnDef("SequenceId", "i64", "INTEGER"),
            ColumnDef("RunningProjectId", "String", "TEXT"),
            ColumnDef("RepoName", "String", "TEXT"),
            ColumnDef("RepoPath", "String", "TEXT"),
            ColumnDef("InstanceId", "String", "TEXT", default_value="'default'"),
            ColumnDef("UpdatedAt", "i64", "INTEGER"),
        ],
        indices=[
            IndexDef("IdxProjectSequenceSeqId", ["SequenceId"], is_unique=True),
        ],
    ),
    TableDef(
        name="ConversationSequence",
        db_file="repo-prompts.db",
        columns=[
            ColumnDef("ConversationSequenceId", "String", "TEXT", is_primary_key=True),
            ColumnDef("SequenceId", "i64", "INTEGER"),
            ColumnDef("ProjectKey", "String", "TEXT"),
            ColumnDef("Title", "String", "TEXT"),
            ColumnDef("InstanceId", "String", "TEXT", default_value="'default'"),
            ColumnDef("UpdatedAt", "i64", "INTEGER"),
        ],
        indices=[
            IndexDef("IdxConversationSequenceSeqId", ["SequenceId"], is_unique=True),
        ],
    ),
    TableDef(
        name="FailedCommand",
        db_file="repo-prompts.db",
        columns=[
            ColumnDef("FailedCommandId", "i64", "INTEGER", is_primary_key=True, is_autoincrement=True),
            ColumnDef("Command", "String", "TEXT", default_value="''"),
            ColumnDef("FullArgs", "String", "TEXT", default_value="''"),
            ColumnDef("Domain", "String", "TEXT", default_value="'root'"),
            ColumnDef("ErrorCode", "String", "TEXT", default_value="'E1001'"),
            ColumnDef("Message", "String", "TEXT", default_value="''"),
            ColumnDef("Suggestions", "String", "TEXT", default_value="''"),
            ColumnDef("HitCount", "i64", "INTEGER", default_value="1"),
            ColumnDef("WorkingDir", "String", "TEXT", default_value="''"),
            ColumnDef("AgmVersion", "String", "TEXT", default_value="''"),
            ColumnDef("IsResolved", "bool", "INTEGER", default_value="0"),
            ColumnDef("CreatedAt", "String", "TEXT", default_value="CURRENT_TIMESTAMP"),
            ColumnDef("LastSeenAt", "String", "TEXT", default_value="CURRENT_TIMESTAMP"),
        ],
        indices=[
            IndexDef("IdxFailedCommandCmdDomain", ["Command", "Domain"]),
            IndexDef("IdxFailedCommandHitCount", ["HitCount DESC"]),
        ],
    ),
    TableDef(
        name="PromptTreeCache",
        db_file="repo-prompts.db",
        columns=[
            ColumnDef("PromptTreeCacheId", "String", "TEXT", is_primary_key=True),
            ColumnDef("InstanceId", "String", "TEXT"),
            ColumnDef("TreeJson", "String", "TEXT"),
            ColumnDef("ProjectCount", "i64", "INTEGER"),
            ColumnDef("ConversationCount", "i64", "INTEGER"),
            ColumnDef("UpdatedAt", "i64", "INTEGER"),
            ColumnDef("TtlSeconds", "i64", "INTEGER", default_value="60"),
        ],
        indices=[],
    ),
    TableDef(
        name="MediaVault",
        db_file="repo-prompts.db",
        columns=[
            ColumnDef("MediaVaultId", "String", "TEXT", is_primary_key=True),
            ColumnDef("Sha256", "String", "TEXT"),
            ColumnDef("LocalPath", "Option<String>", "TEXT", is_nullable=True),
            ColumnDef("PublicUrl", "Option<String>", "TEXT", is_nullable=True),
            ColumnDef("TelegramFileId", "Option<String>", "TEXT", is_nullable=True),
            ColumnDef("MimeType", "Option<String>", "TEXT", is_nullable=True),
            ColumnDef("CreatedAt", "i64", "INTEGER"),
        ],
        indices=[
            IndexDef("IdxMediaVaultSha256", ["Sha256"], is_unique=True),
        ],
    ),

    # -------------------------------------------------------------
    # 3. prompt-backups.db (Switch Resumption Backups)
    # -------------------------------------------------------------
    TableDef(
        name="BackupBatch",
        db_file="prompt-backups.db",
        columns=[
            ColumnDef("BackupBatchId", "String", "TEXT", is_primary_key=True),
            ColumnDef("CreatedAt", "i64", "INTEGER"),
            ColumnDef("PromptsCount", "i64", "INTEGER"),
            ColumnDef("FilePath", "String", "TEXT"),
            ColumnDef("RetentionDays", "i64", "INTEGER", default_value="1"),
            ColumnDef("IsFullyRestored", "bool", "INTEGER", default_value="0"),
        ],
        indices=[],
    ),
    TableDef(
        name="PromptBackup",
        db_file="prompt-backups.db",
        columns=[
            ColumnDef("PromptBackupId", "String", "TEXT", is_primary_key=True),
            ColumnDef("BackupBatchId", "String", "TEXT", foreign_key="BackupBatch(BackupBatchId)"),
            ColumnDef("OriginalPromptId", "String", "TEXT"),
            ColumnDef("ProjectName", "String", "TEXT"),
            ColumnDef("ProjectPath", "String", "TEXT"),
            ColumnDef("RunningProjectId", "String", "TEXT"),
            ColumnDef("ConversationId", "String", "TEXT"),
            ColumnDef("ConversationName", "Option<String>", "TEXT", is_nullable=True),
            ColumnDef("SequenceId", "i64", "INTEGER"),
            ColumnDef("PromptText", "String", "TEXT"),
            ColumnDef("HasImages", "bool", "INTEGER", default_value="0"),
            ColumnDef("ImagesPayload", "Option<String>", "TEXT", is_nullable=True),
            ColumnDef("Status", "String", "TEXT", default_value="'queued'"),
            ColumnDef("CreatedAt", "i64", "INTEGER"),
            ColumnDef("IsRestored", "bool", "INTEGER", default_value="0"),
            ColumnDef("RestoredAt", "Option<i64>", "INTEGER", is_nullable=True),
            ColumnDef("InstanceId", "String", "TEXT", default_value="'default'"),
        ],
        indices=[
            IndexDef("IdxPromptBackupBatch", ["BackupBatchId"]),
            IndexDef("IdxPromptBackupRestored", ["IsRestored", "RestoredAt"]),
        ],
    ),
    TableDef(
        name="GreenProject",
        db_file="prompt-backups.db",
        columns=[
            ColumnDef("GreenProjectId", "String", "TEXT", is_primary_key=True),
            ColumnDef("ProjectIdentifier", "String", "TEXT"),
            ColumnDef("ProjectPath", "String", "TEXT"),
            ColumnDef("AddedAt", "i64", "INTEGER"),
            ColumnDef("Status", "String", "TEXT", default_value="'pending'"),
            ColumnDef("LastCheckedAt", "Option<i64>", "INTEGER", is_nullable=True),
        ],
        indices=[
            IndexDef("IdxGreenProjectIdentifier", ["ProjectIdentifier"], is_unique=True),
        ],
    ),

    # -------------------------------------------------------------
    # 4. task-history.db (Task Execution Audit Trail)
    # -------------------------------------------------------------
    TableDef(
        name="TaskHistory",
        db_file="task-history.db",
        columns=[
            ColumnDef("TaskHistoryId", "String", "TEXT", is_primary_key=True),
            ColumnDef("Action", "String", "TEXT"),
            ColumnDef("ActionCode", "Option<i64>", "INTEGER", is_nullable=True),
            ColumnDef("Status", "String", "TEXT"),
            ColumnDef("Subject", "String", "TEXT"),
            ColumnDef("Detail", "String", "TEXT", default_value="''"),
            ColumnDef("InstanceId", "String", "TEXT", default_value="'default'"),
            ColumnDef("SplitPath", "String", "TEXT", default_value="''"),
            ColumnDef("FromEmail", "String", "TEXT", default_value="''"),
            ColumnDef("ToEmail", "String", "TEXT", default_value="''"),
            ColumnDef("CreatedAt", "i64", "INTEGER"),
            ColumnDef("FinishedAt", "Option<i64>", "INTEGER", is_nullable=True),
        ],
        indices=[
            IndexDef("IdxTaskHistoryCreatedAt", ["CreatedAt DESC"]),
            IndexDef("IdxTaskHistoryStatus", ["Status"]),
            IndexDef("IdxTaskHistoryAction", ["Action"]),
        ],
    ),
]

SCHEMA_VIEWS: list[ViewDef] = [
    # -------------------------------------------------------------
    # 1. errors.db Views
    # -------------------------------------------------------------
    ViewDef(
        name="VwRecentErrors",
        db_file="errors.db",
        columns=[
            ("ErrorLogId", "String"),
            ("ErrorCode", "String"),
            ("Severity", "String"),
            ("Message", "String"),
            ("Source", "Option<String>"),
            ("TriggerAction", "Option<String>"),
            ("CreatedAt", "i64"),
        ],
        sql_query="""SELECT 
    ErrorLogId,
    ErrorCode,
    Severity,
    Message,
    Source,
    TriggerAction,
    CreatedAt
FROM ErrorLog
ORDER BY CreatedAt DESC
LIMIT 100""",
    ),
    ViewDef(
        name="VwErrorSummaryByCode",
        db_file="errors.db",
        columns=[
            ("ErrorCode", "String"),
            ("Severity", "String"),
            ("OccurrencesCount", "i64"),
            ("FirstSeenAt", "i64"),
            ("LastSeenAt", "i64"),
        ],
        sql_query="""SELECT 
    ErrorCode,
    Severity,
    COUNT(*) AS OccurrencesCount,
    MIN(CreatedAt) AS FirstSeenAt,
    MAX(CreatedAt) AS LastSeenAt
FROM ErrorLog
GROUP BY ErrorCode, Severity
ORDER BY OccurrencesCount DESC""",
    ),

    # -------------------------------------------------------------
    # 2. repo-prompts.db Views
    # -------------------------------------------------------------
    ViewDef(
        name="VwActivePromptsWithProjects",
        db_file="repo-prompts.db",
        columns=[
            ("ActivePromptId", "String"),
            ("RunningProjectId", "String"),
            ("InstanceId", "String"),
            ("RepoName", "Option<String>"),
            ("RepoPath", "String"),
            ("PromptContent", "String"),
            ("Model", "Option<String>"),
            ("SessionId", "Option<String>"),
            ("Status", "String"),
            ("CreatedAt", "i64"),
            ("UpdatedAt", "i64"),
            ("ImagePayload", "Option<String>"),
            ("IsProjectRunning", "Option<bool>"),
        ],
        sql_query="""SELECT 
    p.ActivePromptId,
    p.RunningProjectId,
    p.InstanceId,
    r.RepoName,
    p.RepoPath,
    p.PromptContent,
    p.Model,
    p.SessionId,
    p.Status,
    p.CreatedAt,
    p.UpdatedAt,
    p.ImagePayload,
    r.IsRunning AS IsProjectRunning
FROM ActivePrompt p
LEFT JOIN RunningProject r ON p.RunningProjectId = r.RunningProjectId""",
    ),
    ViewDef(
        name="VwQueuedPromptsFifo",
        db_file="repo-prompts.db",
        columns=[
            ("ActivePromptId", "String"),
            ("RunningProjectId", "String"),
            ("InstanceId", "String"),
            ("RepoName", "Option<String>"),
            ("RepoPath", "String"),
            ("PromptContent", "String"),
            ("Model", "Option<String>"),
            ("SessionId", "Option<String>"),
            ("Status", "String"),
            ("CreatedAt", "i64"),
        ],
        sql_query="""SELECT 
    p.ActivePromptId,
    p.RunningProjectId,
    p.InstanceId,
    r.RepoName,
    p.RepoPath,
    p.PromptContent,
    p.Model,
    p.SessionId,
    p.Status,
    p.CreatedAt
FROM ActivePrompt p
LEFT JOIN RunningProject r ON p.RunningProjectId = r.RunningProjectId
WHERE p.Status = 'queued'
ORDER BY p.CreatedAt ASC, p.ActivePromptId ASC""",
    ),

    # -------------------------------------------------------------
    # 3. prompt-backups.db Views
    # -------------------------------------------------------------
    ViewDef(
        name="VwUnrestoredBackups",
        db_file="prompt-backups.db",
        columns=[
            ("PromptBackupId", "String"),
            ("BackupBatchId", "String"),
            ("OriginalPromptId", "String"),
            ("ProjectName", "String"),
            ("ProjectPath", "String"),
            ("ConversationId", "String"),
            ("SequenceId", "i64"),
            ("PromptText", "String"),
            ("HasImages", "bool"),
            ("Status", "String"),
            ("CreatedAt", "i64"),
            ("InstanceId", "String"),
            ("BatchFilePath", "String"),
        ],
        sql_query="""SELECT 
    pb.PromptBackupId,
    pb.BackupBatchId,
    pb.OriginalPromptId,
    pb.ProjectName,
    pb.ProjectPath,
    pb.ConversationId,
    pb.SequenceId,
    pb.PromptText,
    pb.HasImages,
    pb.Status,
    pb.CreatedAt,
    pb.InstanceId,
    bb.FilePath AS BatchFilePath
FROM PromptBackup pb
JOIN BackupBatch bb ON pb.BackupBatchId = bb.BackupBatchId
WHERE pb.IsRestored = 0
ORDER BY pb.CreatedAt ASC, pb.PromptBackupId ASC""",
    ),
    ViewDef(
        name="VwLatestBackupBatches",
        db_file="prompt-backups.db",
        columns=[
            ("BackupBatchId", "String"),
            ("CreatedAt", "i64"),
            ("PromptsCount", "i64"),
            ("FilePath", "String"),
            ("RetentionDays", "i64"),
            ("IsFullyRestored", "bool"),
            ("TotalBackupsCount", "i64"),
            ("RestoredCount", "i64"),
        ],
        sql_query="""SELECT 
    bb.BackupBatchId,
    bb.CreatedAt,
    bb.PromptsCount,
    bb.FilePath,
    bb.RetentionDays,
    bb.IsFullyRestored,
    COUNT(pb.PromptBackupId) AS TotalBackupsCount,
    COALESCE(SUM(CASE WHEN pb.IsRestored = 1 THEN 1 ELSE 0 END), 0) AS RestoredCount
FROM BackupBatch bb
LEFT JOIN PromptBackup pb ON bb.BackupBatchId = pb.BackupBatchId
GROUP BY bb.BackupBatchId
ORDER BY bb.CreatedAt DESC""",
    ),

    # -------------------------------------------------------------
    # 4. task-history.db Views
    # -------------------------------------------------------------
    ViewDef(
        name="VwRecentTaskHistory",
        db_file="task-history.db",
        columns=[
            ("TaskHistoryId", "String"),
            ("Action", "String"),
            ("ActionCode", "Option<i64>"),
            ("Status", "String"),
            ("Subject", "String"),
            ("Detail", "String"),
            ("InstanceId", "String"),
            ("FromEmail", "String"),
            ("ToEmail", "String"),
            ("CreatedAt", "i64"),
            ("FinishedAt", "Option<i64>"),
        ],
        sql_query="""SELECT 
    TaskHistoryId,
    Action,
    ActionCode,
    Status,
    Subject,
    Detail,
    InstanceId,
    FromEmail,
    ToEmail,
    CreatedAt,
    FinishedAt
FROM TaskHistory
ORDER BY CreatedAt DESC
LIMIT 100""",
    ),
]


def generate_ddl_for_table(table: TableDef) -> str:
    lines = [f"CREATE TABLE IF NOT EXISTS {table.name} ("]
    col_defs = []
    for col in table.columns:
        c_str = f"    {col.name} {col.sql_type}"
        if col.is_primary_key:
            if col.is_autoincrement:
                c_str += " PRIMARY KEY AUTOINCREMENT"
            else:
                c_str += " PRIMARY KEY"
        elif not col.is_nullable:
            c_str += " NOT NULL"

        if col.default_value is not None:
            c_str += f" DEFAULT {col.default_value}"

        if col.foreign_key:
            c_str += f" REFERENCES {col.foreign_key} ON DELETE CASCADE"

        col_defs.append(c_str)

    lines.append(",\n".join(col_defs))
    lines.append(");")

    index_lines = []
    for idx in table.indices:
        unique_clause = "UNIQUE " if idx.is_unique else ""
        cols_clause = ", ".join(idx.columns)
        index_lines.append(f"CREATE {unique_clause}INDEX IF NOT EXISTS {idx.name} ON {table.name}({cols_clause});")

    if index_lines:
        lines.append("")
        lines.extend(index_lines)

    return "\n".join(lines)


def generate_entities_rs(tables: list[TableDef]) -> str:
    code = [
        "// Auto-generated by 03-ai-scripts/51-db-orm-generator.py. DO NOT EDIT DIRECTLY.",
        "",
        "use serde::{Deserialize, Serialize};",
        "use rusqlite::Row;",
        "use crate::modules::db_orm::traits::{Entity, FromRow, ToValues, ValueParam};",
        "use crate::error::AppError;",
        "",
    ]

    for table in tables:
        pk_col = next(c for c in table.columns if c.is_primary_key)
        code.append(f"/// Model entity for database table `{table.name}`.")
        code.append("#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]")
        code.append('#[serde(rename_all = "PascalCase")]')
        code.append(f"pub struct {table.name} {{")
        for col in table.columns:
            code.append(f"    pub {to_snake_case(col.name)}: {col.rust_type},")
        code.append("}")
        code.append("")

        # Implement Entity
        code.append(f"impl Entity for {table.name} {{")
        code.append(f'    const TABLE_NAME: &\'static str = "{table.name}";')
        code.append(f'    const PRIMARY_KEY: &\'static str = "{pk_col.name}";')
        code.append(f"    type IdType = {pk_col.rust_type};")
        code.append("")
        code.append("    fn id(&self) -> &Self::IdType {")
        code.append(f"        &self.{to_snake_case(pk_col.name)}")
        code.append("    }")
        code.append("}")
        code.append("")

        # Implement FromRow
        code.append(f"impl FromRow for {table.name} {{")
        code.append("    fn from_row(row: &Row) -> Result<Self, AppError> {")
        code.append("        Ok(Self {")
        for col in table.columns:
            field_name = to_snake_case(col.name)
            col_name = col.name
            if col.rust_type == "bool":
                code.append(f'            {field_name}: row.get::<_, i64>("{col_name}").map(|v| v != 0)?,')
            elif col.rust_type == "Option<bool>":
                code.append(f'            {field_name}: row.get::<_, Option<i64>>("{col_name}").map(|opt| opt.map(|v| v != 0))?,')
            else:
                code.append(f'            {field_name}: row.get("{col_name}")?,')
        code.append("        })")
        code.append("    }")
        code.append("}")
        code.append("")

        # Implement ToValues
        code.append(f"impl ToValues for {table.name} {{")
        code.append("    fn to_columns() -> &'static [&'static str] {")
        cols_quoted = [f'"{c.name}"' for c in table.columns]
        code.append(f"        &[{', '.join(cols_quoted)}]")
        code.append("    }")
        code.append("")
        code.append("    fn to_values(&self) -> Vec<(&'static str, ValueParam)> {")
        code.append("        vec![")
        for col in table.columns:
            field_name = to_snake_case(col.name)
            col_name = col.name
            if col.rust_type == "bool":
                code.append(f'            ("{col_name}", ValueParam::Int(if self.{field_name} {{ 1 }} else {{ 0 }})),')
            elif col.rust_type == "i64":
                code.append(f'            ("{col_name}", ValueParam::Int(self.{field_name})),')
            elif col.rust_type == "f64":
                code.append(f'            ("{col_name}", ValueParam::Float(self.{field_name})),')
            elif col.rust_type == "String":
                code.append(f'            ("{col_name}", ValueParam::Text(self.{field_name}.clone())),')
            elif col.rust_type == "Option<i64>":
                code.append(f'            ("{col_name}", match self.{field_name} {{ Some(v) => ValueParam::Int(v), None => ValueParam::Null }}),')
            elif col.rust_type == "Option<f64>":
                code.append(f'            ("{col_name}", match self.{field_name} {{ Some(v) => ValueParam::Float(v), None => ValueParam::Null }}),')
            elif col.rust_type == "Option<String>":
                code.append(f'            ("{col_name}", match &self.{field_name} {{ Some(v) => ValueParam::Text(v.clone()), None => ValueParam::Null }}),')
            elif col.rust_type == "Option<bool>":
                code.append(f'            ("{col_name}", match self.{field_name} {{ Some(v) => ValueParam::Int(if v {{ 1 }} else {{ 0 }}), None => ValueParam::Null }}),')
            else:
                code.append(f'            ("{col_name}", ValueParam::Text(format!("{{:?}}", self.{field_name}))),')
        code.append("        ]")
        code.append("    }")
        code.append("}")
        code.append("")

    return "\n".join(code)


def generate_tables_rs(tables: list[TableDef]) -> str:
    code = [
        "// Auto-generated by 03-ai-scripts/51-db-orm-generator.py. DO NOT EDIT DIRECTLY.",
        "",
        "pub struct TableSchema {",
        "    pub table_name: &'static str,",
        "    pub db_file: &'static str,",
        "    pub ddl: &'static str,",
        "}",
        "",
    ]

    for table in tables:
        ddl = generate_ddl_for_table(table)
        code.append(f"pub mod {to_snake_case(table.name)}_table {{")
        code.append(f'    pub const TABLE_NAME: &str = "{table.name}";')
        code.append(f'    pub const DB_FILE: &str = "{table.db_file}";')
        for col in table.columns:
            code.append(f'    pub const COL_{to_screaming_snake_case(col.name)}: &str = "{col.name}";')
        code.append("")
        escaped_ddl = ddl.replace('"', '\\"')
        code.append(f'    pub const DDL: &str = "{escaped_ddl}";')
        code.append("}")
        code.append("")

    code.append("pub const ALL_TABLES: &[TableSchema] = &[")
    for table in tables:
        mod_name = to_snake_case(table.name)
        code.append(f"    TableSchema {{")
        code.append(f"        table_name: {mod_name}_table::TABLE_NAME,")
        code.append(f"        db_file: {mod_name}_table::DB_FILE,")
        code.append(f"        ddl: {mod_name}_table::DDL,")
        code.append(f"    }},")
    code.append("];")
    code.append("")

    return "\n".join(code)


def generate_views_rs(views: list[ViewDef]) -> str:
    code = [
        "// Auto-generated by 03-ai-scripts/51-db-orm-generator.py. DO NOT EDIT DIRECTLY.",
        "",
        "use serde::{Deserialize, Serialize};",
        "use rusqlite::Row;",
        "use crate::modules::db_orm::traits::{ViewEntity, FromRow};",
        "use crate::error::AppError;",
        "",
        "pub struct ViewSchema {",
        "    pub view_name: &'static str,",
        "    pub db_file: &'static str,",
        "    pub ddl: &'static str,",
        "}",
        "",
    ]

    for view in views:
        code.append(f"/// Readonly view model for `{view.name}`.")
        code.append("#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]")
        code.append('#[serde(rename_all = "PascalCase")]')
        code.append(f"pub struct {view.name} {{")
        for col_name, rust_type in view.columns:
            code.append(f"    pub {to_snake_case(col_name)}: {rust_type},")
        code.append("}")
        code.append("")

        # Implement ViewEntity
        view_ddl = f"CREATE VIEW IF NOT EXISTS {view.name} AS {view.sql_query};"
        escaped_view_ddl = view_ddl.replace('"', '\\"').replace("\n", " ")

        code.append(f"impl ViewEntity for {view.name} {{")
        code.append(f'    const VIEW_NAME: &\'static str = "{view.name}";')
        code.append(f'    const DB_FILE: &\'static str = "{view.db_file}";')
        code.append(f'    const DDL: &\'static str = "{escaped_view_ddl}";')
        code.append("}")
        code.append("")

        # Implement FromRow
        code.append(f"impl FromRow for {view.name} {{")
        code.append("    fn from_row(row: &Row) -> Result<Self, AppError> {")
        code.append("        Ok(Self {")
        for col_name, rust_type in view.columns:
            field_name = to_snake_case(col_name)
            if rust_type == "bool":
                code.append(f'            {field_name}: row.get::<_, i64>("{col_name}").map(|v| v != 0)?,')
            elif rust_type == "Option<bool>":
                code.append(f'            {field_name}: row.get::<_, Option<i64>>("{col_name}").map(|opt| opt.map(|v| v != 0))?,')
            else:
                code.append(f'            {field_name}: row.get("{col_name}")?,')
        code.append("        })")
        code.append("    }")
        code.append("}")
        code.append("")

    code.append("pub const ALL_VIEWS: &[ViewSchema] = &[")
    for view in views:
        code.append(f"    ViewSchema {{")
        code.append(f"        view_name: {view.name}::VIEW_NAME,")
        code.append(f"        db_file: {view.name}::DB_FILE,")
        code.append(f"        ddl: {view.name}::DDL,")
        code.append(f"    }},")
    code.append("];")
    code.append("")

    return "\n".join(code)


def to_diesel_sql_type(rust_type: str) -> str:
    if rust_type == "i64":
        return "BigInt"
    elif rust_type == "Option<i64>":
        return "Nullable<BigInt>"
    elif rust_type == "String":
        return "Text"
    elif rust_type == "Option<String>":
        return "Nullable<Text>"
    elif rust_type == "bool":
        return "Bool"
    elif rust_type == "Option<bool>":
        return "Nullable<Bool>"
    else:
        raise ValueError(f"Unsupported Rust type for Diesel: {rust_type}")


def generate_diesel_schema_rs(tables: list[TableDef]) -> str:
    code = [
        "// Auto-generated by 03-ai-scripts/51-db-orm-generator.py. DO NOT EDIT DIRECTLY.",
        "#![allow(non_snake_case)]",
        "",
    ]
    for table in tables:
        pk_col = next((c.name for c in table.columns if c.is_primary_key), f"{table.name}Id")
        code.append("diesel::table! {")
        code.append("    use diesel::sql_types::*;")
        code.append("")
        code.append(f"    {table.name} ({pk_col}) {{")
        for col in table.columns:
            sql_type = to_diesel_sql_type(col.rust_type)
            code.append(f"        {col.name} -> {sql_type},")
        code.append("    }")
        code.append("}")
        code.append("")
    return "\n".join(code)


def generate_diesel_models_rs(tables: list[TableDef]) -> str:
    code = [
        "// Auto-generated by 03-ai-scripts/51-db-orm-generator.py. DO NOT EDIT DIRECTLY.",
        "#![allow(non_snake_case)]",
        "",
        "use diesel::prelude::*;",
        "use serde::{Deserialize, Serialize};",
        "use super::diesel_schema;",
        "use super::entities;",
        "",
    ]
    for table in tables:
        struct_name = f"Diesel{table.name}"
        pk_col = next((c.name for c in table.columns if c.is_primary_key), f"{table.name}Id")
        code.append(f"/// Compile-time checked Diesel model for `{table.name}`.")
        code.append("#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone, PartialEq, Serialize, Deserialize)]")
        code.append(f"#[diesel(table_name = diesel_schema::{table.name})]")
        code.append(f"#[diesel(primary_key({pk_col}))]")
        code.append("#[diesel(check_for_backend(diesel::sqlite::Sqlite))]")
        code.append(f"pub struct {struct_name} {{")
        for col in table.columns:
            field_name = to_snake_case(col.name)
            code.append(f"    #[diesel(column_name = {col.name})]")
            code.append(f"    pub {field_name}: {col.rust_type},")
        code.append("}")
        code.append("")

        code.append(f"impl From<{struct_name}> for entities::{table.name} {{")
        code.append(f"    fn from(m: {struct_name}) -> Self {{")
        code.append(f"        entities::{table.name} {{")
        for col in table.columns:
            field_name = to_snake_case(col.name)
            code.append(f"            {field_name}: m.{field_name},")
        code.append("        }")
        code.append("    }")
        code.append("}")
        code.append("")

        code.append(f"impl From<entities::{table.name}> for {struct_name} {{")
        code.append(f"    fn from(e: entities::{table.name}) -> Self {{")
        code.append("        Self {")
        for col in table.columns:
            field_name = to_snake_case(col.name)
            code.append(f"            {field_name}: e.{field_name},")
        code.append("        }")
        code.append("    }")
        code.append("}")
        code.append("")

    return "\n".join(code)


def to_snake_case(name: str) -> str:
    s = re.sub(r"(.)([A-Z][a-z]+)", r"\1_\2", name)
    return re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", s).lower()


def to_screaming_snake_case(name: str) -> str:
    return to_snake_case(name).upper()


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate Rust ORM models and schema DDLs.")
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path("src-tauri/src/modules/db_orm/generated"),
        help="Target directory for generated Rust files",
    )
    parser.add_argument("--dry-run", action="store_true", help="Print summary without writing files")
    args = parser.parse_args()

    # 1. Validate schemas
    print(f"Validating {len(SCHEMA_TABLES)} tables and {len(SCHEMA_VIEWS)} views...")
    for table in SCHEMA_TABLES:
        table.validate()
    for view in SCHEMA_VIEWS:
        view.validate()
    print("Schema validation passed.")

    # 2. Generate code
    entities_code = generate_entities_rs(SCHEMA_TABLES)
    tables_code = generate_tables_rs(SCHEMA_TABLES)
    views_code = generate_views_rs(SCHEMA_VIEWS)
    diesel_schema_code = generate_diesel_schema_rs(SCHEMA_TABLES)
    diesel_models_code = generate_diesel_models_rs(SCHEMA_TABLES)

    mod_code = """// Auto-generated by 03-ai-scripts/51-db-orm-generator.py. DO NOT EDIT DIRECTLY.

pub mod diesel_models;
pub mod diesel_schema;
pub mod entities;
pub mod tables;
pub mod views;

// Note: diesel_schema defines table modules (e.g. ErrorLog).
// diesel_models and diesel_schema are scoped under their submodules to prevent collisions with entity structs.
pub use entities::*;
pub use tables::*;
pub use views::*;
"""

    if args.dry_run:
        print("[Dry Run] Would write:")
        print(f"  {args.out_dir / 'entities.rs'} ({len(entities_code.splitlines())} lines)")
        print(f"  {args.out_dir / 'tables.rs'} ({len(tables_code.splitlines())} lines)")
        print(f"  {args.out_dir / 'views.rs'} ({len(views_code.splitlines())} lines)")
        print(f"  {args.out_dir / 'diesel_schema.rs'} ({len(diesel_schema_code.splitlines())} lines)")
        print(f"  {args.out_dir / 'diesel_models.rs'} ({len(diesel_models_code.splitlines())} lines)")
        print(f"  {args.out_dir / 'mod.rs'} ({len(mod_code.splitlines())} lines)")
        return ExitCodeType.SUCCESS

    args.out_dir.mkdir(parents=True, exist_ok=True)
    (args.out_dir / "entities.rs").write_text(entities_code, encoding="utf-8")
    (args.out_dir / "tables.rs").write_text(tables_code, encoding="utf-8")
    (args.out_dir / "views.rs").write_text(views_code, encoding="utf-8")
    (args.out_dir / "diesel_schema.rs").write_text(diesel_schema_code, encoding="utf-8")
    (args.out_dir / "diesel_models.rs").write_text(diesel_models_code, encoding="utf-8")
    (args.out_dir / "mod.rs").write_text(mod_code, encoding="utf-8")

    # Format generated files with rustfmt if available
    try:
        import subprocess
        subprocess.run(
            [
                "rustfmt",
                str(args.out_dir / "entities.rs"),
                str(args.out_dir / "tables.rs"),
                str(args.out_dir / "views.rs"),
                str(args.out_dir / "diesel_schema.rs"),
                str(args.out_dir / "diesel_models.rs"),
                str(args.out_dir / "mod.rs"),
            ],
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    except Exception:
        pass

    print(f"Successfully generated ORM files in {args.out_dir}:")
    print(f"  - entities.rs ({len(SCHEMA_TABLES)} entities)")
    print(f"  - tables.rs ({len(SCHEMA_TABLES)} table DDLs)")
    print(f"  - views.rs ({len(SCHEMA_VIEWS)} views)")
    print(f"  - diesel_schema.rs ({len(SCHEMA_TABLES)} Diesel tables)")
    print(f"  - diesel_models.rs ({len(SCHEMA_TABLES)} Diesel models)")
    print("  - mod.rs")
    return ExitCodeType.SUCCESS


if __name__ == "__main__":
    sys.exit(main())
