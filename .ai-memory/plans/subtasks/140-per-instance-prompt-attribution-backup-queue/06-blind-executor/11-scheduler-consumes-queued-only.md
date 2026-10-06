# Step 11: Scheduler Consumes Queued Rows Only

Goal: the 10-minute prompt scheduler sends only `status = 'queued'` rows, one row per (instance, repo) per cycle, oldest first. It claims each row atomically, sends through `dispatch_one_prompt_with` (step 10), and writes `dispatched` only when `agy` actually started. A spawn failure leaves the row `failed` with a reason. `backed_up` rows are no longer sent by the scheduler; restore (step 12) and resend (step 13) move them to `queued` first.

The function name and signature `check_and_dispatch_enqueued_prompts(target_instance: Option<&str>) -> Result<usize, String>` stay, so every caller keeps compiling unchanged. The return value now counts only rows that were really sent.

## 1. Depends on

- Step 01 (`canonical_instance_id`).
- Step 08 (`normalize_path_for_compare` is `pub(crate)`).
- Step 09 (`write_resume_handoff_in`, `RESUME_TASK_FILE_NAME`).
- Step 10 (`prompt_status`, `prompt_status_reason`, `DISPATCHING_TIMEOUT_SECS`, `SpawnOutcome`, `DispatchResult`, `claim_prompt_for_dispatch`, `sweep_stuck_dispatching`, `load_prompts_by_status`, `dispatch_one_prompt_with`, `spawn_prompt_via_agy_outcome`; test helpers `seed_dispatch_prompt`, `prompt_row_state`).

## 2. Files you may edit

- `src-tauri/src/modules/repo_db.rs`
- `src-tauri/tests/per_instance_prompt_liveness_test.rs` (one integration test whose assertions describe the old behavior; Change C)

## 3. Find it

GitMap does not descend into `src-tauri/src/bin` or `src-tauri/tests` when you search `src-tauri/src`; run every line:

```text
gitmap aum search "pub fn check_and_dispatch_enqueued_prompts" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "check_and_dispatch_enqueued_prompts(" src-tauri/src --ext .rs
gitmap aum search "check_and_dispatch_enqueued_prompts(" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "check_and_dispatch_enqueued_prompts" src-tauri/tests/auto_switcher_e2e_test.rs --ext .rs
gitmap aum search "check_and_dispatch_enqueued_prompts" src-tauri/tests/instance_cloning_and_sync_test.rs --ext .rs
gitmap aum search "check_and_dispatch_enqueued_prompts" src-tauri/tests/per_instance_prompt_liveness_test.rs --ext .rs
gitmap aum search "pub(crate) fn dispatch_one_prompt_with" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "fn seed_dispatch_prompt" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "dispatch_queued_prompts_in" src-tauri/src --ext .rs
```

The `dispatch_one_prompt_with` and `seed_dispatch_prompt` commands must return one hit each (step 10). If not, STOP. `dispatch_queued_prompts_in` must return 0 hits before you start (name collision check).

| Change | Anchor | Unique search literal | Line hint (2026-10-06) |
|---|---|---|---|
| A. Replace the whole function and its 3 doc lines | `pub fn check_and_dispatch_enqueued_prompts(target_instance: Option<&str>) -> Result<usize, String>` | `/// Bookkeep and check enqueued/backed_up prompts across projects.` | `:2051` to `:2241` |
| B. Tests | `mod tests` | end of module | last line |
| C1. Integration test setup | `async fn test_e2e_prompt_queue_dispatch_strict_instance_scoping() {` in `src-tauri/tests/per_instance_prompt_liveness_test.rs` | `fn test_e2e_prompt_queue_dispatch_strict_instance_scoping` | `:896` to `:902` |
| C2. Integration test repo path (2 lines) | same function | `"d:/work/shared-proj",` (exactly 2 hits in the file, both in this function) | `:914`, `:932` |
| C3. Integration test assertions | same function | `Dispatching for "default" must NOT steal prompt-8159` | `:942` to `:984` |

Callers of `check_and_dispatch_enqueued_prompts` (signature unchanged, so all compile):

