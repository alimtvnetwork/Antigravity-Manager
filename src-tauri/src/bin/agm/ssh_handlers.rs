//! ssh_handlers — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub(crate) fn handle_ssh_deploy_keys(args: &[String]) {
    let mut gitmap_args = vec!["deploy".to_string(), "keys".to_string()];
    gitmap_args.extend(args.iter().cloned());
    if crate::update_helpers::forward_to_gitmap_ssh(&gitmap_args) {
        return;
    }

    println!("🔑 AGM Mesh SSH Public Key Deployment (Native Engine)");
    let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-n");
    let except = args
        .iter()
        .position(|a| a == "--except")
        .and_then(|idx| args.get(idx + 1).map(|s| s.as_str()));
    let target = args
        .iter()
        .find(|a| {
            !a.starts_with('-') && *a != "keys" && *a != "deploy" && Some(a.as_str()) != except
        })
        .map(|s| s.as_str())
        .unwrap_or("all");

    match ssh_manager::deploy_mesh_keys(target, except, dry_run) {
        Ok(summary) => {
            println!(
                "  • Unique Public Keys Identified: {}",
                summary.gathered_keys.len()
            );
            for k in &summary.gathered_keys {
                let preview = if k.public_key.len() > 40 {
                    format!(
                        "{}...{}",
                        &k.public_key[..20],
                        &k.public_key[k.public_key.len() - 15..]
                    )
                } else {
                    k.public_key.clone()
                };
                println!("    - {} ({}): {}", k.key_name, k.source_node, preview);
            }
            println!("  • Target Fleet: {}", target);
            if dry_run {
                println!("  [DRY-RUN] No remote authorized_keys were modified.");
            } else {
                println!(
                    "  • Local authorized_keys updated: {}",
                    summary.local_files_updated
                );
                for rep in &summary.node_reports {
                    let mark = if rep.online && rep.batch_auth_verified {
                        "✓"
                    } else {
                        "✗"
                    };
                    println!(
                        "    {} {} ({}) - {}",
                        mark, rep.alias, rep.ip_address, rep.detail
                    );
                }
            }
            println!("✓ SSH public key mesh deployment complete.");
            println!("\n  💡 SSH Key Deploy & Fleet Optimization Suggestions:");
            println!("    • Verify Remote Access:      agm ssh <alias>");
            println!("    • Check Fleet Nodes:         agm ssh nodes");
            println!("    • Test Fleet Command:        agm ssh exec all \"uname -a\"");
            println!("    • Health Probe Fleet:        gitmap ssh health (or agm ssh nodes)");
            println!("    • Inspect Failed Commands:   agm failed-commands\n");
        }
        Err(e) => eprintln!("[ERROR] Deploy mesh keys failed: {}", e),
    }
}

pub(crate) fn handle_ssh_fix_auth(args: &[String]) {
    let clean_args: Vec<String> = if args.first().map(|s| s.as_str()) == Some("deploy") {
        args[1..].to_vec()
    } else {
        args.to_vec()
    };

    if clean_args.is_empty() || clean_args[0] == "-h" || clean_args[0] == "--help" {
        println!("\nAGM SSH Fix Authentication / Deploy Public Key");
        println!("\nUsage:");
        println!("  agm ssh fix-auth <target> [-i <pubkey>]");
        println!("  agm ssh auth-key deploy <target> [-i <pubkey>]");
        println!("  agm ssh copy-id <target> [-i <pubkey>]");
        println!("\nExamples:");
        println!("  agm ssh fix-auth w3");
        println!("  agm ssh fix-auth root@192.168.1.50 -i ~/.ssh/id_ed25519.pub");
        if clean_args.is_empty() {
            std::process::exit(1);
        }
        return;
    }

    let mut gitmap_args = vec!["fix-auth".to_string()];
    gitmap_args.extend(clean_args.iter().cloned());
    if crate::update_helpers::forward_to_gitmap_ssh(&gitmap_args) {
        return;
    }

    let target = &clean_args[0];
    let mut pubkey_path: Option<&str> = None;
    let mut idx = 1;
    while idx < clean_args.len() {
        if (clean_args[idx] == "-i" || clean_args[idx] == "--identity")
            && idx + 1 < clean_args.len()
        {
            pubkey_path = Some(&clean_args[idx + 1]);
            break;
        }
        idx += 1;
    }

    println!(
        "[*] Deploying public key to target '{}' (native engine)...",
        target
    );
    match ssh_manager::deploy_auth_key_to_target(target, pubkey_path) {
        Ok(reports) => {
            let mut any_success = false;
            for r in &reports {
                if r.online && r.keys_deployed > 0 {
                    any_success = true;
                    println!(
                        "✓ Successfully deployed public key to {} ({}@{}) [status: {}, verified: {}]",
                        r.alias, r.username, r.ip_address, r.status, r.batch_auth_verified
                    );
                } else {
                    eprintln!(
                        "[WARN] Target {} ({}): {} ({})",
                        r.alias, r.ip_address, r.status, r.detail
                    );
                }
            }
            if !any_success && !reports.is_empty() {
                std::process::exit(1);
            }
            if any_success {
                println!("\n  💡 Next Steps & Key Deployment Suggestions:");
                println!("    • Connect without password:  agm ssh {}", target);
                println!("    • Verify public key status:  agm ssh nodes");
                println!(
                    "    • Execute remote test:       agm ssh exec {} \"whoami\"",
                    target
                );
                println!("    • Inspect Failed Commands:   agm failed-commands\n");
            }
        }
        Err(e) => {
            eprintln!("[ERROR] Deploy public key failed: {}", e);
            std::process::exit(1);
        }
    }
}

