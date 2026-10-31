//! running_prompts_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::cli::CliEnvelope;
use antigravity_tools_lib::modules::repo_db::ActivePrompt;
use antigravity_tools_lib::modules::{
    account, backup_prompts_db, instance, json_envelope, repo_db,
};
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::fs;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize)]
pub struct InspectedPrompt {
    pub seq: usize,
    pub id: String,
    pub short_id: String,
    pub project_id: String,
    pub workspace_path: String,
    pub project_dual_seq: String,
    pub conversation_dual_seq: String,
    pub conversation_id: String,
    pub status: String,
    pub elapsed_seconds: i64,
    pub elapsed_time: String,
    pub word_count: usize,
    pub image_count: usize,
    pub has_images: bool,
    pub prompt_preview: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct InspectedInstance {
    pub seq_num: u32,
    pub instance_number: String,
    pub id: String,
    pub name: String,
    pub sequence: String,
    pub status: String,
    pub is_running: bool,
    pub pid: Option<u32>,
    pub bound_email: String,
    pub running_prompts_count: usize,
    pub prompts: Vec<InspectedPrompt>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RunningPromptDisplayRow {
    pub instance_seq: u32,
    pub instance_id: String,
    pub instance_name: String,
    pub bound_email: String,
    pub pid: Option<u32>,
    pub prompt_id: String,
    pub project_id: String,
    pub repo_path: String,
    pub conversation_id: String,
    pub project_dual_seq: String,
    pub conversation_dual_seq: String,
    pub status: String,
    pub word_count: usize,
    pub image_count: usize,
    pub prompt_preview: String,
    pub created_at: i64,
    pub updated_at: i64,
}

fn print_running_prompts_help() {
    println!("AGM Running Prompts & Instances Inspection:");
    println!("  agm running [--words <N>] [-i <instance>] [--json]");
    println!("  agm active [--words <N>] [-i <instance>] [--json]");
    println!("  agm running-prompts [--words <N>] [-i <instance>] [--json]");
    println!("  agm running backup [instance] [-f <path.db>] [--json]");
    println!("  agm running restore [instance] [--keep] [--json] [-f <path.db>]");
    println!("  agm running export [-f <path>] [--wc <N>]");
    println!("  agm running import [-f <path>] [--wc <N>]");
    println!("\nDescription:");
    println!("  Parallel inspection, snapshotting, and restoration of active running prompts");
    println!(
        "  across all Antigravity workspace projects and instances with dual sequence tracking."
    );
    println!("\nAliases: agm running, agm active, agm running-prompts, agm which-prompts-running, agm wpr");
    println!("\nSubcommands:");
    println!("  ls, list            List running prompts across active workspaces (default)");
    println!("  backup, brp         Snapshot active running prompts to split SQLite DB");
    println!("  restore, rrp        Restore and re-inject saved prompts into workspace queue");
    println!("  export, pe          Export prompts database to file or JSON");
    println!("  import, pi          Import prompts from file into local execution queue");
    println!("\nOptions:");
    println!(
        "    -w, --words, --wc <N>  Maximum words to display per prompt snippet (default: 200)"
    );
    println!("    --full                 Display full prompt text without truncation");
    println!("    -l, --limit <N>        Limit number of prompts per instance");
    println!(
        "    -i, --instance <id>    Filter running prompts by instance ID, name, or sequence (#1)"
    );
    println!("    -j, --json             Format output as structured JSON envelope");
    println!("    -h, --help             Show this help information");
    println!("\nExamples:");
    println!(
        "  agm running                         # Comprehensive inspection of instances & prompts"
    );
    println!("  agm running --words 150             # Inspect with 150-word preview");
    println!("  agm running -i #1                   # Inspect instance #1 running prompts");
    println!(
        "  agm running --json                  # Structured JSON output with instances and prompts"
    );
    println!(
        "  agm backup running default          # Snapshot running prompts for default instance"
    );
    println!("  agm restore running default         # Restore prompts for default instance");
}

pub(crate) fn format_dual_seq(agm_seq: &str, gm_seq: &str, prefix: char) -> String {
    let clean_agm = agm_seq.trim().trim_start_matches("AGM:").trim();
    let agm_formatted = if clean_agm.starts_with(prefix) {
        clean_agm.to_string()
    } else {
        format!("{}{:03}", prefix, clean_agm.parse::<u32>().unwrap_or(1))
    };

    let clean_gm = gm_seq.trim();
    let clean_gm = clean_gm.trim_start_matches("GM:").trim();
    let gm_formatted = if prefix == 'P' && !clean_gm.starts_with('#') {
        format!("GM:#{}", clean_gm)
    } else {
        format!("GM:{}", clean_gm)
    };

    format!("[AGM:{} | {}]", agm_formatted, gm_formatted)
}

pub(crate) fn format_elapsed_time(elapsed_seconds: i64) -> String {
    if elapsed_seconds < 60 {
        format!("{}s", elapsed_seconds.max(0))
    } else if elapsed_seconds < 3600 {
        format!("{}m {}s", elapsed_seconds / 60, elapsed_seconds % 60)
    } else {
        let hours = elapsed_seconds / 3600;
        let mins = (elapsed_seconds % 3600) / 60;
        format!("{}h {}m", hours, mins)
    }
}

pub(crate) fn count_prompt_images(prompt_content: &str, image_payload: Option<&str>) -> usize {
    let mut unique_images: HashSet<String> = HashSet::new();

    // 1. Scan prompt content for data URIs and file paths
    let (content_uris, content_paths) =
        repo_db::detection::extract_image_uris_or_paths(prompt_content);
    for u in content_uris {
        if !u.trim().is_empty() {
            unique_images.insert(u);
        }
    }
    for p in content_paths {
        if !p.trim().is_empty() {
            unique_images.insert(p);
        }
    }

    // 2. Scan image_payload if present
    if let Some(payload) = image_payload {
        let trimmed = payload.trim();
        if !trimmed.is_empty() {
            // Check if JSON array
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(trimmed) {
                    for item in arr {
                        if let Some(s) = item.as_str() {
                            let s_trim = s.trim();
                            if !s_trim.is_empty() {
                                unique_images.insert(s_trim.to_string());
                            }
                        }
                    }
                }
            }

            // Extract URIs and paths from payload text
            let (payload_uris, payload_paths) =
                repo_db::detection::extract_image_uris_or_paths(trimmed);
            for u in payload_uris {
                if !u.trim().is_empty() {
                    unique_images.insert(u);
                }
            }
            for p in payload_paths {
                if !p.trim().is_empty() {
                    unique_images.insert(p);
                }
            }

            // Semicolon-separated paths
            if trimmed.contains(';') {
                let img_exts = [".png", ".jpg", ".jpeg", ".webp", ".gif", ".svg", ".bmp"];
                for part in trimmed.split(';') {
                    let part_clean = part.trim();
                    let lower = part_clean.to_lowercase();
                    if img_exts.iter().any(|ext| lower.ends_with(ext))
                        || lower.starts_with("data:image/")
                    {
                        unique_images.insert(part_clean.to_string());
                    }
                }
            }
        }
    }

    unique_images.len()
}

pub(crate) fn cmd_running_inspection(args: &[String]) {
    cmd_running_prompts(args);
}

pub(crate) fn cmd_running_prompts(args: &[String]) {
    if let Some(first) = args.first() {
        let first_lower = first.to_lowercase();
        if first_lower == "backup" {
            crate::running_backup_cmds::cmd_backup_running_prompts(&args[1..]);
            return;
        }
        if first_lower == "restore" {
            crate::running_backup_cmds::cmd_restore_running_prompts(&args[1..]);
            return;
        }
        if first_lower == "export" {
            cmd_running_prompts_export(&args[1..]);
            return;
        }
        if first_lower == "import" {
            cmd_running_prompts_import(&args[1..]);
            return;
        }
    }

    let mut is_json = false;
    let mut is_full = false;
    let mut words_arg: Option<usize> = None;
    let mut limit_arg: Option<usize> = None;
    let mut instance_filter: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--help" || arg == "-h" || arg == "help" {
            print_running_prompts_help();
            return;
        }
        if arg == "--json" || arg == "-j" {
            is_json = true;
            i += 1;
            continue;
        }
        if arg == "--full" {
            is_full = true;
            i += 1;
            continue;
        }
        if (arg == "--limit" || arg == "-l") && i + 1 < args.len() {
            if let Ok(l) = args[i + 1].parse::<usize>() {
                limit_arg = Some(l.max(1));
            }
            i += 2;
            continue;
        }
        if (arg == "--words" || arg == "-w" || arg == "--wc" || arg == "--wordcount")
            && i + 1 < args.len()
        {
            if let Ok(w) = args[i + 1].parse::<usize>() {
                words_arg = Some(w.max(1));
            }
            i += 2;
            continue;
        }
        if arg.starts_with("--words=")
            || arg.starts_with("-w=")
            || arg.starts_with("--wc=")
            || arg.starts_with("--wordcount=")
        {
            if let Some((_, val)) = arg.split_once('=') {
                if let Ok(w) = val.parse::<usize>() {
                    words_arg = Some(w.max(1));
                }
            }
            i += 1;
            continue;
        }
        if (arg == "--instance" || arg == "-i" || arg == "--inst") && i + 1 < args.len() {
            instance_filter = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        if arg.starts_with("--instance=") || arg.starts_with("-i=") || arg.starts_with("--inst=") {
            if let Some((_, val)) = arg.split_once('=') {
                instance_filter = Some(val.to_string());
            }
            i += 1;
            continue;
        }
        if arg == "ls" || arg == "list" || arg == "running" || arg == "active" {
            i += 1;
            continue;
        }
        if !arg.starts_with('-') {
            if let Ok(w) = arg.parse::<usize>() {
                if words_arg.is_none() {
                    words_arg = Some(w);
                }
            } else if instance_filter.is_none() {
                instance_filter = Some(arg.clone());
            }
        }
        i += 1;
    }