- `src-tauri/src/modules/scheduler.rs:412` with `None`. Do not edit it.
- `src-tauri/src/bin/agm.rs:3908` in `fn run_scheduler_single_cycle(target_instance: Option<&str>)`, which passes the raw `agm queue-scheduler [instance_id]` argument. After this step a value that is not a registry id (for example `#2` or a display name) makes the call return `Err`, which `run_scheduler_single_cycle` prints; nothing is sent. Do not edit it here: step 19 (Change F, `cmd_queue_scheduler`) resolves that argument with the instance selector.
- `src-tauri/tests/per_instance_prompt_liveness_test.rs:943` (`Some("default")`) and `:971` (`Some("default-copy-8159")`). They compile unchanged, but the test asserts the old behavior (count `1`, status `dispatched` for a repo folder that does not exist) and writes no registry, so `canonical_instance_id("default-copy-8159")` would fail. Change C rewrites that test for the new behavior.
- `auto_switcher_e2e_test.rs`, `instance_cloning_and_sync_test.rs`: no callers.

Earlier steps may have changed two spots inside this function: step 03 may have added a `source_dir: ...,` line after `image_payload: row.get(10).ok(),`, and step 08 left the two `resolve_instance_id` calls here on purpose. If your copy differs from section 4 only in those spots, still replace the whole function. Any other difference: STOP and report.

## 4. Current code (`:2051` to `:2241`), whole function

