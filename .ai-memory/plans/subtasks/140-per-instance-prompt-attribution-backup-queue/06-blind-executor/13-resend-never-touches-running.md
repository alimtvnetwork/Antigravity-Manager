# Step 13: Resend Never Touches Running Rows

Goal: the explicit resend command (`rrc`) sends only `backed_up` and `failed` rows of one named instance, never a `running` row, sends at most one prompt per repository per run, and marks a row `dispatched` only through the shared dispatch step. Fixes B16. Spec section 6.6 (last paragraph) and 6.5 rule 4.

## 1. Depends on

- Step 12 (`select_instance_prompts_by_status`, `dispatch_prompt_for_restore`).
- Step 10 (`prompt_status`, `claim_prompt_for_dispatch`, `dispatch_one_prompt`).
- Step 01 (`canonical_instance_id`).

## 2. Files you may edit

- `src-tauri/src/modules/repo_db.rs`
- `src-tauri/src/modules/auto_switcher.rs` (one call site only, so the crate still compiles; step 16 replaces that block)
- `src-tauri/src/bin/agm.rs` (five call sites, Change C; `clippy --all-targets` compiles the `agm` bin, so they must change in this step)

## 3. Find it

| Change | File | `fn` signature | Unique search literal | Line hint |
|---|---|---|---|---|
| A | `repo_db.rs` | `pub fn resend_all_running_commands(limit: usize) -> Result<Vec<ActivePrompt>, String>` and `pub fn resend_running_commands_for_instance(` | `Resend and restore all previous running/backed-up/dispatched commands across all instances` | `:3229` |
| B | `auto_switcher.rs` | inside `pub async fn execute_profile_rotation_with_context(` | `let resent_count = match crate::modules::repo_db::resend_running_commands_for_instance(` | `:1690` |
| C1 | `agm.rs` | restore command (the function whose help text contains `agm restore --keep`) | `let _ = repo_db::resend_running_commands_for_instance(Some(&target_instance), 20);` (first of two hits) | `:4169` |
| C2 | `agm.rs` | `agm instances ff` branch | `let _ = repo_db::resend_running_commands_for_instance(Some(&resolved_id), 20);` | `:10645` |
| C3 | `agm.rs` | fast-forward rotation command (`let result = rt.block_on(auto_switcher::trigger_manual_rotation_for_instance(Some(`) | `let _ = repo_db::resend_running_commands_for_instance(Some(&target_instance), 20);` (second of two hits) | `:10927` |
| C4 | `agm.rs` | e2e command, Step 5f | `repo_db::resend_running_commands_for_instance(Some(&new_inst.id), 20).unwrap_or_default();` (first of two hits) | `:14612` |
| C5 | `agm.rs` | e2e command, Step 6e | same literal (second of two hits) | `:14788` |

GitMap does not descend into `src-tauri/src/bin` or `src-tauri/tests` when you search `src-tauri/src`; run every line:

```text
gitmap aum search "Resend and restore all previous running" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "Auto-resume recent prompts for projects active within" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "resend_running_commands_for_instance(" src-tauri/src --ext .rs
gitmap aum search "resend_running_commands_for_instance(" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "resend_running_commands_for_instance|resend_all_running_commands" src-tauri/tests/auto_switcher_e2e_test.rs --ext .rs
gitmap aum search "resend_running_commands_for_instance|resend_all_running_commands" src-tauri/tests/instance_cloning_and_sync_test.rs --ext .rs
gitmap aum search "resend_running_commands_for_instance|resend_all_running_commands" src-tauri/tests/per_instance_prompt_liveness_test.rs --ext .rs
gitmap aum search "resend_prompts_in" src-tauri/src --ext .rs
```

Expected after step 12:

- `src-tauri/src`: exactly three call/definition hits: the definition in `repo_db.rs`, the call inside `resend_all_running_commands`, and `auto_switcher.rs:1690`. (Step 12 removed the calls in `backup_prompts_db.rs` and `instance.rs`.) If any other caller exists, STOP.
- `src-tauri/src/bin/agm.rs`: exactly five hits, C1 to C5, each passing `Some(&<String>)`.
- The three test files: 0 hits.
- `resend_prompts_in`: 0 hits.

`resend_all_running_commands(limit: usize)` keeps its signature; its callers (`telegram_inbound.rs:1354`, `:2732`, `:2791`; `agm.rs:3376`, `:7981`) need no change.

Before adding `resend_prompts_in`, search for it; if it already exists, STOP (another step took the name).