    let max_words = match words_arg {
        Some(w) => w,
        None => 200,
    };

    let all_instances = instance::list_instances().unwrap_or_default();
    let candidate_instances = if all_instances.is_empty() {
        vec![instance::InstanceStatus {
            config: instance::InstanceConfig {
                id: "default".to_string(),
                name: "Default".to_string(),
                seq_num: Some(1),
                is_default: true,
                bound_email: None,
                bound_account_id: None,
                data_dir: "".to_string(),
                executable_path: None,
                extensions_dir: None,
                pid: None,
                created_at: 0,
                last_used: 0,
            },
            is_running: false,
            pid: None,
            memory_mb: None,
        }]
    } else {
        all_instances
    };

    let resolved_filter = instance_filter
        .as_ref()
        .map(|spec| instance::resolve_instance_id(spec).unwrap_or_else(|_| spec.clone()));

    let instances: Vec<_> = candidate_instances
        .into_iter()
        .filter(|inst| {
            if let Some(ref filt) = resolved_filter {
                inst.config.id.eq_ignore_ascii_case(filt)
                    || inst.config.name.eq_ignore_ascii_case(filt)
                    || format!("#{}", inst.config.seq_num.unwrap_or(0)) == *filt
                    || (filt == "default" && inst.config.is_default)
            } else {
                true
            }
        })
        .collect();

