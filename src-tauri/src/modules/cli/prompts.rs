use crate::modules::{account, auto_switcher, config, instance, repo_db};

use super::*;

pub fn handle_prompts_subcommand(args: &[String], entry_cmd: &str) {
    if entry_cmd == "tree" {
        return execute_prompts_tree(args);
    }

    if args.is_empty()
        || args
            .iter()
            .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        print_prompts_cli_help();
        std::process::exit(0);
    }

    let sub = args[0].to_lowercase();
    let sub_args = &args[1..];

    match sub.as_str() {
        "ls" | "list" => execute_prompts_list(sub_args),
        "tree" => execute_prompts_tree(sub_args),
        "send" | "run" => execute_prompts_send(sub_args),
        "enqueue" | "queue" => execute_prompts_enqueue(sub_args),
        "backup" => execute_prompts_backup(sub_args),
        "restore" => execute_prompts_restore(sub_args),
        _ => {
            eprintln!("Unknown prompts subcommand: {}", sub);
            eprintln!("Run 'antigravity-manager prompts help' for usage instructions.");
            std::process::exit(1);
        }
    }
}

pub(crate) fn execute_prompts_list(sub_args: &[String]) {
    let is_json = is_flag_present(sub_args, &["--json", "-j"]);
    let is_all = is_flag_present(sub_args, &["--all", "-a"]);
    let instance_opt = extract_flag_value(sub_args, &["--instance", "-i", "--profile"]);

    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let target_inst = if is_all {
        None
    } else if let Some(ref inst) = instance_opt {
        Some(instance::resolve_instance_id(inst).unwrap_or_else(|_| inst.clone()))
    } else {
        None
    };

    let prompts: Vec<_> = if let Some(ref inst) = target_inst {
        all_prompts
            .into_iter()
            .filter(|p| {
                p.instance_id == *inst
                    || (*inst == "default"
                        && (p.instance_id == "__default__" || p.instance_id.is_empty()))
            })
            .collect()
    } else {
        all_prompts
    };

    if is_json {
        let payload = serde_json::json!({
            "total_prompts": prompts.len(),
            "prompts": prompts,
        });
        CliEnvelope::ok("prompts ls", target_inst, payload).print_and_exit();
    }

    println!("\nActive & Queued Prompts ({} total):", prompts.len());
    println!(
        "{:<5} {:<18} {:<15} {:<12} {:<30} {}",
        "#", "ID", "INSTANCE", "STATUS", "REPO", "PROMPT PREVIEW"
    );
    println!("{}", "-".repeat(110));
    for (idx, p) in prompts.iter().enumerate() {
        let clean_content = p.prompt_content.replace('\n', " ");
        let preview = if clean_content.len() > 40 {
            format!("{}...", &clean_content[..40])
        } else {
            clean_content
        };
        println!(
            "#{:<4} {:<18} {:<15} {:<12} {:<30} {}",
            idx + 1,
            p.id,
            p.instance_id,
            p.status,
            p.project_id,
            preview
        );
    }
    println!();
    std::process::exit(0);
}

