# Step 17: Shared Enqueue, Send-Now, List and Trace

Goal: one set of library functions that both the CLI (`agm`) and the desktop UI (Tauri IPC) call to enqueue a prompt, send a prompt now, list rows of one instance, back up one instance, and trace one prompt's identity chain. After this step the CLI and the UI cannot write differently shaped rows. Subtask `../03-cli-and-ipc-parity.md` Step 3 and Step 4 (shared core part).

Read `00-start-here.md` first. This file never overrides it.

## 1. Depends on

- Step 01: `InstanceScope`, `canonical_instance_id`, `canonical_instance_id_in`, `default_instance_id_in` in `src-tauri/src/modules/instance.rs`.
- Step 03: columns `status_reason`, `source_dir`, `attempts` on `active_prompts`; field `ActivePrompt.source_dir: Option<String>`.
- Step 09: `repo_owner_instances_in(conn, repo_path)`, `RESUME_TASK_FILE_NAME`.
- Step 10: `pub mod prompt_status`, `pub mod prompt_status_reason`, `claim_prompt_for_dispatch`, `dispatch_one_prompt_with`, `spawn_prompt_via_agy_outcome`, `SpawnOutcome`.
- Step 12: `init_tables` is `pub(crate)`; `backup_prompts_db::RestoreReport`.

Confirm with `git log --oneline -20` that the commits of steps 01, 03, 09, 10 and 12 are present. If any is missing, STOP.

## 2. Files you may edit

- `src-tauri/src/modules/instance.rs`
- `src-tauri/src/modules/repo_db.rs`
- `src-tauri/src/modules/backup_prompts_db.rs`
- Call sites of `list_running_projects()` and `list_backed_up_prompts()` (signature change, one-line edits only):
  - `src-tauri/src/commands/instance.rs`
  - `src-tauri/src/modules/agy_cleaner.rs`
  - `src-tauri/src/modules/auto_switcher.rs`
  - `src-tauri/src/modules/notification_hub.rs`
  - `src-tauri/src/modules/telegram_inbound.rs`
  - `src-tauri/src/modules/email_inbound.rs`
  - `src-tauri/src/proxy/server.rs`
  - `src-tauri/src/bin/agm.rs`

No other file. No edit in `src/`.

## 3. Find it

| Change | File | `fn` signature or place | Unique search literal | Line hint |
|---|---|---|---|---|
| A | `instance.rs` | insert above `pub fn resolve_instance_id(specifier: &str) -> Result<String, String> {` and its doc comment | `Resolve an instance query string` | `:4370` (moves after step 01) |
| B | `repo_db.rs` | `pub fn list_running_projects() -> Result<Vec<RunningProject>, String> {` | `List all running projects across instances` | `:2327` |
| C | `repo_db.rs` | `pub fn list_backed_up_prompts() -> Result<Vec<ActivePrompt>, String> {` | `pub fn list_backed_up_prompts()` | `:2364` |
| D | `repo_db.rs` | insert directly above `pub struct SwitchPromptSnap {` | `pub struct SwitchPromptSnap {` | `:2396` |
| E | `backup_prompts_db.rs` | insert directly above `/// List all backup batches and total counts` | `pub fn list_backup_batches(custom_file: Option<&str>)` | `:412` |
| F | every caller | `list_running_projects()` and `list_backed_up_prompts()` | see the caller table in section 5 | many |
| T | `repo_db.rs` | `#[cfg(test)] mod tests {` then `    use super::*;` | `fn test_uri_decoding()` | `:5879` |

Search commands (run all of them; `agm.rs` must be searched by its full path because the directory search skips it):

```text
gitmap aum search "Resolve an instance query string" src-tauri/src/modules/instance.rs --ext .rs
gitmap aum search "List all running projects across instances" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "pub fn list_backed_up_prompts" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "pub struct SwitchPromptSnap" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "pub fn list_backup_batches" src-tauri/src/modules/backup_prompts_db.rs --ext .rs
gitmap aum search "list_running_projects|list_backed_up_prompts" src-tauri/src --ext .rs
gitmap aum search "list_running_projects|list_backed_up_prompts" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "list_running_projects|list_backed_up_prompts" src-tauri/tests/auto_switcher_e2e_test.rs --ext .rs
gitmap aum search "list_running_projects|list_backed_up_prompts" src-tauri/tests/instance_cloning_and_sync_test.rs --ext .rs
gitmap aum search "list_running_projects|list_backed_up_prompts" src-tauri/tests/per_instance_prompt_liveness_test.rs --ext .rs
gitmap aum search "list_running_projects|list_backed_up_prompts" src --ext .tsx
gitmap aum search "list_running_projects|list_backed_up_prompts" src --ext .ts
gitmap aum search "backup_active_running_prompts_for_instance" src-tauri/src/modules/backup_prompts_db.rs --ext .rs
```

The patterns have no backslash on purpose (GitMap interprets backslashes), so they also hit the two definitions in `repo_db.rs`, the two command wrappers in `commands/instance.rs` and the registration in `lib.rs:1103` to `:1104`. The tests files have 0 hits (verified). The frontend hits (`src/services/instanceService.ts:443`, `:447`, `src/components/instances/PromptTreeViewModal.tsx:1104`) call the IPC commands with no arguments. They need no edit because the command wrappers keep zero parameters in this step; step 20 makes the parameter `Option<String>`, which is still compatible with a no-argument `invoke`.

Before adding any new name, search for it. If one already exists, STOP (another step took the name):

```text
gitmap aum search "instance_name_in|instance_seq_in|instance_short_label_in" src-tauri/src --ext .rs
gitmap aum search "InstancePromptRow|PromptRequest|INSTANCE_PROMPT_COLUMNS|enqueue_prompt_on|send_prompt_now_on|send_prompt_now_with|PromptTrace|trace_prompts_on|list_queued_prompts_on|list_running_projects_on|list_backed_up_prompts_on|ensure_queue_project_row|repo_slug" src-tauri/src --ext .rs
gitmap aum search "BackupReport|backup_prompts_for_instance" src-tauri/src --ext .rs
```

Confirm the names this step uses from earlier steps exist (each search must return at least one hit):

```text
gitmap aum search "pub fn canonical_instance_id_in" src-tauri/src/modules/instance.rs --ext .rs
gitmap aum search "pub enum InstanceScope" src-tauri/src/modules/instance.rs --ext .rs
gitmap aum search "pub fn repo_owner_instances_in" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "pub const RESUME_TASK_FILE_NAME" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "pub mod prompt_status" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "pub fn claim_prompt_for_dispatch" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "fn dispatch_one_prompt_with" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "pub fn spawn_prompt_via_agy_outcome" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "pub(crate) fn init_tables" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "pub source_dir: Option<String>" src-tauri/src/modules/repo_db.rs --ext .rs
```

