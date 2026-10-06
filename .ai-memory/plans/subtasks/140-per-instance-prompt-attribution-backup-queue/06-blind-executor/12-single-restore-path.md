# Step 12: Single Restore Path

Goal: after an instance is relaunched, its backed-up prompts are restored and sent by exactly one function, for exactly that instance, and each row is sent at most once. Fixes B29 and B30. Spec section 6.6.

## 1. Depends on

- Step 10 (`prompt_status`, `claim_prompt_for_dispatch`, `finish_prompt_dispatch`, `dispatch_one_prompt`).
- Step 03 (`source_dir`, `status_reason`, `attempts` columns; `ActivePrompt.source_dir`).
- Step 08 (`normalize_path_for_compare` is `pub(crate)`).
- Step 01 (`canonical_instance_id`).

## 2. Files you may edit

- `src-tauri/src/modules/repo_db.rs`
- `src-tauri/src/modules/backup_prompts_db.rs`
- `src-tauri/src/modules/instance.rs`

## 3. Find it

| Change | File | `fn` signature | Unique search literal | Line hint |
|---|---|---|---|---|
| A | `repo_db.rs` | `fn init_tables(conn: &Connection) -> Result<(), String>` | `fn init_tables(conn: &Connection)` | `:163` |
| B | `repo_db.rs` | `pub fn dispatch_running_prompts(instance_id: &str) -> Result<usize, String>` | `Directly dispatch/send backed-up prompts` | `:1490` |
| C | `backup_prompts_db.rs` | `pub fn restore_running_prompts_for_instance(` | `Automatically trigger resend and execute restored prompts` | `:613` |
| D | `backup_prompts_db.rs` | `fn prompt_status_after_restore(saved: &str) -> &'static str` | `fn prompt_status_after_restore(` | `:622` |
| E | `instance.rs` | `pub fn restore_and_inject_prompts_for_instance(` | `Restore and re-inject prompts for a freshly restarted instance` | `:2804` |
| F | `instance.rs` | inside `launch_instance_inner` (macOS branch) | `let _ = record_instance_pid(instance_id, actual_pid, &data_dir);` | `:3562` |
| G | `instance.rs` | inside `launch_instance_inner` (non-macOS branch) | `let _ = record_instance_pid(instance_id, child.id(), &data_dir);` | `:3667` |
| H | `instance.rs` | inside `pub async fn switch_account_to_instance(` | `5s post-launch delay elapsed. Restoring prompts for instance` | `:4910` |

Search commands:

```text
gitmap aum search "fn init_tables(conn: &Connection)" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "Directly dispatch/send backed-up prompts" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "Automatically trigger resend and execute restored prompts" src-tauri/src/modules/backup_prompts_db.rs --ext .rs
gitmap aum search "fn prompt_status_after_restore" src-tauri/src/modules/backup_prompts_db.rs --ext .rs
gitmap aum search "Restore and re-inject prompts for a freshly restarted instance" src-tauri/src/modules/instance.rs --ext .rs
gitmap aum search "actual_pid, &data_dir" src-tauri/src/modules/instance.rs --ext .rs
gitmap aum search "child.id(), &data_dir" src-tauri/src/modules/instance.rs --ext .rs
gitmap aum search "post-launch delay elapsed. Restoring prompts for instance" src-tauri/src/modules/instance.rs --ext .rs
```

Before adding any new name below, search for it. If it already exists with the same signature (an earlier step added it), do not add it twice:

```text
gitmap aum search "DispatchTally|dispatch_backed_up_prompts_in|dispatch_prompt_for_restore|is_registered_instance_alive|select_instance_prompts_by_status" src-tauri/src --ext .rs
gitmap aum search "RestoreReport|restore_backed_up_prompts_for_instance|restore_backed_up_prompts_core|move_instance_backups_into_active_prompts|has_pending_restore" src-tauri/src --ext .rs
```

## 4. Current code

### Change A (`repo_db.rs:163`)

```rust
/// Initialize SQLite schema for running projects and active prompts
fn init_tables(conn: &Connection) -> Result<(), String> {
```

If the line already reads `pub(crate) fn init_tables`, Change A is already done; skip it.

### Change B (`repo_db.rs:1490`, function is 158 lines)

The whole function is replaced. Expected drift: step 08 replaced the `resolve_instance_id` blocks inside this function and step 10 may have changed the send loop. That drift is fine because every line of the function is deleted. Verify only the first two lines and the last lines below, then delete everything from the doc comment line through the closing `}` that sits directly above `/// Robust multi-format timestamp parsing`.

First lines today:

```rust
/// Directly dispatch/send backed-up prompts to the running projects without queuing
pub fn dispatch_running_prompts(instance_id: &str) -> Result<usize, String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();
```

