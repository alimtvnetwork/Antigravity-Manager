# Step 20: IPC Wrappers and Registration

Goal: the desktop UI reaches the same shared library functions as the CLI, through thin Tauri commands that take an explicit `instance_id`. No command shells out to `agm`. Long operations run on a blocking thread so the UI does not freeze. Subtask `../03-cli-and-ipc-parity.md` Step 5.

Read `00-start-here.md` first. This file never overrides it.

## 1. Depends on

- Step 17 (`list_running_projects(&InstanceScope)`, `list_backed_up_prompts(&InstanceScope)`, `enqueue_prompt_for_instance`, `send_prompt_now_for_instance`, `list_queued_prompts_for_instance`, `trace_prompts_for_instance`, `InstancePromptRow`, `PromptTrace`, `PromptRequest`, `enqueue_prompt_on`, `backup_prompts_db::backup_prompts_for_instance`, `BackupReport`).
- Step 12 (`backup_prompts_db::restore_backed_up_prompts_for_instance`, `RestoreReport` with `Serialize`; `repo_db::init_tables` is `pub(crate)`).
- Step 01 (`instance::resolve_instance_input`).

## 2. Files you may edit

- `src-tauri/src/commands/instance.rs`
- `src-tauri/src/lib.rs`

`src-tauri/src/commands/mod.rs` already has `pub use instance::*;` (line 24), so the new commands are visible as `commands::<name>` without editing it. No other `commands/*.rs` file defines these names (verified with `gitmap aum search "fn enqueue_prompt|fn send_prompt_now|fn backup_running_prompts_for_instance|fn restore_running_prompts_for_instance|fn list_queued_prompts|fn trace_prompts" src-tauri/src/commands --ext .rs`: 0 hits).

## 3. Find it

| Change | File | Place | Unique search literal | Line hint |
|---|---|---|---|---|
| A | `commands/instance.rs` | `pub fn list_running_projects()` and `pub fn list_backed_up_prompts()` | `pub fn list_running_projects()` | `:206` to `:214` |
| B | `commands/instance.rs` | end of file, after `pub async fn sync_all_instances_and_quotas()` | `pub async fn sync_all_instances_and_quotas()` | `:354` to `:357` |
| C | `lib.rs` | `tauri::generate_handler![` list | `commands::list_backed_up_prompts,` | `:1104` |

```text
gitmap aum search "pub fn list_running_projects|pub fn list_backed_up_prompts" src-tauri/src/commands/instance.rs --ext .rs
gitmap aum search "pub async fn sync_all_instances_and_quotas" src-tauri/src/commands/instance.rs --ext .rs
gitmap aum search "commands::list_backed_up_prompts," src-tauri/src/lib.rs --ext .rs
gitmap aum search "mod tests" src-tauri/src/commands/instance.rs --ext .rs
```

The last search must return 0 hits (the file has no test module yet). If it returns a hit, paste the test into that module instead of creating a new one.

Confirm these exist (each must return a hit):

```text
gitmap aum search "pub fn send_prompt_now_for_instance|pub fn enqueue_prompt_for_instance|pub fn list_queued_prompts_for_instance|pub fn trace_prompts_for_instance|pub fn enqueue_prompt_on|pub struct PromptRequest" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "pub fn backup_prompts_for_instance|pub fn restore_backed_up_prompts_for_instance|pub struct RestoreReport" src-tauri/src/modules/backup_prompts_db.rs --ext .rs
gitmap aum search "pub fn resolve_instance_input\(" src-tauri/src/modules/instance.rs --ext .rs
```

Open `pub struct RestoreReport` and confirm its derive line contains `Serialize`. If not, STOP (a Tauri command cannot return it).

## 4. Current code

### Change A (`commands/instance.rs:206` to `:214`, as left by step 17)

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

(`cargo fmt` may have wrapped the long body lines; that is fine.)

### Change B (`commands/instance.rs`, end of file)

```rust
#[tauri::command]
pub async fn sync_all_instances_and_quotas() -> Result<Vec<InstanceStatus>, String> {
    instance::sync_all_instances_and_quotas_logic().await
}
```

### Change C (`lib.rs:1102` to `:1109`)

