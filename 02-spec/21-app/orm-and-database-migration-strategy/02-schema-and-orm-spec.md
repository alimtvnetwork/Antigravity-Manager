# Schema and ORM Specification: PascalCase Split-DB & Views Engine

**Specification File:** `02-spec/21-app/orm-and-database-migration-strategy/02-schema-and-orm-spec.md`  
**Parent Document:** [`readme.md`](readme.md)  
**Status:** DRAFT / IN PROGRESS  

---

## 1. Universal Schema Standards

All SQLite databases in Antigravity-Manager adhere to the following strict conventions:
1. **Database Pragmas:**
   ```sql
   PRAGMA journal_mode = WAL;
   PRAGMA synchronous = NORMAL;
   PRAGMA busy_timeout = 5000;
   PRAGMA foreign_keys = ON;
   ```
2. **Table Names:** Strictly singular PascalCase (e.g. `ErrorLog`, `ActivePrompt`, `RunningProject`, `PromptBackup`, `BackupBatch`).
3. **Column Names:** Strictly PascalCase (e.g. `ErrorLogId`, `PromptContent`, `CreatedAt`, `IsRunning`, `HasImages`).
4. **Primary Key Convention:** Primary keys are named `{TableName}Id` (e.g. `ErrorLogId`, `ActivePromptId`, `RunningProjectId`, `BackupBatchId`).
5. **Foreign Key Convention:** Foreign keys match the exact name of the referenced primary key.
6. **Positive Booleans:** Prefixed with `Is` or `Has` (`IsActive`, `HasImages`, `IsRestored`, `IsResolved`, `IsRunning`). Stored as `INTEGER NOT NULL DEFAULT 0` or `1`. Negative names (`IsNotActive`, `HasNoImages`) are strictly forbidden.
7. **Views for Complex Queries:** Any joined or aggregated query must be encapsulated in a pre-defined SQLite View prefixed with `Vw` (e.g. `VwActivePromptsWithProjects`, `VwRecentErrors`).

---

## 2. Split Database Schemas & DDL Definitions

### 2.1 `errors.db` (System & Client Diagnostics)

#### Table: `SchemaVersion`
```sql
CREATE TABLE IF NOT EXISTS SchemaVersion (
    SchemaVersionId INTEGER PRIMARY KEY AUTOINCREMENT,
    VersionNumber INTEGER NOT NULL,
    AppliedAt INTEGER NOT NULL,
    Description TEXT NOT NULL,
    Checksum TEXT NOT NULL
);
```

#### Table: `ErrorLog`
```sql
CREATE TABLE IF NOT EXISTS ErrorLog (
    ErrorLogId TEXT PRIMARY KEY,
    ErrorCode TEXT NOT NULL,
    Severity TEXT NOT NULL,
    Message TEXT NOT NULL,
    StackTrace TEXT,
    Source TEXT,
    TriggerAction TEXT,
    MetadataJson TEXT,
    CreatedAt INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS IdxErrorLogCreatedAt ON ErrorLog(CreatedAt DESC);
CREATE INDEX IF NOT EXISTS IdxErrorLogErrorCode ON ErrorLog(ErrorCode);
CREATE INDEX IF NOT EXISTS IdxErrorLogSeverity ON ErrorLog(Severity);
```

#### Views:
```sql
CREATE VIEW IF NOT EXISTS VwRecentErrors AS
SELECT 
    ErrorLogId,
    ErrorCode,
    Severity,
    Message,
    Source,
    TriggerAction,
    CreatedAt
FROM ErrorLog
ORDER BY CreatedAt DESC
LIMIT 100;

CREATE VIEW IF NOT EXISTS VwErrorSummaryByCode AS
SELECT 
    ErrorCode,
    Severity,
    COUNT(*) AS OccurrencesCount,
    MIN(CreatedAt) AS FirstSeenAt,
    MAX(CreatedAt) AS LastSeenAt
FROM ErrorLog
GROUP BY ErrorCode, Severity
ORDER BY OccurrencesCount DESC;
```

---

### 2.2 `repo-prompts.db` (Active Prompts & Running Projects)

#### Table: `RunningProject`
```sql
CREATE TABLE IF NOT EXISTS RunningProject (
    RunningProjectId TEXT PRIMARY KEY,
    InstanceId TEXT NOT NULL,
    RepoName TEXT NOT NULL,
    RepoPath TEXT NOT NULL,
    WorkspaceStoragePath TEXT,
    IsRunning INTEGER NOT NULL DEFAULT 1,
    LastDetectedAt INTEGER NOT NULL,
    UpdatedAt INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS IdxRunningProjectInstanceRunning 
ON RunningProject(InstanceId, IsRunning);
```

