//! supabase_format — config format helpers, split from agm.rs.

pub(crate) fn cmd_which_format(args: &[String]) {
    use std::path::PathBuf;

    let mut paths: Vec<PathBuf> = Vec::new();
    let mut auto_yes = false;
    let mut auto_run = false;

    for arg in args {
        match arg.as_str() {
            "-y" | "--yes" => auto_yes = true,
            "-r" | "--run" | "--import" | "-i" => auto_run = true,
            "-h" | "--help" => {
                println!("================================================================================");
                println!("  AGM Universal Format Inspector & Importer");
                println!("================================================================================");
                println!("Usage:");
                println!("  agm which-format [files_or_dir...] [-y] [--run]");
                println!("  agm format inspect [files_or_dir...]");
                println!();
                println!("Description:");
                println!(
                    "  Scans specified JSON file(s) or folder, classifies each format against AGM"
                );
                println!("  envelope standards (attributes + data), displays what importing them will change,");
                println!(
                    "  and generates both individual and single-line bulk execution commands."
                );
                println!();
                println!("Options:");
                println!("  -y, --yes          Bypass confirmation prompts in generated commands");
                println!("  -r, --run          Immediately execute import for all matched schemas");
                println!("================================================================================");
                return;
            }
            other if !other.starts_with('-') => {
                paths.push(PathBuf::from(other));
            }
            _ => {}
        }
    }

    let targets = resolve_json_targets(&paths);
    if targets.is_empty() {
        println!("⚠️ No JSON files found in target path(s).");
        return;
    }

    let mut matched = Vec::new();
    let mut unmatched = Vec::new();

    for target in &targets {
        let res = inspect_json_file(target);
        if res.detected_type.is_some() {
            matched.push(res);
        } else {
            unmatched.push(res);
        }
    }

    println!("================================================================================");
    println!("  AGM Universal Format Inspector & Schema Classifier");
    println!("================================================================================");
    println!(
        "  Discovered: {} JSON file(s) across target path(s)\n",
        targets.len()
    );

    if !matched.is_empty() {
        println!(
            "  --- Matched Supported Schemas ({} file(s)) ---",
            matched.len()
        );
        for (i, m) in matched.iter().enumerate() {
            let env_badge = if m.is_envelope {
                let ver = m.version.as_deref().unwrap_or("2.0");
                if m.variables_count > 0 {
                    format!("Envelope v{} ({} variables)", ver, m.variables_count)
                } else {
                    format!("Envelope v{}", ver)
                }
            } else if m.is_legacy {
                "Legacy Flat (Auto-Compatible)".to_string()
            } else {
                "Custom".to_string()
            };
            println!(
                "  #{:<2} [{}] ({})",
                i + 1,
                m.detected_type.as_deref().unwrap_or("unknown"),
                env_badge
            );
            println!("      Path:     {}", m.file_path);
            if let Some(ref wd) = m.work_directory {
                println!("      WorkDir:  {}", wd);
            }
            if let Some(ref note) = m.notes {
                println!("      Notes:    {}", note);
            }
            println!("      Changes:  {}", m.mutation_summary);
            println!("      Command:  {}", m.recommended_command);
            if let Some(ref exp) = m.export_command {
                println!("      Export:   {}", exp);
            }
            println!();
        }
    }

    if !unmatched.is_empty() {
        println!(
            "  --- Unmatched / Incompatible Schemas ({} file(s)) ---",
            unmatched.len()
        );
        for u in &unmatched {
            println!("  ✖ {}", u.file_path);
            if let Some(ref err) = u.error_detail {
                println!("    Reason:   {}", err);
            } else {
                println!("    Reason:   {}", u.mutation_summary);
            }
            println!();
        }
    }

    if let Some(bulk_cmd) = build_bulk_import_command(&matched) {
        println!("  --- Bulk Execution Command (Import All Matched) ---");
        println!(
            "  To import all {} matched file(s) in one command, run:",
            matched.len()
        );
        println!("  {}", bulk_cmd);
        println!();
        println!("  💡 Tip: Append '-y' or '--yes' to bypass all confirmation prompts.");
    }

    println!("================================================================================");

    if auto_run && !matched.is_empty() {
        println!(
            "\n🚀 Auto-executing import for {} matched file(s)...",
            matched.len()
        );
        let mut supabase_targets = Vec::new();
        for m in &matched {
            if let Some(ref dtype) = m.detected_type {
                if dtype.contains("supabase") {
                    supabase_targets.push(m.file_path.clone());
                }
            }
        }
        if !supabase_targets.is_empty() {
            let mut sb_args = supabase_targets;
            if auto_yes {
                sb_args.push("-y".to_string());
            }
            crate::supabase_config_cmds::cmd_supabase_load_json(&sb_args);
        }
        println!("✅ Auto-import completed successfully.");
    }
}