```rust
            commands::trigger_manual_profile_rotation,
            commands::list_running_projects,
            commands::list_backed_up_prompts,
            commands::clean_and_restart_workspace,
            commands::resume_recent_project_prompts,
            commands::assign_project_to_instance,
            commands::get_instance_workspace_folders,
            commands::get_project_conversation_tree,
```

## 5. New code

### Change A: replace the two commands with

```rust
fn ipc_instance_scope(
    instance_id: Option<&str>,
) -> Result<crate::modules::instance::InstanceScope, String> {
    match instance_id
        .map(str::trim)
        .filter(|id| !id.is_empty() && !id.eq_ignore_ascii_case("all"))
    {
        Some(raw) => Ok(crate::modules::instance::InstanceScope::One(
            instance::resolve_instance_input(raw)?,
        )),
        None => Ok(crate::modules::instance::InstanceScope::All),
    }
}

#[tauri::command]
pub fn list_running_projects(
    instance_id: Option<String>,
) -> Result<Vec<crate::modules::repo_db::RunningProject>, String> {
    let scope = ipc_instance_scope(instance_id.as_deref())?;
    crate::modules::repo_db::list_running_projects(&scope)
}

#[tauri::command]
pub fn list_backed_up_prompts(
    instance_id: Option<String>,
) -> Result<Vec<crate::modules::repo_db::ActivePrompt>, String> {
    let scope = ipc_instance_scope(instance_id.as_deref())?;
    crate::modules::repo_db::list_backed_up_prompts(&scope)
}
```

### Change B: append at the end of `commands/instance.rs`

```rust
#[tauri::command]
pub fn enqueue_prompt(
    instance_id: String,
    repo_path: String,
    prompt_content: String,
    conversation_id: Option<String>,
) -> Result<crate::modules::repo_db::InstancePromptRow, String> {
    let inst = instance::resolve_instance_input(&instance_id)?;
    crate::modules::repo_db::enqueue_prompt_for_instance(
        &inst,
        &repo_path,
        &prompt_content,
        conversation_id.as_deref(),
        "ui",
    )
}

#[tauri::command]
pub async fn send_prompt_now(
    instance_id: String,
    repo_path: String,
    prompt_content: String,
    conversation_id: Option<String>,
) -> Result<crate::modules::repo_db::InstancePromptRow, String> {
    tokio::task::spawn_blocking(move || {
        let inst = instance::resolve_instance_input(&instance_id)?;
        crate::modules::repo_db::send_prompt_now_for_instance(
            &inst,
            &repo_path,
            &prompt_content,
            conversation_id.as_deref(),
            "ui",
        )
    })
    .await
    .map_err(|e| format!("Spawn blocking failed: {}", e))?
}

#[tauri::command]
pub fn list_queued_prompts(
    instance_id: String,
) -> Result<Vec<crate::modules::repo_db::InstancePromptRow>, String> {
    let inst = instance::resolve_instance_input(&instance_id)?;
    crate::modules::repo_db::list_queued_prompts_for_instance(&inst)
}

#[tauri::command]
pub fn trace_prompts(
    instance_id: String,
    conversation_id: Option<String>,
) -> Result<Vec<crate::modules::repo_db::PromptTrace>, String> {
    let inst = instance::resolve_instance_input(&instance_id)?;
    crate::modules::repo_db::trace_prompts_for_instance(&inst, conversation_id.as_deref())
}

#[tauri::command]
pub async fn backup_running_prompts_for_instance(
    instance_id: String,
) -> Result<crate::modules::backup_prompts_db::BackupReport, String> {
    tokio::task::spawn_blocking(move || {
        let inst = instance::resolve_instance_input(&instance_id)?;
        crate::modules::backup_prompts_db::backup_prompts_for_instance(&inst, None)
    })
    .await
    .map_err(|e| format!("Spawn blocking failed: {}", e))?
}

#[tauri::command]
pub async fn restore_running_prompts_for_instance(
    instance_id: String,
    keep_backup: Option<bool>,
) -> Result<crate::modules::backup_prompts_db::RestoreReport, String> {
    tokio::task::spawn_blocking(move || {
        let inst = instance::resolve_instance_input(&instance_id)?;
        crate::modules::backup_prompts_db::restore_backed_up_prompts_for_instance(
            &inst,
            keep_backup.unwrap_or(false),
            None,
        )
    })
    .await
    .map_err(|e| format!("Spawn blocking failed: {}", e))?
}
```

