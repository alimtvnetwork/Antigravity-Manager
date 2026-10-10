//! Repo DB: goal heartbeat

use super::models::{ActivePrompt, PromptGoalHeartbeatConfig};
use super::prompts_crud::save_or_requeue_prompt;
use super::state::get_active_agy_workers;
use chrono::Utc;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use uuid::Uuid;

/// Start an active prompt goal writing current date & time every 5 seconds to a specific file
pub fn start_prompt_goal_heartbeat(
    instance_id: &str,
    data_dir: &str,
    repo_path: &str,
    heartbeat_file: &str,
    prompt: &str,
    interval_secs: u64,
) -> Result<PromptGoalHeartbeatConfig, String> {
    let ws_dir = PathBuf::from(repo_path);
    if !ws_dir.exists() {
        // Justification: workspace directory creation is best-effort; subsequent writes surface their own errors
        crate::error::record_ignored(fs::create_dir_all(&ws_dir), "create workspace directory");
    }

    let interval = if interval_secs == 0 { 5 } else { interval_secs };
    let prompt_text = if prompt.trim().is_empty() {
        "Continuous 5-second prompt goal heartbeat verification".to_string()
    } else {
        prompt.trim().to_string()
    };

    let prompt_id = format!("goal-{}-{}", instance_id, Uuid::new_v4().simple());
    let now = Utc::now().timestamp();

    let account_email = crate::modules::instance::load_registry()
        .ok()
        .and_then(|r| r.instances.into_iter().find(|i| i.id == instance_id))
        .and_then(|i| i.bound_email)
        .unwrap_or_else(|| "unbound".to_string());

    let hb_target = if Path::new(heartbeat_file).is_absolute() {
        PathBuf::from(heartbeat_file)
    } else {
        ws_dir.join(heartbeat_file)
    };
    if let Some(p) = hb_target.parent() {
        // Justification: heartbeat parent directory creation is best-effort; the heartbeat write below surfaces failures
        crate::error::record_ignored(fs::create_dir_all(p), "create heartbeat parent directory");
    }
    let hb_str = hb_target.to_string_lossy().to_string();

    let mut cfg = PromptGoalHeartbeatConfig {
        prompt_id: prompt_id.clone(),
        instance_id: instance_id.to_string(),
        data_dir: data_dir.to_string(),
        repo_path: repo_path.to_string(),
        prompt_content: prompt_text.clone(),
        heartbeat_file: hb_str.clone(),
        interval_secs: interval,
        account_email: account_email.clone(),
        worker_pid: None,
        started_at: now,
        last_heartbeat_at: Some(now),
    };

    // Save .antigravity_goal_heartbeat.json inside project repo
    let config_path = ws_dir.join(".antigravity_goal_heartbeat.json");
    let json_cfg = serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
    fs::write(&config_path, json_cfg).map_err(|e| format!("Failed to write goal config: {}", e))?;

    // Also update AGM_INSTANCE_STATUS.md inside project repo for visual verification
    update_workspace_status_file(
        &ws_dir,
        instance_id,
        &account_email,
        &prompt_text,
        &hb_str,
        "RUNNING",
    );

    // Write initial heartbeat line immediately
    append_goal_heartbeat_line(&hb_str, instance_id, &account_email, 0, &prompt_text);

    // Save prompt to active_prompts with status 'running'
    let active_p = ActivePrompt {
        id: prompt_id.clone(),
        project_id: ws_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "project".to_string()),
        instance_id: instance_id.to_string(),
        repo_path: repo_path.to_string(),
        prompt_content: prompt_text.clone(),
        model: Some("gemini-2.5-pro".to_string()),
        session_id: Some(format!("goal-session-{}", instance_id)),
        status: "running".to_string(),
        created_at: now,
        updated_at: now,
        image_payload: None,
    };
    // Justification: prompt tracking save is auxiliary; the goal worker spawn is the authoritative outcome
    crate::error::record_ignored(
        save_or_requeue_prompt(&active_p),
        "save prompt goal before worker spawn",
    );

    // Spawn detached goal worker process
    if let Ok(cur_exe) = std::env::current_exe() {
        let mut cmd = Command::new(&cur_exe);
        cmd.args([
            "prompts",
            "goal-worker",
            "--instance",
            instance_id,
            &format!("--user-data-dir={}", data_dir),
            "--project",
            repo_path,
            "--heartbeat-file",
            &hb_str,
            "--interval",
            &interval.to_string(),
            "--prompt",
            &prompt_text,
        ]);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000 | 0x00000200); // CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP
        }

        if let Ok(child) = cmd.spawn() {
            let pid = child.id();
            cfg.worker_pid = Some(pid);
            // Justification: goal-config PID write is auxiliary bookkeeping; the worker is already spawned
            crate::error::record_ignored(
                fs::write(
                    &config_path,
                    serde_json::to_string_pretty(&cfg).unwrap_or_default(),
                ),
                "persist goal worker PID to config",
            );
            crate::modules::logger::log_info(&format!(
                "[PromptGoal] Spawned goal worker PID {} for instance '{}' (interval: {}s, file: {})",
                pid, instance_id, interval, hb_str
            ));
        }
    }

    Ok(cfg)
}

