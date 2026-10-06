# Step 10: Status Machine, Atomic Claim, and One Dispatch Step

Goal: add the prompt status constants, an atomic claim (`queued`/`backed_up` to `dispatching`), a finish step (`dispatched` or `failed` with a reason), a release step for a busy worker, a timeout sweep, a status-based row loader, and `dispatch_one_prompt`. `spawn_prompt_via_agy` uses the canonical instance id in its worker key and in its HOME branch, and reports why it did not send through a new `SpawnOutcome`.

DR-2: `agy` has no "resume conversation by id" flag that we know of. Keep the current `-p` call. On a successful send, `status_reason = 'new_chat_no_resume_flag'`. Do not add any flag.

This step adds the building blocks only. The scheduler switches to them in step 11; restore and resend switch in steps 12 and 13.

## 1. Depends on

- Step 03 (columns `status_reason`, `attempts` on `active_prompts`; `ActivePrompt.source_dir`).
- Step 08 (`normalize_path_for_compare` `pub(crate)`; test helper `prompt_tests_registry`).
- Step 09 (`write_resume_handoff_in`, `ResumeHandoffOutcome`; test helpers `handoff_test_prompt`, `seed_running_project`).

## 2. Files you may edit

- `src-tauri/src/modules/repo_db.rs`

## 3. Find it

```text
gitmap aum search "pub fn spawn_prompt_via_agy" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "fn get_memory_prompts_map" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "pub fn write_resume_handoff_in" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "status_reason TEXT" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "attempts INTEGER NOT NULL DEFAULT 0" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "spawn_prompt_via_agy(" src-tauri/src --ext .rs
gitmap aum search "spawn_prompt_via_agy(" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "spawn_prompt_via_agy" src-tauri/tests/auto_switcher_e2e_test.rs --ext .rs
gitmap aum search "spawn_prompt_via_agy" src-tauri/tests/instance_cloning_and_sync_test.rs --ext .rs
gitmap aum search "spawn_prompt_via_agy" src-tauri/tests/per_instance_prompt_liveness_test.rs --ext .rs
gitmap aum search "mod prompt_status|SpawnOutcome|DispatchResult|DISPATCHING_TIMEOUT_SECS|fn claim_prompt_for_dispatch|fn finish_prompt_dispatch|fn release_prompt_claim|fn sweep_stuck_dispatching|fn load_prompts_by_status|fn dispatch_one_prompt|fn agy_worker_key|fn sync_memory_prompt_status" src-tauri/src --ext .rs
gitmap aum search "mod prompt_status|SpawnOutcome|DispatchResult|DISPATCHING_TIMEOUT_SECS|fn claim_prompt_for_dispatch|fn finish_prompt_dispatch|fn release_prompt_claim|fn sweep_stuck_dispatching|fn load_prompts_by_status|fn dispatch_one_prompt|fn agy_worker_key|fn sync_memory_prompt_status" src-tauri/src/bin/agm.rs --ext .rs
```

The 3rd, 4th and 5th commands must each return a hit (steps 03 and 09). If not, STOP. The three test-file searches and the last two (name collision) searches must return 0 hits (checked 2026-10-06).

| Change | Anchor | Unique search literal | Line hint (2026-10-06) |
|---|---|---|---|
| A. New status machine block | insert directly above the doc comment of `spawn_prompt_via_agy` | `/// Helper to spawn \`agy\` CLI to execute a prompt in a workspace.` | `:3104` |
| B. Replace `spawn_prompt_via_agy` | `pub fn spawn_prompt_via_agy(prompt: &ActivePrompt) -> bool` | `An agy worker (PID: {}) is already active for workspace` | `:3104` to `:3227` |
| C. Tests | `mod tests` | end of module | last line |

Callers of `spawn_prompt_via_agy` (all keep compiling, the `bool` signature stays): `repo_db.rs` `dispatch_running_prompts`, `check_and_dispatch_enqueued_prompts`, `resend_running_commands_for_instance`, `auto_resume_recent_prompts`, the AGM sequence dispatch near `:5113`; `src-tauri/src/modules/telegram_inbound.rs:2450`; `src-tauri/src/bin/agm.rs` `cmd_prompt_dispatch` (`let _ = repo_db::spawn_prompt_via_agy(&active_p);`, about `:3184` before step 09 moved it). No caller in `src-tauri/tests`. Do not edit them in this step.

How the in-memory prompt map works (read it before editing): `get_memory_prompts_map()` (`:29`) returns a process-global `Mutex<HashMap<String, ActivePrompt>>` keyed by prompt row id. Writers insert after a DB write (`map.insert(p.id.clone(), p)`), and status changes update `status` and `updated_at` in place (`map.get_mut(&id)`). Readers (`get_memory_prompt`, `is_prompt_running_for_project` Gate 1) trust it. Every status change in this step must call `sync_memory_prompt_status` so the map never disagrees with the row.

