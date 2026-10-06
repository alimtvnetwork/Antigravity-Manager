# Step 16: Switch Flows Back Up and Restore the Source Instance Only

Goal: every switch flow backs up the instance it is leaving (canonical id, never `None`), and restores exactly that instance, only after that instance was relaunched. Restore goes through step 12's single path `restore_backed_up_prompts_for_instance`. Fixes B27, B28, B29 and the Telegram part of B30. Spec sections 6.9 and 6.10.

Flows covered:

- Auto switcher (`execute_profile_rotation_with_context`): canonical source id, backup of the source, audit-only requeue, restore of the source only when the source itself was relaunched (same-instance rotation with `auto_reopen_on_switch`). A cross-instance rotation closes the source and never relaunches it, so nothing is restored there; the source's rows stay `backed_up` until the source is launched again (step 12's launch reinject restores them then). The target's own prompts are restored inside `switch_account_to_instance` (step 12, change H).
- Desktop switch (`integration.rs`, `on_account_switch`): the three restore calls become one call for `"default"`.
- Telegram: `/restore` and `/backup` accept `-i <instance>` / `--instance <instance>` / `--instance=<instance>` (and `/restore <instance>`); `/ff`, `/rotate` and `ff:` no longer back up or resend `"default"` around the rotation, because the rotation itself backs up and restores each rotated instance.

`src-tauri/src/modules/account.rs:1743` keeps `"default"`. Do not touch it.

## 1. Depends on

- Step 01 (`canonical_instance_id`, `resolve_instance_input`).
- Step 12 (`restore_backed_up_prompts_for_instance`, `RestoreReport`, `select_instance_prompts_by_status`).
- Step 13 (bridge in `auto_switcher.rs` post-switch block).
- Step 15 (`backup_active_running_prompts_for_instance(&str, Option<&str>)`; `integration.rs:342` passes `"default"`).
- Step 10 (`repo_db::prompt_status`).

## 2. Files you may edit

- `src-tauri/src/modules/auto_switcher.rs`
- `src-tauri/src/modules/integration.rs`
- `src-tauri/src/modules/telegram_inbound.rs`
- `src-tauri/src/modules/repo_db.rs` (one new function)

## 3. Find it

| Change | File | Function | Unique search literal | Line hint |
|---|---|---|---|---|
| A | `auto_switcher.rs` | `pub async fn execute_profile_rotation_with_context(` (source id and backup) | `// Step 0: Ensure all running and queued prompts are snapshotted and backed up before profile switch` | `:1421` to `:1436` |
| B | `auto_switcher.rs` | same function, cross-instance branch | `// a) Save in-flight prompts` | `:1594` to `:1597` |
| C | `auto_switcher.rs` | same function, post-switch block | `// Step 3: Asynchronous 5-second post-launch prompt re-injection and status notification` | `:1684` to `:1729` |
| D | `auto_switcher.rs` | new helper above `/// Execute rotation to target candidate with rich telemetry context` | `/// Execute rotation to target candidate with rich telemetry context` | `:1413` |
| E | `repo_db.rs` | new function above `/// Re-enqueue running conversations for an instance before switch/restart.` | `/// Re-enqueue running conversations for an instance before switch/restart.` | `:2243` |
| F | `integration.rs` | `on_account_switch` (desktop), restore block | `let workspace_roots =` (the hit inside the `if needs_reinject {` block) | `:426` to `:437` |
| G | `telegram_inbound.rs` | `pub fn execute_backup_command(args_str: &str) -> String` (whole function) | `/// Execute Prompt Backup ("Backpack") or Restore command` | `:1310` to `:1405` |
| H | `telegram_inbound.rs` | `pub async fn process_telegram_command_text(` (`restore` alias) | `"restore" => Some(execute_backup_command("restore")),` | `:2712` |
| I | `telegram_inbound.rs` | same function, `ff` / `rotate` arm | `"ff" \| "rotate" => {` | `:2715` to `:2748` |
| J | `telegram_inbound.rs` | same function, `ff:` prefix | `if lower_full.starts_with("ff:") {` | `:2774` to `:2806` |
| K | `telegram_inbound.rs` | new `execute_fast_forward_command`, inserted directly above `/// Unified inbound Telegram command processor` | `/// Unified inbound Telegram command processor` | `:2562` |