    let tree = repo_db::get_project_conversation_tree(max_words, false);
    let mut proj_seq_map: HashMap<String, (String, String)> = HashMap::new();
    let mut conv_seq_map: HashMap<String, (String, String)> = HashMap::new();

    for proj in &tree {
        let p_seq = proj.seq_code.clone();
        let p_gm = proj.gitmap_seq_code.clone();
        proj_seq_map.insert(proj.project_id.clone(), (p_seq.clone(), p_gm.clone()));
        if !proj.repo_path.is_empty() {
            let norm_path = proj.repo_path.replace('\\', "/").to_lowercase();
            proj_seq_map.insert(norm_path, (p_seq.clone(), p_gm.clone()));
        }

        for conv in &proj.conversations {
            let c_seq = conv.seq_code.clone();
            let c_gm = conv.gitmap_seq_code.clone();
            conv_seq_map.insert(conv.conversation_id.clone(), (c_seq.clone(), c_gm.clone()));
            if !conv.short_id.is_empty() {
                conv_seq_map.insert(conv.short_id.clone(), (c_seq.clone(), c_gm.clone()));
            }
        }
    }

    let all_db_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let now = Utc::now().timestamp();

    let mut inspected_instances: Vec<InspectedInstance> = Vec::new();
    let mut total_running_prompts = 0usize;

