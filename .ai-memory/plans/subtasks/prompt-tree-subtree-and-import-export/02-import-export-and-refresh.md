# Subtask Plan: TASK-02 - Self-Contained Import/Export (JSON, SQLite DB, YAML) with Embedded Base64 Images & Refresh Fix

> **Parent Plan:** `.ai-memory/plans/prompt-tree-subtree-and-import-export.md`  
> **Subtask Code:** `TASK-02`  
> **Subtask Slug:** `tree-import-export-with-images`  
> **Target Spec:** `02-spec/21-app/prompt-tree-subtree-and-import-export/02-import-export-and-refresh-spec.md`  
> **Assigned Role:** `Worker 02`  
> **Status:** `COMPLETED`  
> **Target Files Owned:**  
> - `src-tauri/src/modules/repo_db/sqlite_export.rs`  
> - `src-tauri/src/modules/repo_db/export.rs`  
> - `src-tauri/src/modules/repo_db/import.rs`  
> - `src-tauri/src/commands/project_cmds.rs`  
> - `src/components/instances/PromptTreeView/usePromptTreeCoreActions.ts`  
> - `src/components/instances/PromptTreeView/PromptTreeExportModal.tsx`  
> - `src/components/instances/PromptTreeView/PromptTreeImportModal.tsx`  
> - `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`  
> - `src/components/instances/PromptTreeView/ProjectContextMenu.tsx`  
> - `src/components/instances/PromptTreeView/ConversationContextMenu.tsx`  
> - `src/components/instances/PromptTreeView/PromptNodeContextMenu.tsx`  

---

## 1. Objectives & Executive Summary

Worker 02 is tasked with implementing end-to-end section-level portability and resolving fundamental synchronization defects in the Antigravity-Manager Prompt Tree View:
1. **Section-Level Import / Export Engine**: Enable users to export and import across 4 hierarchical scopes (**Full Tree**, **Project**, **Conversation**, and **Prompt Subtree**) in 3 standardized formats (**JSON**, **SQLite DB**, and **YAML**).
2. **Base64 Image Embedding Invariant**: Intercept all multimodal assets (screenshots, architecture diagrams, user uploads) referenced in prompts, encode them as Base64 data URIs on export, and unpack/materialize them to `<target_dir>/.antigravity/attachments/` on import, guaranteeing 100% self-contained transfers without broken file paths across machines.
3. **Refresh Button 4-Point Root Cause Remediation**:
   - Fix **Root Cause A**: Eliminate selection whiplash by changing `onRefreshAll` calls from `loadTree(true, true)` to `loadTree(false, true)` in `PromptTreeViewModal.tsx`.
   - Fix **Root Cause B**: Force live workspace discovery (`detect_running_projects`) before querying `running_projects` in `tree.rs`.
   - Fix **Root Cause C**: Eradicate `immutable=1` URI mode in `tree_collect.rs`, `project_queries.rs`, and `text_utils.rs`, configuring 5000ms `busy_timeout` to read active SQLite WAL frames.
   - Fix **Root Cause D**: Extend in-flight prompt status expiration in `tree_phases.rs` from 45s to 600s.

---

## 2. Step-by-Step Implementation Guide for Worker 02

### Step 1: Create Scoped Export Engine (`src-tauri/src/modules/repo_db/export.rs`)

Create `src-tauri/src/modules/repo_db/export.rs` to handle section-scoped data assembly, multimodal image discovery, and JSON/YAML formatting.

#### 1.1 Data Structures:
```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportScope {
    pub target_type: String, // "full_tree" | "project" | "conversation" | "prompt_subtree"
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub conversation_id: Option<String>,
    pub prompt_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub embed_images: bool,
    pub format: String, // "json" | "sqlite" | "yaml"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStats {
    pub path: String,
    pub project_count: usize,
    pub conversation_count: usize,
    pub prompt_count: usize,
    pub image_count: usize,
    pub total_image_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportEnvelope {
    pub schema_version: String,
    pub kind: String,
    pub exported_at: String,
    pub agm_version: String,
    pub scope: ExportScope,
    pub stats: ExportStats,
    pub data: ExportData,
    pub embedded_images: HashMap<String, String>, // image_key -> data:image/png;base64,...
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub projects: Vec<super::models::RunningProject>,
    pub active_prompts: Vec<super::models::ActivePrompt>,
    pub project_sequences: Vec<super::sequences::ProjectSequenceRow>,
    pub conversation_sequences: Vec<super::sequences::ConversationSequenceRow>,
}
```