Steps 12, 14 to 22 may later change the lines around C1 to C5 (for example step 19 replaces C1's block). Only the single call line is edited here, so this step does not depend on them.

## 4. Current code

### Change A (`repo_db.rs:3229`, two functions, about 205 lines together)

Both functions are replaced together. Expected drift: step 08 replaced the `resolve_instance_id` calls inside `resend_running_commands_for_instance`, and step 09 replaced the inline resume JSON with `write_resume_handoff`. That drift is fine because every line is deleted. Verify the first lines and the last lines below, then delete from the line `/// Resend and restore all previous running/backed-up/dispatched commands across all instances.` through the closing `}` directly above `/// Auto-resume recent prompts for projects active within max_age_seconds`.

First lines today:

```rust
/// Resend and restore all previous running/backed-up/dispatched commands across all instances.
pub fn resend_all_running_commands(limit: usize) -> Result<Vec<ActivePrompt>, String> {
    resend_running_commands_for_instance(None, limit)
}

/// Resend and restore previous running/backed-up/queued commands scoped to an optional `instance_id`.
/// Strictly filters to unrestored/backed-up/queued prompts, prioritizes the latest running prompts,
/// and deduplicates against already dispatched prompts and workspace repositories.
pub fn resend_running_commands_for_instance(
    instance_id: Option<&str>,
    limit: usize,
) -> Result<Vec<ActivePrompt>, String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();

    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload 
             FROM active_prompts 
             WHERE status IN ('backed_up', 'queued', 'pending', 'running')
             ORDER BY created_at ASC LIMIT ?",
        )
        .map_err(|e| format!("Failed to prepare resend query: {}", e))?;
```

Last lines today:

```rust
        prompt.status = "dispatched".to_string();
        prompt.updated_at = now;
        resent.push(prompt);
    }

    invalidate_prompt_tree_cache(target_inst_opt.as_deref());
    Ok(resent)
}

/// Auto-resume recent prompts for projects active within max_age_seconds (strictly skips projects older than threshold)
```

### Change B (`auto_switcher.rs:1690`)

```rust
        let resent_count = match crate::modules::repo_db::resend_running_commands_for_instance(
            if target_inst_id_clone == "default" {
                None
            } else {
                Some(&target_inst_id_clone)
            },
            20,
        ) {
```

### Change C (`src-tauri/src/bin/agm.rs`, five single lines)

In every site the argument is an owned `String` (`target_instance`, `resolved_id`, `new_inst.id`), so the only change is dropping the `Some(...)` wrapper.

C1 (`:4169`, restore command; the next line is `let _ = repo_db::dispatch_running_prompts(&target_instance);` followed by `match backup_prompts_db::restore_running_prompts(`):

```rust
    let _ = repo_db::resend_running_commands_for_instance(Some(&target_instance), 20);
```

C2 (`:10645`, inside `Ok(msg) => {` after `trigger_manual_rotation_for_instance(Some(&resolved_id))`):

```rust
                    let _ = repo_db::resend_running_commands_for_instance(Some(&resolved_id), 20);
```

C3 (`:10927`, inside `if result.is_ok() {`; the next line is `let _ = repo_db::dispatch_running_prompts(&target_instance);` followed by `}`):

```rust
        let _ = repo_db::resend_running_commands_for_instance(Some(&target_instance), 20);
```

C4 (`:14611` to `:14612`, Step 5f of the e2e command):

```rust
    let resent_count =
        repo_db::resend_running_commands_for_instance(Some(&new_inst.id), 20).unwrap_or_default();
```

C5 (`:14787` to `:14788`, Step 6e of the e2e command):

```rust
    let resent_count_2 =
        repo_db::resend_running_commands_for_instance(Some(&new_inst.id), 20).unwrap_or_default();
```

## 5. New code

### Change A (paste where the two old functions were, directly above `/// Auto-resume recent prompts ...`)

```rust
/// Resend backed-up and failed prompts for every instance whose IDE is running, one instance at a time.
pub fn resend_all_running_commands(limit: usize) -> Result<Vec<ActivePrompt>, String> {
    let registry = crate::modules::instance::load_registry()?;
    let mut resent = Vec::new();
    for inst in &registry.instances {
        if !crate::modules::instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid) {
            continue;
        }
        match resend_running_commands_for_instance(&inst.id, limit) {
            Ok(mut rows) => resent.append(&mut rows),
            Err(e) => crate::modules::logger::log_warn(&format!(
                "[RepoDB] Resend skipped for instance '{}': {}",
                inst.id, e
            )),
        }
    }
    Ok(resent)
}

/// Resend `backed_up` and `failed` prompts of one instance (canonical id required). Never selects
/// `running` rows. Sends at most one prompt per repository per run; later rows of the same
/// repository become `queued`. A row becomes `dispatched` only through the shared dispatch step.
pub fn resend_running_commands_for_instance(
    instance_id: &str,
    limit: usize,
) -> Result<Vec<ActivePrompt>, String> {
    let inst = crate::modules::instance::canonical_instance_id(instance_id)?;
    let conn = connect_db()?;
    let resent = resend_prompts_in(&conn, &inst, limit, dispatch_prompt_for_restore);
    invalidate_prompt_tree_cache(Some(&inst));
    Ok(resent)
}

/// Testable core of `resend_running_commands_for_instance`; `dispatch` is injected.
pub(crate) fn resend_prompts_in<F>(
    conn: &Connection,
    instance_id: &str,
    limit: usize,
    mut dispatch: F,
) -> Vec<ActivePrompt>
where
    F: FnMut(&Connection, &ActivePrompt),
{
    let mut resent = Vec::new();
    let mut busy_repos: HashSet<String> = HashSet::new();
    let mut seen_identities: HashSet<(String, String)> = HashSet::new();
    let rows = select_instance_prompts_by_status(
        conn,
        instance_id,
        &[prompt_status::BACKED_UP, prompt_status::FAILED],
    );
    for mut prompt in rows {
        if resent.len() >= limit {
            break;
        }
        let repo_key = normalize_path_for_compare(&prompt.repo_path);
        let session_key = prompt.session_id.clone().unwrap_or_default();
        if !session_key.trim().is_empty()
            && !seen_identities.insert((repo_key.clone(), session_key))
        {
            continue;
        }
        if !busy_repos.insert(repo_key) {
            let now = Utc::now().timestamp();
            let _ = conn.execute(
                "UPDATE active_prompts SET status = ?1, updated_at = ?2 WHERE id = ?3 AND status = ?4",
                params![prompt_status::QUEUED, now, &prompt.id, &prompt.status],
            );
            continue;
        }
        if !claim_prompt_for_dispatch(conn, &prompt.id, &prompt.status) {
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
            prompt.status = status;
            prompt.updated_at = Utc::now().timestamp();
            resent.push(prompt);
        }
    }
    resent
}
```

Notes for the compiler:

- `HashSet`, `params`, `Utc` and `Connection` are already imported at the top of `repo_db.rs`.
- `get_dispatched_prompts_cache` and `reset_dispatched_prompts_cache` stay in the file (other code still calls them). Only this function stops using the cache.
- If clippy reports `extract_image_payload_or_path` or `resume_task_document` as unused after this change, they are not: other functions still call them. Do not delete them.
- If step 10 named the constants differently, use step 10's names.

### Change B (`auto_switcher.rs`)

```rust
        let resent_count = match crate::modules::repo_db::resend_running_commands_for_instance(
            &target_inst_id_clone,
            20,
        ) {
```

This is a bridge so the crate compiles; step 16 replaces the whole post-switch block.

### Change C (`agm.rs`)

C1 becomes:

```rust
    let _ = repo_db::resend_running_commands_for_instance(&target_instance, 20);
```

C2 becomes:

```rust
                    let _ = repo_db::resend_running_commands_for_instance(&resolved_id, 20);
```

C3 becomes:

```rust
        let _ = repo_db::resend_running_commands_for_instance(&target_instance, 20);
```

C4 becomes:

```rust
    let resent_count =
        repo_db::resend_running_commands_for_instance(&new_inst.id, 20).unwrap_or_default();
```

C5 becomes:

```rust
    let resent_count_2 =
        repo_db::resend_running_commands_for_instance(&new_inst.id, 20).unwrap_or_default();
```

`resend_running_commands_for_instance` now calls `canonical_instance_id` first. `target_instance` in C1 and C3 comes from `resolve_instance_id(spec).unwrap_or_else(|_| spec...)` or `get_active_instance_id()`, so an unknown spec now makes the resend return `Err`; all five sites already discard or `unwrap_or_default` the result, so nothing else changes. Step 19 replaces C1's surrounding block with the instance selector.

## 6. Tests

Paste into the existing `#[cfg(test)] mod tests` of `src-tauri/src/modules/repo_db.rs` (starts near `:5880`), before its final closing `}`.

```rust
    fn resend_seed_row(
        conn: &Connection,
        id: &str,
        instance_id: &str,
        repo_path: &str,
        session_id: &str,
        status: &str,
        created_at: i64,
    ) {
        conn.execute(
            "INSERT INTO active_prompts
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES (?1, 'proj', ?2, ?3, 'keep going', NULL, ?4, ?5, ?6, ?6, NULL)",
            params![id, instance_id, repo_path, session_id, status, created_at],
        )
        .unwrap();
    }

    fn resend_status_and_attempts(conn: &Connection, id: &str) -> (String, i64) {
        conn.query_row(
            "SELECT status, attempts FROM active_prompts WHERE id = ?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap()
    }

    #[test]
    fn resend_never_selects_running_rows() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        resend_seed_row(&conn, "p-run", "inst-a", "/work/app-one", "cid-run", "running", 10);
        resend_seed_row(&conn, "p-bak", "inst-a", "/work/app-two", "cid-bak", "backed_up", 11);
        resend_seed_row(&conn, "p-other", "inst-b", "/work/app-three", "cid-other", "backed_up", 12);

        let mut sent: Vec<String> = Vec::new();
        let resent = resend_prompts_in(&conn, "inst-a", 20, |c, p| {
            sent.push(p.id.clone());
            c.execute(
                "UPDATE active_prompts SET status = 'dispatched' WHERE id = ?1",
                params![&p.id],
            )
            .unwrap();
        });

        assert_eq!(sent, vec!["p-bak".to_string()]);
        assert_eq!(resent.len(), 1);
        assert_eq!(resent[0].status, "dispatched");
        assert_eq!(
            resend_status_and_attempts(&conn, "p-run"),
            ("running".to_string(), 0)
        );
        assert_eq!(
            resend_status_and_attempts(&conn, "p-other"),
            ("backed_up".to_string(), 0)
        );
    }

    #[test]
    fn resend_sends_one_prompt_per_repo_and_queues_the_rest() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        resend_seed_row(&conn, "p-1", "inst-a", "D:/Work/App", "cid-1", "backed_up", 10);
        resend_seed_row(&conn, "p-2", "inst-a", "d:\\work\\app\\", "cid-2", "backed_up", 11);
        resend_seed_row(&conn, "p-3", "inst-a", "/work/other", "cid-3", "failed", 12);

        let mut sent: Vec<String> = Vec::new();
        let resent = resend_prompts_in(&conn, "inst-a", 20, |c, p| {
            sent.push(p.id.clone());
            c.execute(
                "UPDATE active_prompts SET status = 'dispatched' WHERE id = ?1",
                params![&p.id],
            )
            .unwrap();
        });

        assert_eq!(sent, vec!["p-1".to_string(), "p-3".to_string()]);
        assert_eq!(resent.len(), 2);
        assert_eq!(resend_status_and_attempts(&conn, "p-1"), ("dispatched".to_string(), 1));
        assert_eq!(resend_status_and_attempts(&conn, "p-2"), ("queued".to_string(), 0));
        assert_eq!(resend_status_and_attempts(&conn, "p-3"), ("dispatched".to_string(), 1));
    }

    #[test]
    fn resend_failed_send_is_not_reported_as_resent() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        resend_seed_row(&conn, "p-1", "inst-a", "/work/app", "cid-1", "backed_up", 10);

        let resent = resend_prompts_in(&conn, "inst-a", 20, |c, p| {
            c.execute(
                "UPDATE active_prompts SET status = 'failed' WHERE id = ?1",
                params![&p.id],
            )
            .unwrap();
        });

        assert!(resent.is_empty());
        assert_eq!(resend_status_and_attempts(&conn, "p-1"), ("failed".to_string(), 1));
    }
```

## 7. Gate

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 resend_; cd ..
```

All commands must exit 0. The filter also runs the existing `test_resend_deduplication_and_reinjection_prevention`; it must still pass (it uses only an in-memory DB and the cache, both unchanged).

## 8. Commit

```text
Fix: prompts - resend never touches running rows, instance required
```

Stage only:

```text
git add src-tauri/src/modules/repo_db.rs src-tauri/src/modules/auto_switcher.rs src-tauri/src/bin/agm.rs
```

## 9. Done when

- [ ] `resend_running_commands_for_instance` takes `instance_id: &str` and calls `canonical_instance_id` first.
- [ ] Its selection is `backed_up` and `failed` of that one instance only; the word `'running'` does not appear in its query.
- [ ] No inline resume JSON, no `spawn_prompt_via_agy` call, no dispatched-cache shortcut, and no `UPDATE ... 'dispatched'` remain in it.
- [ ] `resend_all_running_commands` loops over running instances and calls the per-instance function.
- [ ] `auto_switcher.rs` passes `&target_inst_id_clone` (no `None`).
- [ ] Callers were searched in `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file. `gitmap aum search "resend_running_commands_for_instance(Some" src-tauri/src/bin/agm.rs --ext .rs` and the same search on `src-tauri/src` return 0 hits.
- [ ] Gate exits 0; the three new `resend_` tests pass.
- [ ] Only the three files above are staged.