## 4. Current code

### A. Insertion point (`:3100` to `:3106`)

```rust
    }
    label
}

/// Helper to spawn `agy` CLI to execute a prompt in a workspace.
/// Strips prompt envelope wrappers and executes cleanly via -p without failing on GUI trajectory IDs.
pub fn spawn_prompt_via_agy(prompt: &ActivePrompt) -> bool {
```

### B. `spawn_prompt_via_agy` (`:3104` to `:3227`), whole function

```rust
/// Helper to spawn `agy` CLI to execute a prompt in a workspace.
/// Strips prompt envelope wrappers and executes cleanly via -p without failing on GUI trajectory IDs.
pub fn spawn_prompt_via_agy(prompt: &ActivePrompt) -> bool {
    let ws_dir = PathBuf::from(&prompt.repo_path);
    if !ws_dir.exists() {
        crate::modules::logger::log_warn(&format!(
            "[RepoDB] Workspace directory does not exist: '{}', skipping agy spawn",
            prompt.repo_path
        ));
        return false;
    }

    let clean_prompt = extract_clean_user_prompt(&prompt.prompt_content);
    if clean_prompt.trim().is_empty() {
        crate::modules::logger::log_warn(&format!(
            "[RepoDB] Prompt content for '{}' is empty, skipping agy spawn",
            prompt.id
        ));
        return false;
    }

    let ws_key = format!("{}:{}", prompt.instance_id, prompt.repo_path);
    {
        let mut workers = get_active_agy_workers().lock().unwrap();
        if let Some(&existing_pid) = workers.get(&ws_key) {
            let mut sys = sysinfo::System::new();
            let target_pid = sysinfo::Pid::from_u32(existing_pid);
            sys.refresh_processes_specifics(
                sysinfo::ProcessesToUpdate::Some(&[target_pid]),
                sysinfo::ProcessRefreshKind::new().with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
            );
            if sys.process(target_pid).is_some() {
                crate::modules::logger::log_info(&format!(
                    "[RepoDB] An agy worker (PID: {}) is already active for workspace '{}'. Prompt '{}' was not sent again.",
                    existing_pid, ws_key, prompt.id
                ));
                return false;
            } else {
                workers.remove(&ws_key);
            }
        }
    }

    if let Some(agy_bin) = crate::modules::process::get_antigravity_cli_executable_path() {
        let mut cmd = std::process::Command::new(&agy_bin);
        cmd.current_dir(&ws_dir);
        cmd.arg("--dangerously-skip-permissions");

        if clean_prompt.len() <= 24000 {
            cmd.arg("-p").arg(&clean_prompt);
        } else {
            cmd.arg("-p").arg(&clean_prompt[..24000]);
        }

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        if prompt.instance_id != "default" && !prompt.instance_id.is_empty() {
            if let Ok(inst_home) =
                crate::modules::instance::get_instance_home_dir(&prompt.instance_id)
            {
                #[cfg(target_os = "windows")]
                {
                    cmd.env("USERPROFILE", &inst_home);
                }
                cmd.env("HOME", &inst_home);
                cmd.env("SSH_CONNECTION", "127.0.0.1 50000 127.0.0.1 22");
                cmd.env("SSH_CLIENT", "127.0.0.1 50000 22");
                cmd.env("SSH_TTY", "pty/0");
                cmd.env("WSL_DISTRO_NAME", "antigravity-isolated");
                cmd.env("DOCKER_CONTAINER", "1");

                if let Ok(registry) = crate::modules::instance::load_registry() {
                    if let Some(inst) = registry
                        .instances
                        .iter()
                        .find(|i| i.id == prompt.instance_id)
                    {
                        if let Some(ref acc_id) = inst.bound_account_id {
                            if let Ok(acc) = crate::modules::account::load_account(acc_id) {
                                cmd.env("JETSKI_OAUTH_TOKEN", &acc.token.access_token);
                                cmd.env("GEMINI_CLI_OAUTH_TOKEN", &acc.token.access_token);
                            }
                        }
                    }
                }
            }
        }

        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());

        match cmd.spawn() {
            Ok(child) => {
                let child_pid = child.id();
                {
                    let mut workers = get_active_agy_workers().lock().unwrap();
                    workers.insert(ws_key, child_pid);
                }
                crate::modules::logger::log_info(&format!(
                    "[RepoDB] Dispatched agy execution for prompt '{}' (PID: {:?}, inst: '{}') in '{}': {:.60}...",
                    prompt.id, child_pid, prompt.instance_id, prompt.repo_path, clean_prompt
                ));
                true
            }
            Err(e) => {
                crate::modules::logger::log_error(&format!(
                    "[RepoDB] Failed to spawn agy execution for prompt '{}': {}",
                    prompt.id, e
                ));
                false
            }
        }
    } else {
        crate::modules::logger::log_warn(
            "[RepoDB] agy executable not found, cannot resume prompt via CLI",
        );
        false
    }
}
```