pub(crate) fn handle_ssh_auth(args: &[String]) {
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" {
        println!("\nAGM SSH Authorization Management (GitMap Parity)");
        println!("\nUsage:");
        println!("  agm ssh auth deploy <target> [-i <pubkey>]");
        println!("  agm ssh auth-key deploy <target> [-i <pubkey>]");
        println!("  agm ssh auth add <public-key-or-file>");
        println!("  agm ssh auth-key-add <public-key-or-file>");
        println!("  agm ssh auth export [dir]");
        println!("  agm ssh auth import [dir]");
        println!("  agm ssh auth list");
        println!("\nExamples:");
        println!("  agm ssh auth-key deploy w3");
        println!("  agm ssh auth-key-add ~/.ssh/id_ed25519.pub");
        println!("  agm ssh auth add \"ssh-ed25519 AAAAC3... user@host\"");
        println!("  agm ssh auth export");
        return;
    }

    let subcmd = args[0].to_lowercase();
    match subcmd.as_str() {
        "deploy" | "fix" => {
            let mut gitmap_forward = vec!["fix-auth".to_string()];
            gitmap_forward.extend(args[1..].iter().cloned());
            if crate::update_helpers::forward_to_gitmap_ssh(&gitmap_forward) {
                return;
            }
            handle_ssh_fix_auth(&args[1..]);
        }
        "add" | "install" => {
            let mut gitmap_forward = vec!["auth-key-add".to_string()];
            gitmap_forward.extend(args[1..].iter().cloned());
            if crate::update_helpers::forward_to_gitmap_ssh(&gitmap_forward) {
                return;
            }
            let key_arg = args.get(1).map(|s| s.as_str());
            if let Some(target) = key_arg {
                match ssh_manager::install_authorized_key_local(target) {
                    Ok(updated_files) => {
                        if updated_files.is_empty() {
                            println!("[SUCCESS] Public key already authorized in authorized_keys (deduplicated).");
                        } else {
                            println!(
                                "[SUCCESS] Installed authorized key into: {:?}",
                                updated_files
                            );
                        }
                        println!("\n  💡 Next Steps & Key Deployment Suggestions:");
                        println!("    • Deploy keys to mesh nodes: agm ssh deploy-keys");
                        println!("    • Verify SSH Fleet Nodes:    agm ssh nodes");
                        println!("    • Inspect Failed Commands:   agm failed-commands\n");
                    }
                    Err(e) => {
                        eprintln!("[ERROR] Failed to install authorized key: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("[ERROR] Missing public key string or .pub file path. Example: agm ssh auth add ~/.ssh/id_ed25519.pub");
                std::process::exit(1);
            }
        }
        "export" | "export-all" => {
            let dir_opt = args.get(1).map(|s| s.as_str());
            match ssh_manager::export_all_bundle(dir_opt) {
                Ok((path, keys_cnt, nodes_cnt)) => {
                    println!(
                        "[SUCCESS] Exported SSH authorization bundle to {}:",
                        path.display()
                    );
                    println!("  ● Public Keys Exported: {}", keys_cnt);
                    println!("  ● Fleet Nodes Exported: {}", nodes_cnt);
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to export SSH authorization bundle: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "import" | "import-all" => {
            let dir_opt = args.get(1).map(|s| s.as_str());
            match ssh_manager::import_all_bundle(dir_opt) {
                Ok((keys_installed, stats)) => {
                    println!("[SUCCESS] Imported SSH authorization bundle:");
                    println!("  ● Authorized Keys Installed Locally: {}", keys_installed);
                    println!(
                        "  ● Fleet Nodes Synced: Total: {}, Inserted: {}, Updated: {}, Unchanged: {}",
                        stats.total, stats.inserted, stats.updated, stats.unchanged
                    );
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to import SSH authorization bundle: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "list" | "ls" => {
            handle_ssh_keys(&["ls".to_string()]);
        }
        _ => {
            if subcmd.starts_with("ssh-")
                || subcmd.starts_with("ecdsa-")
                || std::path::Path::new(&subcmd).exists()
            {
                let mut gitmap_forward = vec!["auth-key-add".to_string()];
                gitmap_forward.push(args[0].clone());
                if crate::update_helpers::forward_to_gitmap_ssh(&gitmap_forward) {
                    return;
                }
                match ssh_manager::install_authorized_key_local(&args[0]) {
                    Ok(updated_files) => {
                        if updated_files.is_empty() {
                            println!("[SUCCESS] Public key already authorized (deduplicated).");
                        } else {
                            println!(
                                "[SUCCESS] Installed authorized key into: {:?}",
                                updated_files
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("[ERROR] Failed to install authorized key: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                let mut gitmap_forward = vec!["fix-auth".to_string()];
                gitmap_forward.extend(args.iter().cloned());
                if crate::update_helpers::forward_to_gitmap_ssh(&gitmap_forward) {
                    return;
                }
                handle_ssh_fix_auth(args);
            }
        }
    }
}

pub(crate) fn handle_ssh_keys(args: &[String]) {
    let subcmd = args
        .first()
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "ls".to_string());
    let gitmap_args = match subcmd.as_str() {
        "ls" | "list" => vec!["list".to_string()],
        "create" | "add" | "gen" => {
            let mut v = vec!["create".to_string()];
            v.extend(args.get(1..).unwrap_or(&[]).iter().cloned());
            v
        }
        "copy" | "cp" => {
            let mut v = vec!["copy".to_string()];
            v.extend(args.get(1..).unwrap_or(&[]).iter().cloned());
            v
        }
        "cat" | "view" => {
            let mut v = vec!["cat".to_string()];
            v.extend(args.get(1..).unwrap_or(&[]).iter().cloned());
            v
        }
        "rm" | "delete" => {
            let mut v = vec!["delete".to_string()];
            v.extend(args.get(1..).unwrap_or(&[]).iter().cloned());
            v
        }
        "config" => vec!["config".to_string()],
        "export" | "export-json" | "export-bundle" | "import" | "import-json" | "import-bundle"
        | "authorize" | "auth" | "install" | "deploy" | "distribute" => Vec::new(),
        _ => vec!["list".to_string()],
    };

    if !gitmap_args.is_empty() && crate::update_helpers::forward_to_gitmap_ssh(&gitmap_args) {
        return;
    }

    match subcmd.as_str() {
        "create" | "add" | "gen" => {
            let name = args.get(1).map(|s| s.as_str()).unwrap_or("agm_ed25519");
            let comment = args.get(2).map(|s| s.as_str());
            match ssh_manager::create_ssh_key(name, comment) {
                Ok(rec) => {
                    println!(
                        "[SUCCESS] Created SSH key pair '{}' ({})",
                        rec.name, rec.key_type
                    );
                    println!("  ● Public Path:  {}", rec.public_path);
                    println!("  ● Private Path: {}", rec.private_path);
                    println!("  ● Fingerprint:  {}", rec.fingerprint);
                }
                Err(e) => eprintln!("[ERROR] Failed to create SSH key: {}", e),
            }
        }
        "delete" | "rm" => {
            if let Some(name) = args.get(1) {
                match ssh_manager::delete_ssh_key(name) {
                    Ok(msg) => println!("[SUCCESS] {}", msg),
                    Err(e) => eprintln!("[ERROR] {}", e),
                }
            } else {
                eprintln!(
                    "[ERROR] Missing key name to delete. Example: agm ssh keys rm id_ed25519"
                );
            }
        }
        "copy" | "cp" => {
            let name = args.get(1).map(|s| s.as_str());
            match ssh_manager::copy_ssh_public_key(name) {
                Ok(rec) => {
                    println!("[SUCCESS] Copied public key '{}' to clipboard!", rec.name);
                    println!("  ● Key: {}", rec.public_key);
                }
                Err(e) => eprintln!("[ERROR] Failed to copy key: {}", e),
            }
        }
        "cat" | "view" => {
            let name = args.get(1).map(|s| s.as_str());
            match ssh_manager::copy_ssh_public_key(name) {
                Ok(rec) => println!("{}", rec.public_key),
                Err(e) => eprintln!("[ERROR] Failed to read key: {}", e),
            }
        }
        "config" => match ssh_manager::update_ssh_config(false) {
            Ok(msg) => println!("[SUCCESS] {}", msg),
            Err(e) => eprintln!("[ERROR] Failed to update ssh config: {}", e),
        },
        "export" | "export-json" | "export-bundle" => {
            let dir_opt = args.get(1).map(|s| s.as_str());
            match ssh_manager::export_all_bundle(dir_opt) {
                Ok((path, keys_cnt, nodes_cnt)) => {
                    println!("[SUCCESS] Exported SSH bundle to {}:", path.display());
                    println!("  ● Public Keys Exported: {}", keys_cnt);
                    println!("  ● Fleet Nodes Exported: {}", nodes_cnt);
                }
                Err(e) => eprintln!("[ERROR] Failed to export SSH bundle: {}", e),
            }
        }
        "import" | "import-json" | "import-bundle" => {
            let dir_opt = args.get(1).map(|s| s.as_str());
            match ssh_manager::import_all_bundle(dir_opt) {
                Ok((keys_installed, stats)) => {
                    println!("[SUCCESS] Imported SSH bundle:");
                    println!("  ● Authorized Keys Installed Locally: {}", keys_installed);
                    println!(
                        "  ● Fleet Nodes Synced: Total: {}, Inserted: {}, Updated: {}, Unchanged: {}",
                        stats.total, stats.inserted, stats.updated, stats.unchanged
                    );
                }
                Err(e) => eprintln!("[ERROR] Failed to import SSH bundle: {}", e),
            }
        }
        "authorize" | "auth" | "install" => {
            if let Some(target) = args.get(1) {
                match ssh_manager::install_authorized_key_local(target) {
                    Ok(updated_files) => {
                        if updated_files.is_empty() {
                            println!("[SUCCESS] Public key already authorized in ~/.ssh/authorized_keys (deduplicated).");
                        } else {
                            println!(
                                "[SUCCESS] Installed authorized key into: {:?}",
                                updated_files
                            );
                        }
                    }
                    Err(e) => eprintln!("[ERROR] Failed to install authorized key: {}", e),
                }
            } else {
                eprintln!("[ERROR] Missing public key string or .pub file path. Example: agm ssh keys authorize ~/.ssh/id_ed25519.pub");
            }
        }
        "deploy" | "distribute" => {
            handle_ssh_deploy_keys(&args[1..]);
        }
        _ => match ssh_manager::discover_local_ssh_keys() {
            Ok(keys) => {
                println!("\n  Managed Local SSH Keys ({} total):", keys.len());
                println!("  --------------------------------------------------------------------------------");
                println!(
                    "  {:<20} {:<10} {:<30} {:<20}",
                    "NAME", "TYPE", "FINGERPRINT", "CREATED"
                );
                println!("  --------------------------------------------------------------------------------");
                for k in &keys {
                    let fp = if k.fingerprint.len() > 28 {
                        &k.fingerprint[..28]
                    } else {
                        &k.fingerprint
                    };
                    println!(
                        "  {:<20} {:<10} {:<30} {:<20}",
                        k.name, k.key_type, fp, k.created_at
                    );
                }
                println!("  --------------------------------------------------------------------------------\n");
            }
            Err(e) => eprintln!("[ERROR] Failed to list SSH keys: {}", e),
        },
    }
}
