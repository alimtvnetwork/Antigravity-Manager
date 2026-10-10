use crate::modules::{account, auto_switcher, config, instance, repo_db};

use super::*;

// -----------------------------------------------------------------------------
// Auto-Switch Command Handler
// -----------------------------------------------------------------------------

pub(crate) fn handle_auto_switch_command(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        print_auto_switch_cli_help();
        std::process::exit(0);
    }

    let mut app_cfg = match config::load_app_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ERROR] Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    let is_json = is_flag_present(args, &["--json", "-j"]);
    let mut non_flag_args: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    if non_flag_args
        .first()
        .map(|s| {
            s.eq_ignore_ascii_case("switch")
                || s.eq_ignore_ascii_case("switcher")
                || s.eq_ignore_ascii_case("swtich")
        })
        .unwrap_or(false)
    {
        non_flag_args.remove(0);
    }
    let sub = non_flag_args.first().map(|s| s.to_lowercase());

    match sub.as_deref() {
        Some("enable") | Some("on") | Some("start") => {
            app_cfg.auto_profile_switcher.is_enabled = true;
            if let Err(e) = config::save_app_config(&app_cfg) {
                eprintln!("[ERROR] Failed to save config: {}", e);
                std::process::exit(1);
            }
            println!("[SUCCESS] Auto-profile switcher daemon is now ENABLED.");
        }
        Some("disable") | Some("off") | Some("stop") => {
            app_cfg.auto_profile_switcher.is_enabled = false;
            if let Err(e) = config::save_app_config(&app_cfg) {
                eprintln!("[ERROR] Failed to save config: {}", e);
                std::process::exit(1);
            }
            println!("[SUCCESS] Auto-profile switcher daemon is now DISABLED.");
        }
        Some("toggle") => {
            let new_val = !app_cfg.auto_profile_switcher.is_enabled;
            app_cfg.auto_profile_switcher.is_enabled = new_val;
            if let Err(e) = config::save_app_config(&app_cfg) {
                eprintln!("[ERROR] Failed to save config: {}", e);
                std::process::exit(1);
            }
            println!(
                "[SUCCESS] Auto-profile switcher daemon is now {}.",
                if new_val { "ENABLED" } else { "DISABLED" }
            );
        }
        Some("threshold") | Some("thresh") => {
            if let Some(val_str) = non_flag_args.get(1) {
                if let Ok(val) = val_str.parse::<f64>() {
                    let clamped = val.clamp(0.0, 100.0);
                    app_cfg.auto_profile_switcher.low_quota_threshold_percent = clamped;
                    if let Err(e) = config::save_app_config(&app_cfg) {
                        eprintln!("[ERROR] Failed to save config: {}", e);
                        std::process::exit(1);
                    }
                    println!(
                        "[SUCCESS] Auto-switch low quota threshold set to {:.1}%.",
                        clamped
                    );
                } else {
                    eprintln!("[ERROR] Invalid numeric threshold value: '{}'", val_str);
                    std::process::exit(1);
                }
            } else {
                println!(
                    "Current auto-switch low quota threshold: {:.1}%",
                    app_cfg.auto_profile_switcher.low_quota_threshold_percent
                );
            }
        }
        Some("interval") | Some("int") => {
            if let Some(val_str) = non_flag_args.get(1) {
                if let Ok(val) = val_str.parse::<u32>() {
                    let clamped = val.clamp(10, 86400);
                    app_cfg.auto_profile_switcher.check_interval_seconds = clamped;
                    if let Err(e) = config::save_app_config(&app_cfg) {
                        eprintln!("[ERROR] Failed to save config: {}", e);
                        std::process::exit(1);
                    }
                    println!("[SUCCESS] Auto-switch check interval set to {}s.", clamped);
                } else {
                    eprintln!("[ERROR] Invalid numeric interval seconds: '{}'", val_str);
                    std::process::exit(1);
                }
            } else {
                println!(
                    "Current auto-switch check interval: {}s",
                    app_cfg.auto_profile_switcher.check_interval_seconds
                );
            }
        }
        Some("model") => {
            if let Some(val_str) = non_flag_args.get(1) {
                app_cfg.auto_profile_switcher.target_model = val_str.to_string();
                if let Err(e) = config::save_app_config(&app_cfg) {
                    eprintln!("[ERROR] Failed to save config: {}", e);
                    std::process::exit(1);
                }
                println!("[SUCCESS] Auto-switch target model set to '{}'.", val_str);
            } else {
                println!(
                    "Current auto-switch target model: {}",
                    app_cfg.auto_profile_switcher.target_model
                );
            }
        }
        Some("run") | Some("trigger") | Some("eval") | Some("check") | Some("rotate") => {
            println!("[*] Triggering immediate auto-switch evaluation across instances...");
            let rt = tokio::runtime::Runtime::new().expect("Failed to initialize async runtime");
            match rt.block_on(auto_switcher::check_and_rotate_if_needed()) {
                Ok(Some(reason)) => {
                    println!("[SUCCESS] Auto-switch rotation triggered: {}", reason);
                }
                Ok(None) => {
                    println!("[INFO] Quota healthy across all monitored instances. No rotation required.");
                }
                Err(e) => {
                    eprintln!("[ERROR] Auto-switch evaluation failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            // Show status
            let status = auto_switcher::get_status();
            if is_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&status).unwrap_or_default()
                );
            } else {
                println!("================================================================================");
                println!("                   AGM Auto-Profile Switcher Status                             ");
                println!("================================================================================");
                println!(
                    "  Daemon Status       : {}",
                    if status.is_running {
                        "RUNNING (Active)"
                    } else {
                        "STOPPED (Disabled)"
                    }
                );
                println!("  Active Instance     : {}", status.active_instance_id);
                println!(
                    "  Bound Account       : {}",
                    status.active_account_email.as_deref().unwrap_or("(none)")
                );
                println!(
                    "  Current Quota %     : {}",
                    status
                        .current_quota_percent
                        .map(|p| format!("{:.1}%", p))
                        .unwrap_or_else(|| "N/A".to_string())
                );
                println!(
                    "  Check Interval      : {}s",
                    app_cfg.auto_profile_switcher.check_interval_seconds
                );
                println!(
                    "  Low Quota Threshold : {:.1}%",
                    app_cfg.auto_profile_switcher.low_quota_threshold_percent
                );
                println!(
                    "  Target Model        : {}",
                    app_cfg.auto_profile_switcher.target_model
                );
                if let Some(ref reason) = status.last_switch_reason {
                    println!("  Last Switch Reason  : {}", reason);
                }
                println!("================================================================================");
            }
        }
    }
    std::process::exit(0);
}