```text
gitmap aum search "Ensure all running and queued prompts are snapshotted" src-tauri/src/modules/auto_switcher.rs --ext .rs
gitmap aum search "Save in-flight prompts" src-tauri/src/modules/auto_switcher.rs --ext .rs
gitmap aum search "Asynchronous 5-second post-launch prompt re-injection" src-tauri/src/modules/auto_switcher.rs --ext .rs
gitmap aum search "target_inst_id_clone" src-tauri/src/modules/auto_switcher.rs --ext .rs
gitmap aum search "Re-enqueue running conversations for an instance" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "let workspace_roots" src-tauri/src/modules/integration.rs --ext .rs
gitmap aum search "Execute Prompt Backup" src-tauri/src/modules/telegram_inbound.rs --ext .rs
gitmap aum search "execute_backup_command" src-tauri/src --ext .rs
gitmap aum search "check_and_rotate_if_needed" src-tauri/src/modules/telegram_inbound.rs --ext .rs
```

Before adding new names, check they do not exist yet:

```text
gitmap aum search "fn compute_restore_instance|fn record_requeue_audit_for_instance|fn split_instance_flag|fn execute_fast_forward_command" src-tauri/src --ext .rs
```

## 4. Current code

Expected drift: step 13 replaced the `resend_running_commands_for_instance(...)` argument in Change C (quoted below as step 13 leaves it); step 15 replaced the backup call in `integration.rs:342` (not touched here). Compare only the blocks below.

### Change A (`auto_switcher.rs:1421` to `:1436`)

```rust
    let current_instance_id = ctx
        .as_ref()
        .and_then(|c| c.current_instance_id.clone())
        .unwrap_or_else(|| {
            crate::modules::instance::get_active_instance_id()
                .unwrap_or_else(|_| "default".to_string())
        });
    let inst_id = &target.instance_id;

    // Step 0: Ensure all running and queued prompts are snapshotted and backed up before profile switch
    let backup_res = crate::modules::repo_db::backup_running_prompts(&current_instance_id);
    let backed_up_count = backup_res.as_ref().copied().unwrap_or(0);
    let _ = crate::modules::backup_prompts_db::backup_active_running_prompts(
        Some(&current_instance_id),
        None,
    );
```

### Change B (`auto_switcher.rs:1594` to `:1597`)

```rust
        // a) Save in-flight prompts
        let _ = crate::modules::repo_db::requeue_running_conversations_for_instance(
            &current_instance_id,
        );
```

