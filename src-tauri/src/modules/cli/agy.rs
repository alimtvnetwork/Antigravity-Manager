use super::*;

pub(crate) fn is_flag_present(args: &[String], flags: &[&str]) -> bool {
    args.iter().any(|a| flags.iter().any(|f| a == f))
}

pub(crate) fn parse_keep_count(args: &[String], default_val: usize) -> usize {
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

pub(crate) fn handle_agy_subcommand(sub_args: &[String]) {
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

pub(crate) fn handle_clear_action(extra_args: &[String], default_keep: usize) {
    let keep = parse_keep_count(extra_args, default_keep);
    let is_preflight = is_flag_present(extra_args, &["--precheck", "--pre", "--preflight", "-p"]);
    let is_yes = is_flag_present(extra_args, &["-y", "--yes", "-f"]);
    execute_agy_clear(keep, is_preflight, is_yes);
}

pub(crate) fn execute_agy_clear(keep_count: usize, is_preflight: bool, is_yes: bool) {
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
        // Justification: best-effort stream flush; buffered output is lost only on abnormal exit
        crate::error::record_ignored(io::stdout().flush(), "flush");
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

pub(crate) fn execute_agy_undo(tx_id: Option<&str>) {
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

pub(crate) fn print_preflight_report(report: &crate::modules::agy_cleaner::PreflightReport) {
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
