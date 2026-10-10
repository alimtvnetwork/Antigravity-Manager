//! instance_import_export — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::fs;
use std::io::{self, BufRead, Write};

pub(crate) fn cmd_instances_export(args: &[String]) {
    match instance::export_instances_envelope() {
        Ok(json_str) => {
            let file_arg = args
                .iter()
                .position(|a| a == "--file" || a == "-o")
                .and_then(|idx| args.get(idx + 1));
            if let Some(target_file) = file_arg {
                if let Err(e) = fs::write(target_file, &json_str) {
                    eprintln!(
                        "[ERROR] Failed to write instances to {}: {}",
                        target_file, e
                    );
                } else {
                    println!(
                        "✅ Successfully exported instances envelope to {}",
                        target_file
                    );
                }
            } else {
                println!("{}", json_str);
            }
        }
        Err(e) => eprintln!("[ERROR] Failed to export instances envelope: {}", e),
    }
}

pub(crate) fn cmd_instances_import(args: &[String]) {
    let non_flag_args: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    let target_file = if non_flag_args.len() > 1 {
        Some(non_flag_args[1].as_str())
    } else {
        None
    };

    let path_str = match target_file {
        Some(p) => p,
        None => {
            eprintln!("Usage: agm instances import <file_path>");
            return;
        }
    };

    let resolved_path = json_envelope::resolve_relative_json_path(path_str);
    let raw_json = match fs::read_to_string(&resolved_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ERROR] Failed to read file '{}': {}", path_str, e);
            return;
        }
    };

    match json_envelope::extract_payload::<instance::InstanceRegistry>(&raw_json) {
        Ok((imported_reg, attrs)) => {
            let count = imported_reg.instances.len();
            match instance::save_registry(&imported_reg) {
                Ok(_) => {
                    println!(
                        "✅ Successfully imported {} instances from '{}' (Envelope v{}).",
                        count,
                        resolved_path.display(),
                        attrs.version
                    );
                }
                Err(e) => eprintln!("[ERROR] Failed to save instances registry: {}", e),
            }
        }
        Err(e) => eprintln!("[ERROR] Failed to parse instances envelope: {}", e),
    }
}

pub(crate) fn cmd_instance_duplicate_or_clone(args: &[String]) {
    let is_json = args.iter().any(|a| a == "--json" || a == "-j");
    let copy_projects = args
        .iter()
        .any(|a| a.eq_ignore_ascii_case("--copy-projects") || a.eq_ignore_ascii_case("-cp"));
    let should_launch = args
        .iter()
        .any(|a| a.eq_ignore_ascii_case("--launch") || a.eq_ignore_ascii_case("-l"));

    let non_flag_args: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with('-'))
        .cloned()
        .collect();

    let start_idx = if non_flag_args
        .first()
        .map(|s| s.eq_ignore_ascii_case("duplicate") || s.eq_ignore_ascii_case("clone"))
        .unwrap_or(false)
    {
        1
    } else {
        0
    };

    if non_flag_args.len() < start_idx + 2 {
        eprintln!("Usage: agm instance duplicate <source> <new_name> [--copy-projects] [--launch]");
        std::process::exit(1);
    }

    let source_spec = &non_flag_args[start_idx];
    let new_name = non_flag_args[start_idx + 1].clone();

    let resolved_src = match instance::resolve_instance_id(source_spec) {
        Ok(id) => id,
        Err(e) => {
            eprintln!(
                "[ERROR] Could not resolve source instance '{}': {}",
                source_spec, e
            );
            std::process::exit(1);
        }
    };

    match instance::copy_instance_with_options(&resolved_src, new_name, Some("full"), copy_projects)
    {
        Ok(mut cfg) => {
            if let Ok(exe_path) = instance::clone_instance_executable(&cfg.id) {
                cfg.executable_path = Some(exe_path);
            }
            if should_launch {
                let _ = instance::launch_instance(&cfg.id);
            }

            if is_json {
                println!("{}", serde_json::to_string_pretty(&cfg).unwrap_or_default());
            } else {
                println!(
                    "[SUCCESS] Duplicated instance '{}' into '{}' (ID: {}, copy_projects: {})",
                    source_spec, cfg.name, cfg.id, copy_projects
                );
                if should_launch {
                    println!("  [✓] Launched instance '{}' window", cfg.id);
                }
            }
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to duplicate instance: {}", e);
            std::process::exit(1);
        }
    }
}