Last lines today:

```rust
    crate::modules::logger::log_info(&format!(
        "[RepoDB] Successfully dispatched {} prompts to running projects for instance '{}'",
        dispatched_count, instance_id
    ));

    invalidate_prompt_tree_cache(Some(&target_inst));
    Ok(dispatched_count)
}

/// Robust multi-format timestamp parsing handling RFC3339/ISO8601 with fractional seconds,
```

### Change C (`backup_prompts_db.rs:613`, inside `restore_running_prompts_for_instance`)

```rust
    // Automatically trigger resend and execute restored prompts via CLI scoped to this instance
    let _ = repo_db::resend_running_commands_for_instance(Some(target_inst), 20);
    let _ = repo_db::ensure_prompt_goals_running_for_instance(target_inst);

    Ok(records)
}
```

Step 07 may have changed other lines of `restore_running_prompts_for_instance` (its SELECT and UPDATE). Leave them; only the two lines named in the New code are removed.

### Change D (`backup_prompts_db.rs:620`)

```rust
/// Queued stays queued. A running, dispatched, or already backed-up prompt is
/// marked backed_up so the dispatcher sends it again.
fn prompt_status_after_restore(saved: &str) -> &'static str {
    if saved == "queued" {
        "queued"
    } else {
        "backed_up"
    }
}
```

This function stays unchanged. The new code is inserted directly after its closing `}`.

### Change E (`instance.rs:2804`)

```rust
/// Restore and re-inject prompts for a freshly restarted instance
pub fn restore_and_inject_prompts_for_instance(
    instance_id: &str,
    _workspace_roots: &[String],
) -> Result<usize, String> {
    let _ =
        crate::modules::backup_prompts_db::restore_running_prompts(Some(instance_id), false, None);
    let resent =
        crate::modules::repo_db::resend_running_commands_for_instance(Some(instance_id), 20)
            .unwrap_or_default();
    let dispatched = crate::modules::repo_db::dispatch_running_prompts(instance_id).unwrap_or(0);
    let _ = crate::modules::repo_db::ensure_prompt_goals_running_for_instance(instance_id);
    Ok(resent.len() + dispatched)
}
```

### Change F (`instance.rs:3562`, macOS branch of `launch_instance_inner`)

```rust
        let _ = record_instance_pid(instance_id, actual_pid, &data_dir);
        if reinject_prompts {
            wait_for_instance_prompt_channel(instance_id);
            let _ = crate::modules::backup_prompts_db::restore_running_prompts(
                Some(instance_id),
                false,
                None,
            );
            let _ = crate::modules::repo_db::resend_running_commands_for_instance(
                Some(instance_id),
                20,
            );
            let _ = crate::modules::repo_db::dispatch_running_prompts(instance_id);
            let _ = crate::modules::repo_db::ensure_prompt_goals_running_for_instance(instance_id);
        }
        crate::modules::repo_db::invalidate_prompt_tree_cache(Some(instance_id));
        return Ok(());
```

### Change G (`instance.rs:3667`, non-macOS branch of `launch_instance_inner`)

```rust
        let _ = record_instance_pid(instance_id, child.id(), &data_dir);
        if reinject_prompts {
            wait_for_instance_prompt_channel(instance_id);
            let _ = crate::modules::backup_prompts_db::restore_running_prompts(
                Some(instance_id),
                false,
                None,
            );
            let _ = crate::modules::repo_db::resend_running_commands_for_instance(
                Some(instance_id),
                20,
            );
            let _ = crate::modules::repo_db::dispatch_running_prompts(instance_id);
            let _ = crate::modules::repo_db::ensure_prompt_goals_running_for_instance(instance_id);
        }
        crate::modules::repo_db::invalidate_prompt_tree_cache(Some(instance_id));
        Ok(())
```

### Change H (`instance.rs:4910`, inside `switch_account_to_instance`)

```rust
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        crate::modules::logger::log_info(&format!(
            "[PromptRestore] 5s post-launch delay elapsed. Restoring prompts for instance {}",
            target_inst_id
        ));
        let workspace_roots =
            crate::modules::instance::get_instance_workspace_paths(&target_inst_id);
        let _ = crate::modules::instance::restore_and_inject_prompts_for_instance(
            &target_inst_id,
            &workspace_roots,
        );
        let _ = crate::modules::backup_prompts_db::restore_running_prompts_for_instance(
            Some(&target_inst_id),
            false,
            None,
        );
        let _ = crate::modules::repo_db::dispatch_running_prompts(&target_inst_id);
    });
```

Do not touch the line `let target_inst_data_dir = instance.data_dir.clone();` above this block, even though it is unused today.

## 5. New code

### Change A

