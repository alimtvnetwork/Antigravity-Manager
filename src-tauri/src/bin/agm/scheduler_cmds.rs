//! scheduler_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::time::Duration;

pub(crate) fn cmd_auto_switch(args: &[String]) {
    let mut app_cfg = match config::load_app_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ERROR] Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Auto-Switch & Intelligent Rotator CLI:");
        println!("  agm auto-switch [status] [--json]");
        println!("  agm auto-switch enable | on");
        println!("  agm auto-switch disable | off");
        println!("  agm auto-switch toggle");
        println!("  agm auto-switch run | trigger | eval");
        println!("  agm auto-switch threshold [N]    (Query or set low quota threshold %)");
        println!("  agm auto-switch interval [N]     (Query or set check interval in seconds)");
        println!("  agm auto-switch model [name]     (Query or set target model)");
        println!(
            "  agm auto-switch lockout [N]      (Query or set account lockout window in minutes)"
        );
        println!("  agm auto-switch test [N]         (Run immediate test with optional test threshold %)");
        println!("\nDescription:");
        println!(
            "  Inspects, configures, toggles, or triggers the autonomous background auto-profile"
        );
        println!("  switcher that monitors rolling 4-hour quota windows across running instances");
        println!("  and proactively rotates accounts before depletion.");
        println!("\nAliases: agm auto-switch, agm auto, agm autoswitch, agm auto_switch, agm switcher, agm test-switcher");
        println!("\nExamples:");
        println!(
            "  agm auto-switch status           # Show auto-switcher status & monitored instances"
        );
        println!("  agm auto-switch enable           # Turn ON background auto-switch daemon");
        println!("  agm auto-switch disable          # Turn OFF background auto-switch daemon");
        println!("  agm auto-switch toggle           # Toggle auto-switch daemon state");
        println!(
            "  agm auto-switch run              # Trigger immediate quota evaluation & rotation"
        );
        println!("  agm auto-switch threshold 20     # Set low quota threshold to 20%");
        println!("  agm auto-switch interval 60      # Set polling interval to 60 seconds");
        println!("  agm auto-switch model gemini-2.5-pro # Set evaluation target model");
        println!("  agm auto-switch test 90          # Test simulation with 90% threshold");
        return;
    }

    let is_json = args.iter().any(|a| a == "--json" || a == "-j");
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
                        "[SUCCESS] Auto-switch low quota threshold set to {:.1}% (default: 15.0%).",
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
        Some("interval") => {
            if let Some(val_str) = non_flag_args.get(1) {
                if let Ok(val) = val_str.parse::<u32>() {
                    let clamped = val.clamp(10, 86400);
                    app_cfg.auto_profile_switcher.check_interval_seconds = clamped;
                    if let Err(e) = config::save_app_config(&app_cfg) {
                        eprintln!("[ERROR] Failed to save config: {}", e);
                        std::process::exit(1);
                    }
                    println!(
                        "[SUCCESS] Auto-switch check interval set to {}s (default: 300s).",
                        clamped
                    );
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
        Some("lockout") | Some("lockout-window") => {
            if let Some(val_str) = non_flag_args.get(1) {
                let mins: u32 = match val_str.parse() {
                    Ok(m) => m,
                    Err(_) => {
                        eprintln!(
                            "[ERROR] Invalid minutes value: '{}'. Must be a positive integer.",
                            val_str
                        );
                        std::process::exit(1);
                    }
                };
                app_cfg.auto_profile_switcher.account_lockout_window_minutes = mins;
                if let Err(e) = config::save_app_config(&app_cfg) {
                    eprintln!("[ERROR] Failed to save config: {}", e);
                    std::process::exit(1);
                }
                println!("[SUCCESS] Account lockout window set to {} minutes.", mins);
            } else {
                println!(
                    "Current account lockout window: {} minutes",
                    app_cfg.auto_profile_switcher.account_lockout_window_minutes
                );
            }
        }
        Some("run") | Some("trigger") | Some("eval") | Some("check") | Some("rotate") => {
            println!("[*] Triggering immediate auto-switch evaluation across instances...");
            let rt = match tokio::runtime::Runtime::new() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Failed to initialize async runtime: {}", e);
                    std::process::exit(1);
                }
            };
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
        Some("test") => {
            let test_args: Vec<String> =
                non_flag_args.iter().skip(1).map(|s| (*s).clone()).collect();
            crate::test_cmds::cmd_test_auto_switch(&test_args);
        }
        _ => {
            let status = auto_switcher::get_status();
            if is_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&status).unwrap_or_default()
                );
                return;
            }

            println!("\nAGM Auto-Profile Switcher Status:");
            println!(
                "  Daemon Active:       {}",
                if app_cfg.auto_profile_switcher.is_enabled {
                    "ENABLED (monitoring)"
                } else {
                    "DISABLED"
                }
            );
            println!(
                "  Target Model:        {}",
                app_cfg.auto_profile_switcher.target_model
            );
            println!(
                "  Low Quota Threshold: {:.1}%",
                app_cfg.auto_profile_switcher.low_quota_threshold_percent
            );
            println!(
                "  Check Interval:      {}s",
                app_cfg.auto_profile_switcher.check_interval_seconds
            );
            println!(
                "  Lockout Window:      {}m",
                app_cfg.auto_profile_switcher.account_lockout_window_minutes
            );
            println!("  Active Instance:     {}", status.active_instance_id);
            println!(
                "  Active Account:      {}",
                status.active_account_email.as_deref().unwrap_or("none")
            );
            println!(
                "  Current Quota:       {:.1}%",
                status.current_quota_percent.unwrap_or(100.0)
            );
            println!("  Monitored Instances: {}", status.monitored_instance_count);

            if !status.monitored_instances.is_empty() {
                println!("\nMonitored Instance Quotas:");
                println!(
                    "{:<16} {:<24} {:<12} {:<12} RESET TIME",
                    "INSTANCE", "BOUND EMAIL", "QUOTA", "STATUS"
                );
                println!("{}", "-".repeat(80));
                for inst in &status.monitored_instances {
                    let quota_str = inst
                        .quota_percent
                        .map(|q| format!("{:.1}%", q))
                        .unwrap_or_else(|| "N/A".to_string());
                    let run_str = if inst.is_running { "Running" } else { "Idle" };
                    let reset_str = inst.reset_time_iso.as_deref().unwrap_or("unknown");
                    println!(
                        "{:<16} {:<24} {:<12} {:<12} {}",
                        inst.instance_name,
                        inst.bound_email.as_deref().unwrap_or("unassigned"),
                        quota_str,
                        run_str,
                        reset_str
                    );
                }
            }
            println!();
        }
    }
}

