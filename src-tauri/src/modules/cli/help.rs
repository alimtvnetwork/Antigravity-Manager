use crate::modules::{account, auto_switcher, config, instance, repo_db};

use super::*;

// -----------------------------------------------------------------------------
// AGY Cleaner & Help Utilities
// -----------------------------------------------------------------------------

pub(crate) fn print_all_cli_help() {
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

pub(crate) fn print_instance_cli_help() {
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

pub(crate) fn print_switch_cli_help() {
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

pub(crate) fn print_auto_switch_cli_help() {
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

pub(crate) fn print_profile_help() {
    print_instance_cli_help();
}

pub(crate) fn print_agy_cli_help() {
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
