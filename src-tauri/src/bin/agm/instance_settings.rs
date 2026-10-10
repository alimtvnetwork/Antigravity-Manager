//! instance_settings — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::fs;

pub(crate) fn cmd_instance_settings(args: &[String]) {
    let non_flag_args: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with('-'))
        .cloned()
        .collect();

    let start_idx = if non_flag_args
        .first()
        .map(|s| s.eq_ignore_ascii_case("settings") || s.eq_ignore_ascii_case("setting"))
        .unwrap_or(false)
    {
        1
    } else {
        0
    };

    let subaction = non_flag_args.get(start_idx).map(|s| s.to_lowercase());

    let mut target_instance: Option<String> = None;
    let is_all = args.iter().any(|a| a.eq_ignore_ascii_case("--all"));
    let mut out_file: Option<String> = None;
    let mut in_file: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg_lower = args[i].to_lowercase();
        if (arg_lower == "--instance" || arg_lower == "-i" || arg_lower == "--inst")
            && i + 1 < args.len()
        {
            target_instance = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        if (arg_lower == "--out" || arg_lower == "-o") && i + 1 < args.len() {
            out_file = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        if (arg_lower == "--file" || arg_lower == "-f") && i + 1 < args.len() {
            in_file = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        i += 1;
    }

    let target_ref = if is_all {
        None
    } else {
        target_instance.as_deref()
    };

    match subaction.as_deref() {
        Some("enforce-defaults") | Some("enforce") | Some("defaults") => {
            match instance::enforce_default_settings(target_ref) {
                Ok(count) => {
                    println!(
                        "[SUCCESS] Enforced default settings across {} instance(s).",
                        count
                    );
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to enforce default settings: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some("set-turbo") | Some("turbo") => {
            let is_disabled = args.iter().any(|a| {
                a.eq_ignore_ascii_case("--disable")
                    || a.eq_ignore_ascii_case("--off")
                    || a.eq_ignore_ascii_case("disable")
                    || a.eq_ignore_ascii_case("off")
                    || a.eq_ignore_ascii_case("false")
            });
            let enabled = !is_disabled;

            match instance::set_instance_turbo_mode(target_ref, enabled) {
                Ok(count) => {
                    println!(
                        "[SUCCESS] Set antigravity.turboMode = {} across {} instance(s).",
                        enabled, count
                    );
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to set turbo mode: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some("set-plan-review") | Some("plan-review") | Some("plan") => {
            let is_ask = args.iter().any(|a| {
                a.eq_ignore_ascii_case("--ask")
                    || a.eq_ignore_ascii_case("--ask-first")
                    || a.eq_ignore_ascii_case("ask")
                    || a.eq_ignore_ascii_case("false")
            });
            let always_proceed = !is_ask;

            match instance::set_instance_plan_review(target_ref, always_proceed) {
                Ok(count) => {
                    println!(
                        "[SUCCESS] Set antigravity.planReviewAlwaysProceed = {} across {} instance(s).",
                        always_proceed, count
                    );
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to set plan review policy: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some("export") => {
            let inst_spec = target_instance.as_deref().unwrap_or("active");
            match instance::export_instance_settings(inst_spec) {
                Ok(json_str) => {
                    if let Some(ref path) = out_file {
                        if let Err(e) = fs::write(path, &json_str) {
                            eprintln!("[ERROR] Failed to write to '{}': {}", path, e);
                            std::process::exit(1);
                        }
                        println!(
                            "[SUCCESS] Exported instance '{}' settings to '{}'.",
                            inst_spec, path
                        );
                    } else {
                        println!("{}", json_str);
                    }
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to export settings: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some("import") => {
            let file_path = match in_file {
                Some(f) => f,
                None => {
                    eprintln!("Usage: agm instance settings import [--instance <id> | --all] --file <file>");
                    std::process::exit(1);
                }
            };
            let content = match fs::read_to_string(&file_path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!(
                        "[ERROR] Failed to read settings file '{}': {}",
                        file_path, e
                    );
                    std::process::exit(1);
                }
            };

            match instance::import_instance_settings(target_ref, &content) {
                Ok(count) => {
                    println!("[SUCCESS] Imported settings into {} instance(s).", count);
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to import settings: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            println!("AGM Instance Settings CLI:");
            println!("  agm instance settings enforce-defaults [--all | --instance <id>]");
            println!("  agm instance settings set-turbo [--all | --instance <id>] [--enable | --disable]");
            println!("  agm instance settings set-plan-review [--all | --instance <id>] [--always-proceed | --ask]");
            println!("  agm instance settings export [--instance <id>] [--out <file>]");
            println!("  agm instance settings import [--instance <id> | --all] --file <file>");
        }
    }
}

pub(crate) fn cmd_theme(args: &[String]) {
    let non_flag_args: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with('-'))
        .cloned()
        .collect();

    let start_idx = if non_flag_args
        .first()
        .map(|s| s.eq_ignore_ascii_case("theme") || s.eq_ignore_ascii_case("profile-theme"))
        .unwrap_or(false)
    {
        1
    } else {
        0
    };

    let mut target_instance: Option<String> = None;
    let is_all = args.iter().any(|a| a.eq_ignore_ascii_case("--all"));

    let mut i = 0;
    while i < args.len() {
        let arg_lower = args[i].to_lowercase();
        if (arg_lower == "--instance" || arg_lower == "-i" || arg_lower == "--inst")
            && i + 1 < args.len()
        {
            target_instance = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        i += 1;
    }

    let subaction = non_flag_args.get(start_idx).map(|s| s.to_lowercase());

    match subaction.as_deref() {
        Some("set") => {
            let theme_id = match non_flag_args.get(start_idx + 1) {
                Some(t) => t.clone(),
                None => {
                    eprintln!("Usage: agm theme set <theme-id> [--instance <id> | --all]");
                    eprintln!("       agm profile-theme set <theme-id> [--instance <id> | --all]");
                    std::process::exit(1);
                }
            };

            if target_instance.is_none() && !is_all {
                if let Some(inst) = non_flag_args.get(start_idx + 2) {
                    target_instance = Some(inst.clone());
                }
            }

            let resolved_target: Option<String> = if is_all {
                None
            } else if let Some(ref inst) = target_instance {
                Some(instance::resolve_instance_id(inst).unwrap_or_else(|_| inst.clone()))
            } else {
                match instance::get_active_instance_id() {
                    Ok(id) => Some(id),
                    Err(_) => Some("default".to_string()),
                }
            };

            match instance::set_instance_theme(resolved_target.as_deref(), &theme_id) {
                Ok(count) => {
                    if let Some(ref inst) = resolved_target {
                        println!(
                            "[SUCCESS] Set workbench.colorTheme = '{}' for instance '{}'.",
                            theme_id, inst
                        );
                    } else {
                        println!(
                            "[SUCCESS] Set workbench.colorTheme = '{}' across {} instance(s).",
                            theme_id, count
                        );
                    }
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to set theme: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some("get") => {
            let target = target_instance
                .as_deref()
                .or_else(|| non_flag_args.get(start_idx + 1).map(|s| s.as_str()))
                .unwrap_or("default");
            match instance::get_instance_theme(target) {
                Ok(Some(t)) => println!("Instance '{}' workbench.colorTheme: {}", target, t),
                Ok(None) => {
                    println!(
                        "Instance '{}' has no custom workbench.colorTheme configured",
                        target
                    )
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to get theme: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            if let Some(theme_id) = non_flag_args.get(start_idx) {
                let resolved_target: Option<String> = if is_all {
                    None
                } else if let Some(ref inst) = target_instance {
                    Some(instance::resolve_instance_id(inst).unwrap_or_else(|_| inst.clone()))
                } else {
                    match instance::get_active_instance_id() {
                        Ok(id) => Some(id),
                        Err(_) => Some("default".to_string()),
                    }
                };

                match instance::set_instance_theme(resolved_target.as_deref(), theme_id) {
                    Ok(count) => {
                        if let Some(ref inst) = resolved_target {
                            println!(
                                "[SUCCESS] Set workbench.colorTheme = '{}' for instance '{}'.",
                                theme_id, inst
                            );
                        } else {
                            println!(
                                "[SUCCESS] Set workbench.colorTheme = '{}' across {} instance(s).",
                                theme_id, count
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("[ERROR] Failed to set theme: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("AGM Theme CLI:");
                println!("  agm theme set <theme-id> [--instance <id> | --all]");
                println!("  agm profile-theme set <theme-id> [--instance <id> | --all]");
                println!("  agm theme get [--instance <id>]");
            }
        }
    }
}
