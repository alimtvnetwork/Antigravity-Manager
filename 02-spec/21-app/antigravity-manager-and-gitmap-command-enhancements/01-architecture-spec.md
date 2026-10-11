# 01 — Canonical Architecture Specification: CLI & Backend Compaction Engine

> **/goal** Architect a portable, cross-machine data compaction and restoration subsystem for Antigravity-Manager (`agm`) and GitMap, featuring 5-tier dynamic work directory resolution, standalone portable SQLite storage, Two-Tier JSON Envelope v2.0 serialization (`agm/compact-export`), and native CLI command workflows (`agm export-compact`, `agm import-compact`, `agm which-format`) with strict relative path immutability and positive boolean safety.

**Version:** 1.0.0 (Canonical)  
**Status:** Approved & Binding  
**Subsystem:** Core CLI & Backend Compaction Engine  
**Target Areas:** `src-tauri/src/modules/json_envelope/`, `src-tauri/src/modules/compact_engine/`, `src-tauri/src/bin/agm/`  
**Related Specs:**
- [02-unified-json-envelope-and-format-inspector-spec.md](../02-unified-json-envelope-and-format-inspector-spec.md)
- [02-gitmap-agm-tree-instance-swap-spec.md](../02-gitmap-agm-tree-instance-swap-spec.md)
- [144-prompt-tree-view-ui-progress-bar-and-agm-cli-enhancement/01-architecture-spec.md](../144-prompt-tree-view-ui-progress-bar-and-agm-cli-enhancement/01-architecture-spec.md)

---

## User Request (Verbatim)

```markdown
You are Spec Author 01 for task antigravity-manager-and-gitmap-command-enhancements.
Your responsibility is to author modular, comprehensive specifications and subtask execution plans for the CLI and Backend Compaction Engine.

### Files to Author (STRICTLY DISJOINT):
1. `02-spec/21-app/antigravity-manager-and-gitmap-command-enhancements/01-architecture-spec.md`
2. `.ai-memory/plans/subtasks/antigravity-manager-and-gitmap-command-enhancements/01-cli-backend-engine.md`

### Core Content to Specify:
1. Dynamic Work Directory Resolution:
   - Priority hierarchy: CLI param `--work-dir` -> env `AGM_WORK_DIR` -> GitMap SQLite `gitmap.db` default work directory -> git root ancestor -> default `D:\work` / `~/work`.
   - Relative path transformation: calculate `relativePath` from work directory and `${workDir}\<rel>` template. Support switching work directory seamlessly.
2. Compact SQLite Schema & Storage:
   - Standalone portable `.db` file containing `meta`, `pinned_projects`, `conversations`, `running_prompts`, `prompt_history`.
   - Pruning, retention, WAL mode pragmas.
3. Compact JSON Envelope v2.0 Schema:
   - `KnownDataType::CompactExport` (`agm/compact-export`).
   - Standard variables `${workDir}` and `${repoDir}` with 2-pass expansion.
4. CLI Commands & Flags:
   - `agm export-compact [--file <path>] [--work-dir <path>] [--no-pinned] [--absolute]`
   - `agm import-compact <file> [--work-dir <path>] [-y]`
   - `agm which-format <file>`
   - Detailed arguments, help text, examples, and exit codes.
5. Invariants:
   - Strictly relative Git paths only (NO absolute paths like `C:\...` or `D:\...`, NO `file:///` URIs).
   - Positive boolean naming (`isRelative`, `isApplied`, `hasCompleted`).
   - AppError error handling, zero swallowed errors.
   - TOTAL BAN on running git commands (`git add`, `git commit`, `git status`).

