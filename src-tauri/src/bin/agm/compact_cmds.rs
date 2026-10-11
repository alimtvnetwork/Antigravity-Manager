//! CLI Command handlers for AGM Workspace Compaction Export & Import.

use antigravity_tools_lib::modules::compact_engine::{
    export_compact, import_compact, ExportOptions, ImportOptions,
};
use std::io::{self, Write};

pub(crate) fn cmd_export_compact(args: &[String]) {
    let mut file: Option<String> = None;
    let mut work_dir: Option<String> = None;
    let mut is_pinned_included = true;
    let mut is_relative = true;

    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "-f" | "--file" => {
                if idx + 1 < args.len() {
                    file = Some(args[idx + 1].clone());
                    idx += 1;
                }
            }
            "-w" | "--work-dir" | "--workdir" => {
                if idx + 1 < args.len() {
                    work_dir = Some(args[idx + 1].clone());
                    idx += 1;
                }
            }
            "--no-pinned" => {
                is_pinned_included = false;
            }
            "--absolute" => {
                is_relative = false;
            }
            "-h" | "--help" => {
                print_export_help();
                return;
            }
            other if !other.starts_with('-') && file.is_none() => {
                file = Some(other.to_string());
            }
            _ => {}
        }
        idx += 1;
    }

    let options = ExportOptions {
        file,
        work_dir,
        is_pinned_included,
        is_relative,
    };

    println!("================================================================================");
    println!("  AGM Workspace Compaction Export");
    println!("================================================================================");
    println!("  Exporting workspace snapshot...");

    match export_compact(&options) {
        Ok(res) => {
            let format_str = if res.is_sqlite {
                "Standalone Portable SQLite DB"
            } else {
                "Two-Tier JSON Envelope v2.0"
            };
            println!("  ✔ Snapshot created successfully.");
            println!("  Output File:   {}", res.output_path);
            println!("  Format:        {}", format_str);
            println!("  Pinned Repos:  {} project(s)", res.pinned_count);
            println!("  Conversations: {} session(s)", res.conversations_count);
            println!("  Active Prompts:{} prompt(s)", res.running_count);
            println!("  History Log:   {} record(s)", res.history_count);
            println!("  Duration:      {} ms", res.duration_ms);
            println!(
                "================================================================================"
            );
        }
        Err(e) => {
            eprintln!("  ✖ Export failed: {}", e);
            println!(
                "================================================================================"
            );
            std::process::exit(2);
        }
    }
}

pub(crate) fn cmd_import_compact(args: &[String]) {
    let mut file: Option<String> = None;
    let mut work_dir: Option<String> = None;
    let mut is_auto_confirmed = false;

    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "-w" | "--work-dir" | "--workdir" => {
                if idx + 1 < args.len() {
                    work_dir = Some(args[idx + 1].clone());
                    idx += 1;
                }
            }
            "-y" | "--yes" => {
                is_auto_confirmed = true;
            }
            "-h" | "--help" => {
                print_import_help();
                return;
            }
            other if !other.starts_with('-') && file.is_none() => {
                file = Some(other.to_string());
            }
            _ => {}
        }
        idx += 1;
    }

    let target_file = match file {
        Some(f) => f,
        None => {
            eprintln!("✖ Error: Missing input file argument.");
            println!();
            print_import_help();
            std::process::exit(1);
        }
    };

    if !is_auto_confirmed {
        print!("Importing '{target_file}' will merge projects and prompts into local SQLite. Proceed? [y/N]: ");
        io::stdout().flush().unwrap_or(());
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() || !input.trim().eq_ignore_ascii_case("y") {
            println!("Import cancelled by user.");
            return;
        }
    }

    println!("================================================================================");
    println!("  AGM Workspace Compaction Import");
    println!("================================================================================");
    println!("  Restoring workspace entities from '{target_file}'...");

    let options = ImportOptions {
        file: target_file,
        work_dir,
        is_auto_confirmed,
    };

    match import_compact(&options) {
        Ok(res) => {
            println!("  ✔ Import completed successfully.");
            println!("  Source:        {}", res.source_path);
            println!("  Target WorkDir:{}", res.target_work_dir);
            println!("  Pinned Repos:  {} restored", res.pinned_restored);
            println!("  Conversations: {} restored", res.conversations_restored);
            println!("  Active Prompts:{} restored", res.running_restored);
            println!("  History Log:   {} restored", res.history_restored);
            println!("  Duration:      {} ms", res.duration_ms);
            println!(
                "================================================================================"
            );
        }
        Err(e) => {
            eprintln!("  ✖ Import failed: {}", e);
            println!(
                "================================================================================"
            );
            std::process::exit(2);
        }
    }
}

fn print_export_help() {
    println!("Usage:");
    println!("  agm export-compact [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -f, --file <PATH>       Target output file path (.json or .db).");
    println!("                          Defaults to: agm-workspace-compact-<date>.json");
    println!(
        "  -w, --work-dir <PATH>   Base workspace directory to calculate relative paths against."
    );
    println!("      --no-pinned         Skip exporting pinned projects; export only prompts and history.");
    println!("      --absolute          Preserve raw absolute filesystem paths instead of relative templates.");
    println!("  -h, --help              Print this help documentation.");
    println!();
    println!("Examples:");
    println!("  agm export-compact");
    println!("  agm export-compact -f backup.json");
    println!("  agm export-compact -f portable.db --work-dir D:\\work");
}

fn print_import_help() {
    println!("Usage:");
    println!("  agm import-compact <FILE> [OPTIONS]");
    println!();
    println!("Arguments:");
    println!(
        "  <FILE>                  Path to compact JSON envelope or SQLite database to import."
    );
    println!();
    println!("Options:");
    println!(
        "  -w, --work-dir <PATH>   Target base workspace directory. Re-anchors all relative paths."
    );
    println!("  -y, --yes               Bypass confirmation prompt; automatically execute import.");
    println!("  -h, --help              Print this help documentation.");
    println!();
    println!("Examples:");
    println!("  agm import-compact backup.json");
    println!("  agm import-compact portable.db -y");
    println!("  agm import-compact backup.json --work-dir /home/ubuntu/work -y");
}
