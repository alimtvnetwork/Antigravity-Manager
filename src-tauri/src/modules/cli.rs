use crate::modules::{account, auto_switcher, config, instance, repo_db};
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
fn attach_parent_console() {
    #[link(name = "Kernel32")]
    extern "system" {
        fn AttachConsole(dw_process_id: u32) -> i32;
    }
    const ATTACH_PARENT_PROCESS: u32 = 0xFFFFFFFF;
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

/// Unified argument extractor for AGM CLI commands
#[derive(Debug, Clone, Default)]
pub struct CliContext {
    pub instance_id: Option<String>,
    pub repo_path: Option<String>,
    pub json_output: bool,
    pub all_instances: bool,
    pub verbose: bool,
    pub dry_run: bool,
    pub positional_args: Vec<String>,
}

impl CliContext {
    pub fn parse(args: &[String]) -> Self {
        let mut ctx = Self::default();
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-i" | "--instance" | "--profile" => {
                    if i + 1 < args.len() {
                        ctx.instance_id = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                "-r" | "--repo" | "--workspace" => {
                    if i + 1 < args.len() {
                        ctx.repo_path = Some(args[i + 1].clone());
                        i += 1;
                    }
                }
                "--json" | "-j" => {
                    ctx.json_output = true;
                }
                "-a" | "--all" => {
                    ctx.all_instances = true;
                }
                "-v" | "--verbose" => {
                    ctx.verbose = true;
                }
                "--dry-run" => {
                    ctx.dry_run = true;
                }
                arg if !arg.starts_with('-') => {
                    ctx.positional_args.push(arg.to_string());
                }
                _ => {}
            }
            i += 1;
        }
        ctx
    }

    /// Resolves canonical instance ID or falls back to active instance
    pub fn resolve_target_instance(&self) -> Result<String, String> {
        if let Some(ref raw_id) = self.instance_id {
            crate::modules::instance::resolve_instance_id(raw_id)
        } else {
            crate::modules::instance::get_active_instance_id()
        }
    }
}