### Change C: insert six lines directly after `commands::list_backed_up_prompts,`

```rust
            commands::list_running_projects,
            commands::list_backed_up_prompts,
            commands::enqueue_prompt,
            commands::send_prompt_now,
            commands::list_queued_prompts,
            commands::trace_prompts,
            commands::backup_running_prompts_for_instance,
            commands::restore_running_prompts_for_instance,
            commands::clean_and_restart_workspace,
```

Only the six middle lines are new. Do not add any name twice: `gitmap aum search "commands::send_prompt_now" src-tauri/src/lib.rs --ext .rs` must return exactly 1 hit after the edit.

### How the UI calls these (for step 21)

Tauri converts each Rust parameter name from snake_case to camelCase for the JavaScript side. A missing key is accepted only for `Option<...>` parameters. Extra keys are ignored.

| Command | `invoke()` call | Returns |
|---|---|---|
| `list_running_projects` | `invoke('list_running_projects', { instanceId })` (or no object for all instances) | `RunningProject[]` |
| `list_backed_up_prompts` | `invoke('list_backed_up_prompts', { instanceId })` (or no object for all instances) | `ActivePrompt[]` |
| `enqueue_prompt` | `invoke('enqueue_prompt', { instanceId, repoPath, promptContent, conversationId })` | row object |
| `send_prompt_now` | `invoke('send_prompt_now', { instanceId, repoPath, promptContent, conversationId })` | row object |
| `list_queued_prompts` | `invoke('list_queued_prompts', { instanceId })` | row object array |
| `trace_prompts` | `invoke('trace_prompts', { instanceId, conversationId })` | trace array |
| `backup_running_prompts_for_instance` | `invoke('backup_running_prompts_for_instance', { instanceId })` | `{ instance_id, batch_id, file_path, captured_from_workspace, prompts_count, created_at, records }` |
| `restore_running_prompts_for_instance` | `invoke('restore_running_prompts_for_instance', { instanceId, keepBackup })` | `{ instance_id, restored_from_backup, dispatched, failed, skipped, skipped_reason }` |

A "row object" is `InstancePromptRow` serialized flat: the `ActivePrompt` fields (`id`, `project_id`, `instance_id`, `repo_path`, `prompt_content`, `model`, `session_id`, `status`, `created_at`, `updated_at`, `image_payload`, `source_dir`) plus `instance_name`, `status_reason`, `attempts`. Response keys stay snake_case.

`instanceId` accepts a canonical id, `#seq`, a seq number, or a display name (`resolve_instance_input`). An empty `instanceId` is an error (`instance required`) for every command except the two list commands, where empty or missing means all instances.

## 6. Tests

Append this test module at the very end of `src-tauri/src/commands/instance.rs`. It proves the UI path (`source = "ui"`) and the CLI path (`source = "cli"`) write rows of the same shape. It uses an in-memory DB, an in-memory registry and a temp folder it deletes. No `load_registry()`, no environment variable changes, no Tauri runtime.