Write both files using `write_to_file`, verify their contents, and report completion.
```

---

## 1. Architectural Overview & System Boundary

The Antigravity-Manager and GitMap ecosystem operates across diverse operating environments (Windows developer workstations with `D:\work`, Linux/Ubuntu cloud VMs with `~/work` or `/work`, and macOS developer environments with `/Users/<user>/work`). Storing host-specific absolute filesystem paths (`D:\work\Antigravity-Manager\...` or `C:\Users\...`) creates fragile, non-portable snapshots that fail when imported across machines, accounts, or work directory reconfigurations.

The **CLI & Backend Compaction Engine** establishes a unified mechanism to package, export, inspect, and import active workspaces, pinned projects, conversation transcripts, in-flight prompts, and execution history into self-contained portable formats.

```
+--------------------------------------------------------------------------------------------------+
|                                    Inbound / Runtime Context                                     |
|  CLI Flag: --work-dir <path> | Env: AGM_WORK_DIR | gitmap.db Workdir | Git Root | OS Default     |
+-------------------------------------------------+------------------------------------------------+
                                                  |
                                                  v
+--------------------------------------------------------------------------------------------------+
|                         Dynamic Work Directory Resolution Engine                                 |
|                               (modules/compact_engine/work_dir.rs)                               |
|   1. Resolve active base workDir with deterministic 5-level priority ladder                      |
|   2. Compute normalized relativePath for projects, transcripts, and prompt targets               |
|   3. Encode paths as portable variable template: ${workDir}/<relativePath>                       |
+-------------------------------------------------+------------------------------------------------+
                                                  |
                  +-------------------------------+-------------------------------+
                  |                                                               |
                  v                                                               v
+---------------------------------------------------+   +------------------------------------------+
|            Compact JSON Envelope v2.0             |   |        Portable Compact SQLite DB        |
|    - Schema URI: agm/compact-export               |   |    - Standalone portable .db container   |
|    - Two-tier { attributes, variables, data }     |   |    - Tables: meta, pinned_projects,      |
|    - 2-pass chained interpolation: ${workDir}     |   |      conversations, running_prompts,     |
|    - Human-readable & diffable backup export      |   |      prompt_history                      |
|    - Format inspection via 'agm which-format'     |   |    - WAL mode & busy timeout pragmas     |
+---------------------------------------------------+   +------------------------------------------+
                  |                                                               |
                  +-------------------------------+-------------------------------+
                                                  |
                                                  v
+--------------------------------------------------------------------------------------------------+
|                             Native CLI Commands & Operations                                     |
|   - agm export-compact [--file <path>] [--work-dir <path>] [--no-pinned] [--absolute]            |
|   - agm import-compact <file> [--work-dir <path>] [-y]                                           |
|   - agm which-format <file>                                                                      |
+--------------------------------------------------------------------------------------------------+
```

### 1.1 Core Subsystem Responsibilities
1. **Dynamic Resolution**: Seamlessly decouple active project structures from machine-local drive letters and user home directories through deterministic multi-tier resolution.
2. **Relative Path Normalization**: Convert internal file representations into forward-slash relative paths anchored to the resolved workspace root.
3. **Dual Storage Parity**: Provide high-performance binary SQLite portability (`.db`) alongside portable, human-readable Two-Tier JSON Envelope v2.0 (`.json`).
4. **Autonomous Format Inspection**: Integrate `agm/compact-export` into `agm which-format` with non-destructive discovery, mutation previews, and automated execution flags.
5. **Robust Error Propagation**: All operations return strongly typed `Result<T, AppError>`, banning silent unwraps and swallowed errors.

---

## 2. Dynamic Work Directory Resolution Engine

### 2.1 Resolution Priority Hierarchy
To eliminate path rigidity, the engine resolves the active work directory `workDir` through a strict 5-stage evaluation hierarchy:

| Priority | Resolution Stage | Source Evaluation Method | Failure / Pass Condition |
|---|---|---|---|
| **P1** | **CLI Parameter** | `--work-dir <path>` argument passed to `agm` command | If present and non-empty, canonicalize path and use immediately. |
| **P2** | **Environment Variable** | `std::env::var("AGM_WORK_DIR")` | If set and non-empty, canonicalize path and use. |
| **P3** | **GitMap Master Store** | Query `gitmap.db` master store table `Workdir` or `Config` (`cli/db/gitmap.db`) | Read default active work directory record if database exists. |
| **P4** | **Git Ancestor Root** | Ancestor scan upwards from current working directory (`cwd`) for `.git` | If within a Git repository worktree, use Git root parent directory. |
| **P5** | **Platform Default** | OS-specific canonical workspace location | Windows: `D:\work` (or `C:\work` if `D:` does not exist); Linux/macOS: `~/work`. |

```rust
pub struct WorkDirResolver;