/// Generic JSON envelope for standard CLI responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliEnvelope<T: Serialize> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<CliErrorPayload>,
    pub meta: CliMetaPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliErrorPayload {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliMetaPayload {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    pub timestamp: String,
    pub version: String,
}

impl<T: Serialize> CliEnvelope<T> {
    pub fn ok(command: &str, instance_id: Option<String>, data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: CliMetaPayload {
                command: command.to_string(),
                instance_id,
                timestamp: Utc::now().to_rfc3339(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    pub fn err(command: &str, instance_id: Option<String>, code: &str, message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(CliErrorPayload {
                code: code.to_string(),
                message: message.to_string(),
            }),
            meta: CliMetaPayload {
                command: command.to_string(),
                instance_id,
                timestamp: Utc::now().to_rfc3339(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        }
    }

    pub fn print_and_exit(&self) -> ! {
        let json_str = serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string());
        println!("{}", json_str);
        if self.success {
            std::process::exit(0);
        } else {
            std::process::exit(1);
        }
    }
}

/// Check if the local Antigravity runtime daemon is running on 127.0.0.1:8045
pub fn is_daemon_running() -> bool {
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;
    let addr = SocketAddr::from(([127, 0, 0, 1], 8045));
    TcpStream::connect_timeout(&addr, Duration::from_millis(150)).is_ok()
}

/// Helper to forward CLI actions to the local authenticated REST API daemon
pub fn forward_to_local_rest<T: serde::de::DeserializeOwned>(
    method: reqwest::Method,
    path: &str,
    body: Option<serde_json::Value>,
) -> Result<T, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let admin_token = std::env::var("AGM_ADMIN_TOKEN")
        .or_else(|_| std::env::var("ABV_API_KEY"))
        .unwrap_or_else(|_| {
            crate::modules::config::load_app_config()
                .ok()
                .and_then(|c| {
                    c.proxy
                        .admin_password
                        .filter(|p| !p.is_empty())
                        .or(Some(c.proxy.api_key))
                })
                .unwrap_or_default()
        });

    let norm_path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };
    let url = format!("http://127.0.0.1:8045/api{}", norm_path);
    let mut req = client.request(method, &url);
    if !admin_token.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", admin_token));
        req = req.header("x-api-key", &admin_token);
    }
    if let Some(json_payload) = body {
        req = req.json(&json_payload);
    }

    let resp = req
        .send()
        .map_err(|e| format!("Daemon communication failed: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("Daemon returned HTTP status {}", resp.status()));
    }

    resp.json::<T>()
        .map_err(|e| format!("Failed to parse response: {}", e))
}

/// Check and handle CLI arguments passed from terminal.
/// Returns true if a CLI command was handled (caller should exit).
pub fn handle_cli_arguments() -> bool {
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        return false;
    }

    #[cfg(target_os = "windows")]
    attach_parent_console();

    let raw_cmd = &args[1];
    let cmd = raw_cmd
        .trim_start_matches('/')
        .trim_start_matches('-')
        .to_lowercase();
    let rest_args = if args.len() > 2 {
        args[2..].to_vec()
    } else {
        Vec::new()
    };

    match cmd.as_str() {
        // Multi-Instance & Profile commands
        "instances" | "instance" | "intrance" | "intrances" | "profile" | "profiles" | "ls" => {
            handle_instance_subcommand(&rest_args);
            true
        }

        // Direct create commands (e.g. agm create, instance-create, intrance-create, create-profile)
        "create" | "create-instance" | "create_instance" | "instance-create"
        | "intrance-create" | "intrance_create" | "create-profile" | "cp" => {
            handle_instance_create(&rest_args);
            true
        }

        // Direct switch commands (e.g. agm switch, switch-account, swtich, swtich-account)
        "switch" | "switch-account" | "switch_account" | "account-switch" | "swtich"
        | "swtich-account" | "swtich_account" | "account-swtich" => {
            handle_switch_command(&rest_args);
            true
        }

        // Direct auto-switch daemon commands
        "auto-switch" | "auto-swtich" | "autoswitch" | "auto_switch" | "auto" | "switcher" => {
            handle_auto_switch_command(&rest_args);
            true
        }

        "list-profiles" | "lp" => {
            handle_instance_subcommand(&["list".to_string()]);
            true
        }

        "copy-profile" | "dp" => {
            if rest_args.len() < 2 {
                eprintln!("Error: Usage: antigravity-manager copy-profile <source_id> <new_name>");
                std::process::exit(1);
            }
            let source_id = &rest_args[0];
            let resolved_src =
                instance::resolve_instance_id(source_id).unwrap_or_else(|_| source_id.clone());
            let target_name = rest_args[1..].join(" ");
            match instance::copy_instance(&resolved_src, target_name, None) {
                Ok(new_config) => {
                    println!("[CLI] Successfully cloned profile:");
                    println!("  ID:       {}", new_config.id);
                    println!("  Name:     {}", new_config.name);
                    println!("  Data Dir: {}", new_config.data_dir);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Error copying profile: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "delete-profile" => {
            if rest_args.is_empty() {
                eprintln!("Error: Usage: antigravity-manager delete-profile <profile_id>");
                std::process::exit(1);
            }
            let profile_id = &rest_args[0];
            match instance::delete_instance(profile_id) {
                Ok(_) => {
                    println!("[CLI] Profile '{}' deleted successfully.", profile_id);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Error deleting profile: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "run-profile" => {
            if rest_args.is_empty() {
                eprintln!("Error: Usage: antigravity-manager run-profile <profile_id>");
                std::process::exit(1);
            }
            let profile_id = &rest_args[0];
            match instance::launch_instance(profile_id) {
                Ok(_) => {
                    println!("[CLI] Launched Antigravity with profile '{}'.", profile_id);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Error launching profile: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "fast-forward" | "ff" => {
            println!("[CLI] Initiating fast-forward profile rotation...");
            let rt = match tokio::runtime::Runtime::new() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error creating async runtime: {}", e);
                    std::process::exit(1);
                }
            };

            let target_spec = rest_args.first().map(|s| s.as_str());
            match rt.block_on(auto_switcher::trigger_manual_rotation_for_instance(
                target_spec,
            )) {
                Ok(msg) => {
                    println!("[CLI] Fast-forward success: {}", msg);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[CLI] Fast-forward failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "profile-help" | "help-profile" => {
            print_instance_cli_help();
            std::process::exit(0);
        }

        "help" | "h" => {
            if let Some(sub) = rest_args.first() {
                let sub_low = sub.trim_start_matches('-').to_lowercase();
                match sub_low.as_str() {
                    "instance" | "instances" | "intrance" | "create" => print_instance_cli_help(),
                    "switch" | "switch-account" | "account" => print_switch_cli_help(),
                    "auto-switch" | "autoswitch" | "auto" => print_auto_switch_cli_help(),
                    "agy" | "cache" | "clean" => print_agy_cli_help(),
                    "supabase" | "sb" => print_supabase_cli_help(),
                    "prompts" | "prompt" | "tree" => print_prompts_cli_help(),
                    "doctor" | "check" | "health" => print_doctor_cli_help(),
                    _ => print_all_cli_help(),
                }
            } else {
                print_all_cli_help();
            }
            std::process::exit(0);
        }

        "agy" | "agi" | "cli" => {
            if rest_args.is_empty() {
                print_agy_cli_help();
                std::process::exit(0);
            }
            handle_agy_subcommand(&rest_args);
            true
        }

        "cache-clear" | "clean-agy" | "clear-agy" => {
            handle_clear_action(&rest_args, 10);
            true
        }

        "cache-clear-keep-one" | "ccko" => {
            handle_clear_action(&rest_args, 1);
            true
        }

        "cache-clear-keep-five" | "cckf" => {
            handle_clear_action(&rest_args, 5);
            true
        }

        "undo" => {
            let tx_id = rest_args.first().map(|s| s.as_str());
            execute_agy_undo(tx_id);
            true
        }

        "delegate-update" | "update-ui" | "ui-update-runner" => {
            crate::modules::delegate_updater::run(&rest_args);
            std::process::exit(0);
        }

        "update" | "update-all" | "ua" => {
            let ok = crate::modules::delegate_updater::run_cli_update(&rest_args);
            std::process::exit(if ok { 0 } else { 1 });
        }

        "open-ui" | "ui" | "launch-ui" | "start-ui" => {
            crate::modules::delegate_updater::open_ui(&rest_args);
            std::process::exit(0);
        }

        "supabase" | "sb" => {
            handle_supabase_subcommand(&rest_args);
            true
        }

        // Prompt management commands
        "prompts" | "prompt" | "tree" => {
            handle_prompts_subcommand(&rest_args, cmd.as_str());
            true
        }

        // System diagnostic doctor command
        "doctor" | "check" | "health" => {
            handle_doctor_subcommand(&rest_args);
            true
        }

        _ => false,
    }
}

// -----------------------------------------------------------------------------
// Instances Subcommand Handler
// -----------------------------------------------------------------------------

fn handle_instance_subcommand(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        print_instance_cli_help();
        std::process::exit(0);
    }

    if args.is_empty() || args[0] == "ls" || args[0] == "list" {
        let is_json = is_flag_present(args, &["--json", "-j"]);
        match instance::list_instances() {
            Ok(instances) => {
                if is_json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&instances).unwrap_or_default()
                    );
                    std::process::exit(0);
                }
                println!(
                    "\nRegistered Antigravity Profiles ({} total):",
                    instances.len()
                );
                println!(
                    "{:<5} {:<18} {:<20} {:<18} {:<28} {}",
                    "#", "ID", "NAME", "STATUS", "BOUND EMAIL", "DATA DIR"
                );
                println!("{}", "-".repeat(115));
                for (idx, inst) in instances.iter().enumerate() {
                    let seq = idx + 1;
                    let status_str = if inst.is_running {
                        format!("Running (PID: {})", inst.pid.unwrap_or(0))
                    } else {
                        "Idle".to_string()
                    };
                    let email = inst.config.bound_email.as_deref().unwrap_or("-");
                    println!(
                        "#{:<4} {:<18} {:<20} {:<18} {:<28} {}",
                        seq,
                        inst.config.id,
                        inst.config.name,
                        status_str,
                        email,
                        inst.config.data_dir
                    );
                }
                println!();
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("[ERROR] Error listing profiles: {}", e);
                std::process::exit(1);
            }
        }
    }

    let sub = args[0].to_lowercase();
    match sub.as_str() {
        "create" | "add" | "new" => {
            handle_instance_create(&args[1..]);
        }
        "switch" | "use" | "swtich" => {
            if args.len() < 3 {
                eprintln!(
                    "[ERROR] Usage: antigravity-manager instance switch <instance> <account>"
                );
                std::process::exit(1);
            }
            handle_switch_command(&[args[1].clone(), args[2].clone()]);
        }
        "launch" | "start" | "run" => {
            if args.len() < 2 {
                eprintln!("[ERROR] Usage: antigravity-manager instance launch <instance_id>");
                std::process::exit(1);
            }
            let target = &args[1];
            let resolved = instance::resolve_instance_id(target).unwrap_or_else(|_| target.clone());
            match instance::launch_instance(&resolved) {
                Ok(_) => {
                    println!("[SUCCESS] Launched instance window for '{}'.", resolved);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to launch instance '{}': {}", resolved, e);
                    std::process::exit(1);
                }
            }
        }
        "stop" | "close" | "kill" => {
            if args.len() < 2 {
                eprintln!("[ERROR] Usage: antigravity-manager instance stop <instance_id>");
                std::process::exit(1);
            }
            let target = &args[1];
            let resolved = instance::resolve_instance_id(target).unwrap_or_else(|_| target.clone());
            match instance::stop_instance(&resolved) {
                Ok(_) => {
                    println!("[SUCCESS] Stopped instance '{}'.", resolved);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to stop instance '{}': {}", resolved, e);
                    std::process::exit(1);
                }
            }
        }
        "delete" | "rm" | "remove" => {
            if args.len() < 2 {
                eprintln!("[ERROR] Usage: antigravity-manager instance delete <instance_id>");
                std::process::exit(1);
            }
            let target = &args[1];
            let resolved = instance::resolve_instance_id(target).unwrap_or_else(|_| target.clone());
            match instance::delete_instance(&resolved) {
                Ok(_) => {
                    println!(
                        "[SUCCESS] Deleted instance '{}' and cleaned storage.",
                        resolved
                    );
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to delete instance '{}': {}", resolved, e);
                    std::process::exit(1);
                }
            }
        }
        "copy" | "clone" => {
            if args.len() < 3 {
                eprintln!(
                    "[ERROR] Usage: antigravity-manager instance copy <source_id> <new_name>"
                );
                std::process::exit(1);
            }
            let source_id = &args[1];
            let resolved_src =
                instance::resolve_instance_id(source_id).unwrap_or_else(|_| source_id.clone());
            let target_name = args[2..].join(" ");
            match instance::copy_instance(&resolved_src, target_name, None) {
                Ok(new_config) => {
                    println!(
                        "[SUCCESS] Cloned profile '{}' -> '{}' ({})",
                        source_id, new_config.name, new_config.id
                    );
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to copy instance: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "ff" | "fast-forward" | "rotate" => {
            let target = args.get(1).map(|s| s.as_str());
            let rt = tokio::runtime::Runtime::new().expect("Failed to initialize tokio runtime");
            match rt.block_on(auto_switcher::trigger_manual_rotation_for_instance(target)) {
                Ok(msg) => {
                    println!("[SUCCESS] {}", msg);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[ERROR] Fast-forward failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("[ERROR] Unknown instance subcommand '{}'.", sub);
            println!("Run 'antigravity-manager instance --help' for usage instructions.");
            std::process::exit(1);
        }
    }
}

fn handle_instance_create(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("Antigravity Instance Create Command:");
        println!("  antigravity-manager instance create <name> [options]");
        println!("\nOptions:");
        println!("  --account, -a <email|id>    Bind specific account (defaults to next available unbound)");
        println!("  --from, -f <source_id>      Clone configuration and extensions from an existing instance");
        println!("  --launch, -l                Immediately launch instance window after creation");
        println!("  --data-only, --do           Create isolated data directory only without cloning binary");
        println!(
            "  --json, -j                  Output created instance details in structured JSON"
        );
        println!("\nExamples:");
        println!(
            "  antigravity-manager instance create \"Work-Project\" -a \"work@gmail.com\" --launch"
        );
        println!("  antigravity-manager instance create \"Dev-Sandbox\"");
        println!("  antigravity-manager instance create \"Client-Clone\" --from \"Work-Project\"");
        std::process::exit(0);
    }

    let is_json = is_flag_present(args, &["--json", "-j"]);
    let should_launch = is_flag_present(args, &["--launch", "-l"]);
    let is_data_only = is_flag_present(args, &["--data-only", "-data-only", "--do", "-do"]);

    let mut target_account: Option<String> = None;
    let mut from_instance: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        let arg_lower = args[i].to_lowercase();
        if (arg_lower == "--account" || arg_lower == "-a" || arg_lower == "--acc")
            && i + 1 < args.len()
        {
            target_account = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        if (arg_lower == "--from" || arg_lower == "-f") && i + 1 < args.len() {
            from_instance = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        i += 1;
    }

    let non_flags: Vec<&String> = args
        .iter()
        .filter(|a| {
            !a.starts_with('-')
                && !target_account
                    .as_ref()
                    .map(|acc| acc == *a)
                    .unwrap_or(false)
                && !from_instance.as_ref().map(|src| src == *a).unwrap_or(false)
                && !a.eq_ignore_ascii_case("create")
                && !a.eq_ignore_ascii_case("add")
                && !a.eq_ignore_ascii_case("new")
        })
        .collect();

    let name = if !non_flags.is_empty() {
        non_flags
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        format!("Instance-{}", chrono::Utc::now().timestamp() % 1000)
    };

    println!("[*] Creating instance profile '{}'...", name);
    let create_res = if let Some(ref source) = from_instance {
        let resolved_src = instance::resolve_instance_id(source).unwrap_or_else(|_| source.clone());
        instance::copy_instance(&resolved_src, name.clone(), Some("full"))
    } else {
        instance::create_instance_with_account(name.clone(), target_account.as_deref())
    };

    match create_res {
        Ok(mut cfg) => {
            if !is_data_only {
                if let Ok(exe_path) = instance::clone_instance_executable(&cfg.id) {
                    cfg.executable_path = Some(exe_path);
                }
            }

            if let Some(ref acc_query) = target_account {
                if from_instance.is_some() {
                    let rt =
                        tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                    if let Ok(index) = account::load_account_index() {
                        let q_lower = acc_query.to_lowercase();
                        if let Some(target) = index.accounts.iter().find(|a| {
                            a.id == *acc_query
                                || a.email.to_lowercase() == q_lower
                                || a.email.to_lowercase().contains(&q_lower)
                        }) {
                            let _ = rt.block_on(instance::switch_account_to_instance(
                                &target.id,
                                Some(&cfg.id),
                            ));
                            cfg.bound_account_id = Some(target.id.clone());
                            cfg.bound_email = Some(target.email.clone());
                        }
                    }
                }
            }

            if should_launch {
                let _ = instance::launch_instance(&cfg.id);
            }

            if is_json {
                println!("{}", serde_json::to_string_pretty(&cfg).unwrap_or_default());
            } else {
                println!(
                    "[SUCCESS] Created profile #{}: '{}' (ID: {}, bound: {}, data_only: {}, dir: {})",
                    cfg.seq_num.unwrap_or(1),
                    cfg.name,
                    cfg.id,
                    cfg.bound_email.as_deref().unwrap_or("unbound"),
                    is_data_only,
                    cfg.data_dir
                );
                if should_launch {
                    println!("  [->] Antigravity IDE window launched immediately.");
                }
            }
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to create instance: {}", e);
            std::process::exit(1);
        }
    }
}

// -----------------------------------------------------------------------------
// Switch Account Command Handler
// -----------------------------------------------------------------------------

fn handle_switch_command(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        print_switch_cli_help();
        std::process::exit(0);
    }

    let is_json = is_flag_present(args, &["--json", "-j"]);
    let raw_non_flags: Vec<&String> = args
        .iter()
        .filter(|a| {
            !a.starts_with('-')
                && !a.eq_ignore_ascii_case("switch")
                && !a.eq_ignore_ascii_case("swtich")
                && !a.eq_ignore_ascii_case("use")
        })
        .collect();

    // Strip semantic filler keywords like "account", "to", "for"
    let mut non_flag_args: Vec<&String> = Vec::new();
    for (idx, arg) in raw_non_flags.iter().enumerate() {
        let lower = arg.to_lowercase();
        if (lower == "account" || lower == "acc" || lower == "to" || lower == "for")
            && (idx == 0 || idx + 1 < raw_non_flags.len())
            && raw_non_flags.len() > 1
        {
            continue;
        }
        non_flag_args.push(arg);
    }

    let index = match account::load_account_index() {
        Ok(idx) => idx,
        Err(e) => {
            eprintln!("[ERROR] Failed to load accounts index: {}", e);
            std::process::exit(1);
        }
    };

    if non_flag_args.is_empty() {
        let curr = account::get_current_account().ok().flatten();
        let active_inst =
            instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string());
        println!("AGM Profile Switch:");
        println!("  Current active instance: {}", active_inst);
        println!(
            "  Current active account:  {}",
            curr.as_ref().map(|a| a.email.as_str()).unwrap_or("(none)")
        );
        println!("\nUsage: antigravity-manager switch <email|id|#seq> [--instance <id>]");
        println!("       antigravity-manager switch <instance> <account>");
        println!("       antigravity-manager switch account <email>");
        println!("Run 'antigravity-manager switch --help' for full guide with examples.");
        std::process::exit(1);
    }

    let explicit_inst_opt = args
        .iter()
        .position(|a| a == "--instance" || a == "-i")
        .and_then(|pos| args.get(pos + 1).map(|s| s.as_str()));

    let (target_inst_spec, acc_query) = if non_flag_args.len() >= 2 {
        let first_is_instance = instance::resolve_instance_id(non_flag_args[0]).is_ok();
        let second_is_instance = instance::resolve_instance_id(non_flag_args[1]).is_ok();

        if first_is_instance && !second_is_instance {
            (Some(non_flag_args[0].as_str()), non_flag_args[1].as_str())
        } else if second_is_instance && !first_is_instance {
            (Some(non_flag_args[1].as_str()), non_flag_args[0].as_str())
        } else if explicit_inst_opt.is_some() {
            (explicit_inst_opt, non_flag_args[0].as_str())
        } else {
            (Some(non_flag_args[0].as_str()), non_flag_args[1].as_str())
        }
    } else {
        (explicit_inst_opt, non_flag_args[0].as_str())
    };

    let query_lower = acc_query.trim().to_lowercase();
    let index_match =
        if query_lower.starts_with('#') || query_lower.chars().all(|c| c.is_ascii_digit()) {
            let num_str = query_lower.trim_start_matches('#');
            if let Ok(num) = num_str.parse::<usize>() {
                if num >= 1 && num <= index.accounts.len() {
                    Some(&index.accounts[num - 1])
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

    let target_account = if let Some(acc) = index_match {
        acc
    } else {
        let matches: Vec<_> = index
            .accounts
            .iter()
            .filter(|a| {
                let email_l = a.email.to_lowercase();
                let id_l = a.id.to_lowercase();
                email_l == query_lower
                    || id_l == query_lower
                    || email_l.starts_with(&query_lower)
                    || email_l.contains(&query_lower)
                    || id_l.contains(&query_lower)
            })
            .collect();

        if matches.is_empty() {
            eprintln!("[ERROR] No account found matching '{}'.", acc_query);
            std::process::exit(1);
        } else if matches.len() == 1 {
            matches[0]
        } else if let Some(exact) = matches
            .iter()
            .find(|a| a.email.to_lowercase() == query_lower)
        {
            *exact
        } else {
            eprintln!(
                "[ERROR] Ambiguous query '{}' matched multiple accounts:",
                acc_query
            );
            for m in &matches {
                eprintln!("  - {} ({})", m.email, m.id);
            }
            std::process::exit(1);
        }
    };

    let resolved_inst_id = match target_inst_spec {
        Some(s) => match instance::resolve_instance_id(s) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("[ERROR] Failed to resolve target instance '{}': {}", s, e);
                std::process::exit(1);
            }
        },
        None => instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string()),
    };

    println!(
        "[*] Switching instance '{}' to account '{}' ({})...",
        resolved_inst_id, target_account.email, target_account.id
    );

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    if let Err(e) = rt.block_on(instance::switch_account_to_instance(
        &target_account.id,
        Some(&resolved_inst_id),
    )) {
        eprintln!("[ERROR] Account switch failed: {}", e);
        std::process::exit(1);
    }

    if is_json {
        let res = serde_json::json!({
            "status": "success",
            "instance_id": resolved_inst_id,
            "account_id": target_account.id,
            "account_email": target_account.email,
        });
        println!("{}", serde_json::to_string_pretty(&res).unwrap_or_default());
    } else {
        println!(
            "[SUCCESS] Instance '{}' successfully switched to '{}'.",
            resolved_inst_id, target_account.email
        );
        println!("  [OK] Credentials injected into instance state.vscdb.");
        println!("  [OK] Running prompts backed up and preserved.");
    }
    std::process::exit(0);
}

// -----------------------------------------------------------------------------
// Auto-Switch Command Handler
// -----------------------------------------------------------------------------

fn handle_auto_switch_command(args: &[String]) {
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

// -----------------------------------------------------------------------------
// Supabase Subcommand Handler
// -----------------------------------------------------------------------------

fn handle_supabase_subcommand(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        print_supabase_cli_help();
        std::process::exit(0);
    }

    let sub = args
        .first()
        .map(|s| {
            s.trim_start_matches('/')
                .trim_start_matches('-')
                .to_lowercase()
        })
        .unwrap_or_else(|| "status".to_string());
    let sub_args = if args.len() > 1 { &args[1..] } else { &[] };

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[ERROR] Tokio runtime error: {}", e);
            std::process::exit(1);
        }
    };

    match sub.as_str() {
        "status" | "info" | "ls" => {
            let mut cfg = match crate::modules::supabase_sync::load_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[ERROR] Failed to load Supabase config: {}", e);
                    std::process::exit(1);
                }
            };

            // Auto-discover endpoints from repo-secrets if available
            let seeded = crate::modules::supabase_sync::auto_seed_from_repo_secrets(&mut cfg);
            if seeded {
                let _ = crate::modules::supabase_sync::save_config(&cfg);
            }

            let node_id = crate::modules::supabase_sync::get_local_node_id();
            let local_ip = crate::modules::supabase_sync::get_local_ip();
            let uptime = crate::modules::supabase_sync::get_uptime_seconds();
            let repo_secrets_files: Vec<_> =
                crate::modules::supabase_sync::candidate_repo_secrets_paths()
                    .into_iter()
                    .filter(|p| p.exists())
                    .collect();

            println!(
                "================================================================================"
            );
            println!("  Antigravity-Manager Supabase Fleet & Synchronization Status");
            println!(
                "================================================================================"
            );
            println!("  Node ID:         {}", node_id);
            println!("  Node Alias:      {}", cfg.node_alias);
            println!("  Local IPv4:      {}", local_ip);
            println!("  Uptime:          {}s ({}m)", uptime, uptime / 60);
            println!(
                "  Sync Active:     {}",
                if cfg.is_sync_enabled {
                    "YES (Enabled)"
                } else {
                    "NO (Disabled)"
                }
            );
            println!("  Heartbeat:       every {}s", cfg.heartbeat_interval_secs);
            println!("  Endpoints Count: {}", cfg.endpoints.len());
            println!(
                "  Repo Secrets:    {} file(s) detected",
                repo_secrets_files.len()
            );
            for rf in &repo_secrets_files {
                println!("                   • {}", rf.display());
            }
            println!();

            println!("  --- Configured Endpoints ({}) ---", cfg.endpoints.len());
            if cfg.endpoints.is_empty() {
                println!("  (No endpoints configured. Use 'agm supabase set ...' or 'agm supabase load-json <file>')");
            } else {
                println!(
                    "  {:<20} {:<10} {:<8} {:<8} {:<12} {:<30}",
                    "ID", "ROLE", "ENABLED", "PRIORITY", "SOURCE", "URL"
                );
                println!("  {}", "-".repeat(95));
                for ep in &cfg.endpoints {
                    let is_repo_secret = ep
                        .notes
                        .as_deref()
                        .map_or(false, |n| n.to_lowercase().contains("repo-secrets"))
                        || ep
                            .tags
                            .iter()
                            .any(|t| t.to_lowercase().contains("repo-secrets"));
                    let source_str = if is_repo_secret {
                        "repo-secrets"
                    } else {
                        "manual"
                    };
                    println!(
                        "  {:<20} {:<10} {:<8} {:<8} {:<12} {:<30}",
                        ep.id,
                        ep.role,
                        if ep.is_enabled { "yes" } else { "no" },
                        ep.priority,
                        source_str,
                        if ep.url.len() > 28 {
                            format!("{}...", &ep.url[..26])
                        } else {
                            ep.url.clone()
                        }
                    );
                }
            }
            println!();

            println!("  --- Active Remote Workspace Leases ---");
            match rt.block_on(crate::modules::workspace_lease_manager::list_active_leases()) {
                Ok(leases) => {
                    if leases.is_empty() {
                        println!("  (No active remote leases held across fleet)");
                    } else {
                        let now = chrono::Utc::now().timestamp();
                        println!(
                            "  {:<30} {:<16} {:<16} {:<16} {:<10}",
                            "ACCOUNT EMAIL / ID",
                            "NODE ALIAS",
                            "IP ADDRESS",
                            "PROFILE",
                            "EXPIRES IN"
                        );
                        println!("  {}", "-".repeat(95));
                        for l in &leases {
                            let display_acc = if !l.account_email.is_empty() {
                                l.account_email.clone()
                            } else {
                                l.account_id.clone()
                            };
                            let exp = if l.expires_at > now {
                                format!("{}s", l.expires_at - now)
                            } else {
                                "expired".to_string()
                            };
                            let display_ip = if !l.ip_address.is_empty() {
                                l.ip_address.clone()
                            } else {
                                "-".to_string()
                            };
                            println!(
                                "  {:<30} {:<16} {:<16} {:<16} {:<10}",
                                if display_acc.len() > 28 {
                                    format!("{}...", &display_acc[..26])
                                } else {
                                    display_acc
                                },
                                l.node_alias,
                                display_ip,
                                l.profile_name,
                                exp
                            );
                        }
                        println!("\n  Total active cluster leases: {}", leases.len());
                    }
                }
                Err(e) => {
                    println!("  (Could not fetch remote leases: {})", e);
                }
            }
            println!(
                "================================================================================"
            );
            std::process::exit(0);
        }

        "sync" => {
            println!("[CLI] Triggering Supabase manual synchronization...");
            match rt.block_on(crate::modules::supabase_sync::trigger_manual_sync()) {
                Ok(_) => {
                    println!("[SUCCESS] Supabase synchronization completed successfully.");
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[ERROR] Supabase synchronization failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "list-leases" | "leases" | "in-use" => {
            println!("[CLI] Fetching active workspace account leases from Supabase Root DB...");
            match rt.block_on(crate::modules::workspace_lease_manager::list_active_leases()) {
                Ok(leases) => {
                    if leases.is_empty() {
                        println!("No active accounts currently leased across the cluster.");
                    } else {
                        let now = chrono::Utc::now().timestamp();
                        println!(
                            "\n{:<32} {:<24} {:<18} {:<16} {:<16} {:<10}",
                            "ACCOUNT EMAIL",
                            "ACCOUNT ID",
                            "NODE ALIAS",
                            "IP ADDRESS",
                            "INSTANCE",
                            "EXPIRES IN"
                        );
                        println!("{}", "-".repeat(120));
                        for l in &leases {
                            let exp = if l.expires_at > now {
                                format!("{}s", l.expires_at - now)
                            } else {
                                "expired".to_string()
                            };
                            let display_ip = if !l.ip_address.is_empty() {
                                l.ip_address.clone()
                            } else {
                                "-".to_string()
                            };
                            println!(
                                "{:<32} {:<24} {:<18} {:<16} {:<16} {:<10}",
                                if l.account_email.len() > 30 {
                                    format!("{}...", &l.account_email[..28])
                                } else {
                                    l.account_email.clone()
                                },
                                if l.account_id.len() > 22 {
                                    format!("{}...", &l.account_id[..20])
                                } else {
                                    l.account_id.clone()
                                },
                                l.node_alias,
                                display_ip,
                                l.profile_name,
                                exp
                            );
                        }
                        println!(
                            "\nTotal active accounts in use across machines: {}",
                            leases.len()
                        );
                    }
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to query workspace leases: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "test" => {
            let cfg = match crate::modules::supabase_sync::load_config() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[ERROR] Failed to load Supabase config: {}", e);
                    std::process::exit(1);
                }
            };
            let target_id = sub_args.first().map(|s| s.as_str());
            let endpoints: Vec<_> = cfg
                .endpoints
                .iter()
                .filter(|ep| {
                    if let Some(id) = target_id {
                        ep.id == id || ep.name.to_lowercase().contains(&id.to_lowercase())
                    } else {
                        ep.is_enabled
                    }
                })
                .collect();

            if endpoints.is_empty() {
                println!("No matching endpoints found to test.");
                std::process::exit(0);
            }

            println!("[CLI] Testing {} Supabase endpoint(s)...", endpoints.len());
            for ep in endpoints {
                print!("  Connecting to [{}] {} ({}) ... ", ep.id, ep.name, ep.role);
                let client = match crate::modules::supabase_client::SupabaseClient::new(ep) {
                    Ok(c) => c,
                    Err(e) => {
                        println!("FAILED (Client init error: {})", e);
                        continue;
                    }
                };
                match rt.block_on(client.test_connection()) {
                    Ok(res) => {
                        if res.is_success {
                            println!("PASS (HTTP {})", res.status_code.unwrap_or(200));
                        } else {
                            println!("FAIL (HTTP {})", res.status_code.unwrap_or(0));
                        }
                        println!("    -> {}", res.message);
                    }
                    Err(e) => {
                        println!("ERROR: {}", e);
                    }
                }
            }
            std::process::exit(0);
        }

        _ => {
            eprintln!(
                "[ERROR] Unknown subcommand: 'supabase {}'. Run 'antigravity-manager supabase --help' for guide.",
                sub
            );
            std::process::exit(1);
        }
    }
}

fn print_supabase_cli_help() {
    println!("================================================================================");
    println!("         Antigravity-Manager: Supabase Fleet Synchronization CLI                ");
    println!("================================================================================");
    println!("Usage:");
    println!("  antigravity-manager supabase <subcommand> [options]");
    println!("  agm supabase <subcommand> [options]");
    println!();
    println!("Subcommands:");
    println!(
        "  status, ls                       Display Supabase node status, endpoints count & leases"
    );
    println!("  sync                             Trigger immediate node & instance profile sync");
    println!("  list-leases, leases              Show active cross-machine account leases");
    println!("  test [endpoint_id]               Test connection to configured Supabase endpoints");
    println!("================================================================================");
}

// -----------------------------------------------------------------------------
// AGY Cleaner & Help Utilities
// -----------------------------------------------------------------------------

fn print_all_cli_help() {
    println!("================================================================================");
    println!("             Antigravity-Manager Native CLI Companion                           ");
    println!("================================================================================");
    println!("Usage:");
    println!("  antigravity-manager <command> [options]");
    println!("  .\\run.ps1 <command> [options]");
    println!("  agm <command> [options]");
    println!();
    println!("Primary Command Suites:");
    println!("  instance, instances, ls      Create, list, switch, and launch isolated profiles");
    println!("  switch, switch-account       Directly switch authenticated Google Gemini accounts");
    println!("  auto-switch, auto            Autonomous rolling quota background monitor daemon");
    println!("  fast-forward, ff             Rotate profile immediately to the healthiest account");
    println!("  agy                          Prune conversation steps and clear local cache");
    println!("  update                       Perform fleet-wide update & diagnostic verification");
    println!("  ui, open-ui                  Launch the graphical desktop interface");
    println!(
        "  supabase, sb                 Fleet synchronization, remote leases & database status"
    );
    println!(
        "  prompts, prompt, tree        List, inspect, dispatch, queue, backup & restore prompts"
    );
    println!(
        "  doctor, check, health        Diagnose system health, processes, DB, and environment"
    );
    println!();
    println!(
        "Run 'antigravity-manager <command> --help' for command-specific guides and examples."
    );
    println!();
    print_instance_cli_help();
    println!();
    print_switch_cli_help();
    println!();
    print_auto_switch_cli_help();
    println!();
    print_supabase_cli_help();
    println!();
    print_prompts_cli_help();
    println!();
    print_doctor_cli_help();
}

fn print_instance_cli_help() {
    println!("================================================================================");
    println!("           Antigravity-Manager: Multi-Instance & Profile CLI                    ");
    println!("================================================================================");
    println!("Usage:");
    println!("  antigravity-manager instance <subcommand> [options]");
    println!("  .\\run.ps1 instance <subcommand> [options]");
    println!("  agm instances <subcommand> [options]");
    println!();
    println!("Aliases: instance, instances, intrance, intrances, profile, profiles");
    println!();
    println!("Subcommands:");
    println!("  create, add <name> [options]     Create a new isolated sandbox profile");
    println!("  list, ls                         List all registered profiles and running PIDs");
    println!("  switch, use <inst> <account>     Switch credentials for a specific profile");
    println!("  launch, start <inst>             Launch Antigravity window for profile");
    println!("  stop, kill <inst>                Safely close profile process");
    println!("  delete, rm <inst>                Remove profile directory and registry entry");
    println!("  copy, clone <src> <new_name>     Duplicate profile configuration & settings");
    println!(
        "  ff, rotate [inst]                Fast-forward rotate profile to healthiest account"
    );
    println!();
    println!("Create Options:");
    println!("  --account, -a <email|id>         Bind specific account (default: next available unbound)");
    println!("  --from, -f <source_id>           Clone settings/extensions from existing profile");
    println!("  --launch, -l                     Immediately launch window after creation");
    println!(
        "  --data-only, --do                Create data directory only without cloning binary"
    );
    println!("  --json, -j                       Output structured JSON format");
    println!();
    println!("Examples:");
    println!("  # Create a profile bound to a specific account and launch immediately");
    println!(
        "  antigravity-manager instance create \"Work-Project\" -a \"work.dev@gmail.com\" --launch"
    );
    println!();
    println!("  # Create a profile cloning settings from an existing profile");
    println!("  antigravity-manager instance create \"Client-B\" --from \"Work-Project\"");
    println!();
    println!("  # List all registered profiles and active PIDs");
    println!("  antigravity-manager instance list");
    println!();
    println!("  # Switch instance #2 to another account");
    println!("  antigravity-manager instance switch #2 \"alex.dev@gmail.com\"");
    println!();
    println!("  # Launch or stop an instance window");
    println!("  antigravity-manager instance launch \"Work-Project\"");
    println!("  antigravity-manager instance stop \"Work-Project\"");
    println!();
    println!("  # Fast-forward rotate instance #2 to the next best account");
    println!("  antigravity-manager instance ff #2");
}

fn print_switch_cli_help() {
    println!("================================================================================");
    println!("             Antigravity-Manager: Account Switching CLI                         ");
    println!("================================================================================");
    println!("Usage:");
    println!("  antigravity-manager switch <account> [--instance <id>] [--json]");
    println!("  antigravity-manager switch <instance> <account>");
    println!("  antigravity-manager switch account <account>");
    println!("  .\\run.ps1 switch <account>");
    println!("  agm switch <account>");
    println!();
    println!("Aliases: switch, switch-account, swtich, swtich-account, account-switch");
    println!();
    println!("Description:");
    println!("  Switches authenticated Google Gemini account credentials for an Antigravity IDE");
    println!(
        "  profile (or active/default profile) without GUI interaction. Automatically injects"
    );
    println!("  tokens into state.vscdb, updates profile storage, and preserves running prompts.");
    println!();
    println!("Arguments & Options:");
    println!(
        "  <account>                        Account email, prefix, internal ID, or index (#1, #2)"
    );
    println!("  <instance>                       Target instance ID, name, sequence (#1, #2), or 'default'");
    println!(
        "  --instance, -i <id>              Specify target instance (defaults to active profile)"
    );
    println!("  --json, -j                       Output result in structured JSON format");
    println!();
    println!("Examples:");
    println!("  # Switch active profile to an account by email");
    println!("  antigravity-manager switch user.dev@gmail.com");
    println!();
    println!("  # Switch using the 'switch account' phrase");
    println!("  antigravity-manager switch account user.dev@gmail.com");
    println!();
    println!("  # Switch by email prefix");
    println!("  antigravity-manager switch user.dev");
    println!();
    println!("  # Switch a specific instance using --instance flag");
    println!("  antigravity-manager switch user.dev@gmail.com --instance #2");
    println!("  antigravity-manager switch user.dev@gmail.com -i \"Work-Project\"");
    println!();
    println!("  # Switch instance #2 directly");
    println!("  antigravity-manager switch #2 user.dev@gmail.com");
    println!();
    println!("  # Switch active profile to account #3 from accounts list");
    println!("  antigravity-manager switch #3");
}

fn print_auto_switch_cli_help() {
    println!("================================================================================");
    println!("             Antigravity-Manager: Auto Profile Switcher CLI                     ");
    println!("================================================================================");
    println!("Usage:");
    println!("  antigravity-manager auto-switch <action> [options]");
    println!("  .\\run.ps1 auto-switch <action>");
    println!("  agm auto-switch <action>");
    println!();
    println!("Aliases: auto-switch, auto-swtich, autoswitch, auto, switcher");
    println!();
    println!("Description:");
    println!("  Monitors rolling 4-hour quota windows across running instances and proactively");
    println!("  rotates accounts before depletion, auto-resuming active workspace prompts.");
    println!();
    println!("Actions:");
    println!(
        "  status                           Show auto-switcher daemon state & monitored profiles"
    );
    println!("  enable, on, start                Turn ON background auto-switch daemon");
    println!("  disable, off, stop               Turn OFF background auto-switch daemon");
    println!("  toggle                           Toggle background auto-switch daemon state");
    println!(
        "  run, trigger, eval, rotate       Immediately evaluate rolling quota and rotate if low"
    );
    println!("  threshold [N]                    Get or set low quota threshold percentage (default: 15%)");
    println!(
        "  interval [N]                     Get or set polling interval in seconds (default: 300s)"
    );
    println!(
        "  model [name]                     Get or set evaluation model (e.g., gemini-2.5-pro)"
    );
    println!(
        "  test [N]                         Simulate quota check with test threshold percentage"
    );
    println!();
    println!("Examples:");
    println!("  # Check daemon status and active profile quota");
    println!("  antigravity-manager auto-switch status");
    println!();
    println!("  # Enable or disable the background switcher daemon");
    println!("  antigravity-manager auto-switch enable");
    println!("  antigravity-manager auto-switch disable");
    println!("  antigravity-manager auto-switch toggle");
    println!();
    println!("  # Trigger immediate quota evaluation & auto-rotation");
    println!("  antigravity-manager auto-switch run");
    println!();
    println!("  # Configure threshold and interval");
    println!("  antigravity-manager auto-switch threshold 20");
    println!("  antigravity-manager auto-switch interval 60");
    println!("  antigravity-manager auto-switch model gemini-2.5-pro");
    println!();
    println!("  # Test auto-rotation simulation with 90% threshold");
    println!("  antigravity-manager auto-switch test 90");
}

fn print_profile_help() {
    print_instance_cli_help();
}

fn print_agy_cli_help() {
    println!("================================================================================");
    println!("             Antigravity-Manager: AGY Cache & Retention CLI                      ");
    println!("================================================================================");
    println!("Usage:");
    println!("  antigravity-manager agy <command> [options]");
    println!("  .\\run.ps1 agy <command> [options]");
    println!();
    println!("Commands:");
    println!("  agy cache-clear [--keep <N>]      Prune older conversations (default: keep 10) & clear caches");
    println!(
        "  agy cache-clear-keep-one (ccko)   Keep only the 1 latest conversation, prune the rest"
    );
    println!(
        "  agy cache-clear-keep-five (cckf)  Keep only the 5 latest conversations, prune the rest"
    );
    println!("  agy undo [tx_id]                  Revert the last (or specific) conversation pruning transaction");
    println!();
    println!("Options & Flags:");
    println!("  --precheck, --pre, --preflight    Simulate and preview what would be pruned without modifying files");
    println!("  -y, --yes                         Bypass interactive confirmation prompt for automated scripting");
    println!("  --keep <N>, -k <N>                Specify number of latest conversations to retain intact");
    println!();
    println!("Temporary Staging & Safety:");
    println!(
        "  Pruned conversations are safely staged in the OS temporary directory before removal."
    );
    println!("  Run 'agy undo' to immediately restore them.");
    println!("  [NOTE] Temporary directories may be pruned by the OS over time; revert promptly if needed.");
}

fn is_flag_present(args: &[String], flags: &[&str]) -> bool {
    args.iter().any(|a| flags.iter().any(|f| a == f))
}

fn parse_keep_count(args: &[String], default_val: usize) -> usize {
    for (idx, arg) in args.iter().enumerate() {
        let is_keep_flag = arg == "--keep" || arg == "-k";
        if is_keep_flag {
            if let Some(val_str) = args.get(idx + 1) {
                if let Ok(parsed) = val_str.parse::<usize>() {
                    return parsed;
                }
            }
        }
        if let Ok(num) = arg.parse::<usize>() {
            return num;
        }
    }
    default_val
}

fn handle_agy_subcommand(sub_args: &[String]) {
    let sub = sub_args[0].as_str();
    match sub {
        "cache-clear" | "clear" | "clean" | "cache-clean" => {
            handle_clear_action(&sub_args[1..], 10);
        }
        "cache-clear-keep-one" | "ccko" => {
            handle_clear_action(&sub_args[1..], 1);
        }
        "cache-clear-keep-five" | "cckf" => {
            handle_clear_action(&sub_args[1..], 5);
        }
        "undo" => {
            let tx_id = sub_args.get(1).map(|s| s.as_str());
            execute_agy_undo(tx_id);
        }
        "help" | "--help" | "-h" => {
            print_agy_cli_help();
            std::process::exit(0);
        }
        _ => {
            eprintln!("Unknown AGY command: {}", sub);
            eprintln!("Run 'antigravity-manager agy help' for usage instructions.");
            std::process::exit(1);
        }
    }
}

fn handle_clear_action(extra_args: &[String], default_keep: usize) {
    let keep = parse_keep_count(extra_args, default_keep);
    let is_preflight = is_flag_present(extra_args, &["--precheck", "--pre", "--preflight", "-p"]);
    let is_yes = is_flag_present(extra_args, &["-y", "--yes", "-f"]);
    execute_agy_clear(keep, is_preflight, is_yes);
}

fn execute_agy_clear(keep_count: usize, is_preflight: bool, is_yes: bool) {
    if is_preflight {
        let report = crate::modules::agy_cleaner::preflight_check(keep_count);
        print_preflight_report(&report);
        std::process::exit(0);
    }

    if !is_yes {
        println!();
        println!("  [!] Antigravity Conversation & Cache Pruner");
        println!(
            "  Retention Plan: Keeping top {} recent conversations intact.",
            keep_count
        );
        println!(
            "  Older conversations will be safely staged in OS temp storage for undo recovery."
        );
        print!("  Proceed with pruning? [y/N]: ");
        use std::io::{self, Write};
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Aborted.");
            std::process::exit(1);
        }
        let trimmed = input.trim().to_lowercase();
        let is_confirmed = trimmed == "y" || trimmed == "yes";
        if !is_confirmed {
            println!("  [--] Operation cancelled by user. (Use -y to run non-interactively)");
            std::process::exit(0);
        }
    }

    println!(
        "  [*] Pruning older conversations and clearing cache (keep {})...",
        keep_count
    );
    match crate::modules::agy_cleaner::prune_and_clean(keep_count) {
        Ok(result) => {
            println!("  [OK] Pruning completed successfully!");
            println!("    Transaction ID      : {}", result.transaction_id);
            println!("    Conversations Kept  : {}", result.preserved_count);
            println!(
                "    Conversations Pruned: {} ({:.2} MB)",
                result.pruned_count,
                result.pruned_bytes as f64 / 1024.0 / 1024.0
            );
            println!(
                "    Cache Cleared       : {:.2} MB",
                result.cache_cleared_bytes as f64 / 1024.0 / 1024.0
            );
            println!(
                "    Total Space Freed   : {:.2} MB",
                result.total_freed_bytes as f64 / 1024.0 / 1024.0
            );
            println!("    Staging Backup Path : {}", result.staging_dir);
            println!();
            println!("  [TIP] To revert this operation, run: antigravity-manager agy undo");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("  [ERROR] Cleanup failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn execute_agy_undo(tx_id: Option<&str>) {
    println!("  [*] Reverting conversation pruning transaction from temporary storage...");
    match crate::modules::agy_cleaner::undo_prune(tx_id) {
        Ok(result) => {
            println!("  [OK] Reversion successful!");
            println!("    Transaction ID        : {}", result.transaction_id);
            println!(
                "    Restored Conversations: {}",
                result.restored_conversations
            );
            println!(
                "    Restored Size         : {:.2} MB",
                result.restored_bytes as f64 / 1024.0 / 1024.0
            );
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("  [ERROR] Undo failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_preflight_report(report: &crate::modules::agy_cleaner::PreflightReport) {
    println!("================================================================================");
    println!(" [==] Antigravity Optimizer & Conversation Pre-Flight Report");
    println!("================================================================================");
    println!(
        " Total Conversations Scanned : {} ({:.2} MB)",
        report.total_conversations,
        report.total_conversation_bytes as f64 / 1024.0 / 1024.0
    );
    println!(
        " Retention Policy            : Keeping latest {} conversations intact",
        report.keep_count
    );
    println!(" Preserved Recent Convs      : {}", report.preserved_count);
    println!(
        " Older Convs to Prune        : {} (will be staged to temporary storage)",
        report.pruned_count
    );
    println!(
        " Projected Prune Reclamation : {:.2} MB",
        report.projected_reclaimed_bytes as f64 / 1024.0 / 1024.0
    );
    println!(
        " Application Cache Targets   : {} folders ({:.2} MB)",
        report.cache_paths_count,
        report.cache_bytes as f64 / 1024.0 / 1024.0
    );
    println!(
        " Total Projected Reclamation : ~{:.2} MB",
        (report.projected_reclaimed_bytes + report.cache_bytes) as f64 / 1024.0 / 1024.0
    );
    println!(" Temporary Staging Directory : {}", report.staging_dir);
    println!("--------------------------------------------------------------------------------");
    println!(" [TIP] Undo / Rollback Capability:");
    println!("  Pruned conversation steps are staged in the temporary directory.");
    println!("  To revert any operation, run: agy undo");
    println!("  [NOTE] Temporary directories may be pruned by the OS over time; revert promptly if needed.");
    println!("================================================================================");
    println!(" [NOTE] Pre-flight mode active. No files modified. No processes terminated.");
    println!("================================================================================");
}

// -----------------------------------------------------------------------------
// Prompts & Doctor Subcommands
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub status: String,
    pub node_name: String,
    pub local_ip: String,
    pub database_connectivity: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_path: Option<String>,
    pub accounts_count: usize,
    pub instances_count: usize,
    pub proxy_gateway_status: String,
    pub antigravity_installation: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub antigravity_path: Option<String>,
    pub running_pids: Vec<u32>,
    pub checks: Vec<DoctorCheckItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorCheckItem {
    pub name: String,
    pub is_passed: bool,
    pub details: String,
}

fn extract_flag_value(args: &[String], flags: &[&str]) -> Option<String> {
    for (idx, arg) in args.iter().enumerate() {
        if flags.iter().any(|f| arg == f) {
            if let Some(val) = args.get(idx + 1) {
                if !val.starts_with('-') {
                    return Some(val.clone());
                }
            }
        }
    }
    None
}

fn extract_positional_args(args: &[String]) -> Vec<String> {
    let mut positional = Vec::new();
    let mut is_skipping_next = false;
    for arg in args {
        if is_skipping_next {
            is_skipping_next = false;
            continue;
        }
        if arg == "-i"
            || arg == "--instance"
            || arg == "--profile"
            || arg == "-r"
            || arg == "--repo"
            || arg == "--workspace"
            || arg == "-l"
            || arg == "--limit"
            || arg == "-k"
            || arg == "--keep"
        {
            is_skipping_next = true;
            continue;
        }
        if arg.starts_with('-') {
            continue;
        }
        positional.push(arg.clone());
    }
    positional
}

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

fn execute_prompts_list(sub_args: &[String]) {
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

fn execute_prompts_tree(sub_args: &[String]) {
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

fn execute_prompts_send(sub_args: &[String]) {
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

fn execute_prompts_enqueue(sub_args: &[String]) {
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

    match repo_db::enqueue_prompt_for_instance(&target_inst, &target_repo, &prompt_text, None) {
        Ok(active_prompt) => {
            if is_json {
                let payload = serde_json::json!({
                    "prompt_id": active_prompt.id,
                    "instance_id": active_prompt.instance_id,
                    "repo_path": active_prompt.repo_path,
                    "status": active_prompt.status,
                    "is_queued": true,
                    "created_at": active_prompt.created_at,
                });
                CliEnvelope::ok("prompts enqueue", Some(target_inst), payload).print_and_exit();
            }
            println!("[CLI] Prompt successfully enqueued:");
            println!("  ID:          {}", active_prompt.id);
            println!("  Instance:    {}", active_prompt.instance_id);
            println!("  Repo:        {}", active_prompt.repo_path);
            println!("  Status:      {}", active_prompt.status);
            std::process::exit(0);
        }
        Err(e) => {
            if is_json {
                CliEnvelope::<()>::err("prompts enqueue", Some(target_inst), "ENQUEUE_FAILED", &e)
                    .print_and_exit();
            }
            eprintln!("[ERROR] Failed to enqueue prompt: {}", e);
            std::process::exit(1);
        }
    }
}

fn execute_prompts_backup(sub_args: &[String]) {
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

fn execute_prompts_restore(sub_args: &[String]) {
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

pub fn handle_doctor_subcommand(args: &[String]) {
    let is_json = is_flag_present(args, &["--json", "-j"]);

    // 1. Node name & Local IP
    let node_name = crate::modules::email_watcher::detect_machine_name();
    let local_ip = crate::modules::supabase_sync::get_local_ip();

    // 2. Database Connectivity
    let db_path_opt = repo_db::get_repo_db_path()
        .ok()
        .map(|p| p.to_string_lossy().to_string());
    let is_db_connected = repo_db::connect_db().is_ok();

    // 3. Accounts count
    let accounts_count = account::list_accounts().map(|accs| accs.len()).unwrap_or(0);

    // 4. Instances count
    let instances = instance::list_instances().unwrap_or_default();
    let instances_count = instances.len();

    // 5. Proxy gateway status
    let is_proxy_running = is_daemon_running();
    let proxy_gateway_status = if is_proxy_running {
        "Running (port 8045)".to_string()
    } else {
        "Stopped / Unreachable".to_string()
    };

    // 6. Antigravity installation
    let ag_exe_opt = crate::modules::process::get_antigravity_executable_path(None)
        .or_else(|| crate::modules::process::get_antigravity_executable_path(Some("ide")))
        .map(|p| p.to_string_lossy().to_string());
    let has_antigravity = ag_exe_opt.is_some();

    // 7. Running PIDs
    let mut running_pids: Vec<u32> = Vec::new();
    let mut system = sysinfo::System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All);
    for (pid, proc) in system.processes() {
        let name = proc.name().to_string_lossy().to_lowercase();
        let exe = proc
            .exe()
            .map(|p| p.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if name.contains("antigravity") || exe.contains("antigravity") {
            running_pids.push(pid.as_u32());
        }
    }
    for inst in &instances {
        if inst.is_running {
            if let Some(pid) = inst.pid {
                if !running_pids.contains(&pid) {
                    running_pids.push(pid);
                }
            }
        }
    }
    running_pids.sort_unstable();
    running_pids.dedup();

    let mut checks = Vec::new();
    checks.push(DoctorCheckItem {
        name: "registry_integrity".to_string(),
        is_passed: instance::load_registry().is_ok(),
        details: format!(
            "instances.json valid with {} registered profile(s)",
            instances_count
        ),
    });
    checks.push(DoctorCheckItem {
        name: "database_connectivity".to_string(),
        is_passed: is_db_connected,
        details: if is_db_connected {
            format!(
                "SQLite database connected: {}",
                db_path_opt.as_deref().unwrap_or("repo_prompts.db")
            )
        } else {
            "Failed to connect to repo_prompts.db SQLite database".to_string()
        },
    });
    checks.push(DoctorCheckItem {
        name: "accounts_store".to_string(),
        is_passed: accounts_count > 0,
        details: format!(
            "{} account(s) registered in credentials store",
            accounts_count
        ),
    });
    checks.push(DoctorCheckItem {
        name: "proxy_gateway".to_string(),
        is_passed: is_proxy_running,
        details: if is_proxy_running {
            "Proxy gateway active on 127.0.0.1:8045".to_string()
        } else {
            "Proxy gateway not running (run 'antigravity-manager' or background daemon)".to_string()
        },
    });
    checks.push(DoctorCheckItem {
        name: "antigravity_installation".to_string(),
        is_passed: has_antigravity,
        details: if has_antigravity {
            format!(
                "Executable located at {}",
                ag_exe_opt.as_deref().unwrap_or("-")
            )
        } else {
            "Antigravity executable not found in default paths".to_string()
        },
    });
    checks.push(DoctorCheckItem {
        name: "running_processes".to_string(),
        is_passed: true,
        details: format!(
            "{} live Antigravity process(es) detected",
            running_pids.len()
        ),
    });

    let overall_status = if is_db_connected && has_antigravity {
        "healthy".to_string()
    } else {
        "degraded".to_string()
    };

    let report = DoctorReport {
        status: overall_status,
        node_name,
        local_ip,
        database_connectivity: is_db_connected,
        database_path: db_path_opt,
        accounts_count,
        instances_count,
        proxy_gateway_status,
        antigravity_installation: has_antigravity,
        antigravity_path: ag_exe_opt,
        running_pids,
        checks,
    };

    if is_json {
        CliEnvelope::ok("doctor", None, report).print_and_exit();
    }

    println!("================================================================================");
    println!("             Antigravity-Manager: System Health & Doctor Diagnostics            ");
    println!("================================================================================");
    println!(
        "  Status:                  {}",
        if report.status == "healthy" {
            "HEALTHY"
        } else {
            "DEGRADED"
        }
    );
    println!("  Node Name:               {}", report.node_name);
    println!("  Local IPv4:              {}", report.local_ip);
    println!(
        "  Database Connectivity:   {}",
        if report.database_connectivity {
            "CONNECTED (SQLite)"
        } else {
            "FAILED"
        }
    );
    if let Some(ref p) = report.database_path {
        println!("  Database Path:           {}", p);
    }
    println!("  Accounts Count:          {}", report.accounts_count);
    println!("  Instances Count:         {}", report.instances_count);
    println!("  Proxy Gateway Status:    {}", report.proxy_gateway_status);
    println!(
        "  Antigravity Installed:   {}",
        if report.antigravity_installation {
            "YES"
        } else {
            "NO"
        }
    );
    if let Some(ref p) = report.antigravity_path {
        println!("  Antigravity Path:        {}", p);
    }
    let pids_str = if report.running_pids.is_empty() {
        "None".to_string()
    } else {
        report
            .running_pids
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    };
    println!("  Running PIDs:            {}", pids_str);
    println!();
    println!("--------------------------------------------------------------------------------");
    println!("  Diagnostic Checks:");
    for check in &report.checks {
        let mark = if check.is_passed { "[PASS]" } else { "[WARN]" };
        println!("  {:<8} {:<24} - {}", mark, check.name, check.details);
    }
    println!("================================================================================");
    std::process::exit(if report.database_connectivity { 0 } else { 1 });
}

fn print_prompts_cli_help() {
    println!("================================================================================");
    println!("             Antigravity-Manager: Prompt Dispatch & Management CLI              ");
    println!("================================================================================");
    println!("Usage:");
    println!("  antigravity-manager prompts <subcommand> [options]");
    println!("  .\\run.ps1 prompts <subcommand> [options]");
    println!("  agm prompts <subcommand> [options]");
    println!("  agm tree [options]");
    println!();
    println!("Aliases: prompts, prompt, tree");
    println!();
    println!("Subcommands:");
    println!("  ls, list [--all] [-i <inst>] [--json]      List active & queued prompts");
    println!("  tree [--running] [-i <inst>] [--json]      Output hierarchical conversation tree");
    println!("  send, run <prompt> [-i <inst>] [-r <repo>] Dispatch prompt immediately via Smart Process Cache");
    println!("  enqueue, queue <prompt> [-i <inst>] [-r <repo>] Enqueue prompt into FIFO execution queue");
    println!(
        "  backup [-i <inst>] [--json]                Backup running prompts to SQLite database"
    );
    println!("  restore [-i <inst>] [-l <limit>] [--json]  Restore previous running commands");
    println!();
    println!("Options:");
    println!("  --instance, -i <id>       Target instance profile (default: active/default)");
    println!("  --repo, -r <path>         Target workspace directory (default: current directory)");
    println!("  --all, -a                 List prompts across all registered instances");
    println!("  --running                 Filter tree to actively running prompts only");
    println!("  --limit, -l <N>           Limit number of prompts to restore (default: 20)");
    println!("  --json, -j                Output machine-readable JSON envelope");
    println!("================================================================================");
}

fn print_doctor_cli_help() {
    println!("================================================================================");
    println!("             Antigravity-Manager: System Health & Doctor Diagnostics            ");
    println!("================================================================================");
    println!("Usage:");
    println!("  antigravity-manager doctor [options]");
    println!("  .\\run.ps1 doctor [options]");
    println!("  agm doctor [options]");
    println!();
    println!("Aliases: doctor, check, health");
    println!();
    println!("Options:");
    println!("  --json, -j                Output machine-readable JSON envelope");
    println!("================================================================================");
}
