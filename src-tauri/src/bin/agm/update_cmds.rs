//! update_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use chrono::Utc;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

pub(crate) fn cmd_update(args: &[String]) {
    let is_json = args.iter().any(|a| a == "--json" || a == "-j");
    let is_help = args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help");
    let is_all = args.iter().any(|a| a == "all" || a == "--all" || a == "-a")
        || env::args().any(|a| a == "update-all" || a == "ua");
    let is_gitmap = args
        .iter()
        .any(|a| a.eq_ignore_ascii_case("gitmap") || a.eq_ignore_ascii_case("gm"));
    let is_ssh_fleet = args.iter().any(|a| a.eq_ignore_ascii_case("ssh"));
    let is_check = args.iter().any(|a| a == "--check" || a == "-c");
    let is_force = args.iter().any(|a| a == "--force" || a == "-f");
    let is_export_zip = args
        .iter()
        .any(|a| a == "export-zip" || a == "--export-zip");

    if is_export_zip {
        crate::update_helpers::cmd_update_export_zip(args);
        return;
    }

    if is_help {
        if is_json {
            let help_obj = serde_json::json!({
                "command": "update",
                "aliases": ["update-all", "ua"],
                "syntax": "agm update [all|gitmap|ssh] [--json] [--check] [--force]",
                "options": {
                    "--json, -j": "Output pure machine-readable JSON payload (zero banners)",
                    "all, --all, -a": "Update AGM binary, GitMap CLI, pull latest repo code, and sync fleet",
                    "gitmap, gm": "Update GitMap CLI via gitmap self-update",
                    "ssh": "Update AGM across all registered SSH cluster nodes (gitmap ssh update agm)",
                    "--check, -c": "Query and compare releases without installing updates",
                    "--force, -f": "Force re-installation even if already at latest version"
                }
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&help_obj).unwrap_or_default()
            );
        } else {
            println!("AGM Update & Fleet Synchronization:");
            println!("  agm update export-zip [-o path]            Download the release zip and print its path. Does not install.");
            println!("  agm update [--json] [--check] [--force]    Check and update AGM binary from GitHub");
            println!(
                "  agm update gitmap                          Update GitMap CLI to latest release"
            );
            println!("  agm update ssh                             Update AGM across all SSH cluster machines");
            println!("  agm update all [--json] [--check]          Update AGM binary, GitMap, pull repo, and sync fleet");
            println!("  agm update-all, agm ua                     Aliases for 'agm update all'");
            println!();
            println!("Options:");
            println!("  --json, -j      Output pure machine-readable JSON payload (zero banners)");
            println!("  --check, -c     Query and compare releases without installing updates");
            println!("  --force, -f     Force re-installation even if already at latest version");
            println!();
            println!("Examples:");
            println!("  agm update                       # Update AGM binary interactively");
            println!("  agm update gitmap                # Update GitMap CLI (gitmap self-update)");
            println!("  agm update ssh                   # Update AGM across SSH fleet (gitmap ssh update agm)");
            println!("  agm update all                   # Update AGM binary, GitMap, and sync local repository");
            println!("  agm update all --json            # Remote machine automation via JSON");
        }
        return;
    }

    if is_gitmap && !is_all {
        println!("[*] Updating GitMap CLI via 'gitmap self-update'...");
        let _ = Command::new("gitmap")
            .arg("self-update")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();
        return;
    }

    if is_ssh_fleet {
        println!("[*] Updating AGM across SSH cluster machines via 'gitmap ssh update agm'...");
        let _ = Command::new("gitmap")
            .args(["ssh", "update", "agm"])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();
        return;
    }

    if is_all && !is_check {
        if !is_json {
            println!("[*] Checking GitMap CLI for updates (gitmap self-update)...");
        }
        let _ = Command::new("gitmap").arg("self-update").output();
    }

    if !is_json {
        println!("[*] Checking GitHub for AGM updates...");
    }

    let client = match reqwest::blocking::Client::builder()
        .user_agent("AGM-CLI-Updater")
        .timeout(std::time::Duration::from_secs(15))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            if is_json {
                let err_obj = serde_json::json!({
                    "success": false,
                    "error": format!("Failed to initialize HTTP client: {}", e),
                    "current_version": crate::common::VERSION,
                    "timestamp": chrono::Utc::now().timestamp(),
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&err_obj).unwrap_or_default()
                );
            } else {
                eprintln!("Failed to initialize HTTP client: {}", e);
            }
            return;
        }
    };

    let is_delegated_stage2 = args.iter().any(|a| {
        a == "--no-launch"
            || a == "--no-relaunch"
            || a == "--delegated-worker"
            || a == "--install-dir"
    });
    if is_delegated_stage2 && !is_json && !is_check && !is_help && !is_gitmap && !is_ssh_fleet {
        let ok = antigravity_tools_lib::modules::delegate_updater::run_cli_update(args);
        std::process::exit(if ok { 0 } else { 1 });
    }

    let api_url = "https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases/latest";
    let updater_cdn_url =
        "https://github.com/alimtvnetwork/Antigravity-Manager/releases/latest/download/updater.json";

    let mut tag_name = String::new();
    let mut release_html_url =
        "https://github.com/alimtvnetwork/Antigravity-Manager/releases".to_string();
    let mut release_name = String::new();

    if let Ok(resp) = client.get(api_url).send() {
        if resp.status().is_success() {
            if let Ok(json) = resp.json::<serde_json::Value>() {
                tag_name = json["tag_name"]
                    .as_str()
                    .unwrap_or("")
                    .trim_start_matches('v')
                    .to_string();
                if let Some(u) = json["html_url"].as_str() {
                    release_html_url = u.to_string();
                }
                if let Some(n) = json["name"].as_str() {
                    release_name = n.to_string();
                }
            }
        }
    }

    if tag_name.is_empty() {
        if let Ok(resp) = client.get(updater_cdn_url).send() {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<serde_json::Value>() {
                    tag_name = json["version"]
                        .as_str()
                        .unwrap_or("")
                        .trim_start_matches('v')
                        .to_string();
                    if !tag_name.is_empty() {
                        release_html_url = format!(
                            "https://github.com/alimtvnetwork/Antigravity-Manager/releases/tag/v{}",
                            tag_name
                        );
                        release_name = format!("v{}", tag_name);
                    }
                }
            }
        }
    }

    if tag_name.is_empty() {
        if is_json {
            let err_obj = serde_json::json!({
                "success": false,
                "error": "Failed to fetch latest release metadata from GitHub API and CDN updater.json",
                "current_version": crate::common::VERSION,
                "timestamp": chrono::Utc::now().timestamp(),
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&err_obj).unwrap_or_default()
            );
        } else {
            eprintln!("Failed to fetch latest release metadata from GitHub API and CDN.");
        }
        return;
    }

    let is_up_to_date = tag_name == crate::common::VERSION;
    let mut updated = false;

    // Repo check if in workspace
    let is_git_repo = Path::new(".git").exists();
    let git_branch = if is_git_repo {
        antigravity_tools_lib::modules::git_info::get_git_branch()
    } else {
        "N/A".to_string()
    };
    let git_hash = if is_git_repo {
        antigravity_tools_lib::modules::git_info::get_git_hash()
    } else {
        "N/A".to_string()
    };
    let mut repo_pulled = false;

    if is_all && is_git_repo && !is_check {
        if !is_json {
            println!(
                "[*] Pulling latest repository commits (git pull origin {})...",
                git_branch
            );
        }
        let pull_res = Command::new("git")
            .args(["pull", "origin", &git_branch])
            .output();
        if let Ok(out) = pull_res {
            repo_pulled = out.status.success();
        }
    }

    // Binary update if needed
    if (!is_up_to_date || is_force) && !is_check {
        let export_path = crate::update_helpers::default_update_export_path();
        match crate::update_helpers::download_release_zip(&export_path) {
            Ok(()) => println!("EXPORT_ZIP={}", export_path.display()),
            Err(err) => eprintln!("[WARN] update zip was not saved: {}", err),
        }
        if !is_json {
            println!(
                "[*] A new version is available: v{} -> v{}",
                crate::common::VERSION,
                tag_name
            );
            println!("[*] Triggering automatic update installation...");
        }

        let ui_running =
            antigravity_tools_lib::modules::delegate_updater::is_ui_process_running_excluding(
                std::process::id(),
            );

        if ui_running && !is_json {
            println!(
                "[*] Antigravity Manager UI is currently running. Delegating to 3-stage Update CLI (agm-update-cli -> agm update -> agm open-ui)..."
            );
            let delegate_args = vec![
                "--relaunch".to_string(),
                "--version".to_string(),
                tag_name.clone(),
            ];
            antigravity_tools_lib::modules::delegate_updater::run(&delegate_args);
            return;
        }

        let mut cli_update_args = vec![
            "--force".to_string(),
            "--no-launch".to_string(),
            "--version".to_string(),
            tag_name.clone(),
        ];
        for i in 0..args.len() {
            if args[i] == "--install-dir" && i + 1 < args.len() {
                cli_update_args.push("--install-dir".to_string());
                cli_update_args.push(args[i + 1].clone());
                break;
            }
        }
        updated =
            antigravity_tools_lib::modules::delegate_updater::run_cli_update(&cli_update_args);
    }

    let (node_alias, local_ip) =
        antigravity_tools_lib::modules::email_sender::get_local_node_identity();
    let instance_count = antigravity_tools_lib::modules::instance::list_instances()
        .map(|i| i.len())
        .unwrap_or(1);

    if is_json {
        let payload = serde_json::json!({
            "success": true,
            "command": if is_all { "update-all" } else { "update" },
            "current_version": format!("v{}", crate::common::VERSION),
            "latest_release": format!("v{}", tag_name),
            "release_name": release_name,
            "release_url": release_html_url,
            "is_up_to_date": is_up_to_date,
            "updated": updated,
            "scope": if is_all { "fleet_and_repo" } else { "binary_only" },
            "repo": {
                "is_git_repo": is_git_repo,
                "branch": git_branch,
                "commit": git_hash,
                "pulled": repo_pulled
            },
            "fleet": {
                "node_alias": node_alias,
                "local_ip": local_ip,
                "instance_count": instance_count,
                "status": "synchronized"
            },
            "timestamp": chrono::Utc::now().timestamp()
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
    } else {
        println!(
            "  ================================================================================"
        );
        println!("    AGM SYSTEM UPDATE & FLEET SYNCHRONIZATION");
        println!(
            "  ================================================================================"
        );
        println!("    ● Binary Target:     agm (Antigravity-Manager)");
        println!("    ● Current Version:   v{}", crate::common::VERSION);
        println!("    ● Latest Release:    v{}", tag_name);
        println!(
            "    ● Update Status:     {}",
            if is_up_to_date {
                "[UP TO DATE] (System is currently running the latest release)".to_string()
            } else if updated {
                format!("[UPDATED] (Successfully updated to v{})", tag_name)
            } else {
                format!("[AVAILABLE] (v{} is available for installation)", tag_name)
            }
        );
        println!("    ● Release URL:       {}", release_html_url);
        if is_all {
            println!(
                "    ────────────────────────────────────────────────────────────────────────────"
            );
            println!(
                "    ● Scope:             Full Fleet Synchronization (Binary + Repo + Instances)"
            );
            if is_git_repo {
                println!(
                    "    ● Local Workspace:   {} (Branch: {}, Commit: {})",
                    env::current_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| ".".to_string()),
                    git_branch,
                    git_hash
                );
                println!(
                    "    ● Repo Git Pull:     {}",
                    if repo_pulled {
                        "Updated (git pull successful)"
                    } else {
                        "Up to date / skipped"
                    }
                );
            }
            println!(
                "    ● Fleet Node:        {} / {} (Instances: {})",
                node_alias, local_ip, instance_count
            );
        }
        println!(
            "  ================================================================================"
        );
    }
}

pub(crate) fn cmd_delegate_update(args: &[String]) {
    antigravity_tools_lib::modules::delegate_updater::run(args);
}