impl WorkDirResolver {
    /// Resolves active workspace base directory using strict 5-tier hierarchy.
    pub fn resolve_work_dir(cli_override: Option<&str>) -> Result<PathBuf, AppError> {
        // 1. CLI parameter override
        if let Some(raw) = cli_override {
            let trimmed = raw.trim();
            if !trimmed.is_empty() {
                return Ok(dunce::canonicalize(trimmed).unwrap_or_else(|_| PathBuf::from(trimmed)));
            }
        }

        // 2. Environment variable AGM_WORK_DIR
        if let Ok(env_val) = std::env::var("AGM_WORK_DIR") {
            let trimmed = env_val.trim();
            if !trimmed.is_empty() {
                return Ok(dunce::canonicalize(trimmed).unwrap_or_else(|_| PathBuf::from(trimmed)));
            }
        }

        // 3. GitMap master store query (gitmap.db)
        if let Some(gitmap_dir) = Self::query_gitmap_default_workdir() {
            return Ok(gitmap_dir);
        }

        // 4. Git repository root ancestor of current working directory
        if let Some(git_ancestor_parent) = Self::detect_git_ancestor_parent() {
            return Ok(git_ancestor_parent);
        }

        // 5. Fallback platform default
        Ok(Self::platform_default_workdir())
    }
}
```

### 2.2 Relative Path Transformation & Variable Templating
When exporting projects, active prompts, or conversation records:
1. **Normalization**: All paths are converted to forward slashes (`/`), stripping trailing slashes.
2. **Relative Extraction**: If a target path starts with the resolved `workDir`, the prefix is stripped to produce `relativePath`:
   - Absolute: `D:\work\Antigravity-Manager\src-tauri`
   - Active `workDir`: `D:\work`
   - Calculated `relativePath`: `Antigravity-Manager/src-tauri`
3. **Template Variable**: The path is stored using the canonical variable template `${workDir}/<relativePath>`.
4. **Cross-Machine Import Re-binding**: When importing on a target machine where `--work-dir /home/ubuntu/work` is supplied:
   - Template `${workDir}/Antigravity-Manager/src-tauri` dynamically expands to `/home/ubuntu/work/Antigravity-Manager/src-tauri`.
   - The user can switch work directories seamlessly without manual find-and-replace.

---

## 3. Compact SQLite Schema & Portable Storage Engine

### 3.1 Standalone Portable Database Architecture
The compaction engine supports generating a standalone, self-contained SQLite file (e.g., `workspace-compact.db`). This file contains no external foreign keys to local machine state and can be copied directly between machines.

### 3.2 Database Pragmas & Concurrency
All connections to the compact database enforce:
- `PRAGMA journal_mode = WAL;` (Write-Ahead Logging for high concurrency and zero reader locks)
- `PRAGMA busy_timeout = 5000;` (5000ms retry backoff on contention)
- `PRAGMA synchronous = NORMAL;` (Optimal durability with minimal disk write amplification)
- `PRAGMA foreign_keys = ON;` (Relational integrity enforcement)

### 3.3 Relational Schema Definition

```sql
-- 1. Metadata Table: Snapshot provenance and environment context
CREATE TABLE IF NOT EXISTS meta (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    schema_version TEXT NOT NULL DEFAULT '2.0',
    created_at TEXT NOT NULL,
    source_machine TEXT NOT NULL,
    source_work_dir TEXT NOT NULL,
    agm_version TEXT NOT NULL,
    notes TEXT,
    is_portable INTEGER NOT NULL DEFAULT 1
);