/// Loop run by `agm prompts goal-worker` in the background
pub fn run_prompt_goal_worker_loop(
    instance_id: &str,
    _data_dir: &str,
    repo_path: &str,
    heartbeat_file: &str,
    interval_secs: u64,
    prompt: &str,
) {
    let ws_dir = PathBuf::from(repo_path);
    let config_path = ws_dir.join(".antigravity_goal_heartbeat.json");
    let interval = if interval_secs == 0 { 5 } else { interval_secs };
    let my_pid = std::process::id();

    loop {
        if !config_path.exists() {
            break;
        }

        let current_email = crate::modules::instance::load_registry()
            .ok()
            .and_then(|r| r.instances.into_iter().find(|i| i.id == instance_id))
            .and_then(|i| i.bound_email)
            .unwrap_or_else(|| "unbound".to_string());

        append_goal_heartbeat_line(heartbeat_file, instance_id, &current_email, my_pid, prompt);

        let now = Utc::now().timestamp();
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(mut cfg) = serde_json::from_str::<PromptGoalHeartbeatConfig>(&content) {
                cfg.account_email = current_email.clone();
                cfg.worker_pid = Some(my_pid);
                cfg.last_heartbeat_at = Some(now);
                // Justification: heartbeat config rewrite is best-effort; the heartbeat line below is the authoritative signal
                crate::error::record_ignored(
                    fs::write(
                        &config_path,
                        serde_json::to_string_pretty(&cfg).unwrap_or_default(),
                    ),
                    "rewrite goal heartbeat config",
                );
            }
        }

        update_workspace_status_file(
            &ws_dir,
            instance_id,
            &current_email,
            prompt,
            heartbeat_file,
            "RUNNING",
        );

        std::thread::sleep(Duration::from_secs(interval));
    }
}

fn append_goal_heartbeat_line(
    file_path: &str,
    instance_id: &str,
    email: &str,
    pid: u32,
    prompt: &str,
) {
    let now = chrono::Local::now();
    let ts_str = now.format("%Y-%m-%d %H:%M:%S%.3f %:z").to_string();
    let line = format!(
        "[{}] instance={} account={} pid={} status=RUNNING goal=\"{}\"\n",
        ts_str, instance_id, email, pid, prompt
    );
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)
    {
        // Justification: heartbeat line write is best-effort; the worker loop continues and retries on the next interval
        crate::error::record_ignored(
            file.write_all(line.as_bytes()),
            "append goal heartbeat line",
        );
        // Justification: heartbeat flush is best-effort; the worker loop continues and retries on the next interval
        crate::error::record_ignored(file.flush(), "flush goal heartbeat line");
    }
}

fn update_workspace_status_file(
    ws_dir: &Path,
    instance_id: &str,
    email: &str,
    prompt: &str,
    heartbeat_file: &str,
    status: &str,
) {
    let status_file = ws_dir.join("AGM_INSTANCE_STATUS.md");
    let now = chrono::Local::now()
        .format("%Y-%m-%d %H:%M:%S %:z")
        .to_string();
    let content = format!(
        "# Antigravity Instance & Running Prompt Goal Status\n\n\
        - **Instance ID**: `{}`\n\
        - **Bound Account**: `{}`\n\
        - **Goal Status**: `{}`\n\
        - **Prompt**: {}\n\
        - **Heartbeat File**: `{}`\n\
        - **Heartbeat Interval**: 5 seconds\n\
        - **Last Verified Timestamp**: `{}`\n\n\
        *This file is updated every 5 seconds by the active AGM prompt goal watchdog.*\n",
        instance_id, email, status, prompt, heartbeat_file, now
    );
    // Justification: workspace status file is informational; the heartbeat line is the authoritative signal
    crate::error::record_ignored(
        fs::write(&status_file, content),
        "write workspace status file",
    );
}