#### 1.2 Image Harvester & Base64 Encoder:
```rust
pub fn harvest_and_embed_images(
    prompts: &mut [super::models::ActivePrompt],
    embedded_images: &mut HashMap<String, String>,
) -> (usize, usize) {
    let mut total_count = 0;
    let mut total_bytes = 0;

    for prompt in prompts.iter_mut() {
        let (raw_payload, paths) = super::detection::extract_image_payload_or_path(&prompt.prompt_content);
        for path_str in paths {
            let path = std::path::Path::new(&path_str);
            if path.exists() {
                if let Ok(bytes) = std::fs::read(path) {
                    let hash = format!("{:x}", md5::compute(&bytes)); // or simple 16-hex hash
                    let key = format!("img_{}", &hash[..12]);
                    let mime = detect_mime_type(path, &bytes);
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    let data_uri = format!("data:{mime};base64,{b64}");
                    
                    total_count += 1;
                    total_bytes += bytes.len();
                    embedded_images.insert(key.clone(), data_uri);

                    // Replace absolute local path in prompt text with portable reference
                    prompt.prompt_content = prompt.prompt_content.replace(&path_str, &format!("agm-asset://{key}"));
                }
            }
        }
    }
    (total_count, total_bytes)
}
```

#### 1.3 Export Scoped Querying:
Implement `build_scoped_export_envelope(scope: &ExportScope, options: &ExportOptions) -> Result<ExportEnvelope, AppError>`:
- Connect to `repo_prompts.db`.
- Based on `scope.target_type`:
  * `full_tree`: Read all rows from `running_projects`, `active_prompts`, `agm_project_sequences`, `agm_conversation_sequences`.
  * `project`: Filter `running_projects` by `id = ?1`, filter `active_prompts` by `project_id = ?1`, filter sequences by `project_key`.
  * `conversation`: Filter `active_prompts` by `session_id = ?1`, include parent project row, filter conversation sequences by `conversation_id = ?1`.
  * `prompt_subtree`: Filter `active_prompts` by `id IN (...)`.
- Enforce strict FIFO ordering on all queries: `ORDER BY created_at ASC, id ASC`.
- Harvest images if `options.embed_images` is true.

#### 1.4 YAML Formatting:
Leverage `yaml-rt` or structured serde YAML formatting to output clean YAML files.

---

### Step 2: Extend SQLite Scoped Export (`src-tauri/src/modules/repo_db/sqlite_export.rs`)

Refactor `src-tauri/src/modules/repo_db/sqlite_export.rs` to support scopes and the `embedded_images` table:

#### 2.1 Update Schema:
Add `embedded_images` table definition to `init_tree_export_schema`:
```sql
CREATE TABLE IF NOT EXISTS embedded_images (
    image_key TEXT PRIMARY KEY,
    mime_type TEXT NOT NULL,
    base64_data TEXT NOT NULL,
    byte_size INTEGER NOT NULL,
    original_rel_path TEXT,
    created_at INTEGER NOT NULL
);
```

#### 2.2 Scoped Write Function:
Update `write_prompt_tree_db` to `write_scoped_prompt_tree_db(path: &Path, scope: &ExportScope, options: &ExportOptions) -> Result<ExportStats, AppError>`:
- Execute filtered insert statements into the destination database:
  ```rust
  let prompt_query = match scope.target_type.as_str() {
      "project" => format!("INSERT INTO active_prompts ({AP_COLS}) SELECT {AP_COLS} FROM srcdb.active_prompts WHERE project_id = ?1 ORDER BY created_at ASC, id ASC"),
      "conversation" => format!("INSERT INTO active_prompts ({AP_COLS}) SELECT {AP_COLS} FROM srcdb.active_prompts WHERE session_id = ?1 ORDER BY created_at ASC, id ASC"),
      "prompt_subtree" => { ... },
      _ => format!("INSERT INTO active_prompts ({AP_COLS}) SELECT {AP_COLS} FROM srcdb.active_prompts ORDER BY created_at ASC, id ASC"),
  };
  ```
- If `options.embed_images` is true, iterate exported prompts, extract image files from disk, encode as Base64, and insert into `embedded_images (image_key, mime_type, base64_data, byte_size, original_rel_path, created_at)`.
- Update `meta` table with `target_type: scope.target_type` and `target_id: scope.project_id.or(scope.conversation_id)`.