```rust
/// Initialize SQLite schema for running projects and active prompts
pub(crate) fn init_tables(conn: &Connection) -> Result<(), String> {
```

### Change B (replaces the whole old `dispatch_running_prompts`)

Paste this where the old function was (directly above `/// Robust multi-format timestamp parsing`). It uses `prompt_status`, `claim_prompt_for_dispatch` and `dispatch_one_prompt` from step 10, `normalize_path_for_compare` from step 08, and `canonical_instance_id` from step 01. `HashSet`, `Connection`, `params` and `Utc` are already imported at the top of `repo_db.rs`.

```rust
/// Directly dispatch backed-up prompts of exactly one instance (canonical id) without queuing
pub fn dispatch_running_prompts(instance_id: &str) -> Result<usize, String> {
    let inst = crate::modules::instance::canonical_instance_id(instance_id)?;
    let conn = connect_db()?;
    let tally = dispatch_backed_up_prompts_in(&conn, &inst, dispatch_prompt_for_restore);
    crate::modules::logger::log_info(&format!(
        "[RepoDB] Dispatched {} backed-up prompts for instance '{}' ({} failed, {} left for later)",
        tally.dispatched, inst, tally.failed, tally.skipped
    ));
    invalidate_prompt_tree_cache(Some(&inst));
    Ok(tally.dispatched)
}

/// Counts from one pass over an instance's backed-up prompts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DispatchTally {
    pub dispatched: usize,
    pub failed: usize,
    pub skipped: usize,
}

/// Sends one claimed prompt through the shared dispatch step.
pub fn dispatch_prompt_for_restore(conn: &Connection, prompt: &ActivePrompt) {
    let _ = dispatch_one_prompt(conn, prompt);
}

/// Rows of one instance whose status is in `statuses`, oldest first.
pub(crate) fn select_instance_prompts_by_status(
    conn: &Connection,
    instance_id: &str,
    statuses: &[&str],
) -> Vec<ActivePrompt> {
    if statuses.is_empty() {
        return Vec::new();
    }
    let placeholders = (0..statuses.len())
        .map(|idx| format!("?{}", idx + 2))
        .collect::<Vec<String>>()
        .join(", ");
    let sql = format!(
        "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload, source_dir
         FROM active_prompts
         WHERE instance_id = ?1 AND status IN ({})
         ORDER BY created_at ASC, id ASC",
        placeholders
    );
    let mut values: Vec<&str> = Vec::with_capacity(statuses.len() + 1);
    values.push(instance_id);
    values.extend_from_slice(statuses);
    let Ok(mut stmt) = conn.prepare(&sql) else {
        return Vec::new();
    };
    let mut prompts = Vec::new();
    if let Ok(mapped) = stmt.query_map(rusqlite::params_from_iter(values), |row| {
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
    }) {
        prompts.extend(mapped.flatten());
    }
    prompts
}

/// Claims and sends each backed_up row of one instance at most once. Only the first row of a
/// repository is sent in this pass; later rows of the same repository become queued so the
/// scheduler sends them after the first one finishes.
pub fn dispatch_backed_up_prompts_in<F>(
    conn: &Connection,
    instance_id: &str,
    mut dispatch: F,
) -> DispatchTally
where
    F: FnMut(&Connection, &ActivePrompt),
{
    let mut tally = DispatchTally::default();
    let mut busy_repos: HashSet<String> = HashSet::new();
    let rows = select_instance_prompts_by_status(conn, instance_id, &[prompt_status::BACKED_UP]);
    for prompt in rows {
        let repo_key = normalize_path_for_compare(&prompt.repo_path);
        if !busy_repos.insert(repo_key) {
            let now = Utc::now().timestamp();
            let moved = conn
                .execute(
                    "UPDATE active_prompts SET status = ?1, updated_at = ?2 WHERE id = ?3 AND status = ?4",
                    params![prompt_status::QUEUED, now, &prompt.id, prompt_status::BACKED_UP],
                )
                .unwrap_or(0);
            if moved > 0 {
                if let Ok(mut map) = get_memory_prompts_map().lock() {
                    if let Some(p) = map.get_mut(&prompt.id) {
                        p.status = prompt_status::QUEUED.to_string();
                        p.updated_at = now;
                    }
                }
            }
            tally.skipped += 1;
            continue;
        }
        if !claim_prompt_for_dispatch(conn, &prompt.id, prompt_status::BACKED_UP) {
            tally.skipped += 1;
            continue;
        }
        dispatch(conn, &prompt);
        let status: String = conn
            .query_row(
                "SELECT status FROM active_prompts WHERE id = ?1",
                params![&prompt.id],
                |r| r.get(0),
            )
            .unwrap_or_default();
        if status == prompt_status::DISPATCHED {
            tally.dispatched += 1;
        } else if status == prompt_status::FAILED {
            tally.failed += 1;
        } else {
            tally.skipped += 1;
        }
    }
    tally
}

/// True when the registry entry for this instance has a live IDE process.
pub fn is_registered_instance_alive(instance_id: &str) -> bool {
    let Ok(inst) = crate::modules::instance::canonical_instance_id(instance_id) else {
        return false;
    };
    let Ok(registry) = crate::modules::instance::load_registry() else {
        return false;
    };
    registry
        .instances
        .iter()
        .find(|i| i.id == inst)
        .map(|i| crate::modules::instance::is_instance_running(&i.id, &i.data_dir, i.pid))
        .unwrap_or(false)
}
```

