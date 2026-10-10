//! prompt_import_export — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::repo_db::ActivePrompt;
use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use chrono::Utc;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub(crate) fn cmd_prompts_export(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Prompts Export:");
        println!("  agm prompts-export [N] [-f <file.json>] [--wc <W>]");
        println!("\nDescription:");
        println!(
            "  Exports stored and in-flight prompts from the split SQLite database into JSON,"
        );
        println!(
            "  preserving full prompt text, attached image payloads/paths, models, and timestamps."
        );
        println!("\nAliases: agm prompts-export, agm pe");
        println!("\nOptions:");
        println!(
            "    [N]                 Maximum number of recent prompts to export (default: 50)"
        );
        println!("    -f, --file <path>   Target JSON export file path (default: agm-<repo>-prompts.json)");
        println!("    --wc <W>            Maximum word count for prompt previews");
        println!("\nExamples:");
        println!(
            "  agm prompts-export                  # Export up to 50 prompts to default JSON file"
        );
        println!("  agm pe 100 -f my-prompts.json       # Export last 100 prompts to custom file");
        println!("  agm pe 10                           # Export last 10 prompts");
        return;
    }

    use base64::engine::general_purpose::STANDARD;

    let mut limit_n: usize = 50;
    let mut target_file: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-f" || arg == "--file" || arg == "-file" {
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                target_file = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if !arg.starts_with('-') {
            if let Ok(n) = arg.parse::<usize>() {
                limit_n = n.max(1);
            } else if target_file.is_none() {
                target_file = Some(arg.clone());
            }
        }
        i += 1;
    }

    let slug = crate::common::derive_current_repo_slug();
    let default_filename = format!("agm-{}-prompts.json", slug);
    let out_path = match target_file {
        Some(ref f) if !f.trim().is_empty() => PathBuf::from(f.trim()),
        _ => PathBuf::from(&default_filename),
    };

    let mut prompts = repo_db::list_all_prompts().unwrap_or_default();
    if let Ok(cwd) = env::current_dir() {
        let cwd_norm = cwd.to_string_lossy().to_lowercase().replace('\\', "/");
        let matched: Vec<repo_db::ActivePrompt> = prompts
            .iter()
            .filter(|p| {
                let rp = p.repo_path.to_lowercase().replace('\\', "/");
                !rp.is_empty() && (cwd_norm.starts_with(&rp) || rp.starts_with(&cwd_norm))
            })
            .cloned()
            .collect();
        if !matched.is_empty() {
            prompts = matched;
        }
    }

    prompts.truncate(limit_n);
    prompts.reverse(); // ASC stack order

    let exported_items: Vec<serde_json::Value> = prompts
        .iter()
        .enumerate()
        .map(|(idx, p)| {
            let b64_image = p.image_payload.as_ref().map(|img| {
                if img.starts_with("data:image/") || img.len() > 128 {
                    img.clone()
                } else if Path::new(img).exists() {
                    fs::read(img)
                        .map(|bytes| format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
                        .unwrap_or_else(|_| STANDARD.encode(img.as_bytes()))
                } else {
                    STANDARD.encode(img.as_bytes())
                }
            });
            serde_json::json!({
                "seq": idx + 1,
                "id": p.id,
                "project_id": p.project_id,
                "instance_id": p.instance_id,
                "repo_path": p.repo_path,
                "prompt_content": p.prompt_content,
                "model": p.model,
                "session_id": p.session_id,
                "status": p.status,
                "created_at": p.created_at,
                "updated_at": p.updated_at,
                "image_base64": b64_image,
            })
        })
        .collect();

    let bundle = serde_json::json!({
        "schema": "agm-prompts-export-v1",
        "repo_slug": slug,
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "count": exported_items.len(),
        "prompts": exported_items,
    });

    let json_str = serde_json::to_string_pretty(&bundle).unwrap_or_else(|_| "{}".to_string());
    if let Err(e) = fs::write(&out_path, &json_str) {
        eprintln!(
            "[ERROR] Failed to write prompts export to {:?}: {}",
            out_path, e
        );
        std::process::exit(1);
    }

    println!(
        "[SUCCESS] Exported {} prompt(s) (with Base64 image encoding) to {:?}",
        exported_items.len(),
        out_path
    );
}