```rust
/// Bookkeep and check enqueued/backed_up prompts across projects.
/// If a project has no prompt running (idle verified), automatically pushes
/// the first enqueued prompt (FIFO) and logs the action to Audit Trail.
pub fn check_and_dispatch_enqueued_prompts(target_instance: Option<&str>) -> Result<usize, String> {
    let conn = connect_db()?;
    let now = Utc::now().timestamp();
    let run_id = format!("sched-{}", Utc::now().format("%Y%m%d-%H%M%S"));

    // 1. Gather candidate projects that have enqueued prompts ('backed_up', 'queued', 'pending')
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT project_id, instance_id, repo_path FROM active_prompts 
             WHERE status IN ('backed_up', 'queued', 'pending')
             ORDER BY updated_at ASC",
        )
        .map_err(|e| format!("Failed to query candidate enqueued projects: {}", e))?;

    let candidate_projects = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .flatten()
        .collect::<Vec<_>>();

    if candidate_projects.is_empty() {
        return Ok(0);
    }

    let mut dispatched_count = 0;

    for (project_id, inst_id, repo_path) in candidate_projects {
        if let Some(target) = target_instance {
            let norm_target = crate::modules::instance::resolve_instance_id(target)
                .unwrap_or_else(|_| target.to_string());
            let norm_inst = crate::modules::instance::resolve_instance_id(&inst_id)
                .unwrap_or_else(|_| inst_id.clone());
            let is_target_default = norm_target == "default" || norm_target == "__default__";
            let is_inst_default =
                norm_inst == "default" || norm_inst == "__default__" || norm_inst.is_empty();
            let is_match = norm_target == "all"
                || norm_inst == norm_target
                || (is_target_default && is_inst_default);
            crate::modules::logger::log_instance_prompt_audit(
                &norm_target,
                "check_and_dispatch_enqueued_prompts",
                &project_id,
                &repo_path,
                "",
                None,
                "PromptDispatcher:InstanceMatching",
                is_match,
                if is_match {
                    "PROMPT_DISPATCH_MATCHED"
                } else {
                    "PROMPT_DISPATCH_REJECTED"
                },
            );
            if !is_match {
                continue;
            }
        }

        // Check if project is currently running a prompt
        let is_running = is_prompt_running_for_project(&project_id, &inst_id)
            || is_prompt_running_for_project(&repo_path, &inst_id);

        let clean_project_name = Path::new(&repo_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| project_id.clone());

        if is_running {
            crate::modules::logger::log_info(&format!(
                "[PromptQueueScheduler] Project '{}' ({}) is currently active/busy; enqueued prompts remain queued",
                clean_project_name, project_id
            ));
            continue;
        }

        // Project is IDLE! Pick the first enqueued prompt (FIFO: earliest created_at)
        let prompt_opt: Option<ActivePrompt> = conn
            .query_row(
                "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload
                 FROM active_prompts
                 WHERE (project_id = ?1 OR repo_path = ?2)
                   AND (instance_id = ?3 OR (?3 = 'default' AND (instance_id = 'default' OR instance_id = '__default__')))
                   AND status IN ('backed_up', 'queued', 'pending')
                 ORDER BY created_at ASC
                 LIMIT 1",
                params![&project_id, &repo_path, &inst_id],
                |row| {
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
                },
            )
            .optional()
            .map_err(|e| format!("Failed to fetch enqueued prompt for {}: {}", project_id, e))?;

        let Some(prompt) = prompt_opt else {
            continue;
        };

        crate::modules::logger::log_info(&format!(
            "[PromptQueueScheduler] Project '{}' verified idle. Dispatching enqueued prompt '{}' (conv: {:?})",
            clean_project_name, prompt.id, prompt.session_id
        ));

        // Write .antigravity_resume_task.json to project directory
        let task_file = PathBuf::from(&prompt.repo_path).join(".antigravity_resume_task.json");
        let (extracted_img, img_paths) = extract_image_payload_or_path(&prompt.prompt_content);
        let final_img = prompt.image_payload.clone().or(extracted_img);
        let mut payload = resume_task_document(&prompt, "dispatched", now, &img_paths);
        if final_img.is_some() {
            payload["image_payload"] = serde_json::json!(final_img);
        }
        let _ = if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
            fs::write(&task_file, json_str).is_ok()
        } else {
            false
        };

        // Spawn prompt via agy
        let sent = spawn_prompt_via_agy(&prompt);

        // Update active_prompts status to 'dispatched'
        let _ = conn.execute(
            "UPDATE active_prompts SET status = 'dispatched', updated_at = ?1 WHERE id = ?2",
            params![now, &prompt.id],
        );

        if let Ok(mut map) = get_memory_prompts_map().lock() {
            if let Some(p) = map.get_mut(&prompt.id) {
                p.status = "dispatched".to_string();
                p.updated_at = now;
            }
        }

        // Record SchedulerFacts in Audit Trail
        let prompt_preview = if prompt.prompt_content.len() > 140 {
            format!("{}...", &prompt.prompt_content[..140])
        } else {
            prompt.prompt_content.clone()
        };

        let action_taken = if sent {
            "dispatched"
        } else {
            "resume_task_written"
        };
        let reason = format!(
            "Project verified idle; enqueued prompt pushed automatically via {}",
            action_taken
        );

        let facts = crate::modules::task_history_db::SchedulerFacts {
            scheduler_run_id: run_id.clone(),
            project_name: clean_project_name.clone(),
            repo_path: prompt.repo_path.clone(),
            prompt_id: prompt.id.clone(),
            prompt_preview,
            conversation_id: prompt.session_id.clone().unwrap_or_default(),
            action_taken: action_taken.to_string(),
            reason,
            idle_check_passed: true,
            instance_id: prompt.instance_id.clone(),
            timestamp: now,
        };

        let _ = crate::modules::task_history_db::record_scheduler_event(&facts);
        dispatched_count += 1;
    }

    invalidate_prompt_tree_cache(target_instance);
    Ok(dispatched_count)
}
```

### C1. `src-tauri/tests/per_instance_prompt_liveness_test.rs` lines 896 to 902

```rust
async fn test_e2e_prompt_queue_dispatch_strict_instance_scoping() {
    let sandbox = tempdir().expect("create sandbox tempdir");
    let prev_data_dir = std::env::var_os("ABV_DATA_DIR");
    std::env::set_var("ABV_DATA_DIR", sandbox.path());

    let conn = antigravity_tools_lib::modules::repo_db::connect_db().expect("connect repo db");
    let now = chrono::Utc::now().timestamp();
```

`setup_mock_instances_registry(sandbox_path: &Path)` is a free function in the same file (`:2111`) that writes `instances/instances.json` with `default` and `default-copy-8159`.

### C2. Same function, lines 914 and 932 (one line each, inside the two `params![...]` lists)

```rust
            "d:/work/shared-proj",
```

### C3. Same function, lines 942 to 984