pub(crate) fn cmd_instance_count(args: &[String]) {
    let is_json = args.iter().any(|a| a == "--json" || a == "-j");
    match instance::count_instances() {
        Ok(val) => {
            if is_json {
                println!("{}", serde_json::to_string_pretty(&val).unwrap_or_default());
            } else {
                let total = val.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
                let active = val.get("active").and_then(|v| v.as_u64()).unwrap_or(0);
                let running = val.get("running").and_then(|v| v.as_u64()).unwrap_or(0);
                let active_id = val
                    .get("active_instance_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                println!("\nAntigravity Instances Summary:");
                println!("  Total registered:  {}", total);
                println!("  Active profile:    {} (count: {})", active_id, active);
                println!("  Running instances: {}", running);
            }
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to count instances: {}", e);
            std::process::exit(1);
        }
    }
}

pub(crate) fn cmd_instance_copy_projects(args: &[String]) {
    let non_flag_args: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with('-'))
        .cloned()
        .collect();

    let mut from_spec: Option<String> = None;
    let mut to_spec: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg_lower = args[i].to_lowercase();
        if (arg_lower == "--from" || arg_lower == "-f" || arg_lower == "--src")
            && i + 1 < args.len()
        {
            from_spec = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        if (arg_lower == "--to" || arg_lower == "-t" || arg_lower == "--dst") && i + 1 < args.len()
        {
            to_spec = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        i += 1;
    }

    if from_spec.is_none() || to_spec.is_none() {
        let start_idx = if non_flag_args
            .first()
            .map(|s| {
                s.eq_ignore_ascii_case("copy-projects") || s.eq_ignore_ascii_case("copy_projects")
            })
            .unwrap_or(false)
        {
            1
        } else {
            0
        };

        if from_spec.is_none() && non_flag_args.len() > start_idx {
            from_spec = Some(non_flag_args[start_idx].clone());
        }
        if to_spec.is_none() && non_flag_args.len() > start_idx + 1 {
            to_spec = Some(non_flag_args[start_idx + 1].clone());
        }
    }

    let (src, dst) = match (from_spec, to_spec) {
        (Some(s), Some(d)) => (s, d),
        _ => {
            eprintln!("Usage: agm instance copy-projects --from <src> --to <dest>");
            std::process::exit(1);
        }
    };

    match instance::copy_instance_projects(&src, &dst) {
        Ok(count) => {
            println!(
                "[SUCCESS] Copied {} workspace project(s) from '{}' to '{}'.",
                count, src, dst
            );
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to copy projects: {}", e);
            std::process::exit(1);
        }
    }
}

pub(crate) fn cmd_instance_copy_settings(args: &[String]) {
    let non_flag_args: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with('-'))
        .cloned()
        .collect();

    let mut from_spec: Option<String> = None;
    let mut to_spec: Option<String> = None;
    let mut exe_path: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg_lower = args[i].to_lowercase();
        if (arg_lower == "--from" || arg_lower == "-f" || arg_lower == "--src")
            && i + 1 < args.len()
        {
            from_spec = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        if (arg_lower == "--to" || arg_lower == "-t" || arg_lower == "--dst") && i + 1 < args.len()
        {
            to_spec = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        if (arg_lower == "--exe" || arg_lower == "-e") && i + 1 < args.len() {
            exe_path = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        i += 1;
    }

    if let Some(ref ep) = exe_path {
        if let Some(detected_inst) = instance::find_instance_by_executable(ep) {
            println!(
                "[*] Resolved instance '{}' from executable path '{}'",
                detected_inst, ep
            );
            if from_spec.is_none() {
                from_spec = Some(detected_inst);
            } else if to_spec.is_none() {
                to_spec = Some(detected_inst);
            }
        } else {
            eprintln!(
                "[WARN] Could not find any registered instance matching executable '{}'",
                ep
            );
        }
    }

    if from_spec.is_none() || to_spec.is_none() {
        let start_idx = if non_flag_args
            .first()
            .map(|s| {
                s.eq_ignore_ascii_case("copy-settings")
                    || s.eq_ignore_ascii_case("copy_settings")
                    || s.eq_ignore_ascii_case("sync-settings")
                    || s.eq_ignore_ascii_case("sync_settings")
            })
            .unwrap_or(false)
        {
            1
        } else {
            0
        };

        if from_spec.is_none() && non_flag_args.len() > start_idx {
            from_spec = Some(non_flag_args[start_idx].clone());
        }
        if to_spec.is_none() && non_flag_args.len() > start_idx + 1 {
            to_spec = Some(non_flag_args[start_idx + 1].clone());
        }
    }

    let (src, dst) = match (from_spec, to_spec) {
        (Some(s), Some(d)) => (s, d),
        _ => {
            eprintln!("Usage: agm instance copy-settings --from <src> --to <dest> [--exe <path>]");
            std::process::exit(1);
        }
    };

    match instance::copy_instance_settings(&src, &dst) {
        Ok(_) => {
            println!(
                "[SUCCESS] Copied theme and Antigravity settings from '{}' to '{}'.",
                src, dst
            );
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to copy settings: {}", e);
            std::process::exit(1);
        }
    }
}

pub(crate) fn cmd_instances_all(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Rotate All Instances:");
        println!("  agm instances-all [ff] [--help]");
        println!("\nDescription:");
        println!("  Triggers fast-forward account rotation across all registered multi-instance sandboxes.");
        println!("\nAliases: agm instances-all, agm instances all ff");
        println!("\nExamples:");
        println!("  agm instances-all                   # Rotate accounts across all instances");
        println!("  agm instances all ff                # Equivalent invocation");
        return;
    }

    let instances = match instance::list_instances() {
        Ok(list) => list,
        Err(e) => {
            eprintln!("[ERROR] Failed to query instances: {}", e);
            std::process::exit(1);
        }
    };

    println!(
        "[*] Triggering fast-forward rotation across all {} registered instance(s)...",
        instances.len()
    );
    let rt = tokio::runtime::Runtime::new().expect("Failed to initialize async runtime");

    for inst in instances {
        print!(
            "  -> Instance '{}' ({}): ",
            inst.config.name, inst.config.id
        );
        let _ = io::stdout().flush();
        match rt.block_on(auto_switcher::trigger_manual_rotation_for_instance(Some(
            &inst.config.id,
        ))) {
            Ok(msg) => println!("[OK] {}", msg),
            Err(e) => println!("[SKIPPED/WARN] {}", e),
        }
    }
}