## 5. New code

### A. Insert directly above `/// Helper to spawn \`agy\` CLI to execute a prompt in a workspace.`

```rust
/// Every value the `active_prompts.status` column may hold.
pub mod prompt_status {
    pub const RUNNING: &str = "running";
    pub const BACKED_UP: &str = "backed_up";
    pub const QUEUED: &str = "queued";
    pub const DISPATCHING: &str = "dispatching";
    pub const DISPATCHED: &str = "dispatched";
    pub const FAILED: &str = "failed";
    pub const ORPHANED: &str = "orphaned";
}

/// Values written to `active_prompts.status_reason`.
pub mod prompt_status_reason {
    pub const NEW_CHAT_NO_RESUME_FLAG: &str = "new_chat_no_resume_flag";
    pub const REPO_MISSING: &str = "repo_missing";
    pub const HANDOFF_WRITE_FAILED: &str = "handoff_write_failed";
    pub const AGY_MISSING: &str = "agy_missing";
    pub const SPAWN_FAILED: &str = "spawn_failed";
    pub const EMPTY_PROMPT: &str = "empty_prompt";
    pub const INVALID_INSTANCE: &str = "invalid_instance";
    pub const DISPATCH_TIMEOUT: &str = "dispatch_timeout";
}

pub const DISPATCHING_TIMEOUT_SECS: i64 = 120;

/// Why `spawn_prompt_via_agy_outcome` did or did not start an `agy` process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnOutcome {
    Spawned,
    WorkerBusy,
    AgyMissing,
    RepoMissing,
    EmptyPrompt,
    InvalidInstance,
    Failed,
}

/// Result of `dispatch_one_prompt` for one claimed row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchResult {
    Dispatched,
    Failed(&'static str),
    WorkerBusy,
}

fn sync_memory_prompt_status(id: &str, status: &str, now: i64) {
    if let Ok(mut map) = get_memory_prompts_map().lock() {
        if let Some(p) = map.get_mut(id) {
            p.status = status.to_string();
            p.updated_at = now;
        }
    }
}

/// Moves one row from `from_status` to `dispatching`. Returns true only if this caller won the claim.
pub fn claim_prompt_for_dispatch(conn: &Connection, id: &str, from_status: &str) -> bool {
    let now = Utc::now().timestamp();
    let changed = conn
        .execute(
            "UPDATE active_prompts
             SET status = ?1, attempts = attempts + 1, updated_at = ?2
             WHERE id = ?3 AND status = ?4",
            params![prompt_status::DISPATCHING, now, id, from_status],
        )
        .unwrap_or(0);
    let is_claimed = changed == 1;
    if is_claimed {
        sync_memory_prompt_status(id, prompt_status::DISPATCHING, now);
    }
    is_claimed
}

/// Writes `dispatched` (sent) or `failed` (not sent) for a row in `dispatching`.
/// A failed row always gets a reason; a sent row may carry one (DR-2).
pub fn finish_prompt_dispatch(conn: &Connection, id: &str, sent: bool, reason: Option<&str>) -> bool {
    let now = Utc::now().timestamp();
    let (status, status_reason) = if sent {
        (prompt_status::DISPATCHED, reason)
    } else {
        (
            prompt_status::FAILED,
            Some(reason.unwrap_or(prompt_status_reason::SPAWN_FAILED)),
        )
    };
    let changed = conn
        .execute(
            "UPDATE active_prompts SET status = ?1, status_reason = ?2, updated_at = ?3
             WHERE id = ?4 AND status = ?5",
            params![status, status_reason, now, id, prompt_status::DISPATCHING],
        )
        .unwrap_or(0);
    let is_finished = changed == 1;
    if is_finished {
        sync_memory_prompt_status(id, status, now);
    }
    is_finished
}

/// Puts a claimed row back to `back_to_status` without counting the attempt (worker busy).
pub fn release_prompt_claim(conn: &Connection, id: &str, back_to_status: &str) -> bool {
    let now = Utc::now().timestamp();
    let changed = conn
        .execute(
            "UPDATE active_prompts
             SET status = ?1, attempts = MAX(attempts - 1, 0), updated_at = ?2
             WHERE id = ?3 AND status = ?4",
            params![back_to_status, now, id, prompt_status::DISPATCHING],
        )
        .unwrap_or(0);
    let is_released = changed == 1;
    if is_released {
        sync_memory_prompt_status(id, back_to_status, now);
    }
    is_released
}

/// Moves `dispatching` rows older than `max_age_secs` to `failed` with `dispatch_timeout`.
pub fn sweep_stuck_dispatching(conn: &Connection, max_age_secs: i64) -> usize {
    let now = Utc::now().timestamp();
    let cutoff = now - max_age_secs;
    let mut stuck_ids: Vec<String> = Vec::new();
    if let Ok(mut stmt) =
        conn.prepare("SELECT id FROM active_prompts WHERE status = ?1 AND updated_at < ?2")
    {
        if let Ok(rows) = stmt.query_map(params![prompt_status::DISPATCHING, cutoff], |r| {
            r.get::<_, String>(0)
        }) {
            stuck_ids.extend(rows.flatten());
        }
    }
    let changed = conn
        .execute(
            "UPDATE active_prompts SET status = ?1, status_reason = ?2, updated_at = ?3
             WHERE status = ?4 AND updated_at < ?5",
            params![
                prompt_status::FAILED,
                prompt_status_reason::DISPATCH_TIMEOUT,
                now,
                prompt_status::DISPATCHING,
                cutoff
            ],
        )
        .unwrap_or(0);
    for id in &stuck_ids {
        sync_memory_prompt_status(id, prompt_status::FAILED, now);
    }
    changed
}

/// Rows with one status, oldest first. `instance_id = None` means every instance.
pub(crate) fn load_prompts_by_status(
    conn: &Connection,
    status: &str,
    instance_id: Option<&str>,
) -> Result<Vec<ActivePrompt>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload, source_dir
             FROM active_prompts
             WHERE status = ?1 AND (?2 IS NULL OR instance_id = ?2)
             ORDER BY created_at ASC, id ASC",
        )
        .map_err(|e| format!("Failed to prepare prompt status query: {}", e))?;
    let rows: Vec<ActivePrompt> = stmt
        .query_map(params![status, instance_id], |row| {
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
        })
        .map_err(|e| format!("Failed to query prompts by status: {}", e))?
        .flatten()
        .collect();
    Ok(rows)
}

/// Sends one prompt that the caller already claimed with `claim_prompt_for_dispatch`.
/// `prompt.status` must still hold the status the row had before the claim.
pub fn dispatch_one_prompt(conn: &Connection, prompt: &ActivePrompt) -> DispatchResult {
    dispatch_one_prompt_with(conn, prompt, spawn_prompt_via_agy_outcome)
}

/// Pure core of `dispatch_one_prompt`; tests inject `spawn` instead of starting `agy`.
pub(crate) fn dispatch_one_prompt_with<F>(
    conn: &Connection,
    prompt: &ActivePrompt,
    spawn: F,
) -> DispatchResult
where
    F: Fn(&ActivePrompt) -> SpawnOutcome,
{
    if !Path::new(&prompt.repo_path).exists() {
        finish_prompt_dispatch(conn, &prompt.id, false, Some(prompt_status_reason::REPO_MISSING));
        return DispatchResult::Failed(prompt_status_reason::REPO_MISSING);
    }
    if let ResumeHandoffOutcome::Failed(reason) =
        write_resume_handoff_in(conn, prompt, prompt_status::DISPATCHED)
    {
        crate::modules::logger::log_error(&format!(
            "[RepoDB] Resume hand-off failed for prompt '{}': {}",
            prompt.id, reason
        ));
        finish_prompt_dispatch(
            conn,
            &prompt.id,
            false,
            Some(prompt_status_reason::HANDOFF_WRITE_FAILED),
        );
        return DispatchResult::Failed(prompt_status_reason::HANDOFF_WRITE_FAILED);
    }
    let failure = match spawn(prompt) {
        SpawnOutcome::Spawned => {
            finish_prompt_dispatch(
                conn,
                &prompt.id,
                true,
                Some(prompt_status_reason::NEW_CHAT_NO_RESUME_FLAG),
            );
            return DispatchResult::Dispatched;
        }
        SpawnOutcome::WorkerBusy => {
            release_prompt_claim(conn, &prompt.id, &prompt.status);
            return DispatchResult::WorkerBusy;
        }
        SpawnOutcome::AgyMissing => prompt_status_reason::AGY_MISSING,
        SpawnOutcome::RepoMissing => prompt_status_reason::REPO_MISSING,
        SpawnOutcome::EmptyPrompt => prompt_status_reason::EMPTY_PROMPT,
        SpawnOutcome::InvalidInstance => prompt_status_reason::INVALID_INSTANCE,
        SpawnOutcome::Failed => prompt_status_reason::SPAWN_FAILED,
    };
    finish_prompt_dispatch(conn, &prompt.id, false, Some(failure));
    DispatchResult::Failed(failure)
}

/// Key of the active `agy` worker map: canonical instance id plus normalized repo path.
pub(crate) fn agy_worker_key(canonical_instance_id: &str, repo_path: &str) -> String {
    format!(
        "{}:{}",
        canonical_instance_id,
        normalize_path_for_compare(repo_path)
    )
}

pub(crate) fn agy_worker_key_in(
    registry: &crate::modules::instance::InstanceRegistry,
    raw_instance_id: &str,
    repo_path: &str,
) -> Result<String, String> {
    if raw_instance_id.trim().is_empty() {
        return Err("prompt has no instance id".to_string());
    }
    let inst = crate::modules::instance::canonical_instance_id_in(registry, raw_instance_id)?;
    Ok(agy_worker_key(&inst, repo_path))
}

```

