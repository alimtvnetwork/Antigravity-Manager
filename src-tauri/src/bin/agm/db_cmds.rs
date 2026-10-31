//! CLI commands for Split-DB management, pre-migration snapshots, and zero-loss verification.

use antigravity_tools_lib::modules::db_migration::*;
use antigravity_tools_lib::modules::db_orm::*;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

pub fn handle_db_command(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args[0] == "help" || args[0] == "--help" || args[0] == "-h" {
        print_db_help();
        return Ok(());
    }

    match args[0].as_str() {
        "status" => cmd_db_status(args),
        "backup" => cmd_db_backup(args),
        "restore" => cmd_db_restore(args),
        "verify" => cmd_db_verify(args),
        other => Err(format!(
            "Unknown db command '{}'. Run 'agm db help' for usage.",
            other
        )),
    }
}

fn print_db_help() {
    println!(
        r#"AGM Database Management & Split-DB Migration Suite

Usage:
  agm db status [--json]
  agm db backup [--file <path>] [--json]
  agm db restore [--file <path>] [--json]
  agm db verify [--json]

Subcommands:
  status   Inspect all Split SQLite databases, schema versions, and row counts
  backup   Export single-file pre-migration snapshot (pre_migration_v4_snapshot.json)
  restore  Freshly initialize PascalCase Split-DBs and re-import snapshot data
  verify   Validate zero account loss and SQLite integrity pragmas
"#
    );
}

fn cmd_db_status(args: &[String]) -> Result<(), String> {
    let json_mode = args.iter().any(|a| a == "--json");
    let data_dir = antigravity_tools_lib::modules::account::get_data_dir()
        .map_err(|e| format!("Failed to get data dir: {e}"))?;

    let split_dbs = [
        ("errors.db", data_dir.join("errors.db")),
        ("repo_prompts.db", data_dir.join("repo_prompts.db")),
        (
            "prompt-backups.db",
            data_dir.join("backup-prompts").join("backup-prompts.db"),
        ),
    ];

    let mut report = Vec::new();

    for (name, path) in &split_dbs {
        let exists = path.exists();
        let size_bytes = if exists {
            fs::metadata(path).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        let mut version = 0i64;
        let mut is_pascal = false;
        let mut tables_count = 0usize;

        if exists {
            if let Ok(conn) = Connection::open(path) {
                version = get_schema_version(&conn).unwrap_or(0);
                is_pascal = is_migrated_to_pascalcase(&conn);

                let count: usize = conn
                    .query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
                        [],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                tables_count = count;
            }
        }

        report.push(serde_json::json!({
            "database": name,
            "path": path.to_string_lossy(),
            "exists": exists,
            "sizeBytes": size_bytes,
            "schemaVersion": version,
            "isPascalCase": is_pascal,
            "tablesCount": tables_count
        }));
    }

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
        return Ok(());
    }

    println!(
        "\n╔════════════════════════════════════════════════════════════════════════════════╗"
    );
    println!("║                    AGM Split Database Status Overview                          ║");
    println!(
        "╚════════════════════════════════════════════════════════════════════════════════╝\n"
    );
    println!(
        "{:<20} {:<10} {:<12} {:<16} {:<10}",
        "DATABASE", "STATUS", "SIZE", "SCHEMA VERSION", "PASCALCASE"
    );
    println!("{:-<78}", "");

    for item in &report {
        let db = item["database"].as_str().unwrap_or("");
        let exists = item["exists"].as_bool().unwrap_or(false);
        let size_kb = item["sizeBytes"].as_u64().unwrap_or(0) / 1024;
        let version = item["schemaVersion"].as_i64().unwrap_or(0);
        let pascal = item["isPascalCase"].as_bool().unwrap_or(false);

        let status_str = if exists { "ONLINE" } else { "NOT FOUND" };
        let pascal_str = if pascal { "YES (v1)" } else { "NO (Legacy)" };

        println!(
            "{:<20} {:<10} {:<12} {:<16} {:<10}",
            db,
            status_str,
            format!("{} KB", size_kb),
            format!("v{}", version),
            pascal_str
        );
    }
    println!();
    Ok(())
}