If `init_tables` is still private (`fn init_tables(conn: &Connection)` without `pub(crate)`), change only that keyword to `pub(crate) fn init_tables(conn: &Connection) -> Result<(), String> {`. That is the one allowed extra edit.

## 4. Current code

### Change A (`instance.rs`, insertion point only)

```rust
/// Resolve an instance query string (seq_num like "1", ID like "inst-xyz", name like "Instance 1", or "default"/"active")
/// to a valid concrete instance ID.
pub fn resolve_instance_id(specifier: &str) -> Result<String, String> {
```

### Change B (`repo_db.rs:2327` to `:2361`)

```rust
/// List all running projects across instances
pub fn list_running_projects() -> Result<Vec<RunningProject>, String> {
    let conn = connect_db()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at 
             FROM running_projects 
             WHERE workspace_storage_path IS NOT NULL 
               AND trim(workspace_storage_path) != '' 
               AND instr(id, '__') > 0 
               AND trim(instance_id) != ''
             ORDER BY last_detected_at DESC",
        )
        .map_err(|e| format!("Failed to prepare list projects query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            let running_int: i32 = row.get(5)?;
            let is_running = running_int != 0;
            Ok(RunningProject {
                id: row.get(0)?,
                instance_id: row.get(1)?,
                repo_name: row.get(2)?,
                repo_path: row.get(3)?,
                workspace_storage_path: row.get(4)?,
                is_running,
                last_detected_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("Failed to query running projects: {}", e))?
        .flatten()
        .collect();

    Ok(rows)
}
```

### Change C (`repo_db.rs:2363` to `:2394`)

Expected drift: step 03 added `, source_dir` to the SELECT and `source_dir: row.get(11).ok(),` to the initializer. That drift is fine; the whole function is replaced.

```rust
/// List all backed up prompts
pub fn list_backed_up_prompts() -> Result<Vec<ActivePrompt>, String> {
    let conn = connect_db()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload 
             FROM active_prompts WHERE status = 'backed_up' ORDER BY created_at DESC",
        )
        .map_err(|e| format!("Failed to prepare list prompts query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ActivePrompt {
                id: row.get(0)?,
                project_id: row.get(1)?,
                instance_id: row.get(2)?,
                repo_path: row.get(3)?,
                prompt_content: row.get(4)?,
                model: row.get(5)?,
                session_id: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                image_payload: row.get(10).ok(),
            })
        })
        .map_err(|e| format!("Failed to query backed-up prompts: {}", e))?
        .flatten()
        .collect();

    Ok(rows)
}
```

### Change D (`repo_db.rs`, insertion point only)

```rust
pub struct SwitchPromptSnap {
    pub prompt_id: String,
    pub prompt_text: String,
```

### Change E (`backup_prompts_db.rs`, insertion point only)

```rust
    Ok((batch_info, records))
}

/// List all backup batches and total counts
pub fn list_backup_batches(custom_file: Option<&str>) -> Result<Vec<BackupBatchInfo>, String> {
```

The function that ends with `Ok((batch_info, records))` is `backup_active_running_prompts_for_instance`. Step 15 (which runs before this step) changed its first parameter from `Option<&str>` to `&str`, so its signature is now:

```rust
pub fn backup_active_running_prompts_for_instance(
    instance_id: &str,
    custom_file: Option<&str>,
) -> Result<(BackupBatchInfo, Vec<PromptBackupRecord>), String> {
```

Change E calls it with `&inst`. If the search still shows `instance_id: Option<&str>`, step 15 was not applied: STOP and report.

### Change F (every caller line, verbatim; line hints are from before steps 01 to 16)

Each row is the exact current line (trimmed of leading spaces) and the exact new line. Keep the original indentation. All 27 calls are listed: 20 in `src-tauri/src` (state after step 16) and 7 in `agm.rs`. Step 16 already collapsed the four telegram calls (`:1341`, `:1372`, `:2716`, `:2775`) into the single line inside `fn running_project_display()` (or `fn backup_project_display()` if step 16 had to use that name). If those four old telegram lines still exist, step 16 was not applied: STOP and report. If any other row is missing, or a search in section 3 finds a call that is not in this table, STOP and report.

`ALL` below is short for `&crate::modules::instance::InstanceScope::All`, so write the full path in the code. In `agm.rs` it is `&instance::InstanceScope::All` because agm already imports `instance` in its `use antigravity_tools_lib::modules::{...}` list.

