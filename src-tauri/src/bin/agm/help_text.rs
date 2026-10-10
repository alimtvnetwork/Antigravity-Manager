//! help_text — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use chrono::Utc;
use std::env;

pub(crate) fn print_banner() {
    let git_hash = antigravity_tools_lib::modules::git_info::get_git_full_hash();
    let git_branch = antigravity_tools_lib::modules::git_info::get_git_branch();
    let last_release = antigravity_tools_lib::modules::git_info::get_last_release();
    let repo_url = antigravity_tools_lib::modules::git_info::get_repo_url();
    let built_time = antigravity_tools_lib::modules::git_info::get_built_timestamp();
    let db_path = antigravity_tools_lib::modules::repo_db::get_repo_db_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());
    let exe_path = env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());
    let (node_alias, local_ip) =
        antigravity_tools_lib::modules::email_sender::get_local_node_identity();
    let utc_now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    println!("  ────────────────────────────────────────────────────────────");
    println!("  agm binary");
    println!("  ● Name:           agm (Antigravity-Manager)");
    println!("  ● Git URL:        {}", repo_url);
    println!("  ● Version:        v{}", crate::common::VERSION);
    println!("  ● Commit SHA:     {}", git_hash);
    println!("  ● Branch:         {}", git_branch);
    println!("  ● Last Release:   {}", last_release);
    println!("  ● Database:       {}", db_path);
    println!("  ● Installed path: {}", exe_path);
    println!("  ● Built:          {}", built_time);
    println!();
    println!("  ────────────────────────────────────────────────────────────");
    println!("  current workspace / node");
    println!("  ● Node Alias:     {}", node_alias);
    println!("  ● Local IPv4:     {}", local_ip);
    println!("  ● Dispatched At:  {}", utc_now);
    println!("  ────────────────────────────────────────────────────────────");
    println!();
}