```rust
    // 1. Dispatching for "default" must NOT steal prompt-8159 despite prompt-8159 having an earlier created_at
    let def_count = antigravity_tools_lib::modules::repo_db::check_and_dispatch_enqueued_prompts(
        Some("default"),
    )
    .expect("dispatch for default");
    assert_eq!(def_count, 1, "Default should dispatch its own prompt");

    let status_def: String = conn
        .query_row(
            "SELECT status FROM active_prompts WHERE id = 'prompt-def'",
            [],
            |r| r.get(0),
        )
        .expect("query status def");
    assert_eq!(status_def, "dispatched");

    let status_8159: String = conn
        .query_row(
            "SELECT status FROM active_prompts WHERE id = 'prompt-8159'",
            [],
            |r| r.get(0),
        )
        .expect("query status 8159");
    assert_eq!(
        status_8159, "queued",
        "prompt-8159 must remain queued and not stolen by default"
    );

    // 2. Dispatching for "default-copy-8159" should now dispatch prompt-8159
    let s8159_count = antigravity_tools_lib::modules::repo_db::check_and_dispatch_enqueued_prompts(
        Some("default-copy-8159"),
    )
    .expect("dispatch for 8159");
    assert_eq!(s8159_count, 1, "8159 should dispatch its queued prompt");

    let status_8159_after: String = conn
        .query_row(
            "SELECT status FROM active_prompts WHERE id = 'prompt-8159'",
            [],
            |r| r.get(0),
        )
        .expect("query status 8159 after");
    assert_eq!(status_8159_after, "dispatched");
```

## 5. New code

Replace everything from `/// Bookkeep and check enqueued/backed_up prompts across projects.` down to and including the closing `}` of `check_and_dispatch_enqueued_prompts` with:

```rust
/// Pure core of the prompt scheduler. Sends at most one `queued` row per (instance, repo),
/// oldest first. `target_instance` must already be canonical; `None` means every instance.
/// Rows without an instance id are never sent (DR-7).
pub(crate) fn dispatch_queued_prompts_in<B, F>(
    conn: &Connection,
    target_instance: Option<&str>,
    is_busy: B,
    spawn: F,
) -> Result<Vec<(ActivePrompt, DispatchResult)>, String>
where
    B: Fn(&ActivePrompt) -> bool,
    F: Fn(&ActivePrompt) -> SpawnOutcome,
{
    let timed_out = sweep_stuck_dispatching(conn, DISPATCHING_TIMEOUT_SECS);
    if timed_out > 0 {
        crate::modules::logger::log_warn(&format!(
            "[PromptQueueScheduler] {} prompt(s) stuck in dispatching were marked failed",
            timed_out
        ));
    }

    let queued = load_prompts_by_status(conn, prompt_status::QUEUED, target_instance)?;
    let mut seen_repos: HashSet<(String, String)> = HashSet::new();
    let mut outcomes = Vec::new();

    for prompt in queued {
        if prompt.instance_id.trim().is_empty() {
            continue;
        }
        let repo_key = (
            prompt.instance_id.clone(),
            normalize_path_for_compare(&prompt.repo_path),
        );
        if !seen_repos.insert(repo_key) {
            continue;
        }
        if is_busy(&prompt) {
            crate::modules::logger::log_info(&format!(
                "[PromptQueueScheduler] Repo '{}' (inst: '{}') is busy; prompt '{}' stays queued",
                prompt.repo_path, prompt.instance_id, prompt.id
            ));
            continue;
        }
        if !claim_prompt_for_dispatch(conn, &prompt.id, prompt_status::QUEUED) {
            continue;
        }
        let result = dispatch_one_prompt_with(conn, &prompt, &spawn);
        outcomes.push((prompt, result));
    }

    Ok(outcomes)
}

/// Bookkeep queued prompts across projects.
/// If a repo has no prompt running (idle verified), sends its oldest queued prompt
/// and logs the result to the Audit Trail. Returns how many prompts were really sent.
pub fn check_and_dispatch_enqueued_prompts(target_instance: Option<&str>) -> Result<usize, String> {
    let target = match target_instance.map(str::trim) {
        None => None,
        Some(raw) if raw.eq_ignore_ascii_case("all") => None,
        Some(raw) => Some(crate::modules::instance::canonical_instance_id(raw)?),
    };

    let conn = connect_db()?;
    let now = Utc::now().timestamp();
    let run_id = format!("sched-{}", Utc::now().format("%Y%m%d-%H%M%S"));

    let outcomes = dispatch_queued_prompts_in(
        &conn,
        target.as_deref(),
        |p: &ActivePrompt| {
            is_prompt_running_for_project(&p.project_id, &p.instance_id)
                || is_prompt_running_for_project(&p.repo_path, &p.instance_id)
        },
        spawn_prompt_via_agy_outcome,
    )?;

    let mut dispatched_count = 0;
    for (prompt, result) in &outcomes {
        let clean_project_name = Path::new(&prompt.repo_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| prompt.project_id.clone());

        let action_taken = match result {
            DispatchResult::Dispatched => {
                dispatched_count += 1;
                "dispatched".to_string()
            }
            DispatchResult::Failed(reason) => format!("failed:{}", reason),
            DispatchResult::WorkerBusy => "worker_busy".to_string(),
        };

        crate::modules::logger::log_info(&format!(
            "[PromptQueueScheduler] Project '{}' queued prompt '{}' (inst: '{}', conv: {:?}): {}",
            clean_project_name, prompt.id, prompt.instance_id, prompt.session_id, action_taken
        ));

        let prompt_preview = if prompt.prompt_content.chars().count() > 140 {
            format!(
                "{}...",
                prompt.prompt_content.chars().take(140).collect::<String>()
            )
        } else {
            prompt.prompt_content.clone()
        };
        let reason = format!(
            "Project verified idle; queued prompt result: {}",
            action_taken
        );

        let facts = crate::modules::task_history_db::SchedulerFacts {
            scheduler_run_id: run_id.clone(),
            project_name: clean_project_name,
            repo_path: prompt.repo_path.clone(),
            prompt_id: prompt.id.clone(),
            prompt_preview,
            conversation_id: prompt.session_id.clone().unwrap_or_default(),
            action_taken,
            reason,
            idle_check_passed: true,
            instance_id: prompt.instance_id.clone(),
            timestamp: now,
        };

        let _ = crate::modules::task_history_db::record_scheduler_event(&facts);
    }

    invalidate_prompt_tree_cache(target.as_deref());
    Ok(dispatched_count)
}
```