#### Table: `ActivePrompt`
```sql
CREATE TABLE IF NOT EXISTS ActivePrompt (
    ActivePromptId TEXT PRIMARY KEY,
    RunningProjectId TEXT NOT NULL,
    InstanceId TEXT NOT NULL,
    RepoPath TEXT NOT NULL,
    PromptContent TEXT NOT NULL,
    Model TEXT,
    SessionId TEXT,
    Status TEXT NOT NULL,
    CreatedAt INTEGER NOT NULL,
    UpdatedAt INTEGER NOT NULL,
    ImagePayload TEXT,
    FOREIGN KEY(RunningProjectId) REFERENCES RunningProject(RunningProjectId) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS IdxActivePromptInstanceStatus 
ON ActivePrompt(InstanceId, Status);
CREATE INDEX IF NOT EXISTS IdxActivePromptCreatedAt 
ON ActivePrompt(CreatedAt ASC);
```

#### Table: `ProjectSequence`
```sql
CREATE TABLE IF NOT EXISTS ProjectSequence (
    ProjectKey TEXT PRIMARY KEY,
    SequenceId INTEGER NOT NULL UNIQUE,
    RunningProjectId TEXT NOT NULL,
    RepoName TEXT NOT NULL,
    RepoPath TEXT NOT NULL,
    InstanceId TEXT NOT NULL DEFAULT 'default',
    UpdatedAt INTEGER NOT NULL
);
```

#### Table: `ConversationSequence`
```sql
CREATE TABLE IF NOT EXISTS ConversationSequence (
    ConversationId TEXT PRIMARY KEY,
    SequenceId INTEGER NOT NULL UNIQUE,
    ProjectKey TEXT NOT NULL,
    Title TEXT NOT NULL,
    InstanceId TEXT NOT NULL DEFAULT 'default',
    UpdatedAt INTEGER NOT NULL
);
```

#### Table: `FailedCommand`
```sql
CREATE TABLE IF NOT EXISTS FailedCommand (
    FailedCommandId INTEGER PRIMARY KEY AUTOINCREMENT,
    Command TEXT NOT NULL DEFAULT '',
    FullArgs TEXT NOT NULL DEFAULT '',
    Domain TEXT NOT NULL DEFAULT 'root',
    ErrorCode TEXT NOT NULL DEFAULT 'E1001',
    Message TEXT NOT NULL DEFAULT '',
    Suggestions TEXT NOT NULL DEFAULT '',
    HitCount INTEGER NOT NULL DEFAULT 1,
    WorkingDir TEXT NOT NULL DEFAULT '',
    AgmVersion TEXT NOT NULL DEFAULT '',
    IsResolved INTEGER NOT NULL DEFAULT 0,
    CreatedAt TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    LastSeenAt TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS IdxFailedCommandCmdDomain ON FailedCommand(Command, Domain);
CREATE INDEX IF NOT EXISTS IdxFailedCommandHitCount ON FailedCommand(HitCount DESC);
```

#### Table: `PromptTreeCache`
```sql
CREATE TABLE IF NOT EXISTS PromptTreeCache (
    CacheKey TEXT PRIMARY KEY,
    InstanceId TEXT NOT NULL,
    TreeJson TEXT NOT NULL,
    ProjectCount INTEGER NOT NULL,
    ConversationCount INTEGER NOT NULL,
    UpdatedAt INTEGER NOT NULL,
    TtlSeconds INTEGER NOT NULL DEFAULT 60
);
```

#### Table: `MediaVault`
```sql
CREATE TABLE IF NOT EXISTS MediaVault (
    MediaVaultId TEXT PRIMARY KEY,
    Sha256 TEXT NOT NULL UNIQUE,
    LocalPath TEXT,
    PublicUrl TEXT,
    TelegramFileId TEXT,
    MimeType TEXT,
    CreatedAt INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS IdxMediaVaultSha256 ON MediaVault(Sha256);
```

#### Views:
```sql
CREATE VIEW IF NOT EXISTS VwActivePromptsWithProjects AS
SELECT 
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
LEFT JOIN RunningProject r ON p.RunningProjectId = r.RunningProjectId;

CREATE VIEW IF NOT EXISTS VwQueuedPromptsFifo AS
SELECT 
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
ORDER BY p.CreatedAt ASC, p.ActivePromptId ASC;
```

---

### 2.3 `prompt-backups.db` (Instance Resumption Backups)

#### Table: `BackupBatch`
```sql
CREATE TABLE IF NOT EXISTS BackupBatch (
    BackupBatchId TEXT PRIMARY KEY,
    CreatedAt INTEGER NOT NULL,
    PromptsCount INTEGER NOT NULL,
    FilePath TEXT NOT NULL,
    RetentionDays INTEGER NOT NULL DEFAULT 1,
    IsFullyRestored INTEGER NOT NULL DEFAULT 0
);
```