### B. Replace the whole `spawn_prompt_via_agy` function with

```rust
/// Helper to spawn `agy` CLI to execute a prompt in a workspace.
/// Strips prompt envelope wrappers and executes cleanly via -p without failing on GUI trajectory IDs.
pub fn spawn_prompt_via_agy(prompt: &ActivePrompt) -> bool {
    spawn_prompt_via_agy_outcome(prompt) == SpawnOutcome::Spawned
}

pub fn spawn_prompt_via_agy_outcome(prompt: &ActivePrompt) -> SpawnOutcome {
    let ws_dir = PathBuf::from(&prompt.repo_path);
    if !ws_dir.exists() {
        crate::modules::logger::log_warn(&format!(
            "[RepoDB] Workspace directory does not exist: '{}', skipping agy spawn",
            prompt.repo_path
        ));
        return SpawnOutcome::RepoMissing;
    }

    let clean_prompt = extract_clean_user_prompt(&prompt.prompt_content);
    if clean_prompt.trim().is_empty() {
        crate::modules::logger::log_warn(&format!(
            "[RepoDB] Prompt content for '{}' is empty, skipping agy spawn",
            prompt.id
        ));
        return SpawnOutcome::EmptyPrompt;
    }

    let registry = crate::modules::instance::load_registry().unwrap_or_default();
    let ws_key = match agy_worker_key_in(&registry, &prompt.instance_id, &prompt.repo_path) {
        Ok(key) => key,
        Err(e) => {
            crate::modules::logger::log_warn(&format!(
                "[RepoDB] Prompt '{}' not sent: {}",
                prompt.id, e
            ));
            return SpawnOutcome::InvalidInstance;
        }
    };
    let inst = crate::modules::instance::canonical_instance_id_in(&registry, &prompt.instance_id)
        .unwrap_or_else(|_| prompt.instance_id.clone());
    let default_id = crate::modules::instance::canonical_instance_id_in(&registry, "default")
        .unwrap_or_else(|_| "default".to_string());

    {
        let mut workers = get_active_agy_workers().lock().unwrap();
        if let Some(&existing_pid) = workers.get(&ws_key) {
            let mut sys = sysinfo::System::new();
            let target_pid = sysinfo::Pid::from_u32(existing_pid);
            sys.refresh_processes_specifics(
                sysinfo::ProcessesToUpdate::Some(&[target_pid]),
                sysinfo::ProcessRefreshKind::new().with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
            );
            if sys.process(target_pid).is_some() {
                crate::modules::logger::log_info(&format!(
                    "[RepoDB] An agy worker (PID: {}) is already active for workspace '{}'. Prompt '{}' was not sent again.",
                    existing_pid, ws_key, prompt.id
                ));
                return SpawnOutcome::WorkerBusy;
            } else {
                workers.remove(&ws_key);
            }
        }
    }

    let Some(agy_bin) = crate::modules::process::get_antigravity_cli_executable_path() else {
        crate::modules::logger::log_warn(
            "[RepoDB] agy executable not found, cannot resume prompt via CLI",
        );
        return SpawnOutcome::AgyMissing;
    };

    let mut cmd = std::process::Command::new(&agy_bin);
    cmd.current_dir(&ws_dir);
    cmd.arg("--dangerously-skip-permissions");

    if clean_prompt.len() <= 24000 {
        cmd.arg("-p").arg(&clean_prompt);
    } else {
        cmd.arg("-p").arg(&clean_prompt[..24000]);
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    if inst != default_id {
        let inst_home = match crate::modules::instance::get_instance_home_dir(&inst) {
            Ok(home) => home,
            Err(e) => {
                crate::modules::logger::log_error(&format!(
                    "[RepoDB] Prompt '{}' not sent: no home for instance '{}': {}",
                    prompt.id, inst, e
                ));
                return SpawnOutcome::Failed;
            }
        };
        #[cfg(target_os = "windows")]
        {
            cmd.env("USERPROFILE", &inst_home);
        }
        cmd.env("HOME", &inst_home);
        cmd.env("SSH_CONNECTION", "127.0.0.1 50000 127.0.0.1 22");
        cmd.env("SSH_CLIENT", "127.0.0.1 50000 22");
        cmd.env("SSH_TTY", "pty/0");
        cmd.env("WSL_DISTRO_NAME", "antigravity-isolated");
        cmd.env("DOCKER_CONTAINER", "1");

        if let Some(cfg) = registry.instances.iter().find(|i| i.id == inst) {
            if let Some(ref acc_id) = cfg.bound_account_id {
                if let Ok(acc) = crate::modules::account::load_account(acc_id) {
                    cmd.env("JETSKI_OAUTH_TOKEN", &acc.token.access_token);
                    cmd.env("GEMINI_CLI_OAUTH_TOKEN", &acc.token.access_token);
                }
            }
        }
    }

    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());

    match cmd.spawn() {
        Ok(child) => {
            let child_pid = child.id();
            {
                let mut workers = get_active_agy_workers().lock().unwrap();
                workers.insert(ws_key, child_pid);
            }
            crate::modules::logger::log_info(&format!(
                "[RepoDB] Dispatched agy execution for prompt '{}' (PID: {:?}, inst: '{}') in '{}': {:.60}...",
                prompt.id, child_pid, inst, prompt.repo_path, clean_prompt
            ));
            SpawnOutcome::Spawned
        }
        Err(e) => {
            crate::modules::logger::log_error(&format!(
                "[RepoDB] Failed to spawn agy execution for prompt '{}': {}",
                prompt.id, e
            ));
            SpawnOutcome::Failed
        }
    }
}
```