Notes for the compiler:

- If step 10 named the constants differently (for example `STATUS_BACKED_UP`), use step 10's names. If `prompt_status::BACKED_UP` is not a `&str`, STOP.
- If step 10 made `dispatch_one_prompt` `pub`, `dispatch_prompt_for_restore` is still correct; keep it.
- Do not edit `is_instance_running` (DR-4). This step only calls it.

### Change C (inside `restore_running_prompts_for_instance`)

Delete exactly these two lines and nothing else:

```rust
    // Automatically trigger resend and execute restored prompts via CLI scoped to this instance
    let _ = repo_db::resend_running_commands_for_instance(Some(target_inst), 20);
```

The function then ends like this:

```rust
    let _ = repo_db::ensure_prompt_goals_running_for_instance(target_inst);

    Ok(records)
}
```

### Change D (insert directly after the closing `}` of `prompt_status_after_restore`)

```rust
/// Outcome of restoring one instance's backed-up prompts after its relaunch.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RestoreReport {
    pub instance_id: String,
    pub restored_from_backup: usize,
    pub dispatched: usize,
    pub failed: usize,
    pub skipped: usize,
    pub skipped_reason: Option<String>,
}

/// Restore backed-up prompts for one instance after its relaunch. Moves this instance's
/// unrestored backups into active_prompts, then claims and dispatches each backed_up row of
/// this instance exactly once.
pub fn restore_backed_up_prompts_for_instance(
    instance_id: &str,
    keep_backup: bool,
    custom_file: Option<&str>,
) -> Result<RestoreReport, String> {
    let inst = crate::modules::instance::canonical_instance_id(instance_id)?;
    let backup_conn = connect_backup_db(custom_file)?;
    let repo_conn = repo_db::connect_db()?;
    let report = if has_pending_restore(&backup_conn, &repo_conn, &inst) {
        crate::modules::instance::wait_for_instance_prompt_channel(&inst);
        let is_running = repo_db::is_registered_instance_alive(&inst);
        restore_backed_up_prompts_core(
            &backup_conn,
            &repo_conn,
            &inst,
            keep_backup,
            is_running,
            Utc::now().timestamp(),
            repo_db::dispatch_prompt_for_restore,
        )?
    } else {
        RestoreReport {
            instance_id: inst.clone(),
            skipped_reason: Some("nothing_to_restore".to_string()),
            ..Default::default()
        }
    };
    if report.skipped_reason.as_deref() != Some("instance_not_running") {
        let _ = repo_db::ensure_prompt_goals_running_for_instance(&inst);
    }
    repo_db::invalidate_prompt_tree_cache(Some(&inst));
    crate::modules::logger::log_info(&format!(
        "[BackupDB] Restore for '{}': {} from backup, {} dispatched, {} failed, {} skipped{}",
        report.instance_id,
        report.restored_from_backup,
        report.dispatched,
        report.failed,
        report.skipped,
        report
            .skipped_reason
            .as_deref()
            .map(|reason| format!(" ({})", reason))
            .unwrap_or_default()
    ));
    Ok(report)
}

/// True when this instance has an unrestored backup or a backed_up active row.
pub(crate) fn has_pending_restore(
    backup_conn: &Connection,
    repo_conn: &Connection,
    instance_id: &str,
) -> bool {
    let unrestored: i64 = backup_conn
        .query_row(
            "SELECT COUNT(*) FROM prompt_backups WHERE is_restored = 0 AND instance_id = ?1",
            params![instance_id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let backed_up: i64 = repo_conn
        .query_row(
            "SELECT COUNT(*) FROM active_prompts WHERE instance_id = ?1 AND status = ?2",
            params![instance_id, repo_db::prompt_status::BACKED_UP],
            |r| r.get(0),
        )
        .unwrap_or(0);
    unrestored > 0 || backed_up > 0
}

/// Testable core of `restore_backed_up_prompts_for_instance`. `is_running` and `dispatch` are
/// injected so no process is launched or inspected here.
pub(crate) fn restore_backed_up_prompts_core<F>(
    backup_conn: &Connection,
    repo_conn: &Connection,
    instance_id: &str,
    keep_backup: bool,
    is_running: bool,
    now: i64,
    dispatch: F,
) -> Result<RestoreReport, String>
where
    F: FnMut(&Connection, &ActivePrompt),
{
    let mut report = RestoreReport {
        instance_id: instance_id.to_string(),
        ..Default::default()
    };
    if !is_running {
        report.skipped_reason = Some("instance_not_running".to_string());
        return Ok(report);
    }
    report.restored_from_backup = move_instance_backups_into_active_prompts(
        backup_conn,
        repo_conn,
        instance_id,
        keep_backup,
        now,
    )?;
    let tally = repo_db::dispatch_backed_up_prompts_in(repo_conn, instance_id, dispatch);
    report.dispatched = tally.dispatched;
    report.failed = tally.failed;
    report.skipped = tally.skipped;
    Ok(report)
}

/// Copies this instance's unrestored backups into active_prompts (backed_up, or queued when the
/// saved status was queued) and marks them restored unless `keep_backup`. Sends nothing.
pub(crate) fn move_instance_backups_into_active_prompts(
    backup_conn: &Connection,
    repo_conn: &Connection,
    instance_id: &str,
    keep_backup: bool,
    now: i64,
) -> Result<usize, String> {
    let mut stmt = backup_conn
        .prepare(
            "SELECT id, backup_batch_id, prompt_id, project_name, project_path, project_id,
                    conversation_id, conversation_name, sequence_id, prompt_text, has_images,
                    images_payload, status, created_at, is_restored, restored_at, instance_id
             FROM prompt_backups
             WHERE is_restored = 0 AND instance_id = ?1
             ORDER BY created_at ASC, sequence_id ASC",
        )
        .map_err(|e| format!("Failed to prepare restore query: {}", e))?;
    let records: Vec<PromptBackupRecord> = stmt
        .query_map(params![instance_id], parse_prompt_record)
        .map_err(|e| format!("Failed to query unrestored prompts: {}", e))?
        .flatten()
        .collect();

    let mut moved = 0usize;
    for rec in &records {
        let base_project_id = rec
            .project_id
            .split("__")
            .next()
            .unwrap_or(&rec.project_id);
        let session_id = Some(rec.conversation_id.clone())
            .filter(|cid| !cid.trim().is_empty() && cid != "-");
        let prompt = ActivePrompt {
            id: rec.prompt_id.clone(),
            project_id: format!("{}__{}", base_project_id, instance_id),
            instance_id: instance_id.to_string(),
            repo_path: rec.project_path.clone(),
            prompt_content: rec.prompt_text.clone(),
            model: Some("gemini-3.8-flash-high".to_string()),
            session_id,
            status: prompt_status_after_restore(&rec.status).to_string(),
            created_at: now,
            updated_at: now,
            image_payload: rec.images_payload.clone(),
            source_dir: Some("backup_restore".to_string()),
        };
        if let Err(e) = upsert_restored_prompt(repo_conn, &prompt) {
            crate::modules::logger::log_warn(&format!(
                "[BackupDB] Backup {} for '{}' left unrestored: {}",
                rec.id, instance_id, e
            ));
            continue;
        }
        moved += 1;
        if !keep_backup {
            backup_conn
                .execute(
                    "UPDATE prompt_backups SET is_restored = 1, restored_at = ?1 WHERE id = ?2",
                    params![now, &rec.id],
                )
                .map_err(|e| format!("Failed to mark backup {} restored: {}", rec.id, e))?;
        }
    }

    if !keep_backup && moved > 0 {
        let _ = backup_conn.execute(
            "UPDATE backup_batches SET is_fully_restored = 1 WHERE id IN (
                SELECT backup_batch_id FROM prompt_backups GROUP BY backup_batch_id HAVING min(is_restored) = 1
             )",
            [],
        );
    }
    Ok(moved)
}

fn upsert_restored_prompt(repo_conn: &Connection, prompt: &ActivePrompt) -> Result<(), String> {
    repo_conn
        .execute(
            "INSERT INTO active_prompts
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload, source_dir)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                updated_at = excluded.updated_at,
                prompt_content = excluded.prompt_content,
                image_payload = COALESCE(excluded.image_payload, active_prompts.image_payload),
                source_dir = excluded.source_dir,
                status_reason = NULL
             WHERE active_prompts.instance_id = excluded.instance_id
               AND active_prompts.status <> 'dispatching'",
            params![
                &prompt.id,
                &prompt.project_id,
                &prompt.instance_id,
                &prompt.repo_path,
                &prompt.prompt_content,
                &prompt.model,
                &prompt.session_id,
                &prompt.status,
                prompt.created_at,
                prompt.updated_at,
                &prompt.image_payload,
                &prompt.source_dir,
            ],
        )
        .map_err(|e| format!("Failed to restore prompt {}: {}", prompt.id, e))?;
    Ok(())
}
```