-- 2. Pinned Projects Table: Active bookmarked projects
CREATE TABLE IF NOT EXISTS pinned_projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_key TEXT NOT NULL UNIQUE,
    repo_name TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    pinned_at TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    display_order INTEGER NOT NULL DEFAULT 0,
    notes TEXT
);

-- 3. Conversations Table: Active and archived conversation threads
CREATE TABLE IF NOT EXISTS conversations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL UNIQUE,
    project_key TEXT NOT NULL,
    seq_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    last_active_at TEXT NOT NULL,
    message_count INTEGER NOT NULL DEFAULT 0,
    is_archived INTEGER NOT NULL DEFAULT 0,
    has_active_prompt INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY(project_key) REFERENCES pinned_projects(project_key) ON DELETE CASCADE
);

-- 4. Running Prompts Table: In-flight and queued prompts requiring resumption
CREATE TABLE IF NOT EXISTS running_prompts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    prompt_id TEXT NOT NULL UNIQUE,
    project_key TEXT NOT NULL,
    conversation_id TEXT NOT NULL,
    prompt_content TEXT NOT NULL,
    model TEXT,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    image_payload TEXT,
    is_restored INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY(project_key) REFERENCES pinned_projects(project_key) ON DELETE CASCADE
);

-- 5. Prompt History Table: Historic execution records and audit telemetry
CREATE TABLE IF NOT EXISTS prompt_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    history_id TEXT NOT NULL UNIQUE,
    project_key TEXT NOT NULL,
    conversation_id TEXT NOT NULL,
    prompt_content TEXT NOT NULL,
    model TEXT,
    role TEXT NOT NULL DEFAULT 'user',
    executed_at TEXT NOT NULL,
    exit_code INTEGER,
    has_completed INTEGER NOT NULL DEFAULT 1,
    duration_ms INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY(project_key) REFERENCES pinned_projects(project_key) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_pinned_projects_key ON pinned_projects(project_key);