`requeue_running_conversations_for_instance` calls `backup_running_prompts` again (a second backup of the same instance right after Change A's) and then writes audit events. Change B keeps only the audit part.

### Change C (`auto_switcher.rs:1684` to `:1729`, as step 13 leaves it)

```rust
    // Step 3: Asynchronous 5-second post-launch prompt re-injection and status notification
    let target_inst_id_clone = inst_id.clone();
    let proj_names_clone = proj_names.clone();
    let backed_up_count_val = backed_up_count;
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let resent_count = match crate::modules::repo_db::resend_running_commands_for_instance(
            &target_inst_id_clone,
            20,
        ) {
            Ok(resent) => {
                logger::log_info(&format!(
                    "[AutoSwitcher] Post-switch re-injected {} backed-up running prompts across workspaces",
                    resent.len()
                ));
                resent.len()
            }
            Err(e) => {
                logger::log_warn(&format!(
                    "[AutoSwitcher] Failed to resend backed-up running commands after switch: {}",
                    e
                ));
                0
            }
        };

        let dispatched_count =
            crate::modules::repo_db::dispatch_running_prompts(&target_inst_id_clone).unwrap_or(0);
        let verified_running = crate::modules::repo_db::verify_prompts_running();
        logger::log_info(&format!(
            "[AutoSwitcher] Prompt restoration complete: {} resent, {} dispatched, {} verified active running for instance '{}'",
            resent_count, dispatched_count, verified_running, target_inst_id_clone
        ));

        crate::modules::notification_hub::notify_post_switch_prompt_status(
            &target_inst_id_clone,
            backed_up_count_val,
            resent_count + dispatched_count,
            verified_running,
            proj_names_clone,
        );
    });
```

Facts used by the new code (verify each):

- `app_config` is a local loaded at `:1568` (`let app_config = config::load_app_config().unwrap_or_default();`) and is still in scope here. A second `let app_config = ...` at `:1732` (after this block) shadows it; leave that one.
- `target_inst_id_clone` is used only inside this block (`gitmap aum search "target_inst_id_clone" src-tauri/src/modules/auto_switcher.rs --ext .rs` shows hits only in `:1685` to `:1723`). If it has a hit outside this block, STOP.
- `inst_id` stays used later (`auto_resume_recent_prompts(inst_id, threshold)` at `:1737` and the emit at `:1664`).

### Change D (`auto_switcher.rs:1413`)

```rust
/// Execute rotation to target candidate with rich telemetry context
pub async fn execute_profile_rotation_with_context(
```

### Change E (`repo_db.rs:2243` to `:2245`)

```rust
/// Re-enqueue running conversations for an instance before switch/restart.
/// Marks active prompts as 'backed_up', updates resume document, and logs to Audit.
pub fn requeue_running_conversations_for_instance(instance_id: &str) -> Result<usize, String> {
```

This function stays unchanged. Change E only inserts a new function above its doc comment.

### Change F (`integration.rs:420` to `:439`)

```rust
        if needs_reinject {
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                crate::modules::logger::log_info(
                    "[Desktop] [Step 5/5] 5s post-launch delay elapsed. Restoring prompts for instance default",
                );
                let workspace_roots =
                    crate::modules::instance::get_instance_workspace_paths("default");
                let _ = crate::modules::instance::restore_and_inject_prompts_for_instance(
                    "default",
                    &workspace_roots,
                );
                let _ = crate::modules::backup_prompts_db::restore_running_prompts_for_instance(
                    Some("default"),
                    false,
                    None,
                );
                let _ = crate::modules::repo_db::dispatch_running_prompts("default");
            });
            note_prompt_reinjected(true);
```

### Change G (`telegram_inbound.rs:1310` to `:1405`, whole function)

First lines:

```rust
/// Execute Prompt Backup ("Backpack") or Restore command
pub fn execute_backup_command(args_str: &str) -> String {
    let sub = args_str.trim().to_lowercase();
    if sub == "ls" || sub == "list" || sub == "status" {
```

The restore branch (`:1340` to `:1370`):

```rust
    } else if sub == "restore" || sub == "rrp" || sub == "resend" {
        let projs = repo_db::list_running_projects().unwrap_or_default();
        let proj_names = crate::modules::notification_hub::deduplicate_names(
            projs
                .into_iter()
                .map(|p| p.repo_name)
                .filter(|n| !n.is_empty()),
        );
        let proj_display = if !proj_names.is_empty() {
            proj_names.join(", ")
        } else {
            "Antigravity-Manager".to_string()
        };

        let repo_resent = repo_db::resend_all_running_commands(20).unwrap_or_default();
        let _ = repo_db::dispatch_running_prompts("default");
        match backup_prompts_db::restore_running_prompts(Some("default"), false, None) {
```

The backup branch tail (`:1385` to `:1386`):

```rust
        let _ = repo_db::backup_running_prompts("default");
        match backup_prompts_db::backup_active_running_prompts(Some("default"), None) {
```

Last lines:

```rust
            Err(e) => format!(
                "⚠️ <b>Backup Failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            ),
        }
    }
}
```

The function is 96 lines; only the anchors above are quoted. Re-read the whole function before replacing it.

### Change H (`telegram_inbound.rs:2711` to `:2712`)

```rust
        "backup" | "backpack" => Some(execute_backup_command(rest)),
        "restore" => Some(execute_backup_command("restore")),
```

### Change I (`telegram_inbound.rs:2715` to `:2748`)

```rust
        "ff" | "rotate" => {
            let projs = repo_db::list_running_projects().unwrap_or_default();
            let proj_names = crate::modules::notification_hub::deduplicate_names(
                projs
                    .into_iter()
                    .map(|p| p.repo_name)
                    .filter(|n| !n.is_empty()),
            );
            let proj_display = if !proj_names.is_empty() {
                proj_names.join(", ")
            } else {
                "Antigravity-Manager".to_string()
            };

            let backup_count = repo_db::backup_running_prompts("default").unwrap_or(0);
            let _ = backup_prompts_db::backup_active_running_prompts(Some("default"), None);
            let rotate_res = auto_switcher::check_and_rotate_if_needed().await;
            let resent = repo_db::resend_all_running_commands(20).unwrap_or_default();
            let disp = repo_db::dispatch_running_prompts("default").unwrap_or(0);

            Some(format!(
                "⏩ <b>Fast-Forward &amp; Prompt Preservation:</b>\n\n\
                • <b>Projects:</b> <code>{}</code>\n\
                • <b>Pre-Switch Backup:</b> <code>{}</code> running prompt(s) captured\n\
                • <b>Rotation Evaluation:</b> {}\n\
                • <b>Post-Switch Re-injection:</b> <code>{}</code> prompt(s) resent ({} dispatched)\n\n\
                💡 All active prompts are preserved and continue without interruption.",
                clean_for_telegram_html(&proj_display, 80),
                backup_count,
                if rotate_res.is_ok() { "✅ Evaluated successfully" } else { "ℹ️ No switch required" },
                resent.len(),
                disp
            ))
        }
```

### Change J (`telegram_inbound.rs:2774` to `:2806`)

```rust
            if lower_full.starts_with("ff:") {
                let projs = repo_db::list_running_projects().unwrap_or_default();
                let proj_names = crate::modules::notification_hub::deduplicate_names(
                    projs
                        .into_iter()
                        .map(|p| p.repo_name)
                        .filter(|n| !n.is_empty()),
                );
                let proj_display = if !proj_names.is_empty() {
                    proj_names.join(", ")
                } else {
                    "Antigravity-Manager".to_string()
                };

                let backup_count = repo_db::backup_running_prompts("default").unwrap_or(0);
                let _ = backup_prompts_db::backup_active_running_prompts(Some("default"), None);
                let rotate_res = auto_switcher::check_and_rotate_if_needed().await;
                let resent = repo_db::resend_all_running_commands(20).unwrap_or_default();
                let disp = repo_db::dispatch_running_prompts("default").unwrap_or(0);

                return Some(format!(
                    "⏩ <b>Fast-Forward &amp; Prompt Preservation:</b>\n\n\
                    • <b>Projects:</b> <code>{}</code>\n\
                    • <b>Pre-Switch Backup:</b> <code>{}</code> prompt(s) captured\n\
                    • <b>Rotation Evaluation:</b> {}\n\
                    • <b>Post-Switch Re-injection:</b> <code>{}</code> prompt(s) resent ({} dispatched)",
                    clean_for_telegram_html(&proj_display, 80),
                    backup_count,
                    if rotate_res.is_ok() { "✅ Completed" } else { "ℹ️ Evaluated" },
                    resent.len(),
                    disp
                ));
            }
```

Why `/ff` drops the backup and resend: `check_and_rotate_if_needed` rotates each monitored instance through `execute_profile_rotation_with_context`, which already backs up and restores per instance (Changes A to C). The old pre-backup flipped `"default"` rows from `running` to `backed_up` even when no rotation happened, and the old resend then sent them a second time while the original turn was still running.

## 5. New code

### Change A

```rust
    let raw_current_instance_id = ctx
        .as_ref()
        .and_then(|c| c.current_instance_id.clone())
        .unwrap_or_else(|| {
            crate::modules::instance::get_active_instance_id()
                .unwrap_or_else(|_| "default".to_string())
        });
    let current_instance_id =
        match crate::modules::instance::canonical_instance_id(&raw_current_instance_id) {
            Ok(id) => id,
            Err(e) => {
                logger::log_warn(&format!(
                    "[AutoSwitcher] Rotation stopped before backup: source instance '{}' is not registered: {}",
                    raw_current_instance_id, e
                ));
                return Err(e);
            }
        };
    let inst_id = &target.instance_id;

    // Step 0: Ensure all running and queued prompts are snapshotted and backed up before profile switch
    let backup_res = crate::modules::repo_db::backup_running_prompts(&current_instance_id);
    let backed_up_count = backup_res.as_ref().copied().unwrap_or(0);
    let _ = crate::modules::backup_prompts_db::backup_active_running_prompts_for_instance(
        &current_instance_id,
        None,
    );
```

`current_instance_id` stays a `String`, so every later use (`&current_instance_id`, `target.instance_id != current_instance_id`, `.find(|i| i.id == current_instance_id)`) compiles unchanged.

### Change B

```rust
        // a) Save in-flight prompts
        let _ = crate::modules::repo_db::record_requeue_audit_for_instance(&current_instance_id);
```

### Change C

```rust
    // Step 3: Asynchronous 5-second post-launch prompt restoration and status notification
    let restore_instance = compute_restore_instance(
        &current_instance_id,
        &target.instance_id,
        app_config.auto_profile_switcher.auto_reopen_on_switch,
    );
    let proj_names_clone = proj_names.clone();
    let backed_up_count_val = backed_up_count;
    match restore_instance {
        None => {
            logger::log_info(&format!(
                "[AutoSwitcher] Source instance '{}' was not relaunched; {} prompt(s) stay backed_up until it is launched again",
                current_instance_id,
                crate::modules::repo_db::count_backed_up_prompts(&current_instance_id)
            ));
        }
        Some(restore_inst) => {
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                let report =
                    match crate::modules::backup_prompts_db::restore_backed_up_prompts_for_instance(
                        &restore_inst,
                        false,
                        None,
                    ) {
                        Ok(report) => report,
                        Err(e) => {
                            logger::log_warn(&format!(
                                "[AutoSwitcher] Prompt restore failed for instance '{}': {}",
                                restore_inst, e
                            ));
                            return;
                        }
                    };
                if report.skipped_reason.as_deref() == Some("instance_not_running") {
                    logger::log_warn(&format!(
                        "[AutoSwitcher] Instance '{}' is not running after the switch; prompts stay backed_up",
                        restore_inst
                    ));
                    return;
                }
                let verified_running = crate::modules::repo_db::verify_prompts_running();
                logger::log_info(&format!(
                    "[AutoSwitcher] Prompt restoration complete: {} dispatched, {} failed, {} waiting, {} verified active running for instance '{}'",
                    report.dispatched, report.failed, report.skipped, verified_running, restore_inst
                ));

                crate::modules::notification_hub::notify_post_switch_prompt_status(
                    &restore_inst,
                    backed_up_count_val,
                    report.dispatched,
                    verified_running,
                    proj_names_clone,
                );
            });
        }
    }
```

If clippy reports `redundant_clone` on `proj_names.clone()`, change it to `let proj_names_clone = proj_names;` only if `proj_names` has no later use in the function (check with `gitmap aum search "proj_names" src-tauri/src/modules/auto_switcher.rs --ext .rs`).

### Change D (insert directly above `/// Execute rotation to target candidate with rich telemetry context`)

```rust
/// Instance whose backed-up prompts are restored after a rotation: the source, and only when the
/// rotation relaunched the source itself (same-instance rotation with reopen enabled). A
/// cross-instance rotation closes the source, so nothing is restored for it until its next launch;
/// the target restores its own prompts inside `switch_account_to_instance`.
pub(crate) fn compute_restore_instance(
    source_instance_id: &str,
    target_instance_id: &str,
    is_reopen_enabled: bool,
) -> Option<String> {
    let source = source_instance_id.trim();
    if source.is_empty() || source != target_instance_id.trim() || !is_reopen_enabled {
        return None;
    }
    Some(source.to_string())
}

```

### Change E (insert directly above `/// Re-enqueue running conversations for an instance before switch/restart.`)

```rust
/// Writes one requeue audit event per backed_up prompt of the instance (newest 50). Changes no
/// prompt row and runs no backup.
pub fn record_requeue_audit_for_instance(instance_id: &str) -> Result<usize, String> {
    let inst = crate::modules::instance::canonical_instance_id(instance_id)?;
    let conn = connect_db()?;
    let prompts = select_instance_prompts_by_status(&conn, &inst, &[prompt_status::BACKED_UP]);
    let mut recorded = 0;
    for prompt in prompts.iter().rev().take(50) {
        let project_name = Path::new(&prompt.repo_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| prompt.project_id.clone());
        let preview: String = prompt.prompt_content.chars().take(100).collect();
        let _ = crate::modules::task_history_db::record_requeue_event(
            &project_name,
            &inst,
            prompt.session_id.as_deref().unwrap_or_default(),
            "In-flight conversation re-enqueued for restart/switch continuity",
            &preview,
        );
        recorded += 1;
    }
    Ok(recorded)
}

```

`select_instance_prompts_by_status` returns rows oldest first, so `.rev().take(50)` keeps the newest 50, like the old `ORDER BY updated_at DESC LIMIT 50`. The preview uses `chars()` so a multi-byte character at position 100 cannot panic (the old code sliced bytes).

### Change F

```rust
        if needs_reinject {
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                crate::modules::logger::log_info(
                    "[Desktop] [Step 5/5] 5s post-launch delay elapsed. Restoring prompts for instance default",
                );
                match crate::modules::backup_prompts_db::restore_backed_up_prompts_for_instance(
                    "default", false, None,
                ) {
                    Ok(report) => crate::modules::logger::log_info(&format!(
                        "[Desktop] Prompt restore for 'default': {} dispatched, {} failed, {} waiting",
                        report.dispatched, report.failed, report.skipped
                    )),
                    Err(e) => crate::modules::logger::log_warn(&format!(
                        "[Desktop] Prompt restore for 'default' failed: {}",
                        e
                    )),
                }
            });
            note_prompt_reinjected(true);
```

`"default"` stays: this desktop path only switches the default instance (it closes and relaunches `"default"` in its own steps 2 to 4).

### Change G (replace the whole function, and add `split_instance_flag` directly above it)

```rust
/// Splits Telegram arguments into positional words and the value of `-i <id>`,
/// `--instance <id>` or `--instance=<id>`. A flag without a value is ignored.
pub(crate) fn split_instance_flag(args_str: &str) -> (Vec<String>, Option<String>) {
    let mut positionals = Vec::new();
    let mut instance = None;
    let mut tokens = args_str.split_whitespace();
    while let Some(token) = tokens.next() {
        if token == "-i" || token == "--instance" {
            if let Some(value) = tokens.next() {
                instance = Some(value.to_string());
            }
        } else if let Some(value) = token.strip_prefix("--instance=") {
            if !value.is_empty() {
                instance = Some(value.to_string());
            }
        } else {
            positionals.push(token.to_string());
        }
    }
    (positionals, instance)
}

fn running_project_display() -> String {
    let projs = repo_db::list_running_projects().unwrap_or_default();
    let proj_names = crate::modules::notification_hub::deduplicate_names(
        projs
            .into_iter()
            .map(|p| p.repo_name)
            .filter(|n| !n.is_empty()),
    );
    if !proj_names.is_empty() {
        proj_names.join(", ")
    } else {
        "Antigravity-Manager".to_string()
    }
}

/// Execute Prompt Backup ("Backpack") or Restore command
pub fn execute_backup_command(args_str: &str) -> String {
    let (positionals, instance_flag) = split_instance_flag(args_str);
    let sub = positionals
        .first()
        .map(|s| s.to_lowercase())
        .unwrap_or_default();
    if sub == "ls" || sub == "list" || sub == "status" {
        match backup_prompts_db::list_backup_batches(None) {
            Ok(batches) => {
                if batches.is_empty() {
                    return "🎒 <b>Prompt Backup Vault:</b> No backup batches stored.".to_string();
                }
                let mut rows = String::new();
                for (i, b) in batches.iter().take(8).enumerate() {
                    rows.push_str(&format!(
                        "{}. <code>{}</code> — {} prompt(s) [restored: {}]\n",
                        i + 1,
                        clean_for_telegram_html(&b.id, 32),
                        b.prompts_count,
                        b.is_fully_restored
                    ));
                }
                format!(
                    "🎒 <b>Split SQLite Prompt Backups ({} total):</b>\n\n{}",
                    batches.len(),
                    rows
                )
            }
            Err(e) => format!(
                "⚠️ <b>Backup Query Failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            ),
        }
    } else if sub == "restore" || sub == "rrp" || sub == "resend" {
        let spec = instance_flag.or_else(|| positionals.get(1).cloned());
        let inst = match crate::modules::instance::resolve_instance_input(
            spec.as_deref().unwrap_or("default"),
        ) {
            Ok(id) => id,
            Err(e) => {
                return format!(
                    "⚠️ <b>Prompt Restore Failed:</b> <code>{}</code>",
                    clean_for_telegram_html(&e, 200)
                )
            }
        };
        let proj_display = running_project_display();
        match backup_prompts_db::restore_backed_up_prompts_for_instance(&inst, false, None) {
            Ok(report) => format!(
                "♻️ <b>Prompt Restoration Complete:</b>\n\n\
                • <b>Instance:</b> <code>{}</code>\n\
                • <b>Projects:</b> <code>{}</code>\n\
                • Restored <b>{}</b> prompt(s) from split SQLite backup.\n\
                • Sent <b>{}</b>, failed <b>{}</b>, waiting <b>{}</b>.{}",
                clean_for_telegram_html(&inst, 80),
                clean_for_telegram_html(&proj_display, 80),
                report.restored_from_backup,
                report.dispatched,
                report.failed,
                report.skipped,
                report
                    .skipped_reason
                    .as_deref()
                    .map(|reason| format!("\n• Skipped: <code>{}</code>", clean_for_telegram_html(reason, 60)))
                    .unwrap_or_default()
            ),
            Err(e) => format!(
                "⚠️ <b>Prompt Restore Failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            ),
        }
    } else {
        let inst = match crate::modules::instance::resolve_instance_input(
            instance_flag.as_deref().unwrap_or("default"),
        ) {
            Ok(id) => id,
            Err(e) => {
                return format!(
                    "⚠️ <b>Backup Failed:</b> <code>{}</code>",
                    clean_for_telegram_html(&e, 200)
                )
            }
        };
        let proj_display = running_project_display();

        let _ = repo_db::backup_running_prompts(&inst);
        match backup_prompts_db::backup_active_running_prompts_for_instance(&inst, None) {
            Ok((batch, records)) => format!(
                "🎒 <b>Running Prompts Backed Up Successfully!</b>\n\n\
                • <b>Instance:</b> <code>{}</code>\n\
                • <b>Projects:</b> <code>{}</code>\n\
                • <b>Batch ID:</b> <code>{}</code>\n\
                • <b>Captured Prompts:</b> <b>{}</b>\n\
                • <b>Vault Path:</b> <code>{}</code>\n\n\
                💡 Send <code>/restore</code> anytime to resume backed-up prompts.",
                clean_for_telegram_html(&inst, 80),
                clean_for_telegram_html(&proj_display, 80),
                clean_for_telegram_html(&batch.id, 40),
                records.len(),
                clean_for_telegram_html(&batch.file_path, 120)
            ),
            Err(e) => format!(
                "⚠️ <b>Backup Failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            ),
        }
    }
}
```

The emoji characters are copied from the existing messages; keep them byte-for-byte (do not add new ones). If `running_project_display` already exists in the file (the "check they do not exist" search above does not cover it), run `gitmap aum search "fn running_project_display" src-tauri/src --ext .rs`; on a hit, name the new helper `backup_project_display` and use that name in all three places.

Behavior: `/backup` and `/backup now` back up `"default"`; `/backup -i inst-a` backs up `inst-a`; `/backup restore`, `/restore` restore `"default"`; `/restore inst-a`, `/restore -i inst-a`, `/backup restore --instance=inst-a` restore `inst-a`. An unknown instance returns the error text and touches nothing.

### Change H

```rust
        "backup" | "backpack" => Some(execute_backup_command(rest)),
        "restore" => Some(execute_backup_command(&format!("restore {}", rest))),
```

### Change I

```rust
        "ff" | "rotate" => Some(execute_fast_forward_command().await),
```

### Change J

```rust
            if lower_full.starts_with("ff:") {
                return Some(execute_fast_forward_command().await);
            }
```

### Change K (new function in `telegram_inbound.rs`, insert directly above `/// Unified inbound Telegram command processor`)

```rust
/// Runs one quota rotation pass. Each rotated instance backs up its own prompts before the switch
/// and restores them after its relaunch, so this command neither backs up nor resends anything.
async fn execute_fast_forward_command() -> String {
    let proj_display = running_project_display();
    let rotate_res = auto_switcher::check_and_rotate_if_needed().await;
    format!(
        "⏩ <b>Fast-Forward &amp; Prompt Preservation:</b>\n\n\
        • <b>Projects:</b> <code>{}</code>\n\
        • <b>Rotation Evaluation:</b> {}\n\n\
        Each rotated instance backs up its own prompts and restores them after it is relaunched.",
        clean_for_telegram_html(&proj_display, 80),
        if rotate_res.is_ok() { "✅ Evaluated successfully" } else { "ℹ️ No switch required" }
    )
}
```

After Changes G to K, run `gitmap aum search "resend_all_running_commands|dispatch_running_prompts|backup_active_running_prompts\(Some" src-tauri/src/modules/telegram_inbound.rs --ext .rs`. It must return 0 hits. If an import in the `use` lines at the top of `telegram_inbound.rs` becomes unused, clippy names it; remove only that name.

## 6. Tests

### `auto_switcher.rs` (paste inside `mod tests`, before its final `}`)

```rust
    #[test]
    fn auto_switch_default_target_restores_default_only() {
        assert_eq!(
            compute_restore_instance("default", "default", true),
            Some("default".to_string())
        );
        assert_eq!(compute_restore_instance("default", "default", false), None);
        assert_eq!(compute_restore_instance("", "", true), None);
    }

    #[test]
    fn cross_instance_rotation_restores_source_not_target() {
        assert_eq!(compute_restore_instance("inst-a", "inst-b", true), None);
        assert_eq!(compute_restore_instance("default", "inst-b", true), None);
        assert_eq!(compute_restore_instance("inst-b", "default", true), None);
        assert_eq!(
            compute_restore_instance("inst-a", "inst-a", true),
            Some("inst-a".to_string())
        );
    }
```

### `telegram_inbound.rs` (paste inside `mod tests`, before its final `}`)

```rust
    #[test]
    fn telegram_instance_flag_short_and_long_forms() {
        let (pos, inst) = split_instance_flag("restore -i inst-a");
        assert_eq!(pos, vec!["restore".to_string()]);
        assert_eq!(inst.as_deref(), Some("inst-a"));

        let (pos, inst) = split_instance_flag("restore --instance inst-b");
        assert_eq!(pos, vec!["restore".to_string()]);
        assert_eq!(inst.as_deref(), Some("inst-b"));

        let (pos, inst) = split_instance_flag("--instance=inst-c restore");
        assert_eq!(pos, vec!["restore".to_string()]);
        assert_eq!(inst.as_deref(), Some("inst-c"));
    }

    #[test]
    fn telegram_instance_flag_positional_and_missing_value() {
        let (pos, inst) = split_instance_flag("restore inst-d");
        assert_eq!(pos, vec!["restore".to_string(), "inst-d".to_string()]);
        assert!(inst.is_none());

        let (pos, inst) = split_instance_flag("restore -i");
        assert_eq!(pos, vec!["restore".to_string()]);
        assert!(inst.is_none());

        let (pos, inst) = split_instance_flag("");
        assert!(pos.is_empty());
        assert!(inst.is_none());
    }
```

No test calls `execute_backup_command`, `execute_fast_forward_command`, `record_requeue_audit_for_instance` or the rotation itself: they load the registry or open the real data databases. The restore core they call is tested in step 12.

## 7. Gate

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 auto_switch_default_target_restores_default_only; cd ..
cd src-tauri; cargo test --lib -- --test-threads=1 cross_instance_rotation_restores_source_not_target; cargo test --lib -- --test-threads=1 telegram_instance_flag_; cargo test --lib -- --test-threads=1 backup_prompts_db::tests::restore_; cd ..
```

Every command must exit 0 and each filter must run at least one test.

## 8. Commit

```text
Fix: prompts - switch flows restore only the source instance
```

Stage only:

```text
git add src-tauri/src/modules/auto_switcher.rs src-tauri/src/modules/integration.rs src-tauri/src/modules/telegram_inbound.rs src-tauri/src/modules/repo_db.rs
```

## 9. Done when

- [ ] `execute_profile_rotation_with_context` canonicalizes the source id before any backup and returns `Err` (with a log line) for an unknown source.
- [ ] It backs up through `backup_active_running_prompts_for_instance(&current_instance_id, None)`; no `Some(` and no `None` instance argument remains in that function.
- [ ] The cross-instance branch calls `record_requeue_audit_for_instance`, not `requeue_running_conversations_for_instance`.
- [ ] The post-switch block calls only `restore_backed_up_prompts_for_instance`, and only for `compute_restore_instance(..)`; `resend_running_commands_for_instance` and `dispatch_running_prompts` no longer appear in `auto_switcher.rs` (`gitmap aum search "resend_running_commands_for_instance|dispatch_running_prompts" src-tauri/src/modules/auto_switcher.rs --ext .rs` returns 0 hits).
- [ ] `integration.rs` restore block has one call: `restore_backed_up_prompts_for_instance("default", false, None)`.
- [ ] Telegram `/restore` and `/backup` accept `-i`, `--instance`, `--instance=`; `/ff`, `/rotate` and `ff:` call `execute_fast_forward_command()`.
- [ ] `src-tauri/src/modules/account.rs` is unchanged.
- [ ] Callers searched in `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file. No caller edit is needed because every changed function keeps its signature and return type, and `agm.rs:5815` still calls `execute_backup_command` unchanged. The new private `running_project_display` in `telegram_inbound.rs` replaces the four `repo_db::list_running_projects()` calls with one; step 17 updates that single remaining line.
- [ ] Gate exits 0.
- [ ] Only the four files are staged.