A backup that cannot be upserted is skipped, not fatal. The usual cause is the identity index from step 06, when a running or backed_up row for the same `(instance_id, conversation_id)` already exists. The skipped backup stays `is_restored = 0` and is logged, and the remaining backups are still restored. The function returns the number of rows actually moved, so `restore_move_step_does_not_dispatch` still sees 2.

Notes for the compiler:

- `parse_prompt_record` already exists in this file with signature `fn parse_prompt_record(row: &rusqlite::Row) -> rusqlite::Result<PromptBackupRecord>`. Passing it by name to `query_map` is correct. If the compiler complains about lifetimes there, write `|row| parse_prompt_record(row)` instead (clippy may then suggest the by-name form; follow clippy).
- If `ActivePrompt` has more fields than `source_dir` after step 03, the compiler names them; set each new field to `None` or its default.

### Change E (replaces the whole `restore_and_inject_prompts_for_instance`)

```rust
/// Restore and re-inject prompts for a freshly restarted instance
pub fn restore_and_inject_prompts_for_instance(
    instance_id: &str,
    _workspace_roots: &[String],
) -> Result<usize, String> {
    crate::modules::backup_prompts_db::restore_backed_up_prompts_for_instance(
        instance_id,
        false,
        None,
    )
    .map(|report| report.dispatched)
}
```