Notes for your review (do not paste):

- No `resolve_instance_id`, no inline resume JSON, no `'pending'`, no unconditional `dispatched` update remain in this function.
- `is_prompt_running_for_project` is called with both the project id and the repo path, exactly like before.
- The preview uses characters, not bytes, so a multi-byte prompt cannot panic at byte 140.
- `spawn_prompt_via_agy_outcome` is passed as a function item; `dispatch_one_prompt_with` receives `&spawn` (`&F` implements `Fn`).
- If clippy reports `needless_borrow` on `&spawn`, change it to `|p: &ActivePrompt| spawn(p)`. Any other clippy finding: STOP and report.

### C1. Replace the block from 4 C1 with

```rust
async fn test_e2e_prompt_queue_dispatch_strict_instance_scoping() {
    let sandbox = tempdir().expect("create sandbox tempdir");
    let prev_data_dir = std::env::var_os("ABV_DATA_DIR");
    std::env::set_var("ABV_DATA_DIR", sandbox.path());
    setup_mock_instances_registry(sandbox.path());
    let missing_repo = sandbox
        .path()
        .join("missing-shared-proj")
        .to_string_lossy()
        .to_string();

    let conn = antigravity_tools_lib::modules::repo_db::connect_db().expect("connect repo db");
    let now = chrono::Utc::now().timestamp();
```

The repo folder is a sandbox path that never exists, so the scheduler marks the claimed row `failed` / `repo_missing` and never starts `agy` from a test.

### C2. Replace each of the two lines from 4 C2 with

```rust
            missing_repo.as_str(),
```

### C3. Replace the block from 4 C3 with

