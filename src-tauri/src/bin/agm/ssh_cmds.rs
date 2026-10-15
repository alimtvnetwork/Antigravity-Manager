//! ssh_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::env;
use std::io::{self, BufRead, Write};
use std::process::{Command, Stdio};

pub(crate) fn handle_ssh_nodes(args: &[String]) {
    let is_csv = args.iter().any(|a| a.eq_ignore_ascii_case("--csv"))
        || args
            .windows(2)
            .any(|w| w[0].eq_ignore_ascii_case("--format") && w[1].eq_ignore_ascii_case("csv"));
    if is_csv {
        match telegram_inbound::generate_export_dataset("nodes", "csv") {
            Ok((_name, _mime, csv)) => {
                print!("{}", csv);
                return;
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to export SSH nodes CSV: {}", e);
                std::process::exit(1);
            }
        }
    }
    let is_json_flag = args.iter().any(|a| a == "--json" || a == "-j");
    if is_json_flag {
        match telegram_inbound::generate_export_dataset("nodes", "json") {
            Ok((_name, _mime, j)) => {
                println!("{}", j);
                return;
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to export SSH nodes JSON: {}", e);
                std::process::exit(1);
            }
        }
    }

    let mut gitmap_args = Vec::new();
    let first = args.first().map(|s| s.to_lowercase()).unwrap_or_default();
    if first == "nodes" {
        gitmap_args.push("nodes".to_string());
        gitmap_args.extend(args[1..].iter().cloned());
    } else if first == "export-json" || first == "nodes-export" {
        gitmap_args.push("nodes".to_string());
        gitmap_args.push("export-json".to_string());
        gitmap_args.extend(args[1..].iter().cloned());
    } else if first == "import-json" || first == "nodes-import" {
        gitmap_args.push("nodes".to_string());
        gitmap_args.push("import-json".to_string());
        gitmap_args.extend(args[1..].iter().cloned());
    } else {
        gitmap_args.extend(args.iter().cloned());
    }

    if crate::update_helpers::forward_to_gitmap_ssh(&gitmap_args) {
        return;
    }

    let is_export = args.iter().any(|a| a == "export-json" || a == "export");
    let is_import = args.iter().any(|a| a == "import-json" || a == "import");
    let file_arg = args
        .iter()
        .find(|a| {
            !a.starts_with('-')
                && *a != "nodes"
                && *a != "export-json"
                && *a != "import-json"
                && *a != "ls"
                && *a != "list"
        })
        .map(|s| s.as_str());

    if is_export {
        match ssh_manager::export_nodes_json(file_arg) {
            Ok(p) => println!("[SUCCESS] Exported SSH nodes to {}", p.0.display()),
            Err(e) => eprintln!("[ERROR] Failed to export nodes: {}", e),
        }
        return;
    }

    if is_import {
        match ssh_manager::import_nodes_json(file_arg, None) {
            Ok((source, stats)) => {
                println!(
                    "[SUCCESS] Imported SSH nodes from {}. Total: {}, Inserted: {}, Updated: {}, Unchanged: {}",
                    source, stats.total, stats.inserted, stats.updated, stats.unchanged
                );
            }
            Err(e) => eprintln!("[ERROR] Failed to import nodes: {}", e),
        }
        return;
    }

    match ssh_manager::load_ssh_connections() {
        Ok(conns) => {
            println!("\n  Registered SSH Fleet Nodes ({} total):", conns.len());
            println!("  --------------------------------------------------------------------------------");
            println!(
                "  {:<15} {:<22} {:<15} {:<10}",
                "ALIAS", "HOST (IP)", "USER", "OS"
            );
            println!("  --------------------------------------------------------------------------------");
            for c in &conns {
                println!(
                    "  {:<15} {:<22} {:<15} {:<10}",
                    c.alias, c.ip_address, c.username, c.os
                );
            }
            println!("  --------------------------------------------------------------------------------\n");
            if conns.is_empty() {
                println!("  (No SSH nodes currently enrolled)\n");
            }
            println!("  💡 SSH Fleet Optimization & Key Management Suggestions:");
            println!("    • Deploy & Sync SSH Keys:  agm ssh deploy-keys [alias] (or gitmap ssh deploy-keys)");
            println!("    • Add Key to Remote Host:  agm ssh add-key <alias> [key-path]");
            println!("    • Copy ID to Remote Host:  agm ssh copy-id <alias>");
            println!("    • Execute Fleet Command:   agm ssh exec all \"uptime\"");
            println!("    • Backup Fleet Registry:   agm ssh nodes export-json");
            println!("    • Clear Terminal:          agm clear-terminal");
            println!("    • Inspect Failed Commands: agm failed-commands\n");
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to load SSH nodes: {}", e);
            eprintln!("\n  💡 Troubleshooting Suggestions:");
            eprintln!("    • Check SSH registry:      gitmap ssh ls");
            eprintln!("    • Re-import SSH nodes:     agm ssh nodes import-json <file>");
            eprintln!("    • Inspect Failed Commands: agm failed-commands\n");
        }
    }
}