### Change F (macOS branch)

```rust
        let _ = record_instance_pid(instance_id, actual_pid, &data_dir);
        if reinject_prompts {
            let _ = crate::modules::backup_prompts_db::restore_backed_up_prompts_for_instance(
                instance_id,
                false,
                None,
            );
        }
        crate::modules::repo_db::invalidate_prompt_tree_cache(Some(instance_id));
        return Ok(());
```

### Change G (non-macOS branch)

```rust
        let _ = record_instance_pid(instance_id, child.id(), &data_dir);
        if reinject_prompts {
            let _ = crate::modules::backup_prompts_db::restore_backed_up_prompts_for_instance(
                instance_id,
                false,
                None,
            );
        }
        crate::modules::repo_db::invalidate_prompt_tree_cache(Some(instance_id));
        Ok(())
```

The explicit `wait_for_instance_prompt_channel(instance_id);` line is gone on purpose in F and G: `restore_backed_up_prompts_for_instance` waits itself when there is something to restore. If the compiler then reports `wait_for_instance_prompt_channel` as unused, it is not: `restore_backed_up_prompts_for_instance` calls it. Keep the function.

### Change H (inside `switch_account_to_instance`)

```rust
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        crate::modules::logger::log_info(&format!(
            "[PromptRestore] 5s post-launch delay elapsed. Restoring prompts for instance {}",
            target_inst_id
        ));
        let _ = crate::modules::backup_prompts_db::restore_backed_up_prompts_for_instance(
            &target_inst_id,
            false,
            None,
        );
    });
```

### Callers that stay for now

These callers still compile after this step and are replaced in step 16. Do not change them here:

- `src-tauri/src/modules/integration.rs` (`restore_and_inject_prompts_for_instance`, `restore_running_prompts_for_instance`, `dispatch_running_prompts`).
- `src-tauri/src/modules/auto_switcher.rs` (`resend_running_commands_for_instance`, `dispatch_running_prompts`).
- `src-tauri/src/modules/telegram_inbound.rs` (`resend_all_running_commands`, `dispatch_running_prompts`, `restore_running_prompts`).

## 6. Tests

Paste these into the existing `#[cfg(test)] mod tests` of `src-tauri/src/modules/backup_prompts_db.rs` (starts near `:729`, after `use super::*;`). The backup DB is opened on a temp file through `connect_backup_db(Some(path))`, the same way the existing `test_backup_db_lifecycle` does; the repo DB is `Connection::open_in_memory()` plus `repo_db::init_tables`. No environment variable is set and no registry is loaded.

