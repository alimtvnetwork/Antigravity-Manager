use crate::modules::{account, auto_switcher, config, instance, repo_db};

use super::*;

pub(crate) fn print_prompts_cli_help() {
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

pub(crate) fn print_doctor_cli_help() {
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