pub(crate) fn handle_ssh_exec(args: &[String]) {
    let mut gitmap_args = vec!["exec".to_string()];
    gitmap_args.extend(args.iter().cloned());
    if crate::update_helpers::forward_to_gitmap_ssh(&gitmap_args) {
        return;
    }

    if args.is_empty() {
        println!("Execute remote commands across SSH machines with automatic liveness checks.");
        println!("\nUsage:");
        println!("  agm ssh exec [target] \"<command>\" [flags]");
        println!("  agm se [target] \"<command>\" [flags]");
        println!("\nExamples:");
        println!("  agm ssh exec \"uptime\"");
        println!("  agm ssh exec devbox \"uname -a && df -h\"");
        println!("  agm ssh exec all gitmap --version");
        return;
    }

    let target = args[0].as_str();
    let cmd_slice = if args.len() > 1 { &args[1..] } else { args };
    match ssh_manager::exec_ssh_command(target, cmd_slice, None, None, None) {
        Ok(results) => {
            for r in results {
                println!(
                    "\n--- [{}] ({}) exit: {} ({}ms) ---",
                    r.alias, r.ip_address, r.exit_code, r.duration_ms
                );
                if !r.stdout.is_empty() {
                    print!("{}", r.stdout);
                }
                if !r.stderr.is_empty() {
                    eprint!("{}", r.stderr);
                }
            }
        }
        Err(e) => eprintln!("[ERROR] Remote execution failed: {}", e),
    }
}