| File | Line hint | Current line (verbatim) | New line (verbatim) |
|---|---|---|---|
| `src-tauri/src/commands/instance.rs` | `:208` | `crate::modules::repo_db::list_running_projects()` | `crate::modules::repo_db::list_running_projects(&crate::modules::instance::InstanceScope::All)` |
| `src-tauri/src/commands/instance.rs` | `:213` | `crate::modules::repo_db::list_backed_up_prompts()` | `crate::modules::repo_db::list_backed_up_prompts(&crate::modules::instance::InstanceScope::All)` |
| `src-tauri/src/modules/agy_cleaner.rs` | `:368` | `if let Ok(running_projs) = crate::modules::repo_db::list_running_projects() {` | `if let Ok(running_projs) = crate::modules::repo_db::list_running_projects(&crate::modules::instance::InstanceScope::All) {` |
| `src-tauri/src/modules/auto_switcher.rs` | `:536` | `if let Ok(projects) = crate::modules::repo_db::list_running_projects() {` | `if let Ok(projects) = crate::modules::repo_db::list_running_projects(&crate::modules::instance::InstanceScope::All) {` |
| `src-tauri/src/modules/auto_switcher.rs` | `:543` | `if let Ok(prompts) = crate::modules::repo_db::list_backed_up_prompts() {` | `if let Ok(prompts) = crate::modules::repo_db::list_backed_up_prompts(&crate::modules::instance::InstanceScope::All) {` |
| `src-tauri/src/modules/auto_switcher.rs` | `:1529` | `let running_projs = crate::modules::repo_db::list_running_projects().unwrap_or_default();` | `let running_projs = crate::modules::repo_db::list_running_projects(&crate::modules::instance::InstanceScope::All).unwrap_or_default();` |
| `src-tauri/src/modules/email_inbound.rs` | `:1518` | `let projects = crate::modules::repo_db::list_running_projects()?;` | `let projects = crate::modules::repo_db::list_running_projects(&crate::modules::instance::InstanceScope::All)?;` |
| `src-tauri/src/modules/email_inbound.rs` | `:1929` | `let projects = crate::modules::repo_db::list_running_projects().unwrap_or_default();` | `let projects = crate::modules::repo_db::list_running_projects(&crate::modules::instance::InstanceScope::All).unwrap_or_default();` |
| `src-tauri/src/modules/email_inbound.rs` | `:1930` | `let prompts = crate::modules::repo_db::list_backed_up_prompts().unwrap_or_default();` | `let prompts = crate::modules::repo_db::list_backed_up_prompts(&crate::modules::instance::InstanceScope::All).unwrap_or_default();` |
| `src-tauri/src/modules/notification_hub.rs` | `:553` | `if let Ok(projects) = crate::modules::repo_db::list_running_projects() {` | `if let Ok(projects) = crate::modules::repo_db::list_running_projects(&crate::modules::instance::InstanceScope::All) {` |
| `src-tauri/src/modules/notification_hub.rs` | `:615` | `match crate::modules::repo_db::list_running_projects() {` | `match crate::modules::repo_db::list_running_projects(&crate::modules::instance::InstanceScope::All) {` |
| `src-tauri/src/modules/notification_hub.rs` | `:925` | `match crate::modules::repo_db::list_running_projects() {` | `match crate::modules::repo_db::list_running_projects(&crate::modules::instance::InstanceScope::All) {` |
| `src-tauri/src/modules/telegram_inbound.rs` | inside `fn running_project_display()` (added by step 16) | `let projs = repo_db::list_running_projects().unwrap_or_default();` | `let projs = repo_db::list_running_projects(&crate::modules::instance::InstanceScope::All).unwrap_or_default();` |
| `src-tauri/src/modules/repo_db.rs` | `:2642` | `let projects = list_running_projects().unwrap_or_default();` | `let projects = list_running_projects(&crate::modules::instance::InstanceScope::All).unwrap_or_default();` |
| `src-tauri/src/modules/repo_db.rs` | `:4071` | `let all_projects: Vec<RunningProject> = list_running_projects()` | `let all_projects: Vec<RunningProject> = list_running_projects(&crate::modules::instance::InstanceScope::All)` |
| `src-tauri/src/modules/instance.rs` | `:2752` | `if let Ok(all_projs) = crate::modules::repo_db::list_running_projects() {` | `if let Ok(all_projs) = crate::modules::repo_db::list_running_projects(&crate::modules::instance::InstanceScope::All) {` |
| `src-tauri/src/modules/instance.rs` | `:2835` | `if let Ok(projects) = crate::modules::repo_db::list_running_projects() {` | `if let Ok(projects) = crate::modules::repo_db::list_running_projects(&crate::modules::instance::InstanceScope::All) {` |
| `src-tauri/src/modules/instance.rs` | `:4990` | `let running_projs = crate::modules::repo_db::list_running_projects().unwrap_or_default();` | `let running_projs = crate::modules::repo_db::list_running_projects(&crate::modules::instance::InstanceScope::All).unwrap_or_default();` |
| `src-tauri/src/proxy/server.rs` | `:1746` | `match crate::modules::repo_db::list_running_projects() {` | `match crate::modules::repo_db::list_running_projects(&crate::modules::instance::InstanceScope::All) {` |
| `src-tauri/src/proxy/server.rs` | `:1756` | `match crate::modules::repo_db::list_backed_up_prompts() {` | `match crate::modules::repo_db::list_backed_up_prompts(&crate::modules::instance::InstanceScope::All) {` |
| `src-tauri/src/bin/agm.rs` | `:1629` | `let projects = repo_db::list_running_projects().unwrap_or_default();` | `let projects = repo_db::list_running_projects(&instance::InstanceScope::All).unwrap_or_default();` |
| `src-tauri/src/bin/agm.rs` | `:4627` | `let projects = repo_db::list_running_projects().unwrap_or_default();` | `let projects = repo_db::list_running_projects(&instance::InstanceScope::All).unwrap_or_default();` |
| `src-tauri/src/bin/agm.rs` | `:4788` | `let running_p = repo_db::list_running_projects().unwrap_or_default();` | `let running_p = repo_db::list_running_projects(&instance::InstanceScope::All).unwrap_or_default();` |
| `src-tauri/src/bin/agm.rs` | `:4902` | `let running = repo_db::list_running_projects().unwrap_or_default();` | `let running = repo_db::list_running_projects(&instance::InstanceScope::All).unwrap_or_default();` |
| `src-tauri/src/bin/agm.rs` | `:7932` | `if let Ok(projects) = repo_db::list_running_projects() {` | `if let Ok(projects) = repo_db::list_running_projects(&instance::InstanceScope::All) {` |
| `src-tauri/src/bin/agm.rs` | `:8343` | `if let Ok(projects) = repo_db::list_running_projects() {` | `if let Ok(projects) = repo_db::list_running_projects(&instance::InstanceScope::All) {` |
| `src-tauri/src/bin/agm.rs` | `:11731` | `let projects = repo_db::list_running_projects().unwrap_or_default();` | `let projects = repo_db::list_running_projects(&instance::InstanceScope::All).unwrap_or_default();` |

The two `commands/instance.rs` rows are the bodies of the IPC wrappers; their full new form is shown in section 5, Change F. `agm.rs:1629` and `:4627` are replaced again with real per-instance scopes in step 18. `src-tauri/tests/*.rs` has no caller (verified by searching each file by full path). The three frontend `invoke` calls need no edit (see section 3).

### Change T (`repo_db.rs` test module, insertion point only)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_decoding() {
```

Other steps pasted tests and helpers right after `use super::*;`. That is fine; paste this step's tests directly after `    use super::*;` and above whatever is there now.

## 5. New code

### Change A (`instance.rs`): insert this block directly above the `///` doc line of `resolve_instance_id`

Leave one empty line before and after the block.

```rust
/// Display name of a registry entry; the id itself when the entry is missing or unnamed.
pub fn instance_name_in(registry: &InstanceRegistry, instance_id: &str) -> String {
    registry
        .instances
        .iter()
        .find(|inst| inst.id.eq_ignore_ascii_case(instance_id))
        .map(|inst| inst.name.clone())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| instance_id.to_string())
}

pub fn instance_seq_in(registry: &InstanceRegistry, instance_id: &str) -> Option<u32> {
    registry
        .instances
        .iter()
        .find(|inst| inst.id.eq_ignore_ascii_case(instance_id))
        .and_then(|inst| inst.seq_num)
}

/// `#<seq> <name>`, with `-` when the entry has no sequence number.
pub fn instance_short_label_in(registry: &InstanceRegistry, instance_id: &str) -> String {
    let seq = instance_seq_in(registry, instance_id)
        .map(|n| n.to_string())
        .unwrap_or_else(|| "-".to_string());
    format!("#{} {}", seq, instance_name_in(registry, instance_id))
}
```

### Change B (`repo_db.rs`): replace the whole `list_running_projects` function with

```rust
/// List running projects of one instance or of every instance
pub fn list_running_projects(
    scope: &crate::modules::instance::InstanceScope,
) -> Result<Vec<RunningProject>, String> {
    let conn = connect_db()?;
    list_running_projects_on(&conn, scope)
}

fn instance_scope_values(scope: &crate::modules::instance::InstanceScope) -> Vec<String> {
    match scope {
        crate::modules::instance::InstanceScope::One(id) => vec![id.clone()],
        crate::modules::instance::InstanceScope::All => Vec::new(),
    }
}

