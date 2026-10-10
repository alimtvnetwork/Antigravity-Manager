use crate::modules::{account, auto_switcher, config, instance, repo_db};

use super::*;

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