    for (inst_idx, inst) in instances.iter().enumerate() {
        let seq = inst.config.seq_num.unwrap_or((inst_idx + 1) as u32);
        let inst_sequence = format!("[AGM:P{:03} | GM:#{}]", seq, seq);
        let inst_number = format!("#{}", seq);
        let eff_email = inst
            .config
            .bound_email
            .clone()
            .or_else(|| {
                if inst.config.is_default || inst.config.id == "default" {
                    account::get_current_account()
                        .ok()
                        .flatten()
                        .map(|a| a.email)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "-".to_string());

        let detected_proc = instance::get_or_detect_instance_process(&inst.config.id);
        let active_pid = inst.pid.or_else(|| detected_proc.as_ref().map(|p| p.pid));
        let is_running = inst.is_running || detected_proc.is_some();
        let status_str = if is_running {
            if let Some(p) = active_pid {
                format!("RUNNING (PID: {})", p)
            } else {
                "RUNNING".to_string()
            }
        } else {
            "STOPPED".to_string()
        };

        let is_inst_default = inst.config.is_default || inst.config.id == "default";
        let mut inst_prompts: Vec<ActivePrompt> = all_db_prompts
            .iter()
            .filter(|p| {
                let p_inst = p.instance_id.trim();
                let matches_inst = p_inst == inst.config.id
                    || (is_inst_default
                        && (p_inst == "default" || p_inst == "__default__" || p_inst.is_empty()));
                let status_lower = p.status.to_lowercase();
                let is_active_or_queued = status_lower == "running"
                    || status_lower == "in_flight"
                    || status_lower == "dispatched"
                    || status_lower == "queued"
                    || status_lower == "backed_up"
                    || status_lower == "pending";
                matches_inst && is_active_or_queued
            })
            .cloned()
            .collect();

        // Also merge in live discovered prompts from Antigravity workspaceStorage
        let live_prompts = repo_db::discover_running_prompts_from_antigravity(&inst.config.id);
        for lp in live_prompts {
            if !inst_prompts.iter().any(|p| {
                p.id == lp.id
                    || (p.repo_path.to_lowercase().replace('\\', "/")
                        == lp.repo_path.to_lowercase().replace('\\', "/")
                        && p.prompt_content.trim() == lp.prompt_content.trim())
            }) {
                inst_prompts.push(lp);
            }
        }

        // Strict FIFO ordering (ORDER BY created_at ASC, id ASC)
        inst_prompts.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then_with(|| a.id.cmp(&b.id))
        });

        if let Some(lim) = limit_arg {
            inst_prompts.truncate(lim);
        }

        let mut prompt_items = Vec::new();
        for (p_idx, p) in inst_prompts.iter().enumerate() {
            let norm_path = p.repo_path.replace('\\', "/").to_lowercase();
            let (p_seq, p_gm) = proj_seq_map
                .get(&p.project_id)
                .or_else(|| proj_seq_map.get(&norm_path))
                .cloned()
                .unwrap_or_else(|| ("P001".to_string(), "#1".to_string()));
            let proj_dual_seq = format_dual_seq(&p_seq, &p_gm, 'P');

            let conv_dual_seq = if let Some(ref sid) = p.session_id {
                let (c_seq, c_gm) = conv_seq_map
                    .get(sid)
                    .or_else(|| {
                        if sid.len() >= 8 {
                            conv_seq_map.get(&sid[..8])
                        } else {
                            None
                        }
                    })
                    .cloned()
                    .or_else(|| {
                        repo_db::sequences::get_sequence_code_for_conversation(sid)
                            .map(|s| (s, sid.chars().take(8).collect()))
                    })
                    .unwrap_or_else(|| ("C001".to_string(), sid.chars().take(8).collect()));
                format_dual_seq(&c_seq, &c_gm, 'C')
            } else {
                "[AGM:C001 | GM:main]".to_string()
            };

            let img_count = count_prompt_images(&p.prompt_content, p.image_payload.as_deref());
            let clean = repo_db::text_utils::extract_clean_user_prompt(&p.prompt_content);
            let (preview, words) = if is_full {
                (clean.clone(), clean.split_whitespace().count())
            } else {
                repo_db::text_utils::truncate_words_bounded(&clean, max_words)
            };

            let elapsed_sec = (now - p.created_at).max(0);
            let elapsed_time = format_elapsed_time(elapsed_sec);
            let short_id: String = p.id.chars().take(8).collect();
            let workspace = if !p.repo_path.is_empty() {
                p.repo_path.clone()
            } else {
                p.project_id.clone()
            };

            prompt_items.push(InspectedPrompt {
                seq: p_idx + 1,
                id: p.id.clone(),
                short_id,
                project_id: p.project_id.clone(),
                workspace_path: workspace,
                project_dual_seq: proj_dual_seq,
                conversation_dual_seq: conv_dual_seq,
                conversation_id: p.session_id.clone().unwrap_or_else(|| p.id.clone()),
                status: p.status.to_uppercase(),
                elapsed_seconds: elapsed_sec,
                elapsed_time,
                word_count: words,
                image_count: img_count,
                has_images: img_count > 0,
                prompt_preview: preview,
                created_at: p.created_at,
                updated_at: p.updated_at,
            });
        }

        total_running_prompts += prompt_items.len();
        inspected_instances.push(InspectedInstance {
            seq_num: seq,
            instance_number: inst_number,
            id: inst.config.id.clone(),
            name: inst.config.name.clone(),
            sequence: inst_sequence,
            status: status_str,
            is_running,
            pid: active_pid,
            bound_email: eff_email,
            running_prompts_count: prompt_items.len(),
            prompts: prompt_items,
        });
    }