pub(crate) fn cmd_prompts_import(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Prompts Import:");
        println!("  agm prompts-import [-f <file.json>] [-y]");
        println!("\nDescription:");
        println!(
            "  Imports previously exported prompts from JSON back into the local SQLite database"
        );
        println!("  and re-enqueues them for execution.");
        println!("\nAliases: agm prompts-import, agm pi");
        println!("\nOptions:");
        println!("    -f, --file <path>   Source JSON file to import from (default: agm-<repo>-prompts.json)");
        println!("    -y, --yes           Bypass interactive confirmation prompt");
        println!("\nExamples:");
        println!("  agm prompts-import                  # Import from default JSON file");
        println!(
            "  agm pi -f backup.json -y            # Import from specific file without prompting"
        );
        return;
    }

    let mut explicit_file: Option<String> = None;
    let auto_yes = args.iter().any(|a| a == "--yes" || a == "-y");

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-f" || arg == "--file" || arg == "-file" {
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                explicit_file = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if !arg.starts_with('-') && explicit_file.is_none() {
            explicit_file = Some(arg.clone());
        }
        i += 1;
    }

    let slug = crate::common::derive_current_repo_slug();
    let default_filename = format!("agm-{}-prompts.json", slug);

    let mut files_to_import: Vec<PathBuf> = Vec::new();
    if let Some(ref f) = explicit_file {
        files_to_import.push(PathBuf::from(f));
    } else {
        let default_path = PathBuf::from(&default_filename);
        if default_path.exists() {
            files_to_import.push(default_path.clone());
        }

        // Scan current directory for other prompt JSON files
        if let Ok(entries) = fs::read_dir(".") {
            let mut other_jsons = Vec::new();
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("json") {
                    let fname = p
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    if fname != default_filename
                        && (fname.contains("prompt") || fname.starts_with("agm-"))
                    {
                        other_jsons.push(p);
                    }
                }
            }

            if !other_jsons.is_empty() {
                if auto_yes {
                    files_to_import.extend(other_jsons);
                } else {
                    println!(
                        "[*] Found {} additional prompt JSON file(s) in current folder:",
                        other_jsons.len()
                    );
                    for oj in &other_jsons {
                        println!("    - {}", oj.display());
                    }
                    print!("Do you want to import and rerun these additional JSON files as well? [y/N]: ");
                    let _ = io::stdout().flush();
                    let mut answer = String::new();
                    if io::stdin().read_line(&mut answer).is_ok() {
                        let trimmed = answer.trim().to_lowercase();
                        if trimmed == "y" || trimmed == "yes" {
                            files_to_import.extend(other_jsons);
                        }
                    }
                }
            }
        }
    }

    if files_to_import.is_empty() {
        eprintln!(
            "[ERROR] No prompt JSON file found to import (expected '{}' or specify -f <path>).",
            default_filename
        );
        std::process::exit(1);
    }

    let conn = match repo_db::connect_db() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ERROR] Failed to connect to repo_prompts.db: {}", e);
            std::process::exit(1);
        }
    };

    let cwd_str = env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());
    let now = chrono::Utc::now().timestamp();
    let mut total_imported = 0usize;

    for file_path in &files_to_import {
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[WARN] Failed to read {:?}: {}", file_path, e);
                continue;
            }
        };
        let parsed: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[WARN] Invalid JSON in {:?}: {}", file_path, e);
                continue;
            }
        };

        let arr = parsed
            .get("prompts")
            .and_then(|v| v.as_array())
            .or_else(|| parsed.as_array());

        let Some(items) = arr else {
            continue;
        };

        for item in items {
            let prompt_content = item
                .get("prompt_content")
                .or_else(|| item.get("prompt"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if prompt_content.trim().is_empty() {
                continue;
            }

            let id = uuid::Uuid::new_v4().to_string();
            let proj_id = item
                .get("project_id")
                .and_then(|v| v.as_str())
                .unwrap_or(&slug)
                .to_string();
            let inst_id = item
                .get("instance_id")
                .and_then(|v| v.as_str())
                .unwrap_or("default")
                .to_string();
            let repo_path = item
                .get("repo_path")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or(&cwd_str)
                .to_string();
            let model = item
                .get("model")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| Some("gemini-3.8-flash-high".to_string()));
            let img = item
                .get("image_base64")
                .or_else(|| item.get("image_payload"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let _ = conn.execute(
                "INSERT INTO active_prompts \
                 (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'dispatched', ?8, ?8, ?9)",
                rusqlite::params![
                    &id,
                    &proj_id,
                    &inst_id,
                    &repo_path,
                    &prompt_content,
                    &model,
                    &proj_id,
                    now,
                    &img,
                ],
            );

            // Also write .antigravity_resume_task.json in target repo to trigger immediate rerun
            let task_file = PathBuf::from(&repo_path).join(".antigravity_resume_task.json");
            let task_payload = serde_json::json!({
                "prompt_id": id,
                "project_id": proj_id,
                "instance_id": inst_id,
                "repo_path": repo_path,
                "prompt_content": prompt_content,
                "model": model,
                "image_payload": img,
                "auto_boot": true,
                "imported_at": now,
            });
            if let Ok(js) = serde_json::to_string_pretty(&task_payload) {
                let _ = fs::write(&task_file, js);
            }

            total_imported += 1;
        }
    }

    println!(
        "[SUCCESS] Imported and queued {} prompt(s) for rerun across {} file(s).",
        total_imported,
        files_to_import.len()
    );
}