pub(crate) fn cmd_queue_scheduler(args: &[String]) {
    if let Some(first) = args.first() {
        let first_lower = first.to_lowercase();
        if first_lower == "help" || first_lower == "--help" || first_lower == "-h" {
            println!("AGM Prompt Queue Scheduler (10-Minute Loop & Audit Trail):");
            println!("  agm queue-scheduler [--once] [instance_id]   Check enqueued prompts, verify project idleness, and auto-dispatch FIFO");
            println!("\nOptions:");
            println!("  --once, -o, once         Run a single bookkeeping cycle and exit");
            println!(
                "  [instance_id]            Filter to a specific instance (default: all instances)"
            );
            println!("\nAliases: agm queue-scheduler, agm queue_scheduler, agm scheduler, agm qs");
            println!("\nExamples:");
            println!("  agm queue-scheduler --once           # Run single check and push enqueued prompts if projects are idle");
            println!("  agm queue-scheduler                  # Run continuous background daemon with 10-minute intervals");
            println!("  agm queue-scheduler default --once   # Run single check for default instance only");
            return;
        }
    }

    let once = args
        .iter()
        .any(|a| a == "--once" || a == "-o" || a == "once");
    let target_instance = args
        .iter()
        .find(|a| !a.starts_with('-') && *a != "once")
        .map(|s| s.as_str());

    println!("============================================================");
    println!("        ANTIGRAVITY PROMPT QUEUE SCHEDULER (10-MIN LOOP)   ");
    println!("============================================================");
    if let Some(target) = target_instance {
        println!("Target Instance Filter: {}", target);
    } else {
        println!("Target Instance Filter: All Instances");
    }
    println!(
        "Mode:                  {}",
        if once {
            "Single Run (--once)"
        } else {
            "Continuous Loop (Every 10 min)"
        }
    );
    println!("Audit Logging:         Enabled (AuditAction::SchedulePrompt, code 4)");
    println!("============================================================\n");

    if once {
        run_scheduler_single_cycle(target_instance);
        return;
    }

    // Continuous loop: runs cycle, then sleeps 600s
    let interval = std::time::Duration::from_secs(600);
    loop {
        run_scheduler_single_cycle(target_instance);
        println!("\n[Scheduler] Sleeping 10 minutes until next bookkeeping cycle (press Ctrl+C to stop)...");
        std::thread::sleep(interval);
    }
}

pub(crate) fn run_scheduler_single_cycle(target_instance: Option<&str>) {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    println!("[{}] Executing prompt queue bookkeeping cycle...", now);

    match repo_db::check_and_dispatch_enqueued_prompts(target_instance) {
        Ok(dispatched) => {
            if dispatched > 0 {
                println!(
                    "[{}] SUCCESS: Dispatched {} enqueued prompt(s) to idle project(s) and recorded audit trail.",
                    now, dispatched
                );
            } else {
                println!(
                    "[{}] Completed: Evaluated queue. All projects are either active/busy or have no enqueued prompts.",
                    now
                );
            }
        }
        Err(err) => {
            eprintln!("[{}] ERROR running queue scheduler: {}", now, err);
        }
    }
}