```rust
    // 1. Dispatching for "default" must NOT touch prompt-8159 despite prompt-8159 having an earlier created_at.
    // The repo folder does not exist, so the claimed default prompt fails and is not counted as sent.
    let def_count = antigravity_tools_lib::modules::repo_db::check_and_dispatch_enqueued_prompts(
        Some("default"),
    )
    .expect("dispatch for default");
    assert_eq!(def_count, 0, "A prompt whose repo is missing is never counted as sent");

    let status_def: String = conn
        .query_row(
            "SELECT status FROM active_prompts WHERE id = 'prompt-def'",
            [],
            |r| r.get(0),
        )
        .expect("query status def");
    assert_eq!(status_def, "failed");

    let status_8159: String = conn
        .query_row(
            "SELECT status FROM active_prompts WHERE id = 'prompt-8159'",
            [],
            |r| r.get(0),
        )
        .expect("query status 8159");
    assert_eq!(
        status_8159, "queued",
        "prompt-8159 must remain queued and not stolen by default"
    );

    // 2. Dispatching for "default-copy-8159" claims prompt-8159 and nothing else
    let s8159_count = antigravity_tools_lib::modules::repo_db::check_and_dispatch_enqueued_prompts(
        Some("default-copy-8159"),
    )
    .expect("dispatch for 8159");
    assert_eq!(s8159_count, 0, "A prompt whose repo is missing is never counted as sent");

    let status_8159_after: String = conn
        .query_row(
            "SELECT status FROM active_prompts WHERE id = 'prompt-8159'",
            [],
            |r| r.get(0),
        )
        .expect("query status 8159 after");
    assert_eq!(status_8159_after, "failed");
```

The `// Restore env` block after it stays unchanged.

## 6. Tests

Paste at the end of `mod tests` in `src-tauri/src/modules/repo_db.rs`. They use `Connection::open_in_memory()` plus `init_tables(&conn)`, temp folders from `tempfile`, an injected busy check, and an injected spawner. Nothing starts `agy`, nothing calls `load_registry()` or `connect_db()`. `seed_dispatch_prompt` and `prompt_row_state` come from step 10.