---

### Step 3: Implement Import Engine & Image Materializer (`src-tauri/src/modules/repo_db/import.rs`)

Create `src-tauri/src/modules/repo_db/import.rs` to handle universal format detection, validation, image materialization, and database merging:

#### 3.1 Universal Validator & Probe:
```rust
pub enum ImportPayloadKind {
    SqliteDb(std::path::PathBuf),
    JsonEnvelope(ExportEnvelope),
    YamlEnvelope(ExportEnvelope),
}

pub fn probe_import_source(path: Option<&Path>, data_b64: Option<&str>) -> Result<ImportPayloadKind, AppError>;
```
- Inspect file magic headers (`SQLite format 3\0`, `{`, or `version:` / `kind:`).
- Validate envelope schema version (`"2.0"` or `"1.0"` fallback) and kind marker (`agm/prompt-tree-section-export` or `agm/prompt-tree-export`).

#### 3.2 Asset Materialization to Disk:
```rust
pub fn materialize_embedded_images(
    images: &HashMap<String, String>,
    target_data_dir: &Path,
) -> Result<HashMap<String, PathBuf>, AppError> {
    let attachments_dir = target_data_dir.join(".antigravity").join("attachments");
    std::fs::create_dir_all(&attachments_dir)?;

    let mut materialized_map = HashMap::new();
    for (key, data_uri) in images {
        if let Some((mime, b64_payload)) = parse_data_uri(data_uri) {
            let bytes = base64::engine::general_purpose::STANDARD.decode(b64_payload.trim())?;
            let ext = mime_to_extension(&mime);
            // Sanitize filename to prevent directory traversal
            let safe_name = format!("{}.{}", sanitize_key(key), ext);
            let dest_path = attachments_dir.join(safe_name);
            std::fs::write(&dest_path, &bytes)?;
            materialized_map.insert(key.clone(), dest_path);
        }
    }
    Ok(materialized_map)
}
```

#### 3.3 FIFO Database Merge:
- In SQLite transaction, upsert projects and sequences.
- Upsert prompts using `ORDER BY created_at ASC, id ASC`:
  ```sql
  INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
  ON CONFLICT(id) DO UPDATE SET
      prompt_content = excluded.prompt_content,
      status = excluded.status,
      updated_at = excluded.updated_at,
      image_payload = excluded.image_payload;
  ```
- If prompt content contains `agm-asset://<key>`, rewrite to the materialized disk path.
- Invalidate SWR cache via `super::tree::invalidate_prompt_tree_cache(None)`.

---

### Step 4: Apply Refresh Button RCA Fixes in Backend Core

#### 4.1 Fix Root Cause D (Raise In-Flight Cutoff from 45s to 600s):
In `src-tauri/src/modules/repo_db/tree_phases.rs:23`:
```rust
// Modify line 23 from:
// AND (?1 - updated_at > 45)
// To:
AND (?1 - updated_at > 600)
```

#### 4.2 Fix Root Cause C (Eradicate `immutable=1` & Set Busy Timeout):
In `src-tauri/src/modules/repo_db/tree_collect.rs:79-93`:
Remove `immutable=1` fallback. Replace with:
```rust
let s_conn = Connection::open_with_flags(
    &summaries_db,
    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
).and_then(|conn| {
    conn.busy_timeout(std::time::Duration::from_millis(5000))?;
    Ok(conn)
});
```
Apply the identical fix to `text_utils.rs:206` and `project_queries.rs:111`.

#### 4.3 Fix Root Cause B (Force Project Detection Before Reading Tree):
In `src-tauri/src/modules/repo_db/tree.rs`:
Inside `compute_project_conversation_tree`, prior to `super::tree_phases::query_tree_projects(target)`:
```rust
if let Some(target_inst) = target {
    let _ = crate::modules::repo_db::detect_running_projects(target_inst);
} else {
    let _ = crate::modules::repo_db::detect_running_projects("default");
}
```

---

### Step 5: Implement Tauri IPC Commands (`src-tauri/src/commands/project_cmds.rs`)