pub(crate) fn cmd_ssh(args: &[String]) {
    if args.is_empty()
        || args
            .iter()
            .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Remote SSH Fleet & Key Management (GitMap Parity):");
        println!("  agm ssh <[user@]host> [-p <port>] [--password <pwd>] [--update] [cmd...]");
        println!("  agm ssh exec [target] \"<command>\" [--json]");
        println!("  agm ssh deploy-keys [all] [--dry-run] [--json]");
        println!("  agm ssh fix-auth <target> [-i <identity_pubkey>]");
        println!("  agm ssh copy-id <target> [-i <identity_pubkey>]");
        println!("  agm ssh keys [ls|create|copy|cat|rm|config]");
        println!("  agm ssh nodes [ls]");
        println!("  agm ssh nodes export-json [file]");
        println!("  agm ssh nodes import-json [file]");
        println!("  agm ssh bundle export|import [dir]");
        println!("\nDescription:");
        println!("  Manages remote cluster nodes, SSH credentials, authorized_keys distribution,");
        println!("  and executes remote terminal commands or automated updates across machines.");
        println!("\nKey & Fleet Subcommands:");
        println!("  exec [target] \"<cmd>\"     Execute remote command across fleet machines (alias: se)");
        println!("  deploy-keys [all]         Gather, deduplicate, and deploy SSH public keys across fleet");
        println!(
            "  fix-auth <target>         Deploy public key to remote host's ~/.ssh/authorized_keys"
        );
        println!("  copy-id <target>          Alias for fix-auth (native ssh-copy-id style)");
        println!("  keys [ls|create|copy|rm]  Manage local SSH key pairs and ~/.ssh/config");
        println!("  nodes [ls]                List all registered SSH cluster fleet nodes");
        println!("  nodes export-json [file]  Export SSH nodes to portable JSON (default: gitmap-ssh-nodes.json)");
        println!("  nodes import-json [file]  Import SSH nodes from portable JSON envelope");
        println!(
            "  bundle export|import      Export/import all keys, nodes, and public key bundle"
        );
        println!("\nRemote Execution Options:");
        println!("    -p, --port <port>   Custom SSH port (default: 22)");
        println!("    --password <pwd>    Password for SSH authentication");
        println!("    --update            Trigger remote update on target host");
        println!("\nExamples:");
        println!(
            "  agm ssh exec all \"uname -a\"               # Execute command across all nodes"
        );
        println!(
            "  agm ssh deploy-keys                       # Synchronize public keys across fleet"
        );
        println!(
            "  agm ssh fix-auth root@192.168.1.50        # Authorize public key on remote host"
        );
        println!("  agm ssh keys                              # List all managed local SSH keys");
        println!(
            "  agm ssh nodes                             # Display registered SSH cluster nodes"
        );
        println!("  agm ssh nodes export-json                 # Export fleet nodes to gitmap-ssh-nodes.json");
        println!("  agm ssh root@192.168.1.50                 # Connect to remote VM");
        println!(
            "  agm ssh root@192.168.1.50 --update        # Remotely update AGM binary on target"
        );
        println!("  agm ssh root@192.168.1.50 agm status      # Execute remote agm status");
        if args.is_empty() {
            std::process::exit(1);
        }
        return;
    }

    if args
        .iter()
        .any(|a| a.eq_ignore_ascii_case("--install-gitmap"))
    {
        match ssh_manager::install_gitmap_via_powershell(true) {
            Ok(msg) => println!("[SUCCESS] {}", msg),
            Err(e) => eprintln!("[ERROR] {}", e),
        }
        return;
    }

    let gm_status = ssh_manager::check_gitmap_installed_status();
    if !gm_status.gitmap_installed {
        eprintln!(
            "  💡 Tip: GitMap CLI is not installed on this machine. Run `agm ssh install-gitmap` or `{}`",
            gm_status.suggested_install_cmd
        );
    }

    let first = args[0].to_lowercase();
    if first == "install-gitmap" || first == "install_gitmap" {
        match ssh_manager::install_gitmap_via_powershell(true) {
            Ok(msg) => println!("[SUCCESS] {}", msg),
            Err(e) => eprintln!("[ERROR] {}", e),
        }
        return;
    }

    if first == "import-gitmap" || first == "import_gitmap" {
        match ssh_manager::import_ssh_from_gitmap() {
            Ok((source, stats)) => println!(
                "[SUCCESS] Imported SSH nodes from {}. Total: {}, Inserted: {}, Updated: {}, Unchanged: {} (RSA-2048 encrypted)",
                source, stats.total, stats.inserted, stats.updated, stats.unchanged
            ),
            Err(e) => eprintln!("[ERROR] Failed to import SSH nodes from GitMap: {}", e),
        }
        return;
    }

    if first == "balance" || first == "balances" || first == "quota" {
        crate::quota_cmds::cmd_quota_show(&args[1..]);
        return;
    }

    if first == "status" {
        let gm = ssh_manager::check_gitmap_installed_status();
        println!("\n  [*] SSH & GitMap Fleet Status:");
        println!(
            "      GitMap Installed:  {} ({})",
            if gm.gitmap_installed { "YES" } else { "NO" },
            gm.gitmap_version
                .unwrap_or_else(|| "not detected".to_string())
        );
        println!(
            "      RSA Vault Status:  {}",
            if gm.rsa_vault_initialized {
                "RSA-2048 Initialized"
            } else {
                "Pending First KeyGen"
            }
        );
        if let Some(fp) = gm.rsa_public_fingerprint {
            println!("      RSA Public Key:    {}", fp);
        }
        handle_ssh_nodes(&args[1..]);
        return;
    }

    if first == "prompt" {
        if args.len() < 4 {
            eprintln!("Usage: agm ssh prompt <node> <instance> <prompt_text...>");
            std::process::exit(1);
        }
        let node = args[1].clone();
        let inst = args[2].clone();
        let text = args[3..].join(" ");
        let payload = ssh_manager::RemotePromptDispatchPayload {
            node_alias: node,
            instance_name: inst,
            prompt_text: text,
            conversation_id: None,
        };
        if let Ok(rt) = tokio::runtime::Runtime::new() {
            match rt.block_on(ssh_manager::send_remote_ssh_prompt(&payload)) {
                Ok(res) => println!("[SUCCESS] [{}] {}", res.dispatch_route, res.message),
                Err(e) => eprintln!("[ERROR] {}", e),
            }
        }
        return;
    }

    if first == "add" || first == "save" {
        let mut positional = Vec::new();
        let mut port: u16 = 22;
        let mut raw_password: Option<String> = None;
        let mut key_path: Option<String> = None;
        let mut idx = 1;
        while idx < args.len() {
            let a = &args[idx];
            if (a == "--port" || a == "-p") && idx + 1 < args.len() {
                port = args[idx + 1].parse().unwrap_or(22);
                idx += 2;
            } else if a == "--password" && idx + 1 < args.len() {
                raw_password = Some(args[idx + 1].clone());
                idx += 2;
            } else if (a == "--key" || a == "-i") && idx + 1 < args.len() {
                key_path = Some(args[idx + 1].clone());
                idx += 2;
            } else if !a.starts_with('-') {
                positional.push(a.clone());
                idx += 1;
            } else {
                idx += 1;
            }
        }
        if positional.len() < 2 {
            eprintln!("Usage: agm ssh add <alias> <ip> [user] [--port 22] [--password <pw>]");
            std::process::exit(1);
        }
        let alias = positional[0].clone();
        let host = positional[1].clone();
        let username = positional
            .get(2)
            .cloned()
            .unwrap_or_else(|| "Administrator".to_string());
        let payload = ssh_manager::SaveSshNodeRsaPayload {
            id: None,
            alias,
            host,
            port,
            username,
            raw_password,
            key_path,
            os: Some("windows".to_string()),
        };
        match ssh_manager::upsert_ssh_node_rsa(&payload) {
            Ok(rec) => println!(
                "[SUCCESS] Saved SSH node '{}' ({}@{}:{}) with {} encryption.",
                rec.alias, rec.username, rec.host, rec.port, rec.encryption_algo
            ),
            Err(e) => eprintln!("[ERROR] {}", e),
        }
        return;
    }

    if first == "exec" || first == "se" {
        handle_ssh_exec(&args[1..]);
        return;
    }

    if first == "deploy-keys"
        || first == "deploy_keys"
        || (first == "deploy" && args.get(1).map(|s| s.as_str()) == Some("keys"))
    {
        let offset = if first == "deploy" { 2 } else { 1 };
        crate::ssh_handlers::handle_ssh_deploy_keys(&args[offset..]);
        return;
    }

    if first == "fix-auth" || first == "fix_auth" || first == "copy-id" || first == "copy_id" {
        crate::ssh_handlers::handle_ssh_fix_auth(&args[1..]);
        return;
    }

    if first == "auth-key-add" || first == "ssh-key-add" || first == "key-add" || first == "add-key"
    {
        crate::ssh_handlers::handle_ssh_auth(args);
        return;
    }

    if first == "auth-key" || first == "auth-keys" || first == "auth" {
        crate::ssh_handlers::handle_ssh_auth(&args[1..]);
        return;
    }

    if first == "keys" || first == "key" {
        crate::ssh_handlers::handle_ssh_keys(&args[1..]);
        return;
    }

    if first == "bundle" {
        let sub = args.get(1).map(|s| s.as_str()).unwrap_or("export");
        let dir = args.get(2).map(|s| s.as_str());
        if sub == "import" {
            match ssh_manager::import_all_bundle(dir) {
                Ok((keys, stats)) => println!(
                    "[SUCCESS] Imported SSH bundle: {} key(s) installed, {} node(s) synced",
                    keys, stats.total
                ),
                Err(e) => eprintln!("[ERROR] Failed to import SSH bundle: {}", e),
            }
        } else {
            match ssh_manager::export_all_bundle(dir) {
                Ok((p, keys, nodes)) => println!(
                    "[SUCCESS] Exported SSH bundle to {}: {} key(s), {} node(s)",
                    p.display(),
                    keys,
                    nodes
                ),
                Err(e) => eprintln!("[ERROR] Failed to export SSH bundle: {}", e),
            }
        }
        return;
    }

    if first == "nodes"
        || first == "export-json"
        || first == "import-json"
        || first == "nodes-export"
        || first == "nodes-import"
        || first == "export"
        || first == "import"
        || first == "ls"
        || first == "list"
    {
        handle_ssh_nodes(args);
        return;
    }

    let mut target = String::new();
    let mut port = "22".to_string();
    let mut password: Option<String> = None;
    let mut is_update = false;
    let mut remote_cmd: Vec<String> = Vec::new();

    let mut idx = 0;
    while idx < args.len() {
        let arg = &args[idx];
        if arg == "-p" || arg == "--port" {
            if idx + 1 < args.len() {
                port = args[idx + 1].clone();
                idx += 2;
                continue;
            }
        } else if arg == "--password" {
            if idx + 1 < args.len() {
                password = Some(args[idx + 1].clone());
                idx += 2;
                continue;
            }
        } else if arg == "--update" {
            is_update = true;
            idx += 1;
            continue;
        } else if target.is_empty() && !arg.starts_with('-') {
            target = arg.clone();
            idx += 1;
            continue;
        } else {
            remote_cmd.push(arg.clone());
            idx += 1;
        }
    }

    if target.is_empty() {
        eprintln!("[ERROR] Missing SSH host target.");
        eprintln!("\n  💡 It is not there, but here is a suggestion you can try:");
        eprintln!("    • List available SSH nodes:  agm ssh nodes");
        eprintln!("    • Deploy keys to all nodes:  agm ssh deploy-keys");
        eprintln!("    • Connect by alias or IP:    agm ssh <alias|user@ip>");
        eprintln!("    • Run command across fleet:  agm ssh exec all \"uptime\"\n");
        std::process::exit(1);
    }

    let mut effective_target = target.clone();
    let effective_port = port.clone();
    if let Ok(matched_nodes) = ssh_manager::resolve_target_nodes(&target, None) {
        if let Some(node) = matched_nodes.first() {
            if !node.username.is_empty() {
                effective_target = format!("{}@{}", node.username, node.ip_address);
            } else {
                effective_target = node.ip_address.clone();
            }
        }
    }

    println!(
        "[*] Connecting to SSH target '{}' (port {})...",
        effective_target, effective_port
    );

    let final_cmd = if is_update {
        println!("[*] Auto-update mode active: will execute AGM update on remote VM.");
        "curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh 2>/dev/null | bash || powershell -Command \"irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex\"".to_string()
    } else if !remote_cmd.is_empty() {
        remote_cmd.join(" ")
    } else {
        String::new()
    };

    let effective_password = if password.is_some() {
        password
    } else {
        let test_res = Command::new("ssh")
            .args([
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=4",
                "-p",
                &effective_port,
                &effective_target,
                "exit",
            ])
            .output();

        let needs_password = match test_res {
            Ok(out) => out.status.code().unwrap_or(1) != 0,
            Err(_) => true,
        };

        if needs_password {
            print!("Enter SSH password for '{}': ", effective_target);
            let _ = io::stdout().flush();
            let pwd = crate::common::read_password_masked();
            Some(pwd)
        } else {
            None
        }
    };

    let mut ssh = Command::new("ssh");
    ssh.arg("-p").arg(&effective_port);

    if let Some(ref pwd) = effective_password {
        #[cfg(target_os = "windows")]
        {
            env::set_var("SSH_PASSWORD", pwd);
        }
        #[cfg(not(target_os = "windows"))]
        {
            if Command::new("sshpass").arg("-V").output().is_ok() {
                let mut pass_cmd = Command::new("sshpass");
                pass_cmd
                    .arg("-p")
                    .arg(pwd)
                    .arg("ssh")
                    .arg("-p")
                    .arg(&effective_port)
                    .arg(&effective_target);
                if !final_cmd.is_empty() {
                    pass_cmd.arg(&final_cmd);
                }
                let _ = pass_cmd.status();
                return;
            }
        }
    }

    ssh.arg(&effective_target);
    if !final_cmd.is_empty() {
        ssh.arg(&final_cmd);
    }

    ssh.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    match ssh.status() {
        Ok(s) => {
            let code = s.code().unwrap_or(0);
            if code != 0 {
                eprintln!("[*] SSH session exited with code: {}", code);
                eprintln!("\n  💡 SSH Troubleshooting Suggestions:");
                eprintln!(
                    "    • Deploy public key to host: agm ssh deploy-keys {}",
                    effective_target
                );
                eprintln!("    • Install SSH public key:    agm ssh add-key <key>");
                eprintln!("    • Check remote fleet nodes:  agm ssh nodes");
                eprintln!("    • Diagnose via GitMap:       gitmap ssh health\n");
            }
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to execute 'ssh': {}", e);
            eprintln!("Ensure OpenSSH client is installed and accessible in your system PATH.");
            eprintln!("\n  💡 Suggestions:");
            eprintln!(
                "    • Use native GitMap SSH:     gitmap ssh {}",
                effective_target
            );
            eprintln!("    • Check system doctor:       agm doctor\n");
        }
    }
}