What changed and why (for your review, do not paste this list):

- Worker key is `agy_worker_key(canonical id, normalized path)`; `"__default__"` and `"default"` give the same key.
- HOME branch is `inst != default_id`, so `"__default__"` and `""` can no longer reach the named-instance branch. An empty `instance_id` is refused (`InvalidInstance`).
- A named instance whose home cannot be resolved returns `Failed` instead of silently running under the host home.
- The registry is loaded once per call and reused for the bound account lookup.
- `is_prompt_running_for_project` Gate 2 splits worker keys with `split_once(':')`; a key such as `default:d:/work/app` still splits into `default` and `d:/work/app`.

## 6. Tests

Paste at the end of `mod tests` in `src-tauri/src/modules/repo_db.rs`. They use `Connection::open_in_memory()` plus `init_tables(&conn)`, temp folders from `tempfile`, and an injected spawner. Nothing starts `agy`, nothing calls `load_registry()`. Helpers from steps 08 and 09 (`prompt_tests_registry`, `handoff_test_prompt`) are reused. `seed_dispatch_prompt` and `prompt_row_state` are reused by step 11; define them once here.

```rust
    fn seed_dispatch_prompt(
        conn: &Connection,
        id: &str,
        instance_id: &str,
        repo_path: &str,
        status: &str,
        created_at: i64,
    ) {
        conn.execute(
            "INSERT INTO active_prompts
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at)
             VALUES (?1, 'proj', ?2, ?3, 'keep going', NULL, 'conv-1', ?4, ?5, ?5)",
            params![id, instance_id, repo_path, status, created_at],
        )
        .unwrap();
    }

    fn prompt_row_state(conn: &Connection, id: &str) -> (String, Option<String>, i64) {
        conn.query_row(
            "SELECT status, status_reason, attempts FROM active_prompts WHERE id = ?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap()
    }

    #[test]
    fn claim_prompt_for_dispatch_is_exclusive() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        seed_dispatch_prompt(&conn, "q-1", "inst-a", "D:/work/app", prompt_status::QUEUED, 10);

        assert!(claim_prompt_for_dispatch(&conn, "q-1", prompt_status::QUEUED));
        assert!(!claim_prompt_for_dispatch(&conn, "q-1", prompt_status::QUEUED));

        let (status, _, attempts) = prompt_row_state(&conn, "q-1");
        assert_eq!(status, prompt_status::DISPATCHING);
        assert_eq!(attempts, 1);
    }

    #[test]
    fn claim_prompt_for_dispatch_requires_matching_status() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        seed_dispatch_prompt(&conn, "r-1", "inst-a", "D:/work/app", prompt_status::RUNNING, 10);

        assert!(!claim_prompt_for_dispatch(&conn, "r-1", prompt_status::QUEUED));
        let (status, _, attempts) = prompt_row_state(&conn, "r-1");
        assert_eq!(status, prompt_status::RUNNING);
        assert_eq!(attempts, 0);
    }

    #[test]
    fn finish_prompt_dispatch_failed_always_has_reason() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        seed_dispatch_prompt(&conn, "q-1", "inst-a", "D:/work/app", prompt_status::QUEUED, 10);
        assert!(claim_prompt_for_dispatch(&conn, "q-1", prompt_status::QUEUED));

        assert!(finish_prompt_dispatch(&conn, "q-1", false, None));
        let (status, reason, _) = prompt_row_state(&conn, "q-1");
        assert_eq!(status, prompt_status::FAILED);
        assert_eq!(reason.as_deref(), Some(prompt_status_reason::SPAWN_FAILED));
        assert!(!finish_prompt_dispatch(&conn, "q-1", true, None));
    }

    #[test]
    fn stuck_dispatching_moves_to_failed_after_timeout() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        seed_dispatch_prompt(&conn, "old", "inst-a", "D:/work/app", prompt_status::DISPATCHING, 10);
        seed_dispatch_prompt(&conn, "fresh", "inst-a", "D:/work/app2", prompt_status::QUEUED, 10);
        assert!(claim_prompt_for_dispatch(&conn, "fresh", prompt_status::QUEUED));

        let swept = sweep_stuck_dispatching(&conn, DISPATCHING_TIMEOUT_SECS);

        assert_eq!(swept, 1);
        let (old_status, old_reason, _) = prompt_row_state(&conn, "old");
        assert_eq!(old_status, prompt_status::FAILED);
        assert_eq!(old_reason.as_deref(), Some(prompt_status_reason::DISPATCH_TIMEOUT));
        let (fresh_status, _, _) = prompt_row_state(&conn, "fresh");
        assert_eq!(fresh_status, prompt_status::DISPATCHING);
    }

    #[test]
    fn release_prompt_claim_restores_previous_status() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        seed_dispatch_prompt(&conn, "b-1", "inst-a", "D:/work/app", prompt_status::BACKED_UP, 10);
        assert!(claim_prompt_for_dispatch(&conn, "b-1", prompt_status::BACKED_UP));

        assert!(release_prompt_claim(&conn, "b-1", prompt_status::BACKED_UP));
        let (status, _, attempts) = prompt_row_state(&conn, "b-1");
        assert_eq!(status, prompt_status::BACKED_UP);
        assert_eq!(attempts, 0);
    }

    #[test]
    fn load_prompts_by_status_filters_instance_and_orders_fifo() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        seed_dispatch_prompt(&conn, "a-late", "inst-a", "D:/work/app", prompt_status::QUEUED, 30);
        seed_dispatch_prompt(&conn, "a-early", "inst-a", "D:/work/app", prompt_status::QUEUED, 10);
        seed_dispatch_prompt(&conn, "b-1", "inst-b", "D:/work/app", prompt_status::QUEUED, 20);
        seed_dispatch_prompt(&conn, "a-backed", "inst-a", "D:/work/app", prompt_status::BACKED_UP, 5);

        let only_a = load_prompts_by_status(&conn, prompt_status::QUEUED, Some("inst-a")).unwrap();
        let ids: Vec<&str> = only_a.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, vec!["a-early", "a-late"]);

        let all = load_prompts_by_status(&conn, prompt_status::QUEUED, None).unwrap();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn dispatch_one_prompt_marks_repo_missing_without_spawning() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("gone").to_string_lossy().to_string();
        seed_dispatch_prompt(&conn, "q-1", "inst-a", &missing, prompt_status::QUEUED, 10);
        let prompt = load_prompts_by_status(&conn, prompt_status::QUEUED, None).unwrap().remove(0);
        assert!(claim_prompt_for_dispatch(&conn, "q-1", prompt_status::QUEUED));
        let spawn_calls = std::cell::Cell::new(0);

        let result = dispatch_one_prompt_with(&conn, &prompt, |_| {
            spawn_calls.set(spawn_calls.get() + 1);
            SpawnOutcome::Spawned
        });

        assert_eq!(result, DispatchResult::Failed(prompt_status_reason::REPO_MISSING));
        assert_eq!(spawn_calls.get(), 0);
        let (status, reason, _) = prompt_row_state(&conn, "q-1");
        assert_eq!(status, prompt_status::FAILED);
        assert_eq!(reason.as_deref(), Some(prompt_status_reason::REPO_MISSING));
    }

    #[test]
    fn dispatch_one_prompt_success_records_no_resume_flag() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let repo = tempfile::tempdir().unwrap();
        let repo_path = repo.path().to_string_lossy().to_string();
        seed_dispatch_prompt(&conn, "q-1", "inst-a", &repo_path, prompt_status::QUEUED, 10);
        let prompt = load_prompts_by_status(&conn, prompt_status::QUEUED, None).unwrap().remove(0);
        assert!(claim_prompt_for_dispatch(&conn, "q-1", prompt_status::QUEUED));

        let result = dispatch_one_prompt_with(&conn, &prompt, |_| SpawnOutcome::Spawned);

        assert_eq!(result, DispatchResult::Dispatched);
        let (status, reason, attempts) = prompt_row_state(&conn, "q-1");
        assert_eq!(status, prompt_status::DISPATCHED);
        assert_eq!(reason.as_deref(), Some(prompt_status_reason::NEW_CHAT_NO_RESUME_FLAG));
        assert_eq!(attempts, 1);
        assert!(repo.path().join(RESUME_TASK_FILE_NAME).exists());
    }

    #[test]
    fn dispatch_one_prompt_maps_spawn_failures_to_reasons() {
        let cases = [
            (SpawnOutcome::AgyMissing, prompt_status_reason::AGY_MISSING),
            (SpawnOutcome::Failed, prompt_status_reason::SPAWN_FAILED),
            (SpawnOutcome::InvalidInstance, prompt_status_reason::INVALID_INSTANCE),
            (SpawnOutcome::EmptyPrompt, prompt_status_reason::EMPTY_PROMPT),
        ];
        for (outcome, expected_reason) in cases {
            let conn = Connection::open_in_memory().unwrap();
            init_tables(&conn).unwrap();
            let repo = tempfile::tempdir().unwrap();
            let repo_path = repo.path().to_string_lossy().to_string();
            seed_dispatch_prompt(&conn, "q-1", "inst-a", &repo_path, prompt_status::QUEUED, 10);
            let prompt = load_prompts_by_status(&conn, prompt_status::QUEUED, None).unwrap().remove(0);
            assert!(claim_prompt_for_dispatch(&conn, "q-1", prompt_status::QUEUED));

            let result = dispatch_one_prompt_with(&conn, &prompt, |_| outcome);

            assert_eq!(result, DispatchResult::Failed(expected_reason));
            let (status, reason, _) = prompt_row_state(&conn, "q-1");
            assert_eq!(status, prompt_status::FAILED);
            assert_eq!(reason.as_deref(), Some(expected_reason));
        }
    }

    #[test]
    fn dispatch_one_prompt_worker_busy_returns_row_to_previous_status() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let repo = tempfile::tempdir().unwrap();
        let repo_path = repo.path().to_string_lossy().to_string();
        seed_dispatch_prompt(&conn, "b-1", "inst-a", &repo_path, prompt_status::BACKED_UP, 10);
        let prompt = load_prompts_by_status(&conn, prompt_status::BACKED_UP, None).unwrap().remove(0);
        assert!(claim_prompt_for_dispatch(&conn, "b-1", prompt_status::BACKED_UP));

        let result = dispatch_one_prompt_with(&conn, &prompt, |_| SpawnOutcome::WorkerBusy);

        assert_eq!(result, DispatchResult::WorkerBusy);
        let (status, _, attempts) = prompt_row_state(&conn, "b-1");
        assert_eq!(status, prompt_status::BACKED_UP);
        assert_eq!(attempts, 0);
    }

    #[test]
    fn spawn_worker_key_is_canonical() {
        let registry = prompt_tests_registry(&["default", "inst-a"]);
        let from_alias = agy_worker_key_in(&registry, "__default__", "D:\\Work\\App\\").unwrap();
        let from_default = agy_worker_key_in(&registry, "default", "d:/work/app").unwrap();
        assert_eq!(from_alias, from_default);
        assert_eq!(from_default, "default:d:/work/app");

        let named = agy_worker_key_in(&registry, "INST-A", "d:/work/app").unwrap();
        assert_eq!(named, "inst-a:d:/work/app");
        assert_ne!(named, from_default);

        assert!(agy_worker_key_in(&registry, "", "d:/work/app").is_err());
        assert!(agy_worker_key_in(&registry, "ghost", "d:/work/app").is_err());
    }
```