    if is_json {
        let envelope_data = serde_json::json!({
            "instances_count": inspected_instances.len(),
            "total_running_prompts": total_running_prompts,
            "word_limit": if is_full { None } else { Some(max_words) },
            "is_full": is_full,
            "instances": inspected_instances,
        });
        CliEnvelope::ok("running", resolved_filter, envelope_data).print_and_exit();
    }

    println!("\n================================================================================");
    println!("  AGM RUNNING PROMPTS & INSTANCES INSPECTION");
    println!("================================================================================");
    println!(
        "Instances: {} registered | Word Preview: {} | Mode: {}\n",
        inspected_instances.len(),
        if is_full {
            "Full".to_string()
        } else {
            format!("{} words", max_words)
        },
        if is_full { "Full Text" } else { "Truncated" }
    );

    println!(
        "=== Registered Instances ({}) ===",
        inspected_instances.len()
    );
    for inst in &inspected_instances {
        println!(
            "  • Instance {} {} (ID: {}) {} - Status: {}",
            inst.instance_number, inst.name, inst.id, inst.sequence, inst.status
        );
    }
    println!();

    println!("=== Running Prompts by Instance ===");
    for inst in &inspected_instances {
        println!(
            "\n[*] Instance {} {} (ID: {}) {} - Process Status: {}",
            inst.instance_number, inst.name, inst.id, inst.sequence, inst.status
        );
        if inst.prompts.is_empty() {
            println!("    Running Prompts: None (0 active)");
        } else {
            println!("    Running Prompts ({} active):", inst.prompts.len());
            for p in &inst.prompts {
                println!(
                    "      [{}] ID: {} | Status: {} | Elapsed: {} | Images: {} | Words: {}",
                    p.seq, p.short_id, p.status, p.elapsed_time, p.image_count, p.word_count
                );
                println!("          Workspace: {}", p.workspace_path);
                println!("          Prompt:    {}", p.prompt_preview);
            }
        }
    }
    println!();
}

pub(crate) fn cmd_running_prompts_export(args: &[String]) {
    let mut file_path = "agm-running-prompts.db".to_string();
    let mut word_limit: Option<usize> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if (arg == "-f" || arg == "--file" || arg == "-file") && i + 1 < args.len() {
            file_path = args[i + 1].clone();
            i += 2;
            continue;
        }
        if (arg == "--wc" || arg == "-w" || arg == "--wordcount") && i + 1 < args.len() {
            word_limit = args[i + 1].parse().ok();
            i += 2;
            continue;
        }
        if !arg.starts_with('-') && i == 0 {
            file_path = arg.clone();
        }
        i += 1;
    }

    if file_path.ends_with(".json") {
        let prompts = repo_db::list_all_prompts().unwrap_or_default();
        let mut export_items = Vec::new();
        for (idx, p) in prompts.iter().enumerate() {
            let prompt_text = if let Some(wl) = word_limit {
                crate::common::truncate_words(&p.prompt_content, wl).0
            } else {
                p.prompt_content.clone()
            };
            export_items.push(serde_json::json!({
                "sequence": idx + 1,
                "id": p.id,
                "project_id": p.project_id,
                "repo_path": p.repo_path,
                "prompt": prompt_text,
                "has_images": p.image_payload.is_some(),
                "images_payload": p.image_payload,
                "status": p.status,
            }));
        }
        let json_str = serde_json::to_string_pretty(&export_items).unwrap_or_default();
        if let Some(parent) = std::path::Path::new(&file_path).parent() {
            if !parent.as_os_str().is_empty() {
                antigravity_tools_lib::error::record_ignored(
                    fs::create_dir_all(parent),
                    "create parent directory for JSON export",
                ); // Justification: best effort parent dir creation before write
            }
        }
        if let Err(e) = fs::write(&file_path, json_str) {
            eprintln!("[ERROR] Failed to write JSON export: {}", e);
            std::process::exit(1);
        }
        println!(
            "[SUCCESS] Exported {} prompts to JSON file: {}",
            export_items.len(),
            file_path
        );
    } else {
        match backup_prompts_db::backup_active_running_prompts(Some("default"), Some(&file_path)) {
            Ok((batch, records)) => {
                println!(
                    "[SUCCESS] Exported {} prompts to SQLite DB: {}",
                    records.len(),
                    batch.file_path
                );
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to export prompts to SQLite DB: {}", e);
                std::process::exit(1);
            }
        }
    }
}