fn cmd_db_backup(args: &[String]) -> Result<(), String> {
    let json_mode = args.iter().any(|a| a == "--json");
    let mut custom_file: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        if (args[i] == "--file" || args[i] == "-f") && i + 1 < args.len() {
            custom_file = Some(PathBuf::from(&args[i + 1]));
            i += 1;
        }
        i += 1;
    }

    let exported_path = export_pre_migration_snapshot(custom_file.as_deref())
        .map_err(|e| format!("Backup export failed: {e}"))?;

    let size = fs::metadata(&exported_path).map(|m| m.len()).unwrap_or(0);

    if json_mode {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "path": exported_path.to_string_lossy(),
                "sizeBytes": size
            })
        );
    } else {
        println!(
            "✅ Successfully exported pre-migration snapshot to {:?} ({} bytes)",
            exported_path, size
        );
    }
    Ok(())
}

fn cmd_db_restore(args: &[String]) -> Result<(), String> {
    let json_mode = args.iter().any(|a| a == "--json");
    let mut snapshot_file: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        if (args[i] == "--file" || args[i] == "-f") && i + 1 < args.len() {
            snapshot_file = Some(PathBuf::from(&args[i + 1]));
            i += 1;
        }
        i += 1;
    }

    let snapshot_path = match snapshot_file {
        Some(p) => p,
        None => get_default_snapshot_path().map_err(|e| format!("{e}"))?,
    };

    if !snapshot_path.exists() {
        return Err(format!("Snapshot file not found at {:?}", snapshot_path));
    }

    let summary = restore_from_snapshot(&snapshot_path, None)
        .map_err(|e| format!("Restore from snapshot failed: {e}"))?;

    if json_mode {
        println!(
            "{}",
            serde_json::json!({
                "success": true,
                "summary": {
                    "errorsImported": summary.errors_imported,
                    "projectsImported": summary.projects_imported,
                    "promptsImported": summary.prompts_imported,
                    "batchesImported": summary.batches_imported,
                    "backupsImported": summary.backups_imported,
                    "failedCommandsImported": summary.failed_commands_imported
                }
            })
        );
    } else {
        println!("\n✅ Successfully restored and migrated to PascalCase Split-DB:");
        println!("  - Errors migrated:           {}", summary.errors_imported);
        println!(
            "  - Projects migrated:         {}",
            summary.projects_imported
        );
        println!(
            "  - Prompts migrated:          {}",
            summary.prompts_imported
        );
        println!(
            "  - Backup batches migrated:   {}",
            summary.batches_imported
        );
        println!(
            "  - Prompt backups migrated:   {}",
            summary.backups_imported
        );
        println!(
            "  - Failed commands migrated:  {}",
            summary.failed_commands_imported
        );
        println!();
    }
    Ok(())
}

fn cmd_db_verify(args: &[String]) -> Result<(), String> {
    let json_mode = args.iter().any(|a| a == "--json");

    // 1. Verify accounts.json
    let snapshot = capture_accounts_snapshot()
        .map_err(|e| format!("Failed to capture accounts snapshot: {e}"))?;
    verify_zero_account_loss(&snapshot)
        .map_err(|e| format!("Zero account loss check failed: {e}"))?;

    // 2. Check SQLite pragmas across active databases
    let data_dir = antigravity_tools_lib::modules::account::get_data_dir()
        .map_err(|e| format!("Failed to get data dir: {e}"))?;

    let databases = [
        data_dir.join("errors.db"),
        data_dir.join("repo_prompts.db"),
        data_dir.join("backup-prompts").join("backup-prompts.db"),
    ];

    let mut integrity_results = Vec::new();

    for db_path in &databases {
        if db_path.exists() {
            if let Ok(conn) = Connection::open(db_path) {
                let status: String = conn
                    .query_row("PRAGMA integrity_check;", [], |r| r.get(0))
                    .unwrap_or_else(|e| format!("Error: {e}"));

                integrity_results.push(serde_json::json!({
                    "database": db_path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                    "integrity": status
                }));
            }
        }
    }

    if json_mode {
        println!(
            "{}",
            serde_json::json!({
                "zeroAccountLoss": true,
                "accountsCount": snapshot.account_count,
                "integrity": integrity_results
            })
        );
    } else {
        println!("\n✅ Database & Account Integrity Verification Passed:");
        println!(
            "  - Zero Account Loss Verified: {} account(s) active & intact",
            snapshot.account_count
        );
        for item in &integrity_results {
            println!(
                "  - {}: PRAGMA integrity_check = {}",
                item["database"].as_str().unwrap_or(""),
                item["integrity"].as_str().unwrap_or("")
            );
        }
        println!();
    }
    Ok(())
}