```rust
    fn scheduler_test_repo() -> (tempfile::TempDir, String) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_string_lossy().to_string();
        (dir, path)
    }

    #[test]
    fn scheduler_ignores_backed_up_rows() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let (_repo, repo_path) = scheduler_test_repo();
        seed_dispatch_prompt(&conn, "b-1", "inst-a", &repo_path, prompt_status::BACKED_UP, 10);
        let spawn_calls = std::cell::Cell::new(0);

        let outcomes = dispatch_queued_prompts_in(&conn, None, |_| false, |_| {
            spawn_calls.set(spawn_calls.get() + 1);
            SpawnOutcome::Spawned
        })
        .unwrap();

        assert!(outcomes.is_empty());
        assert_eq!(spawn_calls.get(), 0);
        let (status, _, attempts) = prompt_row_state(&conn, "b-1");
        assert_eq!(status, prompt_status::BACKED_UP);
        assert_eq!(attempts, 0);
    }

    #[test]
    fn scheduler_marks_failed_when_spawn_fails() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let (_repo, repo_path) = scheduler_test_repo();
        seed_dispatch_prompt(&conn, "q-1", "inst-a", &repo_path, prompt_status::QUEUED, 10);

        let outcomes =
            dispatch_queued_prompts_in(&conn, None, |_| false, |_| SpawnOutcome::Failed).unwrap();

        assert_eq!(outcomes.len(), 1);
        assert_eq!(
            outcomes[0].1,
            DispatchResult::Failed(prompt_status_reason::SPAWN_FAILED)
        );
        let (status, reason, attempts) = prompt_row_state(&conn, "q-1");
        assert_eq!(status, prompt_status::FAILED);
        assert_eq!(reason.as_deref(), Some(prompt_status_reason::SPAWN_FAILED));
        assert_eq!(attempts, 1);
    }

    #[test]
    fn scheduler_marks_missing_repo_failed_without_spawning() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("gone").to_string_lossy().to_string();
        seed_dispatch_prompt(&conn, "q-1", "inst-a", &missing, prompt_status::QUEUED, 10);
        let spawn_calls = std::cell::Cell::new(0);

        let outcomes = dispatch_queued_prompts_in(&conn, None, |_| false, |_| {
            spawn_calls.set(spawn_calls.get() + 1);
            SpawnOutcome::Spawned
        })
        .unwrap();

        assert_eq!(outcomes.len(), 1);
        assert_eq!(spawn_calls.get(), 0);
        let (status, reason, _) = prompt_row_state(&conn, "q-1");
        assert_eq!(status, prompt_status::FAILED);
        assert_eq!(reason.as_deref(), Some(prompt_status_reason::REPO_MISSING));
    }

    #[test]
    fn scheduler_sends_one_prompt_per_repo_fifo() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let (_repo_a, repo_a) = scheduler_test_repo();
        let (_repo_b, repo_b) = scheduler_test_repo();
        seed_dispatch_prompt(&conn, "a-late", "inst-a", &repo_a, prompt_status::QUEUED, 20);
        seed_dispatch_prompt(&conn, "a-early", "inst-a", &repo_a, prompt_status::QUEUED, 10);
        seed_dispatch_prompt(&conn, "b-1", "inst-a", &repo_b, prompt_status::QUEUED, 15);

        let outcomes =
            dispatch_queued_prompts_in(&conn, None, |_| false, |_| SpawnOutcome::Spawned).unwrap();

        let sent: Vec<&str> = outcomes.iter().map(|(p, _)| p.id.as_str()).collect();
        assert_eq!(sent, vec!["a-early", "b-1"]);
        let (early_status, early_reason, _) = prompt_row_state(&conn, "a-early");
        assert_eq!(early_status, prompt_status::DISPATCHED);
        assert_eq!(
            early_reason.as_deref(),
            Some(prompt_status_reason::NEW_CHAT_NO_RESUME_FLAG)
        );
        let (late_status, _, late_attempts) = prompt_row_state(&conn, "a-late");
        assert_eq!(late_status, prompt_status::QUEUED);
        assert_eq!(late_attempts, 0);
    }

    #[test]
    fn scheduler_sends_same_repo_once_per_instance() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let (_repo, repo_path) = scheduler_test_repo();
        seed_dispatch_prompt(&conn, "a-1", "inst-a", &repo_path, prompt_status::QUEUED, 10);
        seed_dispatch_prompt(&conn, "b-1", "inst-b", &repo_path, prompt_status::QUEUED, 20);

        let outcomes =
            dispatch_queued_prompts_in(&conn, None, |_| false, |_| SpawnOutcome::Spawned).unwrap();

        assert_eq!(outcomes.len(), 2);
        assert_eq!(prompt_row_state(&conn, "a-1").0, prompt_status::DISPATCHED);
        assert_eq!(prompt_row_state(&conn, "b-1").0, prompt_status::DISPATCHED);
    }

    #[test]
    fn scheduler_skips_busy_repo() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let (_repo_busy, repo_busy) = scheduler_test_repo();
        let (_repo_idle, repo_idle) = scheduler_test_repo();
        seed_dispatch_prompt(&conn, "busy-1", "inst-a", &repo_busy, prompt_status::QUEUED, 10);
        seed_dispatch_prompt(&conn, "idle-1", "inst-a", &repo_idle, prompt_status::QUEUED, 20);
        let busy_path = repo_busy.clone();

        let outcomes = dispatch_queued_prompts_in(
            &conn,
            None,
            |p| p.repo_path == busy_path,
            |_| SpawnOutcome::Spawned,
        )
        .unwrap();

        let sent: Vec<&str> = outcomes.iter().map(|(p, _)| p.id.as_str()).collect();
        assert_eq!(sent, vec!["idle-1"]);
        let (status, _, attempts) = prompt_row_state(&conn, "busy-1");
        assert_eq!(status, prompt_status::QUEUED);
        assert_eq!(attempts, 0);
    }

    #[test]
    fn scheduler_filters_by_target_instance() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let (_repo_a, repo_a) = scheduler_test_repo();
        let (_repo_b, repo_b) = scheduler_test_repo();
        seed_dispatch_prompt(&conn, "a-1", "inst-a", &repo_a, prompt_status::QUEUED, 10);
        seed_dispatch_prompt(&conn, "b-1", "inst-b", &repo_b, prompt_status::QUEUED, 20);

        let outcomes = dispatch_queued_prompts_in(
            &conn,
            Some("inst-b"),
            |_| false,
            |_| SpawnOutcome::Spawned,
        )
        .unwrap();

        let sent: Vec<&str> = outcomes.iter().map(|(p, _)| p.id.as_str()).collect();
        assert_eq!(sent, vec!["b-1"]);
        assert_eq!(prompt_row_state(&conn, "a-1").0, prompt_status::QUEUED);
    }

    #[test]
    fn scheduler_skips_rows_without_instance() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let (_repo, repo_path) = scheduler_test_repo();
        seed_dispatch_prompt(&conn, "orphan-1", "", &repo_path, prompt_status::QUEUED, 10);

        let outcomes =
            dispatch_queued_prompts_in(&conn, None, |_| false, |_| SpawnOutcome::Spawned).unwrap();

        assert!(outcomes.is_empty());
        let (status, _, attempts) = prompt_row_state(&conn, "orphan-1");
        assert_eq!(status, prompt_status::QUEUED);
        assert_eq!(attempts, 0);
    }

    #[test]
    fn scheduler_releases_worker_busy_row_to_queued() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let (_repo, repo_path) = scheduler_test_repo();
        seed_dispatch_prompt(&conn, "q-1", "inst-a", &repo_path, prompt_status::QUEUED, 10);

        let outcomes =
            dispatch_queued_prompts_in(&conn, None, |_| false, |_| SpawnOutcome::WorkerBusy)
                .unwrap();

        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].1, DispatchResult::WorkerBusy);
        let (status, _, attempts) = prompt_row_state(&conn, "q-1");
        assert_eq!(status, prompt_status::QUEUED);
        assert_eq!(attempts, 0);
    }

    #[test]
    fn scheduler_times_out_stuck_dispatching_rows() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let (_repo, repo_path) = scheduler_test_repo();
        seed_dispatch_prompt(&conn, "stuck-1", "inst-a", &repo_path, prompt_status::DISPATCHING, 10);

        let outcomes =
            dispatch_queued_prompts_in(&conn, None, |_| false, |_| SpawnOutcome::Spawned).unwrap();

        assert!(outcomes.is_empty());
        let (status, reason, _) = prompt_row_state(&conn, "stuck-1");
        assert_eq!(status, prompt_status::FAILED);
        assert_eq!(reason.as_deref(), Some(prompt_status_reason::DISPATCH_TIMEOUT));
    }
```