```rust
    fn restore_test_dbs(tag: &str) -> (Connection, Connection, PathBuf) {
        let dir = std::env::temp_dir().join(format!("agm_test_{}_{}", tag, Uuid::new_v4().simple()));
        let backup_path = dir.join("backup-prompts.db");
        let backup = connect_backup_db(Some(backup_path.to_str().unwrap())).unwrap();
        let repo = Connection::open_in_memory().unwrap();
        repo_db::init_tables(&repo).unwrap();
        (backup, repo, dir)
    }

    fn seed_prompt_backup(
        backup: &Connection,
        prompt_id: &str,
        instance_id: &str,
        repo_path: &str,
        status: &str,
    ) {
        backup
            .execute(
                "INSERT OR IGNORE INTO backup_batches (id, created_at, prompts_count, file_path, retention_days, is_fully_restored)
                 VALUES ('batch-test', 1, 1, 'test', 1, 0)",
                [],
            )
            .unwrap();
        backup
            .execute(
                "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id, conversation_id, sequence_id, prompt_text, status, created_at, is_restored, instance_id)
                 VALUES (?1, 'batch-test', ?2, 'app', ?3, 'app-1', ?4, 1, 'keep going', ?5, 1, 0, ?6)",
                params![
                    format!("rec-{}", prompt_id),
                    prompt_id,
                    repo_path,
                    format!("cid-{}", prompt_id),
                    status,
                    instance_id
                ],
            )
            .unwrap();
    }

    fn active_status_and_attempts(repo: &Connection, id: &str) -> Option<(String, i64)> {
        repo.query_row(
            "SELECT status, attempts FROM active_prompts WHERE id = ?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .ok()
    }

    fn is_backup_restored(backup: &Connection, prompt_id: &str) -> bool {
        backup
            .query_row(
                "SELECT is_restored FROM prompt_backups WHERE prompt_id = ?1",
                params![prompt_id],
                |r| r.get::<_, bool>(0),
            )
            .unwrap()
    }

    fn mark_test_dispatched(conn: &Connection, id: &str) {
        conn.execute(
            "UPDATE active_prompts SET status = 'dispatched' WHERE id = ?1",
            params![id],
        )
        .unwrap();
    }

    #[test]
    fn restore_for_instance_does_not_touch_other_instance() {
        let (backup, repo, dir) = restore_test_dbs("restore_other");
        seed_prompt_backup(&backup, "prompt-inst-a-1", "inst-a", "/work/app", "running");
        seed_prompt_backup(&backup, "prompt-inst-b-1", "inst-b", "/work/app", "running");
        repo.execute(
            "INSERT INTO active_prompts
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES ('prompt-inst-b-live', 'app__inst-b', 'inst-b', '/work/other', 'other work', NULL, 'cid-b', 'backed_up', 1, 1, NULL)",
            [],
        )
        .unwrap();

        let mut sent: Vec<String> = Vec::new();
        let report = restore_backed_up_prompts_core(&backup, &repo, "inst-a", false, true, 100, |conn, prompt| {
            sent.push(prompt.id.clone());
            mark_test_dispatched(conn, &prompt.id);
        })
        .unwrap();

        assert_eq!(sent, vec!["prompt-inst-a-1".to_string()]);
        assert_eq!(report.restored_from_backup, 1);
        assert_eq!(report.dispatched, 1);
        assert!(is_backup_restored(&backup, "prompt-inst-a-1"));
        assert!(!is_backup_restored(&backup, "prompt-inst-b-1"));
        assert!(active_status_and_attempts(&repo, "prompt-inst-b-1").is_none());
        assert_eq!(
            active_status_and_attempts(&repo, "prompt-inst-b-live"),
            Some(("backed_up".to_string(), 0))
        );
        let restored_instance: String = repo
            .query_row(
                "SELECT instance_id FROM active_prompts WHERE id = 'prompt-inst-a-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(restored_instance, "inst-a");

        drop(backup);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn restore_dispatches_each_row_at_most_once() {
        let (backup, repo, dir) = restore_test_dbs("restore_once");
        seed_prompt_backup(&backup, "prompt-inst-a-1", "inst-a", "D:/Work/App", "running");
        seed_prompt_backup(&backup, "prompt-inst-a-2", "inst-a", "d:\\work\\app\\", "running");
        seed_prompt_backup(&backup, "prompt-inst-a-3", "inst-a", "/work/other", "backed_up");

        let mut sent: Vec<String> = Vec::new();
        for _ in 0..2 {
            let _ = restore_backed_up_prompts_core(&backup, &repo, "inst-a", false, true, 100, |conn, prompt| {
                sent.push(prompt.id.clone());
                mark_test_dispatched(conn, &prompt.id);
            })
            .unwrap();
        }

        assert_eq!(
            sent,
            vec!["prompt-inst-a-1".to_string(), "prompt-inst-a-3".to_string()]
        );
        assert_eq!(
            active_status_and_attempts(&repo, "prompt-inst-a-1"),
            Some(("dispatched".to_string(), 1))
        );
        assert_eq!(
            active_status_and_attempts(&repo, "prompt-inst-a-2"),
            Some(("queued".to_string(), 0))
        );
        assert_eq!(
            active_status_and_attempts(&repo, "prompt-inst-a-3"),
            Some(("dispatched".to_string(), 1))
        );

        drop(backup);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn restore_skips_instance_that_is_not_running() {
        let (backup, repo, dir) = restore_test_dbs("restore_dead");
        seed_prompt_backup(&backup, "prompt-inst-a-1", "inst-a", "/work/app", "running");

        let mut sent: Vec<String> = Vec::new();
        let report = restore_backed_up_prompts_core(&backup, &repo, "inst-a", false, false, 100, |_, prompt| {
            sent.push(prompt.id.clone());
        })
        .unwrap();

        assert!(sent.is_empty());
        assert_eq!(report.skipped_reason.as_deref(), Some("instance_not_running"));
        assert!(!is_backup_restored(&backup, "prompt-inst-a-1"));
        assert!(active_status_and_attempts(&repo, "prompt-inst-a-1").is_none());

        drop(backup);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn restore_move_step_does_not_dispatch() {
        let (backup, repo, dir) = restore_test_dbs("restore_move");
        seed_prompt_backup(&backup, "prompt-inst-a-1", "inst-a", "/work/app", "running");
        seed_prompt_backup(&backup, "prompt-inst-a-2", "inst-a", "/work/other", "queued");

        let moved = move_instance_backups_into_active_prompts(&backup, &repo, "inst-a", false, 100).unwrap();

        assert_eq!(moved, 2);
        assert_eq!(
            active_status_and_attempts(&repo, "prompt-inst-a-1"),
            Some(("backed_up".to_string(), 0))
        );
        assert_eq!(
            active_status_and_attempts(&repo, "prompt-inst-a-2"),
            Some(("queued".to_string(), 0))
        );
        let dispatching: i64 = repo
            .query_row(
                "SELECT COUNT(*) FROM active_prompts WHERE status IN ('dispatching', 'dispatched')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(dispatching, 0);
        let source_dir: Option<String> = repo
            .query_row(
                "SELECT source_dir FROM active_prompts WHERE id = 'prompt-inst-a-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(source_dir.as_deref(), Some("backup_restore"));

        drop(backup);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn restore_pending_check_is_per_instance() {
        let (backup, repo, dir) = restore_test_dbs("restore_pending");
        seed_prompt_backup(&backup, "prompt-inst-b-1", "inst-b", "/work/app", "running");

        assert!(!has_pending_restore(&backup, &repo, "inst-a"));
        assert!(has_pending_restore(&backup, &repo, "inst-b"));

        drop(backup);
        let _ = fs::remove_dir_all(dir);
    }
```

