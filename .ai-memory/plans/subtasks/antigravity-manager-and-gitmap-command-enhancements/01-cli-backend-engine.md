# Subtask 01: CLI & Backend Compaction Engine Implementation

**Status:** `QUEUED`  
**Priority:** High  
**Parent Task:** `antigravity-manager-and-gitmap-command-enhancements`  
**Spec Reference:** [02-spec/21-app/antigravity-manager-and-gitmap-command-enhancements/01-architecture-spec.md](../../../02-spec/21-app/antigravity-manager-and-gitmap-command-enhancements/01-architecture-spec.md)  
**Assigned Focus:** Rust Backend, SQLite Storage, Two-Tier JSON Envelope v2.0, CLI Dispatch  

---

## 1. Objective & Technical Scope

Implement the native Rust compaction, storage, and CLI engine for Antigravity-Manager, providing:
1. Dynamic 5-tier work directory resolution (`--work-dir`, `AGM_WORK_DIR`, `gitmap.db`, Git root parent, platform default).
2. Standalone portable SQLite storage engine with WAL pragmas and retention pruning.
3. Two-Tier JSON Envelope v2.0 schema extension for `KnownDataType::CompactExport` (`agm/compact-export`).
4. Native CLI commands `agm export-compact`, `agm import-compact`, and `agm which-format` inspection support.
5. Strict adherence to relative path normalization, positive boolean naming, and `AppError` error propagation.

---

## 2. Target Files & Symbol Modifications

| File Path | Action | Symbols / Components to Implement |
|---|---|---|
| `src-tauri/src/modules/compact_engine/mod.rs` | Create | Facade exports for `work_dir`, `models`, `storage`, `export`, and `import`. |
| `src-tauri/src/modules/compact_engine/models.rs` | Create | Structs: `CompactExportData`, `CompactMetadata`, `CompactPinnedProject`, `CompactConversation`, `CompactRunningPrompt`, `CompactPromptHistory`. |
| `src-tauri/src/modules/compact_engine/work_dir.rs` | Create | `WorkDirResolver::resolve_work_dir`, `to_relative_path`, `to_template_path`, `expand_template_path`. |
| `src-tauri/src/modules/compact_engine/storage.rs` | Create | SQLite connection initializer with WAL pragmas, schema tables (`meta`, `pinned_projects`, `conversations`, `running_prompts`, `prompt_history`), pruning logic. |
| `src-tauri/src/modules/compact_engine/export.rs` | Create | `export_to_json_envelope`, `export_to_sqlite_file`. |
| `src-tauri/src/modules/compact_engine/import.rs` | Create | `import_from_json_envelope`, `import_from_sqlite_file`. |
| `src-tauri/src/modules/json_envelope/types.rs` | Modify | Add `KnownDataType::CompactExport`, update `default_commands_for_type`. |
| `src-tauri/src/modules/json_envelope/extraction.rs` | Modify | Map `"agm/compact-export"` in `from_type_str` and `as_type_str`. |
| `src-tauri/src/modules/json_envelope/inspection.rs` | Modify | Add inspection formatter for `KnownDataType::CompactExport`. |
| `src-tauri/src/bin/agm/compact_cmds.rs` | Create | CLI handlers `cmd_export_compact` and `cmd_import_compact`. |
| `src-tauri/src/bin/agm/main.rs` | Modify | Register routes: `export-compact` / `ec`, `import-compact` / `ic`. |
| `src-tauri/src/bin/agm/help_text.rs` | Modify | Append help entries and command examples for compaction commands. |

---

## 3. Step-by-Step Implementation Sequence

### Step 1: Dynamic Work Directory Resolution Engine
- Create `src-tauri/src/modules/compact_engine/work_dir.rs`.
- Implement `WorkDirResolver`:
  1. CLI parameter override (`--work-dir`).
  2. Environment variable (`AGM_WORK_DIR`).
  3. GitMap database lookup (`gitmap.db` workdir table if available).
  4. Git ancestor root discovery (walk up directories checking for `.git` directory without spawning `git`).
  5. Fallback platform default (`D:\work` on Windows, `~/work` on Unix).
- Implement path transformation helpers:
  - `to_relative_path(target_path, base_work_dir) -> Result<String, AppError>`
  - `to_template_path(relative_path) -> String` (`${workDir}/<relative_path>`)
  - `expand_template_path(template_path, active_work_dir) -> PathBuf`