CREATE INDEX IF NOT EXISTS idx_conversations_proj ON conversations(project_key);
CREATE INDEX IF NOT EXISTS idx_running_prompts_status ON running_prompts(status);
CREATE INDEX IF NOT EXISTS idx_prompt_history_executed ON prompt_history(executed_at DESC);
```

### 3.4 Pruning, Retention, and Compaction Policies
1. **Retention Cap**: During export, `prompt_history` records are automatically pruned to retain the most recent 500 entries per project unless explicitly overridden.
2. **Conversation Pruning**: Conversations marked `is_archived = 1` older than 14 days without active prompts are omitted from compact export.
3. **Vacuuming**: Upon completing compaction writes, the engine executes `VACUUM;` to reclaim unused disk pages and minimize archive payload size.

---

## 4. Compact JSON Envelope v2.0 Schema

### 4.1 Two-Tier Envelope Structure
The compaction format integrates directly with the AGM Two-Tier JSON Envelope v2.0 specification (`src-tauri/src/modules/json_envelope/`).

```json
{
  "attributes": {
    "type": "agm/compact-export",
    "version": "2.0",
    "source": "agm-cli",
    "createdAt": "2026-10-11T15:00:00Z",
    "agmVersion": "4.162.0",
    "workDirectory": {
      "path": "${workDir}",
      "defaultPath": "D:\\work",
      "isApplied": true,
      "isEnforced": false
    },
    "defaultWorkDirectory": "D:\\work",
    "isWorkDirectoryApplied": true,
    "isWorkDirectoryEnforced": false,
    "importCommand": "agm import-compact agm-workspace-compact.json -y",
    "exportCommand": "agm export-compact --file agm-workspace-compact.json",
    "helpCommand": "agm which-format agm-workspace-compact.json",
    "description": "Portable workspace compaction snapshot including pinned projects, conversations, running prompts, and execution history."
  },
  "variables": {
    "workDir": "D:\\work",
    "repoDir": "${workDir}\\Antigravity-Manager"
  },
  "data": {
    "meta": {
      "schemaVersion": "2.0",
      "createdAt": "2026-10-11T15:00:00Z",
      "sourceMachine": "DEV-WORKSTATION-01",
      "sourceWorkDir": "D:\\work",
      "isPortable": true
    },
    "pinnedProjects": [
      {
        "projectKey": "P001",
        "repoName": "Antigravity-Manager",
        "relativePath": "Antigravity-Manager",
        "pinnedAt": "2026-10-10T12:00:00Z",
        "isActive": true,
        "displayOrder": 1
      }
    ],
    "conversations": [
      {
        "conversationId": "c7a8b9-1122-3344",
        "projectKey": "P001",
        "seqId": 1,
        "title": "Refactor Compaction Engine",
        "relativePath": "Antigravity-Manager",
        "lastActiveAt": "2026-10-11T14:30:00Z",
        "messageCount": 18,
        "isArchived": false,
        "hasActivePrompt": true
      }
    ],
    "runningPrompts": [
      {
        "promptId": "p-9901-abcd",
        "projectKey": "P001",
        "conversationId": "c7a8b9-1122-3344",
        "promptContent": "Run cargo test --all",
        "model": "gemini-2.5-pro",
        "status": "running",
        "createdAt": "2026-10-11T14:45:00Z",
        "updatedAt": "2026-10-11T14:50:00Z",
        "imagePayload": null,
        "isRestored": false
      }
    ],
    "promptHistory": [
      {
        "historyId": "h-5501-ef01",
        "projectKey": "P001",
        "conversationId": "c7a8b9-1122-3344",
        "promptContent": "Inspect failed tests",
        "model": "gemini-2.5-flash",
        "role": "user",
        "executedAt": "2026-10-11T14:20:00Z",
        "exitCode": 0,
        "hasCompleted": true,
        "durationMs": 4200
      }
    ]
  }
}
```

### 4.2 KnownDataType Integration
In `src-tauri/src/modules/json_envelope/types.rs` and `inspection.rs`:
- Enum variant: `KnownDataType::CompactExport`
- Identifier string: `"agm/compact-export"`
- Recognition in `from_type_str`: matches `"agm/compact-export"`, `"compact-export"`, `"compact_export"`
- Inspection display name: `"Workspace Compaction Archive"`
- Mutation impact description: `"Imports pinned projects, conversation transcripts, and restores in-flight prompts into local SQLite repository database"`
- Suggested command: `agm import-compact "<path>" -y`

### 4.3 Two-Pass Variable Expansion
When unpacking `agm/compact-export`:
- **Pass 1 (Dependency Graph Resolution)**: Evaluate `$variables` so `${workDir}` resolves to the active local `--work-dir`, and `${repoDir}` derives from resolved `${workDir}`.
- **Pass 2 (Data Payload Traversal)**: Recursively substitute variables in `pinnedProjects.relativePath`, `conversations.relativePath`, and any path references in `runningPrompts`.

---

## 5. CLI Commands, Flags, and Contracts

### 5.1 Command 1: `agm export-compact`
Exports workspace metadata, pinned projects, conversations, active prompts, and history into a compact JSON or SQLite file.

```
Usage:
  agm export-compact [OPTIONS]

Options:
  -f, --file <PATH>       Target output file path. Defaults to:
                          'agm-workspace-compact-<date>.json' (or .db if specified).
  -w, --work-dir <PATH>   Base workspace directory to calculate relative paths against.
                          Overrides AGM_WORK_DIR and GitMap configuration.
      --no-pinned         Skip exporting pinned projects; export only prompts and history.
      --absolute          Preserve raw absolute filesystem paths instead of relative templates.
  -h, --help              Print help documentation.