## 7. Gate

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 backup_prompts_db::tests::restore_; cd ..
```

All commands must exit 0. Five tests must run and pass.

## 8. Commit

```text
Fix: prompts - single per-instance restore path for backed-up prompts
```

Stage only:

```text
git add src-tauri/src/modules/repo_db.rs src-tauri/src/modules/backup_prompts_db.rs src-tauri/src/modules/instance.rs
```

## 9. Done when

- [ ] `init_tables` is `pub(crate)`.
- [ ] `dispatch_running_prompts` takes the canonical id, selects only `backed_up` rows of that instance through `dispatch_backed_up_prompts_in`, and contains no `resolve_instance_id`, no `get_dispatched_prompts_cache`, and no direct `UPDATE ... 'dispatched'`.
- [ ] `restore_running_prompts_for_instance` no longer calls `resend_running_commands_for_instance`.
- [ ] `RestoreReport`, `restore_backed_up_prompts_for_instance`, `has_pending_restore`, `restore_backed_up_prompts_core`, `move_instance_backups_into_active_prompts` and `upsert_restored_prompt` exist in `backup_prompts_db.rs`.
- [ ] `restore_and_inject_prompts_for_instance` is a one-call wrapper.
- [ ] Both launch reinject blocks and the `switch_account_to_instance` async block call only `restore_backed_up_prompts_for_instance`.
- [ ] `is_instance_running` is unchanged.
- [ ] `move_instance_backups_into_active_prompts` logs and skips a record whose upsert fails, and returns the moved count, not `records.len()`.
- [ ] Callers searched in `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file. No caller edit is needed because the signatures of `dispatch_running_prompts(&str) -> Result<usize, String>` and `restore_and_inject_prompts_for_instance(&str, &[String]) -> Result<usize, String>` are unchanged, and `init_tables` only widens to `pub(crate)`. The callers are `agm.rs:4170`, `:10928`, `:14613`, `:14789` and `tests/auto_switcher_e2e_test.rs:161`.
- [ ] Gate exits 0; the five `restore_` tests pass.
- [ ] Only the three files above are staged.