pub(crate) fn print_help_json() {
    let help_obj = serde_json::json!({
        "name": "agm",
        "version": crate::common::VERSION,
        "description": "Antigravity Manager CLI & Reverse Proxy Gateway",
        "commands": [
            {
                "group": "Account Rotation & Quota Governance",
                "items": [
                    { "name": "status", "aliases": ["credits"], "flags": ["--json"], "description": "Show node status, immediate & weekly credits" },
                    { "name": "ff", "aliases": ["smart-switch", "fast-forward"], "flags": [], "description": "Trigger fast-forward rotation to freshest 100% quota account" },
                    { "name": "switch-if-low-credit", "aliases": ["swlc", "sfc"], "flags": ["-t <pct>", "--json", "-f [file]", "--force"], "description": "Check live quota & rotate if quota <= threshold" },
                    { "name": "is-low-credit-for-switch", "aliases": ["ilc"], "flags": ["-t <pct>", "--json", "-f [file]"], "description": "Check if active quota <= threshold" },
                    { "name": "accounts", "aliases": ["acc"], "flags": ["--active", "--json"], "description": "List registered accounts, tiers, and quotas" },
                    { "name": "accounts refresh-tier", "aliases": ["refresh-tier"], "flags": ["--all", "--json"], "description": "Fetch and update subscription tiers (PRO / ULTRA / FREE) for accounts" },
                    { "name": "history", "aliases": ["audit"], "flags": ["--page <n>", "--json"], "description": "List the task history audit from the split SQLite files" },
                    { "name": "switch", "aliases": [], "flags": ["<email|prefix|id>"], "description": "Switch active profile directly without GUI" }
                ]
            },
            {
                "group": "Update, Repo Sync & Fleet Orchestration",
                "items": [
                    { "name": "update", "aliases": ["update-all", "ua"], "flags": ["all", "--json", "--check", "--force"], "description": "Check and update AGM binary, pull latest git repo, and sync fleet" },
                    { "name": "delegate-update", "aliases": ["ui-update-runner"], "flags": ["--wait-pid <PID>", "--relaunch", "--target-exe <EXE>"], "description": "Delegated out-of-process updater for AGM UI & CLI (zero file-locks & auto-relaunch)" },
                    { "name": "sync", "aliases": [], "flags": [], "description": "Synchronize local accounts, instances, and DB vaults" },
                    { "name": "pull", "aliases": [], "flags": [], "description": "Execute git pull origin main in repository root" },
                    { "name": "ssh", "aliases": [], "flags": ["<target>", "--update"], "description": "Connect to remote VM via SSH or run remote command" }
                ]
            },
            {
                "group": "Parallel Prompt Backup & Workspace Restoration",
                "items": [
                    { "name": "backup", "aliases": ["backpack", "brp", "backup-running-prompts"], "flags": ["ls", "clean", "-f <file>", "--json"], "description": "Parallel snapshot of active prompts across all workspaces to split SQLite DB" },
                    { "name": "restore", "aliases": ["rrp", "restore-running-prompts", "resend-running"], "flags": ["--keep", "--json", "-f <file>"], "description": "Restore and re-enqueue in-flight prompts into active workspaces" },
                    { "name": "which-prompts-running", "aliases": ["wpr"], "flags": ["--json"], "description": "List running projects, conv IDs, and prompt queues" },
                    { "name": "prompts ls", "aliases": ["running-prompts ls"], "flags": ["[N]", "--json", "--words <W>"], "description": "Show N running prompts in ASC stack order (with friendly project names)" },
                    { "name": "prompt", "aliases": [], "flags": ["\"<text>\"", "--prefix <cat>", "--suffix <cat>"], "description": "Dispatch prompt with git pull & 01-prompts templates" },
                    { "name": "query", "aliases": ["search", "find", "prompts query"], "flags": ["[term]", "--words <W>", "--limit <N>", "--status <S>", "--json"], "description": "Query cached SQLite prompts with ≥200-word preview" },
                    { "name": "prune", "aliases": ["pr", "clear-cache"], "flags": ["--keep <N>", "--preflight", "--undo", "--json"], "description": "Safely prune older conversations (guards active prompts & ≥5 sessions)" },
                    { "name": "resend-running-commands", "aliases": ["rrc"], "flags": ["[N]", "--json", "-f [path]"], "description": "Resend commands before close/switch & sync image paths" },
                    { "name": "queue-scheduler", "aliases": ["scheduler", "qs"], "flags": ["--once", "[instance]"], "description": "10-minute automated queue bookkeeping: auto-push FIFO prompts when project is idle" }
                ]
            },
            {
                "group": "Sandbox Instance & Profile Isolation",
                "items": [
                    { "name": "instances", "aliases": ["instances ls"], "flags": ["--json"], "description": "List all sandbox profiles and running PIDs" },
                    { "name": "instances <target> ff", "aliases": [], "flags": [], "description": "Fast-forward rotate account for specific sandbox instance" },
                    { "name": "instances-all ff", "aliases": [], "flags": [], "description": "Fast-forward rotate accounts across ALL sandbox instances" },
                    { "name": "instances create", "aliases": [], "flags": ["\"<name>\"", "--data-only"], "description": "Create an isolated sandbox instance profile" },
                    { "name": "instances rm", "aliases": [], "flags": ["<seq|id|alias>", "--force"], "description": "Remove a specific sandbox instance profile" }
                ]
            },
            {
                "group": "Email & Telegram Telemetry",
                "items": [
                    { "name": "email", "aliases": [], "flags": ["status", "help", "ls", "add", "rm", "mv", "export", "--json"], "description": "Manage email alerts, sender accounts, and recipients" },
                    { "name": "telegram", "aliases": [], "flags": ["ls", "set", "ping", "cmds", "chat", "setup", "help"], "description": "Manage Telegram bot token, chat ID, automated setup, and alerts" },
                    { "name": "broadcast-email", "aliases": [], "flags": ["ls", "add", "rm", "send-to-all", "send", "test"], "description": "Broadcast status cards to all configured recipients" }
                ]
            },
            {
                "group": "Supabase Fleet Sync & Lease Architecture",
                "items": [
                    { "name": "supabase help", "aliases": ["supa help"], "flags": [], "description": "Display complete config locations, JSON examples, and syntax guide" },
                    { "name": "supabase status", "aliases": ["supa status", "supa ls"], "flags": [], "description": "Show local node alias, IP, endpoints, and active remote leases" },
                    { "name": "supabase list-leases", "aliases": ["leases", "in-use"], "flags": [], "description": "List all accounts currently leased and in-use across cluster machines" },
                    { "name": "supabase test", "aliases": ["supa test"], "flags": ["[endpoint_id]"], "description": "Test HTTP connection and table accessibility to Supabase endpoints" },
                    { "name": "supabase set", "aliases": ["set-endpoint"], "flags": ["<id> <name> <url> <key> <role> [notes] [tags]"], "description": "Configure or update Supabase endpoint directly from terminal" },
                    { "name": "supabase load-json", "aliases": ["import"], "flags": ["<file>"], "description": "Ingest and merge endpoints from JSON file" },
                    { "name": "supabase sync", "aliases": ["supa sync"], "flags": [], "description": "Trigger immediate local node heartbeat and instance profile sync" },
                    { "name": "supabase schema", "aliases": ["supa schema"], "flags": ["[root|secondary]"], "description": "Output SQL schema DDL for Supabase SQL Editor" }
                ]
            }
        ]
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&help_obj).unwrap_or_default()
    );
}