Examples:
  agm export-compact
  agm export-compact -f backup-compact.json
  agm export-compact -f portable.db --work-dir D:\work
  agm export-compact --no-pinned -f prompts-only.json
```

### 5.2 Command 2: `agm import-compact`
Ingests a compact archive into local SQLite databases (`repo_prompts.db`, `backup_prompts.db`, `gitmap.db`), seamlessly re-anchoring relative paths to the local active work directory.

```
Usage:
  agm import-compact <FILE> [OPTIONS]

Arguments:
  <FILE>                  Path to compact JSON envelope or SQLite database to import.

Options:
  -w, --work-dir <PATH>   Target base workspace directory. Re-anchors all relative paths.
                          Defaults to local resolved work directory.
  -y, --yes               Bypass confirmation prompt; automatically execute import.
  -h, --help              Print help documentation.

Examples:
  agm import-compact backup-compact.json
  agm import-compact portable.db -y
  agm import-compact backup-compact.json --work-dir /home/ubuntu/work -y
```

### 5.3 Command 3: `agm which-format`
Enhanced inspection tool that automatically recognizes `agm/compact-export`, previews item counts, validates JSON structure, and provides single-line import suggestions.

```
Usage:
  agm which-format [FILES_OR_DIR...] [-y] [--run]

Sample Output for Compact Export:
  ================================================================================
    AGM Universal Format Inspector & Schema Classifier
  ================================================================================
    Discovered: 1 JSON file(s) across target path(s)

    --- Matched Supported Schemas (1 file(s)) ---
    #1  [agm/compact-export] (Envelope v2.0 (2 variables))
        Path:     agm-workspace-compact.json
        WorkDir:  ${workDir}
        Entities: 1 pinned project(s), 1 conversation(s), 1 running prompt(s), 1 history record(s)
        Impact:   Imports pinned projects, conversation transcripts, and restores in-flight prompts into local SQLite repository database
        Command:  agm import-compact agm-workspace-compact.json -y
```

### 5.4 CLI Process Exit Codes

| Exit Code | Classification | Description |
|---|---|---|
| `0` | `SUCCESS` | Operation completed with zero errors. All transactions committed. |
| `1` | `USAGE_ERROR` | Invalid or missing arguments, conflicting flags (e.g. `--file` without path). |
| `2` | `IO_ERROR` | File not found, permission denied, or target directory unwriteable. |
| `3` | `SCHEMA_ERROR` | Invalid JSON syntax, corrupted SQLite header, or unrecognized schema version. |
| `4` | `INTEGRITY_ERROR` | Relational constraint violation or corrupt conversation linkage during import. |

---

## 6. System Invariants & Non-Negotiable Guardrails

### 6.1 Invariant 1: Strictly Relative Git Paths
- **Zero Absolute Paths in Artifacts**: Exported envelopes and portable `.db` records MUST NOT contain raw host-specific drive roots (`C:\`, `D:\`, `/home/...`) unless the `--absolute` flag is explicitly passed.
- **No `file:///` URIs**: All file references in transcripts and metadata must use relative forward-slash POSIX paths.

### 6.2 Invariant 2: Positive Boolean Naming Standard
All boolean fields, variables, database columns, and struct properties MUST use affirmative, positive nomenclature. Negated names (`is_not_active`, `disable_pinned`) are strictly forbidden:
- `isRelative` (NOT `isNotAbsolute`)
- `isApplied` (NOT `isUnapplied`)
- `hasCompleted` (NOT `isIncomplete`)
- `isActive` (NOT `isInactive`)
- `isRestored` (NOT `isUnrestored`)
- `isPortable` (NOT `isNonPortable`)

### 6.3 Invariant 3: Zero Swallowed Errors & AppError Propagation
- Every fallible operation (I/O, database queries, JSON parsing) must return `Result<T, AppError>`.
- Bare `unwrap()`, unhandled `.ok()`, and silent catch blocks are strictly prohibited.
- For non-critical best-effort actions (e.g., temporary cleanup or index optimization), calls must use `crate::error::record_ignored(...)` accompanied by an explicit `// Justification:` comment.