#### Table: `PromptBackup`
```sql
CREATE TABLE IF NOT EXISTS PromptBackup (
    PromptBackupId TEXT PRIMARY KEY,
    BackupBatchId TEXT NOT NULL,
    OriginalPromptId TEXT NOT NULL,
    ProjectName TEXT NOT NULL,
    ProjectPath TEXT NOT NULL,
    RunningProjectId TEXT NOT NULL,
    ConversationId TEXT NOT NULL,
    ConversationName TEXT,
    SequenceId INTEGER NOT NULL,
    PromptText TEXT NOT NULL,
    HasImages INTEGER NOT NULL DEFAULT 0,
    ImagesPayload TEXT,
    Status TEXT NOT NULL DEFAULT 'queued',
    CreatedAt INTEGER NOT NULL,
    IsRestored INTEGER NOT NULL DEFAULT 0,
    RestoredAt INTEGER,
    InstanceId TEXT DEFAULT 'default',
    FOREIGN KEY(BackupBatchId) REFERENCES BackupBatch(BackupBatchId) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS IdxPromptBackupBatch ON PromptBackup(BackupBatchId);
CREATE INDEX IF NOT EXISTS IdxPromptBackupRestored ON PromptBackup(IsRestored, RestoredAt);
```

#### Table: `GreenProject`
```sql
CREATE TABLE IF NOT EXISTS GreenProject (
    GreenProjectId TEXT PRIMARY KEY,
    ProjectIdentifier TEXT NOT NULL UNIQUE,
    ProjectPath TEXT NOT NULL,
    AddedAt INTEGER NOT NULL,
    Status TEXT NOT NULL DEFAULT 'pending',
    LastCheckedAt INTEGER
);
```

#### Views:
```sql
CREATE VIEW IF NOT EXISTS VwUnrestoredBackups AS
SELECT 
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
ORDER BY pb.CreatedAt ASC, pb.PromptBackupId ASC;

CREATE VIEW IF NOT EXISTS VwLatestBackupBatches AS
SELECT 
    bb.BackupBatchId,
    bb.CreatedAt,
    bb.PromptsCount,
    bb.FilePath,
    bb.RetentionDays,
    bb.IsFullyRestored,
    COUNT(pb.PromptBackupId) AS TotalBackupsCount,
    SUM(CASE WHEN pb.IsRestored = 1 THEN 1 ELSE 0 END) AS RestoredCount
FROM BackupBatch bb
LEFT JOIN PromptBackup pb ON bb.BackupBatchId = pb.BackupBatchId
GROUP BY bb.BackupBatchId
ORDER BY bb.CreatedAt DESC;
```

---

## 3. Rust ORM Abstraction Architecture (`db_orm`)

The ORM is structured into four cohesive submodules within `src-tauri/src/modules/db_orm/`:

1. `traits.rs`: Core traits implemented by generated entity models:
   - `Entity`: Provides `table_name()`, `primary_key_column()`, and primary key value.
   - `FromRow`: Type-safe deserialization from a `rusqlite::Row`.
   - `ToValues`: Extraction of column names and parameter values for parameterized INSERT/UPDATE.
   - `ViewEntity`: Readonly view models with `view_name()` and `FromRow`.

2. `builder.rs`: Fluent, type-safe query builder:
   - `QueryBuilder::select::<T>()` -> returns builder for `SELECT ... FROM TableName`.
   - `.where_eq("ColumnName", value)`
   - `.where_in("ColumnName", values)`
   - `.order_by("ColumnName", Order::Asc | Order::Desc)`
   - `.limit(n)`
   - `.fetch_all(conn)` -> `Result<Vec<T>, AppError>`
   - `.fetch_one(conn)` -> `Result<Option<T>, AppError>`
   - `QueryBuilder::insert::<T>(entity, conn)` -> parameterized `INSERT INTO ...`
   - `QueryBuilder::update::<T>(entity, conn)` -> parameterized `UPDATE ... WHERE Id = ?`
   - `QueryBuilder::delete_by_id::<T>(id, conn)` -> parameterized `DELETE FROM ... WHERE Id = ?`

3. `connection.rs`: Pooled or direct connections enforcing WAL mode, busy timeout, and schema initialization.

4. `generated/`:
   - `entities.rs`: Model definitions generated by `03-ai-scripts/51-db-orm-generator.py`.
   - `tables.rs`: Table metadata, column name constants (`ErrorLog::ERROR_LOG_ID`, `ErrorLog::MESSAGE`).
   - `views.rs`: Readonly view models (`VwActivePromptsWithProjects`, `VwRecentErrors`, etc.).
