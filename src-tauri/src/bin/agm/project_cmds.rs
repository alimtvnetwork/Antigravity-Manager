//! project_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use chrono::Utc;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use uuid::Uuid;

pub(crate) fn cmd_recreate_project(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Recreate Project Workspace:");
        println!("  agm recreate-project [repo_paths...]");
        println!("\nDescription:");
        println!("  Re-scans, unbinds stale locks, and recreates workspace configuration for target repos.");
        println!("\nAliases: agm recreate-project, agm recreate");
        println!("\nExamples:");
        println!("  agm recreate-project                # Recreate current repository workspace");
        println!("  agm recreate d:\\work\\my-project      # Recreate specific project workspace");
        return;
    }

    let explicit: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with('-'))
        .flat_map(|a| a.split(','))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if !explicit.is_empty() {
        for t in explicit {
            recreate_single_workspace(&t);
        }
        return;
    }

    let git_root = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty());

    let target = git_root.unwrap_or_else(|| {
        env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string())
    });
    recreate_single_workspace(&target);
}

pub(crate) fn cmd_recreate(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        cmd_recreate_project(args);
        return;
    }

    let targets: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with('-'))
        .flat_map(|a| a.split(','))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if targets.is_empty() {
        cmd_recreate_project(args);
        return;
    }
    for t in targets {
        recreate_single_workspace(&t);
    }
}