If the compiler cannot infer the closure argument type in `|p| p.repo_path == busy_path`, write `|p: &ActivePrompt| p.repo_path == busy_path`. Do the same for any `|_|` closure that fails to infer: `|_: &ActivePrompt|`.

## 7. Gate

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 scheduler_ dispatch_one_prompt claim_prompt_for_dispatch; cd ..
```

Then run:

```text
gitmap aum search "resolve_instance_id" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "fs::write(&task_file" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "'pending'" src-tauri/src/modules/repo_db.rs --ext .rs
```

- The first must return 0 hits.
- The second must return exactly 1 hit, inside `write_resume_handoff_in` (step 09).
- The third must not show any hit inside `check_and_dispatch_enqueued_prompts` or `dispatch_queued_prompts_in`. Hits in `resend_running_commands_for_instance` and in the old test near the end of `mod tests` are expected; step 13 owns the resend one.

## 8. Commit

Stage exactly: `src-tauri/src/modules/repo_db.rs`, `src-tauri/tests/per_instance_prompt_liveness_test.rs`.

```text
Fix: prompts - scheduler dispatches only queued rows
```

## 9. Done when

- [ ] `dispatch_queued_prompts_in` exists, takes `&Connection`, an injected busy check and an injected spawner.
- [ ] The scheduler reads only `status = 'queued'` rows, oldest first, one per (instance, normalized repo).
- [ ] Rows with an empty `instance_id` are never claimed.
- [ ] A row is `dispatched` only when the spawner returned `Spawned`; otherwise `failed` with a reason, or back to `queued` on `WorkerBusy`.
- [ ] Stuck `dispatching` rows older than `DISPATCHING_TIMEOUT_SECS` become `failed` / `dispatch_timeout` at the start of each cycle.
- [ ] `check_and_dispatch_enqueued_prompts` keeps its signature; `scheduler.rs` and `agm.rs` are untouched.
- [ ] Callers were searched in `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file; `test_e2e_prompt_queue_dispatch_strict_instance_scoping` is updated (Change C) and `gitmap aum search "d:/work/shared-proj" src-tauri/tests/per_instance_prompt_liveness_test.rs --ext .rs` returns 0 hits.
- [ ] The three gitmap checks in section 7 pass.
- [ ] Gate exits 0. Only the two files in section 8 are staged.