### Step 2: Portable Compact SQLite Storage Engine
- Create `src-tauri/src/modules/compact_engine/storage.rs`.
- Implement schema initialization for standalone `.db` file:
  - Enforce `PRAGMA journal_mode = WAL;`, `PRAGMA busy_timeout = 5000;`, `PRAGMA synchronous = NORMAL;`.
  - Tables: `meta`, `pinned_projects`, `conversations`, `running_prompts`, `prompt_history`.
- Implement retention and pruning:
  - Max 500 historical prompts per project during export.
  - Skip archived conversations older than 14 days without active prompts.
  - Execute `VACUUM;` on database closure.

### Step 3: Two-Tier JSON Envelope v2.0 Integration
- Update `src-tauri/src/modules/json_envelope/types.rs`:
  - Add variant `KnownDataType::CompactExport`.
  - Register default commands in `default_commands_for_type`:
    - Import: `agm import-compact agm-workspace-compact.json -y`
    - Export: `agm export-compact --file agm-workspace-compact.json`
    - Which-format: `agm which-format agm-workspace-compact.json`
- Update `src-tauri/src/modules/json_envelope/extraction.rs`:
  - Map `"agm/compact-export"`, `"compact-export"`, `"compact_export"` to `KnownDataType::CompactExport`.
- Update `src-tauri/src/modules/json_envelope/inspection.rs`:
  - In `describe_format`, format item counts for pinned projects, conversations, running prompts, and prompt history.

### Step 4: Export & Import Serialization Logic
- Create `src-tauri/src/modules/compact_engine/export.rs`:
  - Read active projects from `repo_prompts.db` / `gitmap.db`.
  - Transform all paths to relative forward-slash POSIX paths.
  - Wrap payload in `JsonEnvelope<CompactExportData>` with variables `workDir` and `repoDir`.
  - Support SQLite binary output if target file ends in `.db`.
- Create `src-tauri/src/modules/compact_engine/import.rs`:
  - Detect input type (`.json` envelope vs `.db` SQLite).
  - Unpack envelope and evaluate `${workDir}` against local resolved work directory.
  - Re-anchor paths and upsert into local SQLite databases inside a single transaction.

### Step 5: CLI Command Handlers & Dispatch
- Create `src-tauri/src/bin/agm/compact_cmds.rs`:
  - Implement `cmd_export_compact(args: &[String])`:
    - Parse `-f`/`--file`, `-w`/`--work-dir`, `--no-pinned`, `--absolute`.
    - Print summary table and execution duration.
  - Implement `cmd_import_compact(args: &[String])`:
    - Parse `<file>`, `-w`/`--work-dir`, `-y`/`--yes`.
    - Provide interactive confirmation prompt if `-y` is absent.
- Register commands in `src-tauri/src/bin/agm/main.rs`:
  - Map `"export-compact"` | `"ec"` -> `compact_cmds::cmd_export_compact`.
  - Map `"import-compact"` | `"ic"` -> `compact_cmds::cmd_import_compact`.
- Add help text documentation in `src-tauri/src/bin/agm/help_text.rs`.

---

## 4. Verification & Testing Protocol

Execute the following test gates locally (note: Git mutating commands are strictly banned):

```bash
# 1. Rust code formatting check
cd src-tauri && cargo fmt -- --check

# 2. Rust clippy linting and compilation gate
cd src-tauri && cargo clippy --all-targets --all-features

# 3. Targeted unit tests for work dir resolution & compact engine
cd src-tauri && cargo test modules::compact_engine

# 4. JSON envelope format inspection tests
cd src-tauri && cargo test modules::json_envelope
```

---

## 5. Definition of Done & Acceptance Gates

- [ ] `WorkDirResolver` resolves through all 5 priority tiers in verified unit tests.
- [ ] Exported JSON envelope conforms strictly to `attributes` (type: `"agm/compact-export"`) and `variables` standard.
- [ ] Exported SQLite `.db` passes `PRAGMA integrity_check;`.
- [ ] `agm which-format` detects `agm/compact-export` with accurate entity counts and `-y` suggested command.
- [ ] Zero raw absolute paths (`C:\...`, `D:\...`, `file:///...`) in exported payloads.
- [ ] All booleans follow affirmative positive naming (`isRelative`, `isApplied`, `hasCompleted`, `isActive`, `isRestored`).
- [ ] All errors map to `AppError` without unhandled `unwrap()` or swallowed results.
- [ ] Zero Git mutating commands invoked.