pub fn list_running_projects_on(
    conn: &Connection,
    scope: &crate::modules::instance::InstanceScope,
) -> Result<Vec<RunningProject>, String> {
    let filter = instance_scope_values(scope);
    let mut sql = String::from(
        "SELECT id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at 
         FROM running_projects 
         WHERE workspace_storage_path IS NOT NULL 
           AND trim(workspace_storage_path) != '' 
           AND instr(id, '__') > 0 
           AND trim(instance_id) != ''",
    );
    if !filter.is_empty() {
        sql.push_str(" AND instance_id = ?1");
    }
    sql.push_str(" ORDER BY last_detected_at DESC");

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare list projects query: {}", e))?;

    let rows: Vec<RunningProject> = stmt
        .query_map(rusqlite::params_from_iter(filter.iter()), |row| {
            let running_int: i32 = row.get(5)?;
            let is_running = running_int != 0;
            Ok(RunningProject {
                id: row.get(0)?,
                instance_id: row.get(1)?,
                repo_name: row.get(2)?,
                repo_path: row.get(3)?,
                workspace_storage_path: row.get(4)?,
                is_running,
                last_detected_at: row.get(6)?,
            })
        })
        .map_err(|e| format!("Failed to query running projects: {}", e))?
        .flatten()
        .collect();

    Ok(rows)
}
```

### Change C (`repo_db.rs`): replace the whole `list_backed_up_prompts` function with

```rust
/// List backed up prompts of one instance or of every instance
pub fn list_backed_up_prompts(
    scope: &crate::modules::instance::InstanceScope,
) -> Result<Vec<ActivePrompt>, String> {
    let conn = connect_db()?;
    list_backed_up_prompts_on(&conn, scope)
}

pub fn list_backed_up_prompts_on(
    conn: &Connection,
    scope: &crate::modules::instance::InstanceScope,
) -> Result<Vec<ActivePrompt>, String> {
    let filter = instance_scope_values(scope);
    let mut sql = format!(
        "SELECT {} FROM active_prompts WHERE status = '{}'",
        INSTANCE_PROMPT_COLUMNS,
        prompt_status::BACKED_UP
    );
    if !filter.is_empty() {
        sql.push_str(" AND instance_id = ?1");
    }
    sql.push_str(" ORDER BY created_at DESC");

    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare list prompts query: {}", e))?;

    let rows: Vec<ActivePrompt> = stmt
        .query_map(rusqlite::params_from_iter(filter.iter()), active_prompt_from_sql)
        .map_err(|e| format!("Failed to query backed-up prompts: {}", e))?
        .flatten()
        .collect();

    Ok(rows)
}
```

### Change D (`repo_db.rs`): insert this block directly above `pub struct SwitchPromptSnap {`

Leave one empty line before and after the block. `prompt_status`, `prompt_status_reason`, `claim_prompt_for_dispatch`, `dispatch_one_prompt_with`, `spawn_prompt_via_agy_outcome`, `SpawnOutcome`, `repo_owner_instances_in`, `RESUME_TASK_FILE_NAME`, `connect_db`, `invalidate_prompt_tree_cache` and `normalize_path_for_compare` are all in this same file already.

```rust
pub const INSTANCE_PROMPT_COLUMNS: &str = "id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload, source_dir, status_reason, attempts";

/// One `active_prompts` row plus the columns `ActivePrompt` does not carry.
#[derive(Debug, Clone, Serialize)]
pub struct InstancePromptRow {
    #[serde(flatten)]
    pub prompt: ActivePrompt,
    pub instance_name: String,
    pub status_reason: Option<String>,
    pub attempts: i64,
}

/// Input shared by the CLI and the UI for one new prompt row.
#[derive(Debug, Clone, Copy)]
pub struct PromptRequest<'a> {
    pub instance_id: &'a str,
    pub repo_path: &'a str,
    pub text: &'a str,
    pub conversation_id: Option<&'a str>,
    pub source: &'a str,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptHandoffTrace {
    pub path: String,
    pub legacy: String,
}

/// Identity chain of one prompt row, as printed by `agm prompts trace` and `trace_prompts`.
#[derive(Debug, Clone, Serialize)]
pub struct PromptTrace {
    pub instance_id: String,
    pub instance_name: String,
    pub source_dir: Option<String>,
    pub conversation_id: Option<String>,
    pub repo_path: String,
    pub row_id: String,
    pub status: String,
    pub status_reason: Option<String>,
    pub attempts: i64,
    pub status_history: Vec<serde_json::Value>,
    pub backup_row_id: Option<String>,
    pub handoff: PromptHandoffTrace,
    pub dispatch_result: Option<String>,
}

fn active_prompt_from_sql(row: &rusqlite::Row<'_>) -> rusqlite::Result<ActivePrompt> {
    Ok(ActivePrompt {
        id: row.get(0)?,
        project_id: row.get(1)?,
        instance_id: row.get(2)?,
        repo_path: row.get(3)?,
        prompt_content: row.get(4)?,
        model: row.get(5)?,
        session_id: row.get(6)?,
        status: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        image_payload: row.get(10).ok(),
        source_dir: row.get(11).ok(),
    })
}

fn instance_prompt_row_from_sql(
    row: &rusqlite::Row<'_>,
    registry: &crate::modules::instance::InstanceRegistry,
) -> rusqlite::Result<InstancePromptRow> {
    let prompt = active_prompt_from_sql(row)?;
    let instance_name = crate::modules::instance::instance_name_in(registry, &prompt.instance_id);
    Ok(InstancePromptRow {
        status_reason: row.get::<_, Option<String>>(12)?,
        attempts: row.get::<_, Option<i64>>(13)?.unwrap_or(0),
        instance_name,
        prompt,
    })
}

fn read_instance_prompt_row(
    conn: &Connection,
    registry: &crate::modules::instance::InstanceRegistry,
    id: &str,
) -> Result<InstancePromptRow, String> {
    conn.query_row(
        &format!(
            "SELECT {} FROM active_prompts WHERE id = ?1",
            INSTANCE_PROMPT_COLUMNS
        ),
        params![id],
        |row| instance_prompt_row_from_sql(row, registry),
    )
    .map_err(|e| format!("Failed to read prompt row '{}': {}", id, e))
}