Expose the new export and import routines to Tauri IPC in `project_cmds.rs`:
1. `export_prompt_tree_section(scope: ExportScope, options: ExportOptions, destination_path: Option<String>) -> Result<ExportStats, AppError>`
2. `export_prompt_tree_section_bytes(scope: ExportScope, options: ExportOptions) -> Result<(String, String, ExportStats), AppError>`
3. `validate_prompt_tree_import(source_path: Option<String>, data_b64: Option<String>) -> Result<ExportStats, AppError>`
4. `import_prompt_tree_section(source_path: Option<String>, data_b64: Option<String>, options: ImportOptions) -> Result<usize, AppError>`

Register these commands in `src-tauri/src/lib/run.rs`.

---

### Step 6: Frontend Core Actions & Non-Whiplash Refresh

#### 6.1 Fix Root Cause A in `PromptTreeViewModal.tsx`:
In `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`:
- Line 499: Change `onRefreshAll={() => loadTree(true, true)}` to:
  ```typescript
  onRefreshAll={() => loadTree(false, true)}
  ```
- Line 509: Change `loadTree(true, true)` to:
  ```typescript
  loadTree(false, true);
  ```

#### 6.2 Update `usePromptTreeCoreActions.ts`:
Add states and execution handlers for section export and import modals:
- `isExportModalOpen`, `setIsExportModalOpen`
- `isImportModalOpen`, `setIsImportModalOpen`
- `exportScopeTarget`, `setExportScopeTarget`
- `executeSectionExport(params: ExportTreeSectionParams): Promise<ExportStats>`
- `executeSectionImport(params: ImportTreeSectionParams): Promise<number>`

---

### Step 7: Create Modals & Wire Context Menus

#### 7.1 Build `PromptTreeExportModal.tsx`:
- Location: `src/components/instances/PromptTreeView/PromptTreeExportModal.tsx`
- Features:
  * Segmented scope selector (Full Tree, Project, Conversation, Prompt Turn).
  * Format selector (JSON, SQLite DB, YAML) styled as clean card options.
  * Image embedding toggle switch with explanation badge.
  * Live statistics preview card (projects, convs, prompts, images, estimated size).
  * Cross-platform execution: native file save dialog (`save({ filters: ... })`) in Tauri; base64 blob download in Web mode.

#### 7.2 Build `PromptTreeImportModal.tsx`:
- Location: `src/components/instances/PromptTreeView/PromptTreeImportModal.tsx`
- Features:
  * Drag-and-drop file upload target supporting `.json`, `.db`, `.yaml`, `.yml`.
  * Real-time file inspection via `validate_prompt_tree_import` displaying stats and scope badge.
  * Conflict resolution toggle: `Merge & Upsert (FIFO)` vs `Clean Scope Replace`.
  * Target instance attachment path display.
  * Completion notification with automatic `loadTree(false, true)` call.

#### 7.3 Wire Context Menus:
- In `ProjectContextMenu.tsx`: Add "Export Section as JSON...", "Export Section as SQLite .db...", "Export Section as YAML...". Clicking opens `PromptTreeExportModal` pre-populated with `scope: { target_type: 'project', project_id }`.
- In `ConversationContextMenu.tsx`: Add identical items pre-populated with `scope: { target_type: 'conversation', conversation_id }`.
- In `PromptNodeContextMenu.tsx`: Add items pre-populated with `scope: { target_type: 'prompt_subtree', prompt_ids: [promptId] }`.

---

## 3. Verification & Acceptance Checklist for Worker 02

Before concluding TASK-02, Worker 02 must execute and verify:
1. **Compilation & Clippy Gates:**
   - `cd src-tauri && cargo fmt -- --check`
   - `cd src-tauri && cargo clippy --all-targets --all-features`
   - `npm run build`
2. **Export Verifications:**
   - Export single project as JSON: inspect envelope, ensure only target project is present.
   - Export single conversation as SQLite: verify `.db` file has valid SQLite header, meta kind, and rows.
   - Export prompt with image: verify `embedded_images` contains `data:image/...;base64,...` and prompt text contains `agm-asset://...`.
3. **Import Verifications:**
   - Import JSON into fresh instance: verify images materialize under `<target_dir>/.antigravity/attachments/` and display in UI without broken links.
   - Import SQLite: verify prompts merge in FIFO order (`ORDER BY created_at ASC, id ASC`).
4. **Refresh Verification:**
   - Select a conversation; click Refresh; verify the selected conversation remains selected and view does NOT snap to the top.
   - Start an IDE task, wait 90 seconds, click Refresh; verify the running status and spinner remain active.
