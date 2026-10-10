//! help_tables — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub(crate) fn print_prompts_reference_table() {
    println!("Usable Prompts Reference List (01-prompts Library):");
    println!(
        "  {:<26} {:<44} {:<30}",
        "PROMPT SLUG", "PURPOSE / ACTION", "EMAIL INVOCATION"
    );
    println!("  {}", "-".repeat(104));
    println!(
        "  {:<26} {:<44} {:<30}",
        "execute-pending-tasks",
        "Run queued tasks in .ai-memory/plans/pending/",
        "Body: execute-pending-tasks"
    );
    println!(
        "  {:<26} {:<44} {:<30}",
        "execute-parent-task",
        "Decompose parent plan and run N-step loop",
        "Body: execute-parent-task <goal>"
    );
    println!(
        "  {:<26} {:<44} {:<30}",
        "ci-cd-fix", "Grounded 4-part RCA & pipeline self-healing", "Body: ci-cd-fix"
    );
    println!(
        "  {:<26} {:<44} {:<30}",
        "minor-bump", "Bump version in manifests & tag release", "Body: minor-bump"
    );
    println!(
        "  {:<26} {:<44} {:<30}",
        "coding-guidelines",
        "Enforce PascalCase, AppError, booleans & enums",
        "Body: coding-guidelines"
    );
    println!(
        "  {:<26} {:<44} {:<30}",
        "smart-test-runner",
        "Incremental targeted test runs without regressing",
        "Body: smart-test-runner"
    );
    println!();
}

pub(crate) fn print_live_projects_table() {
    println!("Discovered Workspaces & Project Identifiers:");
    println!(
        "  {:<32} {:<28} {:<10} {:<30}",
        "PROJECT NAME", "REPLY TARGET", "STATUS", "PATH"
    );
    println!("  {}", "-".repeat(105));
    let projects = repo_db::get_live_project_execution_info();
    if projects.is_empty() {
        println!("  (No active workspace projects detected)");
    } else {
        for p in projects {
            let reply_target = format!("proj-{}", p.project_id);
            let path_snippet: String = if p.repo_path.len() > 30 {
                format!("...{}", &p.repo_path[p.repo_path.len() - 27..])
            } else {
                p.repo_path.clone()
            };
            println!(
                "  {:<32} {:<28} {:<10} {:<30}",
                p.repo_name, reply_target, p.status, path_snippet
            );
        }
    }
    println!();
}

pub(crate) fn print_recent_prompts_table() {
    println!("Recent Prompts Inventory:");
    println!(
        "  {:<26} {:<24} {:<12} {:<38}",
        "PROMPT ID", "PROJECT", "STATUS", "SNIPPET"
    );
    println!("  {}", "-".repeat(105));
    let prompts = repo_db::list_all_prompts().unwrap_or_default();
    if prompts.is_empty() {
        println!("  (No prompt records in repo_prompts.db)");
    } else {
        for p in prompts.iter().take(6) {
            let snippet: String = p
                .prompt_content
                .replace('\n', " ")
                .chars()
                .take(36)
                .collect();
            println!(
                "  {:<26} {:<24} {:<12} {:<38}",
                p.id, p.project_id, p.status, snippet
            );
        }
    }
    println!();
}

pub(crate) fn print_doctor_probe(name: &str, detail: &str, is_pass: bool) {
    if is_pass {
        println!("    [\x1b[32mPASS\x1b[0m] {:<30} {}", name, detail);
    } else {
        println!("    [\x1b[31mFAIL\x1b[0m] {:<30} {}", name, detail);
    }
}