/// Stop any running prompt goal worker processes for an instance
pub fn stop_prompt_goal_workers_for_instance(instance_id: &str, data_dir: &str) -> usize {
    let mut stopped = 0;
    let mut sys = sysinfo::System::new();
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        sysinfo::ProcessRefreshKind::new()
            .with_cmd(sysinfo::UpdateKind::OnlyIfNotSet)
            .with_exe(sysinfo::UpdateKind::OnlyIfNotSet),
    );

    {
        let mut workers = get_active_agy_workers().lock().unwrap();
        let inst_prefix = format!("{}:", instance_id);
        workers.retain(|k, _| !k.starts_with(&inst_prefix));
    }

    let norm_data = data_dir.to_lowercase().replace('\\', "/");
    let clean_data = norm_data.trim_end_matches('/');

    for (pid, proc) in sys.processes() {
        let cmd_str = proc
            .cmd()
            .iter()
            .map(|a| a.to_string_lossy().to_lowercase().replace('\\', "/"))
            .collect::<Vec<String>>()
            .join(" ");

        let is_goal_worker = cmd_str.contains("goal-worker");
        let matches_inst = cmd_str.contains(&format!("--instance {}", instance_id.to_lowercase()))
            || (!clean_data.is_empty() && cmd_str.contains(clean_data));

        if is_goal_worker && matches_inst {
            let pid_u32 = pid.as_u32();
            #[cfg(target_os = "windows")]
            {
                // Justification: killing the worker is best-effort; the process may already be gone and the registry entry was already removed
                crate::error::record_ignored(
                    Command::new("taskkill")
                        .args(["/F", "/T", "/PID", &pid_u32.to_string()])
                        .creation_flags(0x08000000)
                        .output(),
                    "kill goal worker process (taskkill)",
                );
            }
            #[cfg(not(target_os = "windows"))]
            {
                // Justification: killing the worker is best-effort; the process may already be gone and the registry entry was already removed
                crate::error::record_ignored(
                    Command::new("kill")
                        .args(["-9", &pid_u32.to_string()])
                        .output(),
                    "kill goal worker process (kill)",
                );
            }
            stopped += 1;
        }
    }

    crate::modules::logger::log_info(&format!(
        "[PromptGoal] Stopped {} prompt goal worker(s) for instance '{}'",
        stopped, instance_id
    ));
    stopped
}

/// Inspect prompt goal status across workspaces for an instance
pub fn inspect_prompt_goal_status(
    _instance_id: &str,
    workspaces: &[String],
) -> (bool, Option<String>, Option<String>) {
    let now = Utc::now().timestamp();
    for ws in workspaces {
        let ws_p = Path::new(ws);
        let cfg_p = ws_p.join(".antigravity_goal_heartbeat.json");
        if cfg_p.exists() {
            if let Ok(c) = fs::read_to_string(&cfg_p) {
                if let Ok(cfg) = serde_json::from_str::<PromptGoalHeartbeatConfig>(&c) {
                    let hb_p = Path::new(&cfg.heartbeat_file);
                    let mut is_fresh = false;
                    let mut last_line = None;
                    if hb_p.exists() {
                        if let Ok(meta) = fs::metadata(hb_p) {
                            if let Ok(modified) = meta.modified() {
                                let dt: chrono::DateTime<Utc> = modified.into();
                                let age = now - dt.timestamp();
                                if age <= (cfg.interval_secs as i64 + 4) {
                                    is_fresh = true;
                                }
                            }
                        }
                        if let Ok(content) = fs::read_to_string(hb_p) {
                            last_line = content
                                .lines()
                                .rev()
                                .find(|l| !l.trim().is_empty())
                                .map(|s| s.to_string());
                        }
                    }
                    return (is_fresh, Some(cfg.heartbeat_file), last_line);
                }
            }
        }
    }
    (false, None, None)
}

/// Automatically check and re-invoke prompt goals if not actively running
pub fn ensure_prompt_goals_running_for_instance(instance_id: &str) -> usize {
    let mut reinvoked = 0;
    let workspaces = if let Ok(registry) = crate::modules::instance::load_registry() {
        if let Some(inst) = registry.instances.iter().find(|i| i.id == instance_id) {
            crate::modules::instance::get_instance_workspace_folders(instance_id, &inst.data_dir)
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let now = Utc::now().timestamp();
    for ws in &workspaces {
        let ws_p = Path::new(ws);
        let cfg_p = ws_p.join(".antigravity_goal_heartbeat.json");
        if cfg_p.exists() {
            if let Ok(c) = fs::read_to_string(&cfg_p) {
                if let Ok(cfg) = serde_json::from_str::<PromptGoalHeartbeatConfig>(&c) {
                    let hb_p = Path::new(&cfg.heartbeat_file);
                    let mut is_alive = false;
                    if hb_p.exists() {
                        if let Ok(meta) = fs::metadata(hb_p) {
                            if let Ok(mod_time) = meta.modified() {
                                let dt: chrono::DateTime<Utc> = mod_time.into();
                                if (now - dt.timestamp()) <= (cfg.interval_secs as i64 + 4) {
                                    is_alive = true;
                                }
                            }
                        }
                    }
                    if !is_alive {
                        crate::modules::logger::log_info(&format!(
                            "[PromptGoal] Prompt goal heartbeat stopped in '{}', auto-reinvoking for instance '{}'...",
                            ws, instance_id
                        ));
                        if let Ok(registry) = crate::modules::instance::load_registry() {
                            if let Some(inst) =
                                registry.instances.iter().find(|i| i.id == instance_id)
                            {
                                if let Ok(_) = start_prompt_goal_heartbeat(
                                    instance_id,
                                    &inst.data_dir,
                                    ws,
                                    &cfg.heartbeat_file,
                                    &cfg.prompt_content,
                                    cfg.interval_secs,
                                ) {
                                    reinvoked += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    reinvoked
}