pub(crate) fn print_help() {
    println!("  ┌────────────────────────────────────────────────────────────────────────────┐");
    println!("  │ Antigravity-Manager (AGM) CLI Usage & Operational Guide                    │");
    println!("  └────────────────────────────────────────────────────────────────────────────┘");
    println!("  Syntax:");
    println!("    agm <command> [arguments] [flags]");
    println!();
    println!("  ────────────────────────────────────────────────────────────────────────────");
    println!("  ACCOUNT ROTATION & QUOTA GOVERNANCE");
    println!("  ────────────────────────────────────────────────────────────────────────────");
    println!("    status, credits [--json]");
    println!("        Show node status, immediate & weekly credits, active account");
    println!("    ff, smart-switch, fast-forward [instance]");
    println!("        Trigger fast-forward rotation to freshest 100% quota account (for instance or all)");
    println!("    switch, switch-account <email|prefix|id|#seq> [--instance <id|alias>]");
    println!(
        "        Directly switch active profile (or specified instance) without GUI intervention"
    );
    println!("    switch <instance> <account>");
    println!("        Positional switch: switch specified instance profile to account");
    println!(
        "    auto-switch, auto [status|enable|disable|toggle|run|threshold|interval|model|test]"
    );
    println!(
        "        Inspect, toggle, configure, or evaluate rolling 4-hour quota auto-switcher daemon"
    );
    println!("    switch-if-low-credit, swlc, sfc [-t <pct>] [--json] [-f [file]] [--force]");
    println!("        Check live quota; rotate if quota <= threshold (default: 15.0%)");
    println!("    is-low-credit-for-switch, ilc [-t <pct>] [--json]");
    println!("        Check if active quota <= threshold (outputs true/false or JSON)");
    println!("    accounts, acc [--active] [--json]");
    println!("        List registered accounts, tiers, and remaining quotas");
    println!("    accounts refresh-tier [--all] [--json]");
    println!("        Fetch and persist subscription tiers (PRO / ULTRA / FREE)");
    println!("    history, audit [--page <n>] [--json]");
    println!(
        "        Show the last 100 task-history rows and where each split database file lives"
    );
    println!("    account switch <email|id|#seq> [--instance <id>]");
    println!("        Switch account profile via accounts subcommand");
    println!();
    println!("  ────────────────────────────────────────────────────────────────────────────");
    println!("  UPDATE, REPO SYNC & FLEET ORCHESTRATION");
    println!("  ────────────────────────────────────────────────────────────────────────────");
    println!("    update, update-all, ua [all] [--json] [--check] [--force]");
    println!("        Check GitHub releases, update binary, pull git repo, and sync fleet");
    println!("        Pass --json for clean, zero-noise stdout machine automation");
    println!("    sync");
    println!("        Synchronize local accounts, instances, and DB vaults");
    println!("    pull");
    println!("        Execute git pull origin main in repository root");
    println!("    ssh <target> [options], sj, se");
    println!("        Connect to remote VM via SSH, manage keys/nodes, or run remote command");
    println!(
        "        Subcommands: exec <node> \"<cmd>\", deploy-keys, fix-auth, copy-id, keys, nodes"
    );
    println!("        GitMap Parity Examples:");
    println!(
        "          agm ssh exec vm-01 \"gitmap aum status\"      # Run command on target machine"
    );
    println!(
        "          agm ssh exec all \"agm update\"               # Update AGM across entire fleet"
    );
    println!(
        "          agm ssh deploy-keys                         # Distribute SSH keys across nodes"
    );
    println!(
        "          agm ssh nodes ls                            # List connected cluster nodes"
    );
    println!();
    println!("  ────────────────────────────────────────────────────────────────────────────");
    println!("  PARALLEL PROMPT BACKUP & WORKSPACE RESTORATION");
    println!("  ────────────────────────────────────────────────────────────────────────────");
    println!("    backup, backpack, backup-running-prompts [ls|clean] [-f file] [--json]");
    println!(
        "        Parallel snapshot of active prompts across all workspaces to split SQLite DB"
    );
    println!("    restore, restore-running-prompts [--keep] [--json] [-f file]");
    println!("        Restore and re-enqueue in-flight prompts into active workspaces");
    println!("    queue-scheduler, scheduler, qs [--once] [instance]");
    println!("        10-minute automated queue bookkeeping: auto-push FIFO prompts when project is idle");
    println!("    tree [all] [--words <W>] [--json]");
    println!("        Render Project → Conversation → 200-Word Prompt tree with Dual Seq IDs ([AGM:P001 | GM:#1], [AGM:C001 | GM:<cid>])");
    println!("    which-prompts-running, wpr [--json]");
    println!("        List running projects, conversation IDs, and prompt queues");
    println!("    query, search [term] [--words <W>] [--limit <N>] [--status <S>] [--json]");
    println!("        Query cached prompts in SQLite with ≥200-word preview, filtering by status or term");
    println!("    prompts query [term] [--words <W>] [--limit <N>] [--status <S>] [--json]");
    println!("        Query cached prompts in SQLite with ≥200-word preview, filtering by status or term");
    println!("    prompts show <id|seq> [--json]");
    println!("        Show full prompt instructions and metadata for specific ID or sequence");
    println!("    prompt [C001|P001|GM:#1|proj] \"<text>\" [--instance <id|#seq>] [--node <alias>] [--prefix C] [--suffix C]");
    println!("        Dispatch prompt by Dual AGM/GitMap Sequence ID, project, instance, or remote SSH node");
    println!("        Examples:");
    println!("          agm prompt C001 \"continue task\"               # Inject prompt locally into C001");
    println!("          agm prompt P001 \"check build\" --node vm-01    # Select machine and dispatch via SSH");
    println!("          gitmap ssh exec vm-01 \"agm status\"            # Query remote machine via GitMap SSH");
    println!("    resend-running-commands, rrc [N] [--json] [-f [path]]");
    println!("        Resend commands before close/switch & sync image paths to resume file");
    println!("    prune, pr [--keep <N>] [--json]");
    println!("        Safely prune older conversations (guards active prompts & guarantees ≥5 sessions per active workspace)");
    println!();
    println!("  ────────────────────────────────────────────────────────────────────────────");
    println!("  SANDBOX INSTANCES & PROFILE ISOLATION");
    println!("  ────────────────────────────────────────────────────────────────────────────");
    println!("    instances [ls] [--json]");
    println!("        List all sandbox profiles, bound accounts, and running PIDs");
    println!("    instances create <name> [--account <email|id>] [--from default|<inst>] [--data-only] [--launch]");
    println!(
        "        Create a new empty sandbox, or clone one with --from default (or another instance)"
    );
    println!("    test-instance-flow [--from default|<inst>] [--new] [--json]");
    println!(
        "        Switch test. Default clones the default IDE. --new starts an empty instance instead"
    );
    println!("    instances switch <seq|id|alias> <account>");
    println!("        Switch an instance profile's bound account credentials directly");
    println!("    instances launch <seq|id|alias>");
    println!("        Launch Antigravity IDE for the specified instance profile");
    println!("    instances stop <seq|id|alias>");
    println!("        Safely stop running Antigravity IDE process for specified instance");
    println!("    instances assign <seq|id|alias> <repo_path...>");
    println!("        Bind/assign one or more project workspaces to a specific sandbox instance");
    println!("    instances <seq|id|alias> ff");
    println!("        Fast-forward rotate account for specific sandbox instance");
    println!("    instances-all ff");
    println!("        Fast-forward rotate accounts across ALL sandbox instances");
    println!("    instances auto-switch [status|enable|disable|toggle|run]");
    println!("        Inspect or control background auto-profile switcher for instances");
    println!("    instances rm <seq|id|alias> [--force]");
    println!("        Remove a specific sandbox instance profile");
    println!("    instances rm-all");
    println!("        Remove all non-default instances (preserves default)");
    println!();
    println!("  ────────────────────────────────────────────────────────────────────────────");
    println!("  EMAIL & TELEGRAM MULTI-VM TELEMETRY");
    println!("  ────────────────────────────────────────────────────────────────────────────");
    println!("    email [status|help|ls|add|rm|mv|export] [--json]");
    println!("        Show email notification status or manage IMAP/SMTP accounts");
    println!("    telegram [ls|set|ping|cmds|chat|setup|help]");
    println!("        Manage Telegram bot token, chat ID, automated setup, and alerts");
    println!("    broadcast-email [ls|add|rm|send-to-all|send|send-help|test]");
    println!("        Manage multi-recipient email broadcasts");
    println!();
    println!("  ────────────────────────────────────────────────────────────────────────────");
    println!("  SUPABASE FLEET SYNC & LEASE ARCHITECTURE");
    println!("  ────────────────────────────────────────────────────────────────────────────");
    println!("    supabase [help]");
    println!("        Complete operational guide, config file locations, and JSON formatting");
    println!("    supabase status, ls");
    println!("        Show local node ID, alias, IP, configured endpoints, and active leases");
    println!("    supabase list-leases, leases, in-use");
    println!("        Show which accounts are currently selected / in-use across machines");
    println!("    supabase test [endpoint_id]");
    println!("        Test connection & table schema for configured Supabase endpoints");
    println!("    supabase set <id> <name> <url> <key> <role> [notes] [tags]");
    println!("        Add or update Supabase endpoint directly from command line");
    println!("    supabase load-json <file>");
    println!("        Load endpoints and configuration from JSON file");
    println!("    supabase sync");
    println!("        Synchronize local machine node and instance profiles to Supabase");
    println!("    supabase schema [root|secondary]");
    println!("        Output SQL schema DDL to create tables in Supabase SQL Editor");
    println!();
    println!("  ────────────────────────────────────────────────────────────────────────────");
    println!("  REAL-WORLD EXAMPLES (AGM & GITMAP PARITY)");
    println!("  ────────────────────────────────────────────────────────────────────────────");
    println!(
        "    # 1. Inspect Bracketed Project → Conversation → 200-Word Prompt Tree (Dual Seq IDs):"
    );
    println!("    agm tree                            # Shows [AGM:P001 | GM:#1] & [AGM:C001 | GM:<cid>]");
    println!("    agm tree all --words 200");
    println!(
        "    agm running-prompts --words 200     # Query cached prompts with 200-word preview"
    );
    println!("    gitmap agy active");
    println!();
    println!("    # 2. Inject Prompt by AGM or GitMap Sequence ID, Instance, or Remote SSH Node:");
    println!("    agm prompt C001 \"Is it done?\"");
    println!("    agm prompt GM:#1 --instance #2 \"Run cargo clippy\"");
    println!("    agm prompt C001 --instance default --node vm-01 \"Check build status\"");
    println!("    gitmap agy prompt -n read-all -t \"Read memory and continue\"");
    println!("    gitmap agy prompt -n is-done -t \"Verify all tasks\"");
    println!("    gitmap agy prompt-project P001 -n is-done -t \"Check build\"");
    println!();
    println!("    # 3. Backup & Restore Running Storage Prompts + FPUG / SUG (AGM & GitMap):");
    println!("    agm backup && agm backup ls && agm restore");
    println!("    gitmap agy running-prompts ls | backup | restore");
    println!("    gitmap backup-running-prompts && gitmap restore-running-prompts");
    println!("    agm agy fpug ls && agm agy sug ls");
    println!();
    println!("    # 4. Multi-Instance Creation, Account Switching & Auto-Switching:");
    println!("    agm instances create \"Worker-2\" -a dev@gmail.com");
    println!("    agm create \"QA-Test\" --data-only --launch");
    println!("    agm switch #2 dev2@gmail.com");
    println!("    agm switch dev2@gmail.com --instance Worker-2");
    println!("    agm auto-switch status");
    println!("    agm auto-switch enable");
    println!("    agm auto-switch toggle");
    println!("    agm auto-switch run");
    println!("    agm instances assign #2 d:\\work\\Antigravity-Manager d:\\work\\gitmap-v28");
    println!("    agm instances #2 ff");
    println!();
    println!("    # 5. Update AGM & GitMap Locally or Across SSH Fleet:");
    println!("    agm update                  # Update AGM binary");
    println!("    agm update gitmap           # Update GitMap CLI");
    println!("    agm update all --json       # Full fleet + repo + GitMap update");
    println!("    gitmap agm update -y        # Update AGM via GitMap installer");
    println!("    gitmap ssh update agm       # Update AGM across SSH fleet");
    println!();
    println!(
        "    # 6. Conversation Pruning & Hygiene (Preserves Active Prompts & Top 5 Sessions):"
    );
    println!(
        "    agm prune                           # Safe prune keeping 10 latest conversations"
    );
    println!("    agm prune --keep 5                  # Prune keeping 5 latest conversations");
    println!(
        "    agm pr 5                            # Shorthand prune keeping 5 latest conversations"
    );
    println!("    agm clean-conversations --preflight # Dry-run preview of space to be reclaimed");
    println!();
    println!("    # 7. Multi-Node SSH Fleet Execution & Machine Selection (GitMap & AGM):");
    println!("    agm ssh nodes                       # List registered SSH cluster nodes");
    println!("    agm ssh check <node>                # Check SSH connectivity, port 22 & latency");
    println!("    agm ssh exec \"agm status\"           # Run command across entire SSH cluster");
    println!(
        "    agm ssh <node> \"agm accounts\"        # Run AGM command on a specific remote machine"
    );
    println!("    agm prompt P001 \"Build\" --node <node> # Route prompt injection to a specific remote node");
    println!("    gitmap ssh nodes                    # List reachable GitMap SSH fleet nodes");
    println!("    gitmap ssh check <node>             # Diagnostic probe of remote node health");
    println!(
        "    gitmap ssh key export               # Export SSH public key for cluster deployment"
    );
    println!("    gitmap ssh key deploy <node>        # Deploy authorization key to remote node");
    println!("    gitmap ssh <node> \"agm status\"      # Run AGM command on specific remote machine via GitMap");
    println!("    gitmap ssh exec \"gitmap pe\"         # Run pipeline evaluation across all cluster machines");
    println!();
    println!("    # 8. Query Cached Prompts from Terminal (SQLite Cache ≥200 Words Preview):");
    println!("    agm prompts query                   # View latest prompts with 200-word preview");
    println!(
        "    agm prompts query \"pipeline\"        # Search cached prompts containing 'pipeline'"
    );
    println!("    agm prompts query -s running        # Filter only actively running prompts");
    println!(
        "    agm prompts query --words 300 -n 10 # Display top 10 prompts with 300 words each"
    );
    println!();
}