```rust
#[cfg(test)]
mod tests {
    use crate::modules::instance::{InstanceConfig, InstanceRegistry};
    use crate::modules::repo_db::{enqueue_prompt_on, init_tables, PromptRequest};
    use rusqlite::Connection;

    #[test]
    fn ipc_enqueue_matches_cli_enqueue_row_shape() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let registry = InstanceRegistry {
            active_instance_id: "default".to_string(),
            instances: vec![InstanceConfig {
                id: "test-cli-flow-a-0001".to_string(),
                name: "Alpha".to_string(),
                data_dir: std::env::temp_dir()
                    .join("test-cli-flow-a-0001")
                    .to_string_lossy()
                    .to_string(),
                executable_path: None,
                extensions_dir: None,
                bound_account_id: None,
                bound_email: None,
                created_at: 0,
                last_used: 0,
                is_default: false,
                pid: None,
                seq_num: Some(2),
            }],
        };
        let repo = std::env::temp_dir().join(format!(
            "agm-ipc-shape-{}",
            uuid::Uuid::new_v4().simple()
        ));
        std::fs::create_dir_all(&repo).unwrap();
        let repo_str = repo.to_string_lossy().to_string();

        let ui = enqueue_prompt_on(
            &conn,
            &registry,
            &PromptRequest {
                instance_id: "test-cli-flow-a-0001",
                repo_path: &repo_str,
                text: "list files",
                conversation_id: Some("conv-ipc"),
                source: "ui",
            },
        )
        .unwrap();
        let cli = enqueue_prompt_on(
            &conn,
            &registry,
            &PromptRequest {
                instance_id: "test-cli-flow-a-0001",
                repo_path: &repo_str,
                text: "list files",
                conversation_id: Some("conv-ipc"),
                source: "cli",
            },
        )
        .unwrap();

        assert_ne!(ui.prompt.id, cli.prompt.id);
        assert!(ui.prompt.id.starts_with("queued-test-cli-flow-a-0001-"));
        assert!(cli.prompt.id.starts_with("queued-test-cli-flow-a-0001-"));
        assert_eq!(ui.prompt.status, cli.prompt.status);
        assert_eq!(ui.prompt.instance_id, cli.prompt.instance_id);
        assert_eq!(ui.prompt.project_id, cli.prompt.project_id);
        assert_eq!(ui.prompt.repo_path, cli.prompt.repo_path);
        assert_eq!(ui.prompt.session_id, cli.prompt.session_id);
        assert_eq!(ui.prompt.model, cli.prompt.model);
        assert_eq!(ui.status_reason, cli.status_reason);
        assert_eq!(ui.attempts, cli.attempts);
        assert_eq!(ui.instance_name, cli.instance_name);
        assert_eq!(ui.prompt.source_dir.as_deref(), Some("ui"));
        assert_eq!(cli.prompt.source_dir.as_deref(), Some("cli"));

        let ui_keys: Vec<String> = serde_json::to_value(&ui)
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        let cli_keys: Vec<String> = serde_json::to_value(&cli)
            .unwrap()
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        assert_eq!(ui_keys, cli_keys);

        let _ = std::fs::remove_dir_all(&repo);
    }
}
```

## 7. Gate

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 ipc_enqueue_matches_cli_enqueue_row_shape; cd ..
```

`npm run build` is not needed here: no file under `src/` changes. The existing frontend calls `invoke('list_running_projects')` and `invoke('list_backed_up_prompts')` with no arguments (`src/services/instanceService.ts:443`, `:447`); they keep working because the new parameter is `Option<String>`.

## 8. Commit

```text
Feature: prompts - IPC commands for per-instance enqueue and send
```

## 9. Done when

- [ ] `list_running_projects` and `list_backed_up_prompts` take `instance_id: Option<String>`.
- [ ] `enqueue_prompt`, `send_prompt_now`, `list_queued_prompts`, `trace_prompts`, `backup_running_prompts_for_instance`, `restore_running_prompts_for_instance` exist once each in `commands/instance.rs`.
- [ ] `send_prompt_now`, `backup_running_prompts_for_instance` and `restore_running_prompts_for_instance` are `async` and use `tokio::task::spawn_blocking`.
- [ ] No new command calls `std::process::Command`, `agm`, `get_active_instance_id` or `resolve_instance_id`.
- [ ] The six names are registered once each in `src-tauri/src/lib.rs`.
- [ ] Callers searched in `src-tauri/src`, `src-tauri/src/bin/agm.rs`, each `src-tauri/tests/*.rs` file, and `src` (`.ts` and `.tsx`, for the IPC). No Rust code calls the `list_running_projects` or `list_backed_up_prompts` command functions directly, and `agm.rs` and the tests have 0 `commands::` hits. The only references are the `generate_handler!` entries in `lib.rs`. The frontend `invoke('list_running_projects')` and `invoke('list_backed_up_prompts')` at `src/services/instanceService.ts:443`, `:447` and `src/components/instances/PromptTreeViewModal.tsx:1104` pass no arguments, which still deserializes to `instance_id: None` (all instances), so they need no edit here.
- [ ] `ipc_enqueue_matches_cli_enqueue_row_shape` passes; the gate exits 0.
- [ ] Only `src-tauri/src/commands/instance.rs` and `src-tauri/src/lib.rs` are staged.