pub(crate) fn execute_prompts_tree(sub_args: &[String]) {
    let is_json = is_flag_present(sub_args, &["--json", "-j"]);
    let is_running_only = is_flag_present(sub_args, &["--running", "--only-running"]);
    let instance_opt = extract_flag_value(sub_args, &["--instance", "-i", "--profile"]);
    let target_inst = instance_opt.as_deref();

    let tree =
        repo_db::get_project_conversation_tree_cached(target_inst, 200, is_running_only, true);

    if is_json {
        let payload = serde_json::json!({
            "total_projects": tree.len(),
            "projects": tree,
        });
        CliEnvelope::ok("prompts tree", instance_opt, payload).print_and_exit();
    }

    println!(
        "\nAGM Project & Conversation Tree ({} projects):",
        tree.len()
    );
    println!("{}", "=".repeat(90));
    for proj in &tree {
        let clean_proj_badge = if proj.gitmap_seq_code.starts_with("GM:#") {
            proj.gitmap_seq_code
                .strip_prefix("GM:")
                .unwrap_or(&proj.gitmap_seq_code)
        } else if !proj.seq_code.is_empty() {
            &proj.seq_code
        } else {
            "P001"
        };
        let run_badge = if proj.running_count > 0 {
            format!(" [{} RUNNING]", proj.running_count)
        } else {
            String::new()
        };
        println!(
            "{} {} ({}) - Instance: {}{}",
            clean_proj_badge, proj.repo_name, proj.repo_path, proj.instance_id, run_badge
        );
        for conv in &proj.conversations {
            let conv_badge = if conv.seq_code.is_empty() {
                "C001"
            } else {
                &conv.seq_code
            };
            let status_mark = if conv.is_running {
                "● RUNNING"
            } else {
                "IDLE"
            };
            println!(
                "  ↳ {} [{}] {} ({})",
                conv_badge, conv.short_id, conv.title, status_mark
            );
            if !conv.prompt_preview_200w.is_empty() {
                let clean_preview = conv.prompt_preview_200w.replace('\n', " ");
                let preview = if clean_preview.len() > 80 {
                    format!("{}...", &clean_preview[..80])
                } else {
                    clean_preview
                };
                println!("     Preview: {}", preview);
            }
        }
    }
    println!("{}", "=".repeat(90));
    std::process::exit(0);
}

pub(crate) fn execute_prompts_send(sub_args: &[String]) {
    let is_json = is_flag_present(sub_args, &["--json", "-j"]);
    let instance_opt = extract_flag_value(sub_args, &["--instance", "-i", "--profile"]);
    let repo_opt = extract_flag_value(sub_args, &["--repo", "-r", "--workspace"]);
    let positional = extract_positional_args(sub_args);

    if positional.is_empty() {
        eprintln!(
            "Error: Usage: agm prompts send <prompt_text> [-i <instance>] [-r <repo>] [--json]"
        );
        std::process::exit(1);
    }

    let prompt_text = positional.join(" ");
    let target_inst = instance_opt.unwrap_or_else(|| {
        instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string())
    });
    let target_repo = repo_opt.unwrap_or_else(|| {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string())
    });

    match repo_db::send_prompt_now_for_instance(&target_inst, &target_repo, &prompt_text, None) {
        Ok(active_prompt) => {
            if is_json {
                let payload = serde_json::json!({
                    "prompt_id": active_prompt.id,
                    "instance_id": active_prompt.instance_id,
                    "repo_path": active_prompt.repo_path,
                    "status": active_prompt.status,
                    "is_running": active_prompt.status == "running",
                    "created_at": active_prompt.created_at,
                });
                CliEnvelope::ok("prompts send", Some(target_inst), payload).print_and_exit();
            }
            println!("[CLI] Prompt successfully dispatched:");
            println!("  ID:          {}", active_prompt.id);
            println!("  Instance:    {}", active_prompt.instance_id);
            println!("  Repo:        {}", active_prompt.repo_path);
            println!("  Status:      {}", active_prompt.status);
            std::process::exit(0);
        }
        Err(e) => {
            if is_json {
                CliEnvelope::<()>::err("prompts send", Some(target_inst), "DISPATCH_FAILED", &e)
                    .print_and_exit();
            }
            eprintln!("[ERROR] Failed to send prompt: {}", e);
            std::process::exit(1);
        }
    }
}