If clippy reports `unused` for `prompt_status::ORPHANED` or `prompt_status::RUNNING`: the module is `pub` and the file has `#![allow(dead_code)]` at the top, so it should not. If it does, STOP and report; do not delete the constant.

## 7. Gate

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 claim_prompt_for_dispatch finish_prompt_dispatch stuck_dispatching release_prompt_claim load_prompts_by_status dispatch_one_prompt spawn_worker_key; cd ..
```

## 8. Commit

Stage only `src-tauri/src/modules/repo_db.rs`.

```text
Fix: prompts - status machine, atomic claim and canonical agy spawn
```

## 9. Done when

- [ ] `prompt_status`, `prompt_status_reason`, `DISPATCHING_TIMEOUT_SECS`, `SpawnOutcome`, `DispatchResult` exist.
- [ ] `claim_prompt_for_dispatch`, `finish_prompt_dispatch`, `release_prompt_claim`, `sweep_stuck_dispatching` each update the in-memory map through `sync_memory_prompt_status` when the row changed.
- [ ] `load_prompts_by_status` reads `source_dir`.
- [ ] `dispatch_one_prompt` calls `write_resume_handoff_in` and never writes `dispatched` unless the spawner returned `Spawned`.
- [ ] `spawn_prompt_via_agy` still returns `bool` and all existing callers compile unchanged.
- [ ] Callers of `spawn_prompt_via_agy` were searched in `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file; none needed an edit.
- [ ] The worker key comes from `agy_worker_key`; the HOME branch compares with the canonical default id.
- [ ] No `agy` flag was added.
- [ ] Gate exits 0. Only `src-tauri/src/modules/repo_db.rs` is staged.