pub(crate) fn recreate_single_workspace(target_spec: &str) {
    let trimmed_spec = target_spec.trim();
    if trimmed_spec.is_empty() {
        return;
    }

    // Resolve target_spec to a concrete folder path (supports <seq>, #seq, id, repo_name, or path)
    let projects = repo_db::list_running_projects().unwrap_or_default();
    let clean_seq = trimmed_spec.trim_start_matches('#');
    let seq_matched_path = if let Ok(seq_num) = clean_seq.parse::<usize>() {
        if seq_num >= 1 && seq_num <= projects.len() {
            Some(PathBuf::from(&projects[seq_num - 1].repo_path))
        } else {
            None
        }
    } else {
        None
    };

    let resolved_path: PathBuf = if let Some(p) = seq_matched_path {
        p
    } else {
        let direct = PathBuf::from(trimmed_spec);
        if direct.exists() {
            direct.canonicalize().unwrap_or(direct)
        } else if let Some(found) = projects.into_iter().find(|p| {
            p.repo_name.eq_ignore_ascii_case(trimmed_spec)
                || p.id.eq_ignore_ascii_case(trimmed_spec)
                || p.repo_path
                    .to_lowercase()
                    .contains(&trimmed_spec.to_lowercase())
        }) {
            PathBuf::from(found.repo_path)
        } else if let Ok(cwd) = env::current_dir() {
            if let Some(parent) = cwd.parent() {
                let sibling = parent.join(trimmed_spec);
                if sibling.exists() {
                    sibling
                } else {
                    direct
                }
            } else {
                direct
            }
        } else {
            direct
        }
    };

    let path_str = resolved_path.to_string_lossy().to_string();
    let clean_str = path_str.trim_start_matches(r"\\?\").to_string();
    let repo_name = resolved_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| trimmed_spec.to_string());

    println!(
        "[*] Recreating project workspace '{}' ({})...",
        repo_name, clean_str
    );

    // 1. Remove .antigravity_resume_task.json if present
    let resume_file = resolved_path.join(".antigravity_resume_task.json");
    if resume_file.exists() {
        let _ = fs::remove_file(&resume_file);
    }

    // 2. Clean matching workspaceStorage folders across instances
    if let Ok(reg) = instance::load_registry() {
        let norm_target = clean_str.to_lowercase().replace('\\', "/");
        for inst in &reg.instances {
            let ws_root = PathBuf::from(&inst.data_dir)
                .join("User")
                .join("workspaceStorage");
            if ws_root.is_dir() {
                if let Ok(entries) = fs::read_dir(&ws_root) {
                    for entry in entries.flatten() {
                        let ws_json = entry.path().join("workspace.json");
                        if ws_json.is_file() {
                            if let Ok(content) = fs::read_to_string(&ws_json) {
                                let content_norm = content
                                    .to_lowercase()
                                    .replace("%20", " ")
                                    .replace("%3a", ":")
                                    .replace('\\', "/");
                                if content_norm.contains(&norm_target) {
                                    let _ = fs::remove_dir_all(entry.path());
                                    println!(
                                        "    [✓] Purged cached workspaceStorage: {:?}",
                                        entry.file_name()
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Prune matching conversation .db files and brain/<cid> directories under ~/.gemini/antigravity/
    if let Some(ag_root) = agy_cleaner::get_gemini_base_dir() {
        let norm_target = clean_str.to_lowercase().replace('\\', "/");
        let repo_lower = repo_name.to_lowercase();
        let all_convs = agy_cleaner::scan_conversations(5);
        let mut pruned_convs = 0usize;
        for conv in all_convs {
            if conv.is_preserved {
                continue;
            }
            let uris_norm = conv
                .workspace_uris
                .to_lowercase()
                .replace("%20", " ")
                .replace("%3a", ":")
                .replace('\\', "/");
            let is_match = (!norm_target.is_empty() && uris_norm.contains(&norm_target))
                || (!repo_lower.is_empty() && uris_norm.contains(&repo_lower));
            if is_match {
                let db_p = PathBuf::from(&conv.db_path);
                if db_p.exists() {
                    let _ = fs::remove_file(&db_p);
                }
                let brain_dir = ag_root.join("brain").join(&conv.conversation_id);
                if brain_dir.exists() {
                    let _ = fs::remove_dir_all(&brain_dir);
                }
                pruned_convs += 1;
            }
        }
        if pruned_convs > 0 {
            println!(
                "    [✓] Pruned {} previous conversation(s) & brain cache(s) for '{}'",
                pruned_convs, repo_name
            );
        }
    }

    // 4. Reset repo_prompts.db state and seed fresh initial conversation prompt
    let initial_prompt = "read all files and memory to understand the project";
    let now = chrono::Utc::now().timestamp();
    let inst_id = instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string());
    let proj_slug = repo_name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    let proj_id = if proj_slug.is_empty() {
        "workspace".to_string()
    } else {
        proj_slug
    };
    let prompt_id = uuid::Uuid::new_v4().to_string();
    let session_id = uuid::Uuid::new_v4().to_string();

    if let Ok(conn) = repo_db::connect_db() {
        let _ = conn.execute(
            "DELETE FROM active_prompts WHERE LOWER(repo_path) = LOWER(?1) OR LOWER(project_id) LIKE LOWER(?2)",
            rusqlite::params![&clean_str, format!("%{}%", repo_name)],
        );
        let _ = conn.execute(
            "DELETE FROM running_projects WHERE LOWER(repo_path) = LOWER(?1) OR LOWER(repo_name) = LOWER(?2)",
            rusqlite::params![&clean_str, &repo_name],
        );
        let _ = conn.execute(
            "INSERT OR REPLACE INTO running_projects \
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, NULL, 1, ?5, ?5)",
            rusqlite::params![&proj_id, &inst_id, &repo_name, &clean_str, now],
        );
        let _ = conn.execute(
            "INSERT INTO active_prompts \
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, 'gemini-3.8-flash-high', ?6, 'dispatched', ?7, ?7)",
            rusqlite::params![
                &prompt_id,
                &proj_id,
                &inst_id,
                &clean_str,
                initial_prompt,
                &session_id,
                now
            ],
        );
        println!(
            "    [✓] Cleared old state and seeded initial prompt '{}' in repo_prompts.db",
            initial_prompt
        );
    }

    if resolved_path.exists() {
        let task_payload = serde_json::json!({
            "prompt_id": prompt_id,
            "project_id": proj_id,
            "instance_id": inst_id,
            "repo_path": clean_str,
            "session_id": session_id,
            "prompt_content": initial_prompt,
            "auto_boot": true,
            "dispatched_at": now,
        });
        if let Ok(js) = serde_json::to_string_pretty(&task_payload) {
            let _ = fs::write(&resume_file, js);
        }
    }

    // 5. Re-open project in Antigravity IDE (agy)
    if resolved_path.exists() {
        let launched = Command::new("agy")
            .arg(&clean_str)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok();
        if launched {
            println!(
                "    [✓] Spawned fresh Antigravity IDE session for '{}'",
                clean_str
            );
        } else if let Ok(exe) =
            antigravity_tools_lib::modules::process::detect_antigravity_with_diagnostics(None)
        {
            let _ = Command::new(exe)
                .arg("--new-window")
                .arg(&clean_str)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            println!(
                "    [✓] Launched Antigravity binary directly for '{}'",
                clean_str
            );
        }
    }

    println!("[SUCCESS] Project '{}' recreated cleanly.", repo_name);
}