pub(crate) fn execute_prompts_enqueue(sub_args: &[String]) {
    let is_json = is_flag_present(sub_args, &["--json", "-j"]);
    let instance_opt = extract_flag_value(sub_args, &["--instance", "-i", "--profile"]);
    let repo_opt = extract_flag_value(sub_args, &["--repo", "-r", "--workspace"]);
    let positional = extract_positional_args(sub_args);

    if positional.is_empty() {
        eprintln!(
            "Error: Usage: agm prompts enqueue <prompt_text> [-i <instance>] [-r <repo>] [--json]"
        );
        std::process::exit(1);
    }

    let prompt_text = positional.join(" ");
    let target_inst = instance_opt.unwrap_or_else(|| {
        instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string())
    });
    let target_repo = repo_opt.unwrap_or_else(|| {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string())
    });

    match repo_db::enqueue_prompt_for_instance_full(
        &target_inst,
        &prompt_text,
        Some(&target_repo),
        None,
        None,
    ) {
        Ok(row_id) => {
            if is_json {
                let payload = serde_json::json!({
                    "prompt_row_id": row_id,
                    "instance_id": target_inst,
                    "repo_path": target_repo,
                    "status": "queued",
                    "is_queued": true,
                });
                CliEnvelope::ok("prompts enqueue", Some(target_inst), payload).print_and_exit();
            }
            println!("[CLI] Prompt successfully enqueued:");
            println!("  Row ID:      {}", row_id);
            println!("  Instance:    {}", target_inst);
            println!("  Repo:        {}", target_repo);
            println!("  Status:      queued");
            std::process::exit(0);
        }
        Err(e) => {
            let msg = e.to_string();
            if is_json {
                CliEnvelope::<()>::err(
                    "prompts enqueue",
                    Some(target_inst),
                    "ENQUEUE_FAILED",
                    &msg,
                )
                .print_and_exit();
            }
            eprintln!("[ERROR] Failed to enqueue prompt: {}", msg);
            std::process::exit(1);
        }
    }
}

pub(crate) fn execute_prompts_backup(sub_args: &[String]) {
    let is_json = is_flag_present(sub_args, &["--json", "-j"]);
    let instance_opt = extract_flag_value(sub_args, &["--instance", "-i", "--profile"]);
    let target_inst = instance_opt.unwrap_or_else(|| {
        instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string())
    });

    match repo_db::backup_running_prompts(&target_inst) {
        Ok(count) => {
            if is_json {
                let payload = serde_json::json!({
                    "instance_id": target_inst,
                    "backed_up_count": count,
                });
                CliEnvelope::ok("prompts backup", Some(target_inst), payload).print_and_exit();
            }
            println!(
                "[CLI] Successfully backed up {} running prompt(s) for instance '{}'.",
                count, target_inst
            );
            std::process::exit(0);
        }
        Err(e) => {
            if is_json {
                CliEnvelope::<()>::err("prompts backup", Some(target_inst), "BACKUP_FAILED", &e)
                    .print_and_exit();
            }
            eprintln!("[ERROR] Failed to backup running prompts: {}", e);
            std::process::exit(1);
        }
    }
}

pub(crate) fn execute_prompts_restore(sub_args: &[String]) {
    let is_json = is_flag_present(sub_args, &["--json", "-j"]);
    let instance_opt = extract_flag_value(sub_args, &["--instance", "-i", "--profile"]);
    let limit = extract_flag_value(sub_args, &["--limit", "-l"])
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(20);
    let target_inst = instance_opt.as_deref();

    match repo_db::resend_running_commands_for_instance(target_inst, limit) {
        Ok(restored) => {
            if is_json {
                let payload = serde_json::json!({
                    "instance_id": target_inst,
                    "restored_count": restored.len(),
                    "prompts": restored,
                });
                CliEnvelope::ok("prompts restore", target_inst.map(String::from), payload)
                    .print_and_exit();
            }
            println!(
                "[CLI] Successfully restored {} prompt(s) for instance '{:?}':",
                restored.len(),
                target_inst.unwrap_or("all")
            );
            for p in &restored {
                println!("  - [{}] {} ({})", p.id, p.project_id, p.status);
            }
            std::process::exit(0);
        }
        Err(e) => {
            if is_json {
                CliEnvelope::<()>::err(
                    "prompts restore",
                    target_inst.map(String::from),
                    "RESTORE_FAILED",
                    &e,
                )
                .print_and_exit();
            }
            eprintln!("[ERROR] Failed to restore prompts: {}", e);
            std::process::exit(1);
        }
    }
}
