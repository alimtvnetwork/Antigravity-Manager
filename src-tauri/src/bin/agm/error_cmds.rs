//! error_cmds — CLI command handlers for `agm errors` suite.
//!
//! Subcommands:
//!   - `agm errors list [--limit N] [--severity S] [--json]`
//!   - `agm errors clear`
//!   - `agm errors fix`

use antigravity_tools_lib::modules::error_db::{self, ErrorQueryOptions};
use chrono::{DateTime, Local, TimeZone, Utc};

pub(crate) fn dispatch_errors_domain(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        print_errors_help();
        return;
    }

    let first = args.first().map(|s| s.as_str()).unwrap_or("list");

    match first {
        "list" | "ls" => {
            cmd_errors_list(&args[1..]);
        }
        "clear" | "clean" | "purge" => {
            cmd_errors_clear(&args[1..]);
        }
        "fix" | "heal" | "repair" => {
            cmd_errors_fix(&args[1..]);
        }
        "count" => {
            cmd_errors_count(&args[1..]);
        }
        s if s.starts_with('-') => {
            // First argument is a flag (e.g. `agm errors --limit 50` or `agm errors --json`)
            cmd_errors_list(args);
        }
        _ => {
            eprintln!("[WARN] Unknown errors subcommand: '{first}'");
            print_errors_help();
        }
    }
}

fn print_errors_help() {
    println!("AGM Speed Error Database & Self-Healing Diagnostics:");
    println!("  agm errors [list] [--limit <N>] [--severity <S>] [--json]");
    println!("  agm errors clear");
    println!("  agm errors fix [--dry-run] [--json]");
    println!("\nDescription:");
    println!(
        "  Inspect, clear, and heal errors tracked in SQLite speed error database (errors.db)."
    );
    println!(
        "  The automated self-healing engine detects and surgically resolves stale lockfiles,"
    );
    println!("  file permissions, corrupted configurations, and database WAL anomalies.");
    println!("\nCommands:");
    println!("  list, ls            Display recent errors in a formatted table or JSON (default)");
    println!("  clear, clean        Purge error history from errors.db");
    println!("  fix, heal           Automated diagnostic & self-healing engine (stale locks, permissions, configs)");
    println!("  count               Display total error counts by severity");
    println!("\nOptions:");
    println!("  --limit, -l <N>     Maximum errors to display (default: 20)");
    println!("  --severity, -s <S>  Filter by severity level ('error', 'warn', 'info')");
    println!(
        "  --dry-run           Inspect issues without applying surgical fixes (for fix command)"
    );
    println!("  --json, -j          Output in machine-readable JSON format");
    println!("  --help, -h          Show help message");
    println!("\nExamples:");
    println!("  agm errors                          # View recent errors in table format");
    println!("  agm errors list --limit 50          # View up to 50 recent errors");
    println!("  agm errors list --severity error    # View only error-level entries");
    println!(
        "  agm errors fix                      # Run automated self-healing & repair common issues"
    );
    println!("  agm errors clear                    # Purge error history");
}

pub(crate) fn cmd_errors_list(args: &[String]) {
    let is_json = args.iter().any(|a| a == "--json" || a == "-j");

    let mut limit = 20usize;
    let mut severity: Option<String> = None;
    let mut search: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if (arg == "--limit" || arg == "-l") && i + 1 < args.len() {
            if let Ok(val) = args[i + 1].parse::<usize>() {
                limit = val.max(1);
            }
            i += 2;
            continue;
        } else if (arg == "--severity" || arg == "-s") && i + 1 < args.len() {
            severity = Some(args[i + 1].clone());
            i += 2;
            continue;
        } else if arg == "--json" || arg == "-j" {
            i += 1;
            continue;
        } else if !arg.starts_with('-') && arg != "list" && arg != "ls" {
            search = Some(arg.clone());
            i += 1;
            continue;
        }
        i += 1;
    }

    let options = ErrorQueryOptions {
        limit: Some(limit),
        offset: Some(0),
        severity: severity.clone(),
        error_code: None,
        search,
    };

    match error_db::list_errors(&options) {
        Ok(records) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&records).unwrap_or_else(|_| "[]".to_string())
                );
                return;
            }

            if records.is_empty() {
                println!("\n  ✓ No errors recorded in errors.db. System operating normally!\n");
                return;
            }

            let total_count =
                error_db::count_errors(severity.as_deref()).unwrap_or(records.len() as i64);

            println!(
                "================================================================================"
            );
            println!(
                "  AGM Speed Error Database (errors.db) - Total: {} error(s) | Showing: {}",
                total_count,
                records.len()
            );
            println!(
                "================================================================================"
            );
            println!(
                "  {:<4} {:<8} {:<7} {:<19} {:<22} {}",
                "#", "CODE", "SEV", "TIMESTAMP", "SOURCE", "MESSAGE"
            );
            println!(
                "  ------------------------------------------------------------------------------"
            );

            for (idx, r) in records.iter().enumerate() {
                let time_str = format_timestamp(r.created_at);
                let source_display = r
                    .source
                    .as_deref()
                    .unwrap_or(r.trigger_action.as_deref().unwrap_or("backend"));
                let truncated_source = truncate_str(source_display, 21);
                let truncated_msg = truncate_str(&r.message, 38);

                let sev_lower = r.severity.to_lowercase();
                let sev_display = match sev_lower.as_str() {
                    "error" => "ERROR",
                    "warn" | "warning" => "WARN",
                    "info" => "INFO",
                    _ => r.severity.as_str(),
                };

                println!(
                    "  {:<4} {:<8} {:<7} {:<19} {:<22} {}",
                    idx + 1,
                    r.error_code,
                    sev_display,
                    time_str,
                    truncated_source,
                    truncated_msg
                );
            }

            println!(
                "================================================================================"
            );
            println!("  • Clear history:    agm errors clear");
            println!("  • Run self-healing: agm errors fix\n");
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to query errors from errors.db: {e}");
        }
    }
}