pub(crate) fn print_supabase_help() {
    let cfg_path = supabase_sync::get_config_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "%APPDATA%\\antigravity-manager\\supabase_config.json".to_string());
    let node_id = supabase_sync::get_local_node_id();
    let local_ip = supabase_sync::get_local_ip();

    println!("================================================================================");
    println!("  AGM Supabase Multi-Machine Fleet Synchronization & Lease Architecture");
    println!("================================================================================");
    println!();
    println!("📁 CONFIGURATION FILE LOCATION:");
    println!("   Target Path:      {}", cfg_path);
    println!("   Local Node ID:    {}", node_id);
    println!("   Local IPv4:       {}", local_ip);
    println!();
    println!("📄 CONFIGURATION JSON STRUCTURE & EXAMPLE:");
    println!(
        r#"   {{
     "endpoints": [
       {{
         "id": "ep-root-lovable-01",
         "name": "Root Supabase (Lovable)",
         "url": "https://pezjuuddecbyfmqxytrv.supabase.co/rest/v1/",
         "api_key": "sb_publishable_cJdJyIEeXc8bpym9TIOU7w_vVU0_Y69",
         "role": "root",
         "is_enabled": true,
         "prune_threshold_mb": 400,
         "priority": 1,
         "notes": "Root account by Lovable",
         "tags": ["lovable", "root"]
       }},
       {{
         "id": "ep-secondary-01",
         "name": "Secondary Supabase",
         "url": "https://ikwmurmjynhxdpmhzekt.supabase.co/rest/v1/",
         "api_key": "sb_publishable_barsshQom3VcUw1l5soE_A_R1FwQz8o",
         "role": "secondary",
         "is_enabled": true,
         "prune_threshold_mb": 200,
         "priority": 2,
         "notes": "Secondary fallback and command queue",
         "tags": ["secondary"]
       }}
     ],
     "node_alias": "Node-823632",
     "is_sync_enabled": true,
     "auto_prune_root_mb": 400,
     "auto_prune_secondary_mb": 200,
     "heartbeat_interval_secs": 30
   }}"#
    );
    println!();
    println!("🗄️ DATABASE ROLES & PARENT-CHILD RELATIONSHIPS:");
    println!("   • Root Role ('root'):");
    println!("     - Table 'public.nodes' (Parent Machine): id, alias, ip_address, uptime_seconds, project_count, status");
    println!("     - Table 'public.instance_profiles' (Child Instances): id, node_id (FK->nodes.id), profile_name, active_account_email, is_active, status");
    println!("     - Table 'public.workspace_leases' (Cross-Machine In-Use Leases): account_id, account_email, node_id, node_alias, ip_address, profile_name, expires_at");
    println!("     - Function 'acquire_workspace_lease()': Atomic lock acquiring preventing multi-machine account collisions");
    println!("   • Secondary Role ('secondary'):");
    println!("     - Table 'public.command_queue': Remote inbound commands");
    println!("     - Table 'public.command_telemetry': Command stdout/stderr/exit_code logs");
    println!("     - Table 'public.endpoint_health': Storage tracking & FIFO auto-pruning");
    println!();
    println!("🛠️ CLI COMMANDS:");
    println!("   agm supabase status                    Display current configuration, endpoints & active leases");
    println!("   agm supabase list-leases               Show which accounts are currently in-use across machines");
    println!("   agm supabase test [endpoint_id]        Test connectivity & schema tables for configured endpoints");
    println!("   agm supabase set <id> <name> <url> <key> <role> [notes] [tags]");
    println!("                                          Configure/update endpoint directly from command line");
    println!("   agm supabase load-json <file>          Ingest and merge endpoints from JSON file");
    println!("   agm supabase export [--file <path>]    Export Supabase configuration wrapped in JSON envelope");
    println!("   agm supabase sync                      Trigger immediate local node & instance profile sync");
    println!("   agm supabase schema [root|secondary]   Output SQL schema script for Supabase SQL Editor");
    println!("   agm supabase enable / disable          Toggle Supabase synchronization");
    println!("   agm supabase set-alias <alias>         Update local machine alias");
    println!("   agm supabase set-config [--cooldown <m>] [--interval <s>] [--alias <alias>]");
    println!(
        "                                          Update cooldown lockout, interval and alias"
    );
    println!("   agm supabase load-secrets              Auto-discover and load endpoints from repo-secrets folder");
    println!(
        "   agm supabase set-prune <root> <sec>    Configure auto-prune storage thresholds (MB)"
    );
    println!("   agm supabase set-heartbeat <secs>      Configure heartbeat interval seconds");
    println!("================================================================================");
}