pub(crate) fn cmd_running_prompts_import(args: &[String]) {
    let mut file_path = "agm-running-prompts.db".to_string();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if (arg == "-f" || arg == "--file" || arg == "-file") && i + 1 < args.len() {
            file_path = args[i + 1].clone();
            i += 2;
            continue;
        }
        if !arg.starts_with('-') && i == 0 {
            file_path = arg.clone();
        }
        i += 1;
    }

    if file_path.ends_with(".json") {
        let content = match fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "[ERROR] Failed to read JSON import file '{}': {}",
                    file_path, e
                );
                std::process::exit(1);
            }
        };
        let items: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap_or_default();
        let now = Utc::now().timestamp();
        for item in &items {
            let id = item["id"]
                .as_str()
                .unwrap_or(&Uuid::new_v4().to_string())
                .to_string();
            let proj_id = item["project_id"]
                .as_str()
                .unwrap_or("imported")
                .to_string();
            let repo_path = item["repo_path"].as_str().unwrap_or("").to_string();
            let prompt_text = item["prompt"].as_str().unwrap_or("").to_string();
            let images_payload = item["images_payload"].as_str().map(|s| s.to_string());
            let active_p = ActivePrompt {
                id,
                project_id: proj_id,
                instance_id: "default".to_string(),
                repo_path,
                prompt_content: prompt_text,
                model: Some("gemini-3.8-flash-high".to_string()),
                session_id: None,
                status: "queued".to_string(),
                created_at: now,
                updated_at: now,
                image_payload: images_payload,
            };
            antigravity_tools_lib::error::record_ignored(
                repo_db::save_or_requeue_prompt(&active_p),
                "save imported prompt",
            ); // Justification: best effort import across item batch
        }
        println!(
            "[SUCCESS] Imported and enqueued {} prompt(s) from JSON: {}",
            items.len(),
            file_path
        );
    } else {
        match backup_prompts_db::restore_running_prompts(Some("default"), true, Some(&file_path)) {
            Ok(records) => {
                println!(
                    "[SUCCESS] Imported and enqueued {} prompt(s) from SQLite DB: {}",
                    records.len(),
                    file_path
                );
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to import from SQLite DB: {}", e);
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_elapsed_time() {
        assert_eq!(format_elapsed_time(0), "0s");
        assert_eq!(format_elapsed_time(42), "42s");
        assert_eq!(format_elapsed_time(60), "1m 0s");
        assert_eq!(format_elapsed_time(125), "2m 5s");
        assert_eq!(format_elapsed_time(3600), "1h 0m");
        assert_eq!(format_elapsed_time(3665), "1h 1m");
    }

    #[test]
    fn test_format_dual_seq() {
        assert_eq!(format_dual_seq("P001", "#1", 'P'), "[AGM:P001 | GM:#1]");
        assert_eq!(format_dual_seq("C001", "c001", 'C'), "[AGM:C001 | GM:c001]");
        assert_eq!(format_dual_seq("1", "1", 'P'), "[AGM:P001 | GM:#1]");
    }

    #[test]
    fn test_count_prompt_images() {
        assert_eq!(
            count_prompt_images("plain text prompt without any images", None),
            0
        );
        assert_eq!(
            count_prompt_images("check this file:///path/to/screenshot.png please", None),
            1
        );
        assert_eq!(
            count_prompt_images(
                "look at data:image/png;base64,iVBORw0KGgoAAAANSUhEUg here",
                None
            ),
            1
        );
        let payload =
            serde_json::json!(["data:image/png;base64,1", "data:image/jpeg;base64,2"]).to_string();
        assert_eq!(count_prompt_images("two images", Some(&payload)), 2);
    }
}