pub(crate) fn cmd_errors_clear(args: &[String]) {
    let is_json = args.iter().any(|a| a == "--json" || a == "-j");

    match error_db::clear_errors() {
        Ok(cleared) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::json!({
                        "success": true,
                        "cleared": cleared,
                        "message": format!("Successfully cleared {cleared} error records from errors.db")
                    })
                );
            } else {
                println!("✅ Successfully cleared {cleared} recorded error(s) from errors.db.");
            }
        }
        Err(e) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::json!({
                        "success": false,
                        "error": e
                    })
                );
            } else {
                eprintln!("[ERROR] Failed to clear errors database: {e}");
            }
        }
    }
}

pub(crate) fn cmd_errors_fix(args: &[String]) {
    let is_json = args.iter().any(|a| a == "--json" || a == "-j");
    let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-d");
    let apply_fixes = !dry_run;

    match error_db::run_self_healing(apply_fixes) {
        Ok(report) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
                );
                return;
            }

            println!(
                "================================================================================"
            );
            if dry_run {
                println!("  AGM Diagnostic Engine (Dry-Run Preview — No Fixes Applied)                    ");
            } else {
                println!("  AGM Automated Diagnostic & Self-Healing Engine                                ");
            }
            println!(
                "================================================================================"
            );

            for check in &report.checks {
                let badge = if check.passed {
                    if check.auto_fixed {
                        "[🔧 FIXED]"
                    } else {
                        "[✓ PASSED]"
                    }
                } else {
                    "[✗ FAILED]"
                };

                println!("  {:<10} {:<32} {}", badge, check.name, check.message);
            }

            if !report.fixes_applied.is_empty() {
                println!("\n  Surgical Fixes Applied:");
                for fix in &report.fixes_applied {
                    println!("    + {fix}");
                }
            }

            if !report.warnings.is_empty() {
                println!("\n  Diagnostic Warnings:");
                for warn in &report.warnings {
                    println!("    ! {warn}");
                }
            }

            println!(
                "================================================================================"
            );
            let status_label = if report.is_healthy && report.fixed_count > 0 {
                "RECOVERED"
            } else if report.is_healthy {
                "HEALTHY"
            } else {
                "ATTENTION REQUIRED"
            };

            println!(
                "  Summary: {}/{} checks passed. {} surgical fix(es) applied. Status: [{}]",
                report.passed_checks, report.total_checks, report.fixed_count, status_label
            );
            println!("================================================================================\n");
        }
        Err(e) => {
            eprintln!("[ERROR] Self-healing diagnostic failed: {e}");
        }
    }
}

pub(crate) fn cmd_errors_count(args: &[String]) {
    let is_json = args.iter().any(|a| a == "--json" || a == "-j");

    let total = error_db::count_errors(None).unwrap_or(0);
    let errors = error_db::count_errors(Some("error")).unwrap_or(0);
    let warns = error_db::count_errors(Some("warn")).unwrap_or(0);
    let infos = error_db::count_errors(Some("info")).unwrap_or(0);

    if is_json {
        println!(
            "{}",
            serde_json::json!({
                "total": total,
                "error": errors,
                "warn": warns,
                "info": infos,
            })
        );
    } else {
        println!("Error Database Counts:");
        println!("  • Total:   {total}");
        println!("  • Errors:  {errors}");
        println!("  • Warns:   {warns}");
        println!("  • Infos:   {infos}");
    }
}

fn format_timestamp(millis: i64) -> String {
    if let Some(dt) = Utc.timestamp_millis_opt(millis).single() {
        let local_dt: DateTime<Local> = DateTime::from(dt);
        local_dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        "N/A".to_string()
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    let trimmed = s.replace('\n', " ").replace('\r', "");
    if trimmed.chars().count() <= max_len {
        trimmed
    } else {
        let mut truncated: String = trimmed.chars().take(max_len.saturating_sub(3)).collect();
        truncated.push_str("...");
        truncated
    }
}