pub(crate) fn print_commands_table() {
    let (node_alias, _) = antigravity_tools_lib::modules::email_sender::get_local_node_identity();
    println!("Available Inbound Commands & Email Format Table:");
    println!(
        "  {:<26} {:<36} {:<32}",
        "ACTION / GOAL", "EMAIL REPLY SUBJECT", "CLI EQUIVALENT"
    );
    println!("  {}", "-".repeat(98));
    println!(
        "  {:<26} {:<36} {:<32}",
        "Create Instance Profile",
        format!("cmd: {} | agm create", node_alias),
        "agm instances create \"<name>\""
    );
    println!(
        "  {:<26} {:<36} {:<32}",
        "Switch Account Profile",
        format!("cmd: {} | agm switch", node_alias),
        "agm switch <account|#seq>"
    );
    println!(
        "  {:<26} {:<36} {:<32}",
        "Toggle Auto-Switcher",
        format!("cmd: {} | agm auto toggle", node_alias),
        "agm auto-switch toggle"
    );
    println!(
        "  {:<26} {:<36} {:<32}",
        "Send Prompt to Project",
        format!("sub: {} | proj-<project_name>", node_alias),
        "agm prompt -p \"<name>\" \"<text>\""
    );
    println!(
        "  {:<26} {:<36} {:<32}",
        "Broadcast to All",
        format!("sub: {} | all", node_alias),
        "agm broadcast-email send-to-all"
    );
    println!(
        "  {:<26} {:<36} {:<32}",
        "Check Quota & Credits",
        format!("cmd: {} | agm status", node_alias),
        "agm status"
    );
    println!(
        "  {:<26} {:<36} {:<32}",
        "Trigger Auto-Switch",
        format!("cmd: {} | agm switch-if-low-credit", node_alias),
        "agm swlc"
    );
    println!(
        "  {:<26} {:<36} {:<32}",
        "List Running Prompts",
        format!("cmd: {} | agm running-prompts ls", node_alias),
        "agm running-prompts ls"
    );
    println!(
        "  {:<26} {:<36} {:<32}",
        "Inbound Commands Manual", "Subject: help", "agm help"
    );
    println!();
}