### 6.4 Invariant 4: TOTAL BAN on Running Git Commands
- The compaction engine, CLI commands, and automated test runners are **STRICTLY PROHIBITED** from executing Git mutating commands (`git add`, `git commit`, `git push`, `git status`, `git checkout`).
- Repository detection MUST rely on direct filesystem inspection (checking for `.git` directory presence, reading `.git/HEAD`, or reading GitMap configuration databases) without spawning Git child processes.

---

## 7. Data Contracts & Rust Type Definitions

```rust
// File: src-tauri/src/modules/compact_engine/models.rs

use serde::{Deserialize, Serialize};

/// Top-level payload for agm/compact-export
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompactExportData {
    pub meta: CompactMetadata,
    #[serde(default)]
    pub pinned_projects: Vec<CompactPinnedProject>,
    #[serde(default)]
    pub conversations: Vec<CompactConversation>,
    #[serde(default)]
    pub running_prompts: Vec<CompactRunningPrompt>,
    #[serde(default)]
    pub prompt_history: Vec<CompactPromptHistory>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompactMetadata {
    pub schema_version: String,
    pub created_at: String,
    pub source_machine: String,
    pub source_work_dir: String,
    pub is_portable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompactPinnedProject {
    pub project_key: String,
    pub repo_name: String,
    pub relative_path: String,
    pub pinned_at: String,
    pub is_active: bool,
    pub display_order: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompactConversation {
    pub conversation_id: String,
    pub project_key: String,
    pub seq_id: i64,
    pub title: String,
    pub relative_path: String,
    pub last_active_at: String,
    pub message_count: usize,
    pub is_archived: bool,
    pub has_active_prompt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompactRunningPrompt {
    pub prompt_id: String,
    pub project_key: String,
    pub conversation_id: String,
    pub prompt_content: String,
    pub model: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_payload: Option<String>,
    pub is_restored: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompactPromptHistory {
    pub history_id: String,
    pub project_key: String,
    pub conversation_id: String,
    pub prompt_content: String,
    pub model: Option<String>,
    pub role: String,
    pub executed_at: String,
    pub exit_code: Option<i32>,
    pub has_completed: bool,
    pub duration_ms: u64,
}
```

---

## 8. Acceptance Criteria & Verification Matrix

| ID | Category | Verification Condition | Verification Command / Check |
|---|---|---|---|
| **AC-01** | WorkDir Priority | `--work-dir` flag takes precedence over `AGM_WORK_DIR` and system defaults. | Unit test verifying resolution priority ladder. |
| **AC-02** | Path Relativity | Exported JSON files contain relative forward-slash paths and `${workDir}` variables. | String assertion on output payload: contains no drive letters. |
| **AC-03** | SQLite Portability | Exported `.db` file initializes clean schema, inserts all entities, and passes `PRAGMA integrity_check;`. | SQLite integrity check query on generated portable database. |
| **AC-04** | Envelope Schema | `agm/compact-export` is registered in `KnownDataType` and unpacked seamlessly. | `agm which-format` classifies test compact file as `agm/compact-export`. |
| **AC-05** | CLI Export Command | `agm export-compact` outputs valid JSON envelope and reports entity counts. | CLI command test verifying exit code `0` and file existence. |
| **AC-06** | CLI Import Command | `agm import-compact` ingests snapshot into local SQLite and restores active state. | CLI import command test with `-y` followed by query verification. |
| **AC-07** | Non-Destructive Inspect | `agm which-format` reads compact export without modifying any local state. | Verify database unmodified after inspecting candidate archive. |
| **AC-08** | Codebase Hygiene | Zero Git mutating commands invoked; positive boolean naming verified; all errors mapped to `AppError`. | Code review & clippy check `cargo clippy --all-targets --all-features`. |