fn repo_folder_name(repo_path: &str) -> String {
    Path::new(repo_path.trim().trim_end_matches(['/', '\\']))
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Same rule as `agm`'s `derive_current_repo_slug`: lowercase, non-alphanumerics become `-`.
fn repo_slug(repo_path: &str) -> String {
    let slug: String = repo_folder_name(repo_path)
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let trimmed = slug.trim_matches('-');
    if trimmed.is_empty() {
        "workspace".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Reuses this instance's project row for the repo (normalized path equality) or inserts a
/// composite `{slug}__{instance}` row that the startup purge keeps.
fn ensure_queue_project_row(
    conn: &Connection,
    registry: &crate::modules::instance::InstanceRegistry,
    instance_id: &str,
    repo_path: &str,
) -> Result<String, String> {
    let target = normalize_path_for_compare(repo_path);
    let mut stmt = conn
        .prepare("SELECT id, repo_path FROM running_projects WHERE instance_id = ?1")
        .map_err(|e| format!("Failed to prepare project lookup: {}", e))?;
    let existing: Vec<(String, String)> = stmt
        .query_map(params![instance_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| format!("Failed to query project rows: {}", e))?
        .flatten()
        .collect();
    if let Some((id, _)) = existing
        .into_iter()
        .find(|(id, path)| id.contains("__") && normalize_path_for_compare(path) == target)
    {
        return Ok(id);
    }

    let slug = repo_slug(repo_path);
    let project_id = format!("{}__{}", slug, instance_id);
    let data_dir = registry
        .instances
        .iter()
        .find(|inst| inst.id == instance_id)
        .map(|inst| inst.data_dir.clone())
        .unwrap_or_default();
    let ws_path = if data_dir.trim().is_empty() {
        PathBuf::from("workspaceStorage").join(&slug)
    } else {
        PathBuf::from(&data_dir)
            .join("User")
            .join("workspaceStorage")
            .join(&slug)
    };
    let repo_name = {
        let name = repo_folder_name(repo_path);
        if name.is_empty() {
            slug.clone()
        } else {
            name
        }
    };
    let now = Utc::now().timestamp();
    conn.execute(
        "INSERT OR IGNORE INTO running_projects (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?6)",
        params![
            project_id,
            instance_id,
            repo_name,
            repo_path,
            ws_path.to_string_lossy().to_string(),
            now
        ],
    )
    .map_err(|e| format!("Failed to insert project row: {}", e))?;
    Ok(project_id)
}

fn validate_prompt_request(request: &PromptRequest<'_>) -> Result<(), String> {
    if request.text.trim().is_empty() {
        return Err("prompt text is empty".to_string());
    }
    if request.repo_path.trim().is_empty() {
        return Err("repo path is empty".to_string());
    }
    Ok(())
}

/// Inserts one `queued` row for a canonical instance and returns its id.
fn insert_queued_prompt(
    conn: &Connection,
    registry: &crate::modules::instance::InstanceRegistry,
    instance_id: &str,
    id_prefix: &str,
    request: &PromptRequest<'_>,
) -> Result<String, String> {
    let project_id = ensure_queue_project_row(conn, registry, instance_id, request.repo_path)?;
    let id = format!("{}-{}-{}", id_prefix, instance_id, Uuid::new_v4());
    let conversation_id = request
        .conversation_id
        .map(str::trim)
        .filter(|cid| !cid.is_empty());
    let now = Utc::now().timestamp();
    conn.execute(
        "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, source_dir, attempts)
         VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7, ?8, ?8, ?9, 0)",
        params![
            id,
            project_id,
            instance_id,
            request.repo_path,
            request.text,
            conversation_id,
            prompt_status::QUEUED,
            now,
            request.source
        ],
    )
    .map_err(|e| format!("Failed to insert queued prompt: {}", e))?;
    Ok(id)
}

/// Pure core: one `queued` row for exactly the named instance. Unknown ids are rejected.
pub fn enqueue_prompt_on(
    conn: &Connection,
    registry: &crate::modules::instance::InstanceRegistry,
    request: &PromptRequest<'_>,
) -> Result<InstancePromptRow, String> {
    let inst = crate::modules::instance::canonical_instance_id_in(registry, request.instance_id)?;
    validate_prompt_request(request)?;
    if !Path::new(request.repo_path).is_dir() {
        return Err(format!("repo path does not exist: {}", request.repo_path));
    }
    let id = insert_queued_prompt(conn, registry, &inst, "queued", request)?;
    read_instance_prompt_row(conn, registry, &id)
}

/// Pure core of send-now with an injected spawner. A missing repo is recorded as
/// `failed` / `repo_missing` instead of being rejected, so the attempt stays visible.
pub fn send_prompt_now_with<F>(
    conn: &Connection,
    registry: &crate::modules::instance::InstanceRegistry,
    request: &PromptRequest<'_>,
    spawn: F,
) -> Result<InstancePromptRow, String>
where
    F: Fn(&ActivePrompt) -> SpawnOutcome,
{
    let inst = crate::modules::instance::canonical_instance_id_in(registry, request.instance_id)?;
    validate_prompt_request(request)?;
    let id_prefix = if request.source == "cli" { "cli" } else { "ui" };
    let id = insert_queued_prompt(conn, registry, &inst, id_prefix, request)?;
    let queued = read_instance_prompt_row(conn, registry, &id)?;
    if claim_prompt_for_dispatch(conn, &id, prompt_status::QUEUED) {
        let _ = dispatch_one_prompt_with(conn, &queued.prompt, spawn);
    }
    read_instance_prompt_row(conn, registry, &id)
}

pub fn send_prompt_now_on(
    conn: &Connection,
    registry: &crate::modules::instance::InstanceRegistry,
    request: &PromptRequest<'_>,
) -> Result<InstancePromptRow, String> {
    send_prompt_now_with(conn, registry, request, spawn_prompt_via_agy_outcome)
}

/// Pure core: `queued`, `dispatching` and `failed` rows of one instance, oldest first.
pub fn list_queued_prompts_on(
    conn: &Connection,
    registry: &crate::modules::instance::InstanceRegistry,
    instance_id: &str,
) -> Result<Vec<InstancePromptRow>, String> {
    let inst = crate::modules::instance::canonical_instance_id_in(registry, instance_id)?;
    let sql = format!(
        "SELECT {} FROM active_prompts WHERE instance_id = ?1 AND status IN (?2, ?3, ?4) ORDER BY created_at ASC, id ASC",
        INSTANCE_PROMPT_COLUMNS
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare queue query: {}", e))?;
    let rows: Vec<InstancePromptRow> = stmt
        .query_map(
            params![
                inst,
                prompt_status::QUEUED,
                prompt_status::DISPATCHING,
                prompt_status::FAILED
            ],
            |row| instance_prompt_row_from_sql(row, registry),
        )
        .map_err(|e| format!("Failed to query queued prompts: {}", e))?
        .flatten()
        .collect();
    Ok(rows)
}

fn latest_backup_row(
    backup_conn: Option<&Connection>,
    instance_id: &str,
    prompt_id: &str,
) -> Option<(String, i64, bool)> {
    backup_conn.and_then(|conn| {
        conn.query_row(
            "SELECT id, created_at, is_restored FROM prompt_backups WHERE instance_id = ?1 AND prompt_id = ?2 ORDER BY created_at DESC LIMIT 1",
            params![instance_id, prompt_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .ok()
    })
}

fn build_prompt_trace(
    conn: &Connection,
    backup_conn: Option<&Connection>,
    row: InstancePromptRow,
) -> PromptTrace {
    let p = &row.prompt;
    let backup = latest_backup_row(backup_conn, &p.instance_id, &p.id);

    let mut status_history = vec![serde_json::json!({
        "event": "created",
        "at": p.created_at,
        "source_dir": p.source_dir,
    })];
    if let Some((backup_id, backed_up_at, is_restored)) = &backup {
        status_history.push(serde_json::json!({
            "event": "backed_up",
            "at": backed_up_at,
            "backup_row_id": backup_id,
            "is_restored": is_restored,
        }));
    }
    status_history.push(serde_json::json!({
        "event": "current",
        "at": p.updated_at,
        "status": p.status,
        "status_reason": row.status_reason,
        "attempts": row.attempts,
    }));

    let has_dispatch_attempt = (p.status == prompt_status::DISPATCHING
        || p.status == prompt_status::DISPATCHED
        || p.status == prompt_status::FAILED)
        && row.status_reason.as_deref() != Some(prompt_status_reason::REPO_MISSING);
    let legacy = if has_dispatch_attempt {
        match repo_owner_instances_in(conn, &p.repo_path).len() {
            0 => "not_applicable",
            1 => "written",
            _ => "skipped_multi_owner",
        }
    } else {
        "not_applicable"
    };
    let handoff_path = Path::new(&p.repo_path)
        .join(RESUME_TASK_FILE_NAME)
        .to_string_lossy()
        .to_string();

    let dispatch_result = if p.status == prompt_status::DISPATCHED {
        Some(match &row.status_reason {
            Some(reason) => format!("sent:{}", reason),
            None => "sent".to_string(),
        })
    } else if p.status == prompt_status::FAILED {
        Some(format!(
            "failed:{}",
            row.status_reason.as_deref().unwrap_or("unknown")
        ))
    } else {
        None
    };

    PromptTrace {
        instance_id: p.instance_id.clone(),
        instance_name: row.instance_name.clone(),
        source_dir: p.source_dir.clone(),
        conversation_id: p.session_id.clone(),
        repo_path: p.repo_path.clone(),
        row_id: p.id.clone(),
        status: p.status.clone(),
        status_reason: row.status_reason.clone(),
        attempts: row.attempts,
        status_history,
        backup_row_id: backup.map(|(id, _, _)| id),
        handoff: PromptHandoffTrace {
            path: handoff_path,
            legacy: legacy.to_string(),
        },
        dispatch_result,
    }
}

/// Pure core: identity chain of every row of one instance, optionally one conversation.
pub fn trace_prompts_on(
    conn: &Connection,
    backup_conn: Option<&Connection>,
    registry: &crate::modules::instance::InstanceRegistry,
    instance_id: &str,
    conversation_id: Option<&str>,
) -> Result<Vec<PromptTrace>, String> {
    let inst = crate::modules::instance::canonical_instance_id_in(registry, instance_id)?;
    let mut values = vec![inst];
    let mut sql = format!(
        "SELECT {} FROM active_prompts WHERE instance_id = ?1",
        INSTANCE_PROMPT_COLUMNS
    );
    if let Some(cid) = conversation_id.map(str::trim).filter(|c| !c.is_empty()) {
        sql.push_str(" AND session_id = ?2");
        values.push(cid.to_string());
    }
    sql.push_str(" ORDER BY created_at ASC, id ASC");
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Failed to prepare trace query: {}", e))?;
    let rows: Vec<InstancePromptRow> = stmt
        .query_map(rusqlite::params_from_iter(values.iter()), |row| {
            instance_prompt_row_from_sql(row, registry)
        })
        .map_err(|e| format!("Failed to query prompt trace: {}", e))?
        .flatten()
        .collect();
    Ok(rows
        .into_iter()
        .map(|row| build_prompt_trace(conn, backup_conn, row))
        .collect())
}

pub fn enqueue_prompt_for_instance(
    instance_id: &str,
    repo_path: &str,
    text: &str,
    conversation_id: Option<&str>,
    source: &str,
) -> Result<InstancePromptRow, String> {
    let registry = crate::modules::instance::load_registry()?;
    let conn = connect_db()?;
    let request = PromptRequest {
        instance_id,
        repo_path,
        text,
        conversation_id,
        source,
    };
    let row = enqueue_prompt_on(&conn, &registry, &request)?;
    invalidate_prompt_tree_cache(Some(&row.prompt.instance_id));
    Ok(row)
}

pub fn send_prompt_now_for_instance(
    instance_id: &str,
    repo_path: &str,
    text: &str,
    conversation_id: Option<&str>,
    source: &str,
) -> Result<InstancePromptRow, String> {
    let registry = crate::modules::instance::load_registry()?;
    let conn = connect_db()?;
    let request = PromptRequest {
        instance_id,
        repo_path,
        text,
        conversation_id,
        source,
    };
    let row = send_prompt_now_on(&conn, &registry, &request)?;
    invalidate_prompt_tree_cache(Some(&row.prompt.instance_id));
    Ok(row)
}

pub fn list_queued_prompts_for_instance(instance_id: &str) -> Result<Vec<InstancePromptRow>, String> {
    let registry = crate::modules::instance::load_registry()?;
    let conn = connect_db()?;
    list_queued_prompts_on(&conn, &registry, instance_id)
}

pub fn trace_prompts_for_instance(
    instance_id: &str,
    conversation_id: Option<&str>,
) -> Result<Vec<PromptTrace>, String> {
    let registry = crate::modules::instance::load_registry()?;
    let conn = connect_db()?;
    let backup_conn = crate::modules::backup_prompts_db::connect_backup_db(None).ok();
    trace_prompts_on(
        &conn,
        backup_conn.as_ref(),
        &registry,
        instance_id,
        conversation_id,
    )
}
```

Compiler-forced adaptations you may make (and nothing else):

- If the compiler says `ActivePrompt` has a field not listed in `active_prompt_from_sql`, STOP (an unknown step added a field).
- If `trim_end_matches(['/', '\\'])` is rejected by the toolchain, write `trim_end_matches(|c| c == '/' || c == '\\')`.

### Change E (`backup_prompts_db.rs`): insert directly above `/// List all backup batches and total counts`

```rust
/// Result of backing up one instance: Layer 1 capture count plus the split-DB batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupReport {
    pub instance_id: String,
    pub batch_id: String,
    pub file_path: String,
    pub captured_from_workspace: usize,
    pub prompts_count: usize,
    pub created_at: i64,
    pub records: Vec<PromptBackupRecord>,
}

/// Backs up exactly one instance (canonical id required). Shared by `agm brp` and the UI.
pub fn backup_prompts_for_instance(
    instance_id: &str,
    custom_file: Option<&str>,
) -> Result<BackupReport, String> {
    let inst = crate::modules::instance::canonical_instance_id(instance_id)?;
    let captured_from_workspace = repo_db::backup_running_prompts(&inst).unwrap_or(0);
    let (batch, records) = backup_active_running_prompts_for_instance(&inst, custom_file)?;
    Ok(BackupReport {
        instance_id: inst,
        batch_id: batch.id,
        file_path: batch.file_path,
        captured_from_workspace,
        prompts_count: records.len(),
        created_at: batch.created_at,
        records,
    })
}
```

### Change F (callers): one rule for every call found in section 3

Every caller today means "all instances". Pass `All` explicitly:

- In files under `src-tauri/src/` other than `agm.rs`: replace `list_running_projects()` with `list_running_projects(&crate::modules::instance::InstanceScope::All)` and `list_backed_up_prompts()` with `list_backed_up_prompts(&crate::modules::instance::InstanceScope::All)`. Keep whatever path prefix was there (`crate::modules::repo_db::`, `repo_db::`, or none inside `repo_db.rs`).
- In `src-tauri/src/bin/agm.rs`: replace `repo_db::list_running_projects()` with `repo_db::list_running_projects(&instance::InstanceScope::All)` (agm already imports `instance`).

Example, `src-tauri/src/proxy/server.rs:1746`:

```rust
    match crate::modules::repo_db::list_running_projects(&crate::modules::instance::InstanceScope::All) {
```

`src-tauri/src/commands/instance.rs:206` to `:214` becomes (step 20 replaces it again):

```rust
#[tauri::command]
pub fn list_running_projects() -> Result<Vec<crate::modules::repo_db::RunningProject>, String> {
    crate::modules::repo_db::list_running_projects(&crate::modules::instance::InstanceScope::All)
}

#[tauri::command]
pub fn list_backed_up_prompts() -> Result<Vec<crate::modules::repo_db::ActivePrompt>, String> {
    crate::modules::repo_db::list_backed_up_prompts(&crate::modules::instance::InstanceScope::All)
}
```

Do not change any other logic at the call sites. After this, all three searches in section 3 must show no call with empty parentheses.

## 6. Tests

Paste the helpers and tests directly after `    use super::*;` in `mod tests` of `src-tauri/src/modules/repo_db.rs`. They use an in-memory DB, an in-memory registry, and temp folders they create and delete. No `load_registry()`, no environment variable changes. `fs`, `Path`, `PathBuf`, `Uuid`, `Connection` and `params` are already imported by `use super::*;`.

```rust
    fn q140_registry(data_root: &Path) -> crate::modules::instance::InstanceRegistry {
        let make = |id: &str, name: &str, seq: u32, is_default: bool| {
            crate::modules::instance::InstanceConfig {
                id: id.to_string(),
                name: name.to_string(),
                data_dir: data_root.join(id).to_string_lossy().to_string(),
                executable_path: None,
                extensions_dir: None,
                bound_account_id: None,
                bound_email: None,
                created_at: 0,
                last_used: 0,
                is_default,
                pid: None,
                seq_num: Some(seq),
            }
        };
        crate::modules::instance::InstanceRegistry {
            active_instance_id: "default".to_string(),
            instances: vec![
                make("default", "Default", 1, true),
                make("test-cli-flow-a-0001", "Alpha", 2, false),
                make("test-cli-flow-b-0001", "Beta", 3, false),
            ],
        }
    }

    fn q140_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        conn
    }

    fn q140_temp_repo(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("agm-q140-{}-{}", label, Uuid::new_v4().simple()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn q140_request<'a>(
        instance_id: &'a str,
        repo_path: &'a str,
        conversation_id: Option<&'a str>,
    ) -> PromptRequest<'a> {
        PromptRequest {
            instance_id,
            repo_path,
            text: "list files",
            conversation_id,
            source: "cli",
        }
    }

    fn q140_count(conn: &Connection, table: &str) -> i64 {
        conn.query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn enqueue_prompt_for_instance_writes_one_queued_row() {
        let conn = q140_db();
        let registry = q140_registry(&std::env::temp_dir());
        let repo = q140_temp_repo("enqueue");
        let repo_str = repo.to_string_lossy().to_string();

        let row = enqueue_prompt_on(
            &conn,
            &registry,
            &q140_request("test-cli-flow-a-0001", &repo_str, Some("conv-q140-a")),
        )
        .unwrap();

        assert!(row.prompt.id.starts_with("queued-test-cli-flow-a-0001-"));
        assert_eq!(row.prompt.status, prompt_status::QUEUED);
        assert_eq!(row.prompt.instance_id, "test-cli-flow-a-0001");
        assert_eq!(row.instance_name, "Alpha");
        assert_eq!(row.prompt.session_id.as_deref(), Some("conv-q140-a"));
        assert_eq!(row.prompt.source_dir.as_deref(), Some("cli"));
        assert_eq!(row.attempts, 0);
        assert_eq!(
            row.prompt.project_id,
            format!("{}__test-cli-flow-a-0001", repo_slug(&repo_str))
        );
        assert_eq!(q140_count(&conn, "active_prompts"), 1);
        let is_running: i64 = conn
            .query_row(
                "SELECT is_running FROM running_projects WHERE id = ?1",
                params![row.prompt.project_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(is_running, 0);
        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn enqueue_prompt_for_instance_rejects_unknown_instance() {
        let conn = q140_db();
        let registry = q140_registry(&std::env::temp_dir());
        let repo = q140_temp_repo("unknown");
        let repo_str = repo.to_string_lossy().to_string();

        let err = enqueue_prompt_on(&conn, &registry, &q140_request("nope-0000", &repo_str, None))
            .unwrap_err();

        assert!(err.contains("unknown instance id"), "{}", err);
        assert_eq!(q140_count(&conn, "active_prompts"), 0);
        assert_eq!(q140_count(&conn, "running_projects"), 0);
        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn enqueue_two_instances_same_repo_gives_two_rows() {
        let conn = q140_db();
        let registry = q140_registry(&std::env::temp_dir());
        let repo = q140_temp_repo("shared");
        let repo_str = repo.to_string_lossy().to_string();

        let a = enqueue_prompt_on(
            &conn,
            &registry,
            &q140_request("test-cli-flow-a-0001", &repo_str, Some("conv-q140-s")),
        )
        .unwrap();
        let b = enqueue_prompt_on(
            &conn,
            &registry,
            &q140_request("test-cli-flow-b-0001", &repo_str, Some("conv-q140-s")),
        )
        .unwrap();

        assert_ne!(a.prompt.id, b.prompt.id);
        assert_eq!(a.prompt.instance_id, "test-cli-flow-a-0001");
        assert_eq!(b.prompt.instance_id, "test-cli-flow-b-0001");
        assert!(a.prompt.project_id.ends_with("__test-cli-flow-a-0001"));
        assert!(b.prompt.project_id.ends_with("__test-cli-flow-b-0001"));
        assert_eq!(q140_count(&conn, "active_prompts"), 2);
        let _ = fs::remove_dir_all(&repo);
    }

    #[test]
    fn send_prompt_now_marks_failed_on_missing_repo() {
        let conn = q140_db();
        let registry = q140_registry(&std::env::temp_dir());
        let missing = std::env::temp_dir().join(format!("agm-q140-missing-{}", Uuid::new_v4().simple()));
        let missing_str = missing.to_string_lossy().to_string();

        let row = send_prompt_now_with(
            &conn,
            &registry,
            &q140_request("test-cli-flow-a-0001", &missing_str, Some("conv-q140-f")),
            |_p: &ActivePrompt| SpawnOutcome::Spawned,
        )
        .unwrap();

        assert!(row.prompt.id.starts_with("cli-test-cli-flow-a-0001-"));
        assert_eq!(row.prompt.status, prompt_status::FAILED);
        assert_eq!(
            row.status_reason.as_deref(),
            Some(prompt_status_reason::REPO_MISSING)
        );
        assert_eq!(row.attempts, 1);
        assert!(!missing.exists());
    }

    #[test]
    fn cli_dispatch_session_id_is_never_repo_slug() {
        let conn = q140_db();
        let registry = q140_registry(&std::env::temp_dir());
        let missing = std::env::temp_dir().join(format!("agm-q140-slug-{}", Uuid::new_v4().simple()));
        let missing_str = missing.to_string_lossy().to_string();

        let row = send_prompt_now_with(
            &conn,
            &registry,
            &q140_request("test-cli-flow-a-0001", &missing_str, None),
            |_p: &ActivePrompt| SpawnOutcome::Spawned,
        )
        .unwrap();

        assert_eq!(row.prompt.session_id, None);
        assert_ne!(
            row.prompt.session_id.as_deref(),
            Some(repo_slug(&missing_str).as_str())
        );
        assert_ne!(
            row.prompt.session_id.as_deref(),
            Some(row.prompt.project_id.as_str())
        );
    }

    #[test]
    fn list_backed_up_prompts_scope_one_filters_instance() {
        let conn = q140_db();
        for (id, inst) in [
            ("bk-q140-a", "test-cli-flow-a-0001"),
            ("bk-q140-b", "test-cli-flow-b-0001"),
        ] {
            conn.execute(
                "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at)
                 VALUES (?1, 'proj', ?2, 'D:/work/app', 'keep going', NULL, ?1, 'backed_up', 10, 10)",
                params![id, inst],
            )
            .unwrap();
        }

        let one = list_backed_up_prompts_on(
            &conn,
            &crate::modules::instance::InstanceScope::One("test-cli-flow-a-0001".to_string()),
        )
        .unwrap();
        let all =
            list_backed_up_prompts_on(&conn, &crate::modules::instance::InstanceScope::All).unwrap();

        assert_eq!(one.len(), 1);
        assert_eq!(one[0].id, "bk-q140-a");
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn trace_includes_identity_chain_keys() {
        let conn = q140_db();
        let registry = q140_registry(&std::env::temp_dir());
        let repo = q140_temp_repo("trace");
        let repo_str = repo.to_string_lossy().to_string();
        enqueue_prompt_on(
            &conn,
            &registry,
            &q140_request("test-cli-flow-a-0001", &repo_str, Some("conv-q140-t")),
        )
        .unwrap();

        let traces = trace_prompts_on(
            &conn,
            None,
            &registry,
            "test-cli-flow-a-0001",
            Some("conv-q140-t"),
        )
        .unwrap();

        assert_eq!(traces.len(), 1);
        let value = serde_json::to_value(&traces[0]).unwrap();
        for key in [
            "source_dir",
            "conversation_id",
            "repo_path",
            "row_id",
            "status",
            "status_reason",
            "status_history",
            "backup_row_id",
            "handoff",
            "dispatch_result",
            "instance_id",
            "instance_name",
        ] {
            assert!(value.get(key).is_some(), "missing key {}", key);
        }
        assert_eq!(value["handoff"]["legacy"], "not_applicable");
        assert!(value["handoff"]["path"]
            .as_str()
            .unwrap()
            .ends_with(RESUME_TASK_FILE_NAME));
        assert_eq!(value["instance_name"], "Alpha");
        let _ = fs::remove_dir_all(&repo);
    }
```

Why `send_prompt_now_with` and not `send_prompt_now_on` in tests: the injected spawner guarantees no `agy` process can start. The missing repo is caught before the spawner runs.

## 7. Gate

Run from the repo root. All commands must exit 0.

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 enqueue_prompt_for_instance_; cargo test --lib -- --test-threads=1 enqueue_two_instances_same_repo_gives_two_rows; cargo test --lib -- --test-threads=1 send_prompt_now_marks_failed_on_missing_repo; cargo test --lib -- --test-threads=1 cli_dispatch_session_id_is_never_repo_slug; cargo test --lib -- --test-threads=1 list_backed_up_prompts_scope_one_filters_instance; cargo test --lib -- --test-threads=1 trace_includes_identity_chain_keys; cd ..
```

Each `cargo test` line must report at least one test run (`running 1 test` or more). `running 0 tests` means the test was not pasted: fix it.

## 8. Commit

Stage only the files you changed from section 2 (explicit paths). Message:

```text
Feature: prompts - shared per-instance enqueue, send, list and trace
```

## 9. Done when

- [ ] `instance_name_in`, `instance_seq_in`, `instance_short_label_in` exist once in `src-tauri/src/modules/instance.rs`.
- [ ] `list_running_projects` and `list_backed_up_prompts` take `&InstanceScope`; `_on` cores exist.
- [ ] No call `list_running_projects()` or `list_backed_up_prompts()` with empty parentheses is left in `src-tauri/src`, in `src-tauri/src/bin/agm.rs`, or in `src-tauri/tests`.
- [ ] Callers searched by full path in `src-tauri/src`, `src-tauri/src/bin/agm.rs`, each `src-tauri/tests/*.rs` file, and `src` (`.ts` and `.tsx`, for the IPC). All 27 rows of the Change F table are edited exactly as written. `backup_prompts_for_instance` calls `backup_active_running_prompts_for_instance(&inst, custom_file)?`, not `Some(&inst)`.
- [ ] `InstancePromptRow`, `PromptRequest`, `PromptTrace`, `enqueue_prompt_on`, `send_prompt_now_with`, `send_prompt_now_on`, `list_queued_prompts_on`, `trace_prompts_on` and the four `_for_instance` wrappers exist in `repo_db.rs`.
- [ ] No core function calls `load_registry`, `connect_db`, `connect_backup_db` or `invalidate_prompt_tree_cache`; only the `_for_instance` wrappers do.
- [ ] `BackupReport` and `backup_prompts_for_instance` exist in `backup_prompts_db.rs`.
- [ ] The seven tests pass; the gate exits 0.
- [ ] Only files from section 2 are staged.
