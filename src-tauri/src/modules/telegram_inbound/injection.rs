use crate::modules::email_watcher;
use crate::modules::repo_db;
use crate::modules::supabase_client::SupabaseClient;
use crate::modules::supabase_sync;
use chrono::{Local, Utc};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;

use super::types::{PromptExecutionStatusType, RemoteDispatchStatusType};
use super::*;

/// Execute prompt injection to local workspace or remote cluster node (supports Dual AGM/GitMap Seq IDs P001/C001, --instance, and --node)
pub async fn execute_prompt_injection(args_str: &str) -> String {
    let trimmed = args_str.trim();
    if trimmed.is_empty() {
        return "⚠️ <b>Missing Arguments:</b>\nUsage:\n• <code>/prompt C001 &lt;prompt text&gt;</code>\n• <code>/prompt P001 --instance #2 &lt;prompt text&gt;</code>\n• <code>/prompt C001 --instance default --node worker-1 &lt;prompt text&gt;</code>".to_string();
    }

    // Extract optional `--instance <id>` / `ins:<id>`, `--node <node>` / `node:<node>`, and `--base64 <b64>` anywhere in the command
    let mut explicit_instance: Option<String> = None;
    let mut explicit_node: Option<String> = None;
    let mut explicit_base64: Option<String> = None;
    let mut filtered_tokens: Vec<&str> = Vec::new();
    let raw_tokens: Vec<&str> = trimmed.split_whitespace().collect();
    let mut idx = 0;
    while idx < raw_tokens.len() {
        let tok = raw_tokens[idx];
        if tok.eq_ignore_ascii_case("--instance") || tok.eq_ignore_ascii_case("-i") {
            if idx + 1 < raw_tokens.len() {
                let raw_inst = raw_tokens[idx + 1];
                explicit_instance = Some(
                    crate::modules::instance::resolve_instance_id(raw_inst)
                        .unwrap_or_else(|_| raw_inst.to_string()),
                );
                idx += 2;
                continue;
            }
        } else if tok.eq_ignore_ascii_case("--node") {
            if idx + 1 < raw_tokens.len() {
                explicit_node = Some(raw_tokens[idx + 1].to_string());
                idx += 2;
                continue;
            }
        } else if tok.eq_ignore_ascii_case("--base64") || tok.eq_ignore_ascii_case("-b64") {
            if idx + 1 < raw_tokens.len() {
                explicit_base64 = Some(raw_tokens[idx + 1].to_string());
                idx += 2;
                continue;
            }
        } else if let Some(rest_ins) = tok
            .strip_prefix("instance:")
            .or_else(|| tok.strip_prefix("ins:"))
        {
            if !rest_ins.is_empty() {
                explicit_instance = Some(
                    crate::modules::instance::resolve_instance_id(rest_ins)
                        .unwrap_or_else(|_| rest_ins.to_string()),
                );
                idx += 1;
                continue;
            }
        } else if let Some(rest_node) = tok.strip_prefix("node:") {
            if !rest_node.is_empty() {
                explicit_node = Some(rest_node.to_string());
                idx += 1;
                continue;
            }
        }
        filtered_tokens.push(tok);
        idx += 1;
    }

    if filtered_tokens.is_empty() {
        return "Please include both a target and your prompt, e.g. /prompt default Write unit tests".to_string();
    }

    // If `--node` or `--instance` was explicitly provided and the first token is NOT a C001/P001 sequence ID
    // or if only 1 token is left when both `--node` and `--instance` were provided, allow direct prompt text!
    let first_tok = filtered_tokens[0];
    let is_seq_like = first_tok.to_uppercase().starts_with("C0")
        || first_tok.to_uppercase().starts_with("P0")
        || first_tok.to_uppercase().starts_with("GM:")
        || first_tok.to_uppercase().starts_with("AGM:");

    // Support `<machine>:<instance>` colon syntax in first token (e.g. `/prompt W3:default Fix build`)
    if !is_seq_like && first_tok.contains(':') && filtered_tokens.len() >= 2 {
        if let Some((m_part, i_part)) = first_tok.split_once(':') {
            if !m_part.trim().is_empty() && !i_part.trim().is_empty() {
                explicit_node = Some(m_part.trim().to_string());
                explicit_instance = Some(
                    crate::modules::instance::resolve_instance_id(i_part.trim())
                        .unwrap_or_else(|_| i_part.trim().to_string()),
                );
                filtered_tokens[0] = "default";
            }
        }
    } else if !is_seq_like && explicit_instance.is_none() && filtered_tokens.len() >= 2 {
        // Support `/prompt default Write unit tests` where first token is an instance name/id
        if first_tok.eq_ignore_ascii_case("default")
            || crate::modules::instance::resolve_instance_id(first_tok).is_ok()
        {
            let resolved_id = crate::modules::instance::resolve_instance_id(first_tok)
                .unwrap_or_else(|_| first_tok.to_string());
            explicit_instance = Some(resolved_id);
            filtered_tokens[0] = "default";
        }
    } else if !is_seq_like && (explicit_node.is_some() || explicit_instance.is_some()) {
        let is_existing_dir = std::path::Path::new(first_tok).is_dir();
        let is_known_project = repo_db::get_live_project_execution_info().iter().any(|p| {
            p.project_id.eq_ignore_ascii_case(first_tok)
                || p.repo_name.eq_ignore_ascii_case(first_tok)
        });
        if !is_existing_dir && !is_known_project && filtered_tokens.len() < 2 {
            // Support `/prompt --node vm-1 --instance work Audit code` without requiring a dummy project token
            filtered_tokens.insert(0, "default");
        }
    }

    if filtered_tokens.len() < 2 {
        return "Please include both a target and your prompt, e.g. /prompt default Write unit tests".to_string();
    }

    let local_config = supabase_sync::load_config().unwrap_or_default();
    let gitmap_nodes = query_gitmap_cluster_nodes();
    let ssh_nodes = crate::modules::ssh_manager::load_ssh_connections().unwrap_or_default();

    let is_first_token_remote_node = explicit_node.is_none()
        && (gitmap_nodes.iter().any(|n| {
            n.alias.eq_ignore_ascii_case(filtered_tokens[0])
                && !super::node_label::is_current_machine(&n.alias)
        }) || ssh_nodes.iter().any(|n| {
            n.alias.eq_ignore_ascii_case(filtered_tokens[0])
                && !super::node_label::is_current_machine(&n.alias)
        }));

    let (target_node, target_project, mut prompt_text) = if let Some(exp_node) = explicit_node {
        let proj = filtered_tokens[0];
        let text = filtered_tokens[1..].join(" ");
        (exp_node, proj.to_string(), text)
    } else if is_first_token_remote_node && filtered_tokens.len() >= 3 {
        let node = filtered_tokens[0];
        let proj = filtered_tokens[1];
        let text = filtered_tokens[2..].join(" ");
        (node.to_string(), proj.to_string(), text)
    } else {
        let first = filtered_tokens[0];
        if super::node_label::is_current_machine(first) {
            if filtered_tokens.len() >= 3 {
                let proj = filtered_tokens[1];
                (
                    "local".to_string(),
                    proj.to_string(),
                    filtered_tokens[2..].join(" "),
                )
            } else {
                (
                    "local".to_string(),
                    "default".to_string(),
                    filtered_tokens[1..].join(" "),
                )
            }
        } else {
            let proj = filtered_tokens[0];
            let text = filtered_tokens[1..].join(" ");
            ("local".to_string(), proj.to_string(), text)
        }
    };

    if let Some(b64) = explicit_base64 {
        use base64::Engine;
        let clean_b64 = b64.trim().trim_matches('"').trim_matches('\'');
        if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(clean_b64.as_bytes()) {
            if let Ok(decoded) = String::from_utf8(bytes) {
                prompt_text = decoded;
            }
        }
    }

    if prompt_text.trim().is_empty() {
        return "Your prompt text is empty — nothing was sent.".to_string();
    }

    if !super::node_label::is_current_machine(&target_node) {
        return execute_remote_prompt_injection(
            &target_node,
            &target_project,
            &prompt_text,
            explicit_instance.as_deref(),
            &local_config,
        )
        .await;
    }

    // 1. First attempt resolution via Dual AGM/GitMap Sequence ID (`P001`, `GM:#1`, `C001`, `GM:<cid>`, or conv UUID prefix)
    let seq_resolved = repo_db::resolve_agm_sequence_target(&target_project);

    let projects = repo_db::get_live_project_execution_info();
    let matched_proj = projects.iter().find(|p| {
        p.project_id.eq_ignore_ascii_case(&target_project)
            || p.repo_name.eq_ignore_ascii_case(&target_project)
            || p.project_id.starts_with(&target_project)
            || target_project.starts_with(&p.project_id)
    });

    let (final_proj_id, repo_path, resolved_instance, resolved_conv_id, _seq_badge) =
        if let Some(seq) = seq_resolved {
            (
                seq.project_id,
                seq.repo_path,
                explicit_instance.clone().unwrap_or(seq.instance_id),
                seq.conversation_id,
                Some(format!("AGM:{} | {}", seq.seq_code, seq.gitmap_seq_code)),
            )
        } else if let Some(p) = matched_proj {
            (
                p.project_id.clone(),
                p.repo_path.clone(),
                explicit_instance
                    .clone()
                    .unwrap_or_else(|| "default".to_string()),
                None,
                None,
            )
        } else {
            let p_buf = PathBuf::from(&target_project);
            let (pid, rpath) = if p_buf.exists() && p_buf.is_dir() {
                (target_project.clone(), target_project.clone())
            } else if let Some(first_p) = projects.first() {
                (first_p.project_id.clone(), first_p.repo_path.clone())
            } else {
                (
                    "local-project".to_string(),
                    std::env::current_dir()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| ".".to_string()),
                )
            };
            (
                pid,
                rpath,
                explicit_instance
                    .clone()
                    .unwrap_or_else(|| "default".to_string()),
                None,
                None,
            )
        };

    let prompt_id = format!("p-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let resolved_prompt = wrap_telegram_prompt(&prompt_text);
    let (extracted_img, _img_paths) = repo_db::extract_image_payload_or_path(&prompt_text);
    let active_prompt = repo_db::ActivePrompt {
        id: prompt_id.clone(),
        project_id: final_proj_id.clone(),
        instance_id: resolved_instance.clone(),
        repo_path: repo_path.clone(),
        prompt_content: resolved_prompt.clone(),
        model: None,
        session_id: resolved_conv_id.clone(),
        status: "running".to_string(),
        created_at: Utc::now().timestamp(),
        updated_at: Utc::now().timestamp(),
        image_payload: extracted_img,
    };

    if let Err(e) = repo_db::save_or_requeue_prompt(&active_prompt) {
        return format!(
            "Couldn't save your prompt: {}",
            clean_for_telegram_html(&e, 200)
        );
    }

    crate::modules::scheduler::notify_prompt_queue();
    let dispatch_inst = resolved_instance.clone();
    tauri::async_runtime::spawn_blocking(move || {
        crate::error::record_ignored(
            crate::modules::repo_db::check_and_dispatch_enqueued_prompts(Some(&dispatch_inst)),
            "telegram inbound immediate dispatch probe",
        );
        // Justification: Best-effort immediate dispatch probe; background scheduler will retry if probe fails.
    });

    let spawned = match repo_db::spawn_prompt_via_agy(&active_prompt) {
        Ok(()) => true,
        Err(spawn_err) => {
            crate::modules::logger::log_warn(&format!(
                "[Telegram] agy spawn failed for inbound prompt: {spawn_err}"
            ));
            false
        }
    };
    let exec_status = PromptExecutionStatusType::from_spawned(spawned);
    let ide_line = ide_exe_line_for(&resolved_instance).unwrap_or_default();
    let id8 = prompt_id.strip_prefix("p-").unwrap_or(&prompt_id);
    let now_utc = Utc::now();
    let utc_str = now_utc.to_rfc3339();
    let local_str = Local::now().format("%Y-%m-%d %I:%M:%S %p").to_string();
    let (_, detected_images) = repo_db::extract_image_payload_or_path(&prompt_text);
    let media_count = detected_images
        .len()
        .max(if active_prompt.image_payload.is_some() {
            1
        } else {
            0
        });

    format!(
        "✅ <b>Prompt Dispatched & Acknowledged</b>\n\
        • <b>Dispatched At:</b> <code>{}</code> ({})\n\
        • <b>Target Node:</b> <code>local</code>\n\
        • <b>Instance:</b> <code>{}</code> · <code>{}</code>\n\
        • <b>Status:</b> {}\n\
        • <b>Reference:</b> <code>{}</code>\n\
        • <b>Attached Media:</b> {} image(s)\n\
        • <b>Quick Actions:</b> <code>/active</code> · <code>/tree</code> · <code>/backup</code> · <code>/restore</code>\n\
        {}\
        <b>Your prompt:</b>\n<pre>{}</pre>",
        utc_str,
        local_str,
        clean_for_telegram_html(&resolved_instance, 32),
        clean_for_telegram_html(&final_proj_id, 40),
        exec_status,
        clean_for_telegram_html(id8, 16),
        media_count,
        ide_line,
        clean_for_telegram_html(&repo_db::strip_internal_id_wrappers(&prompt_text), 800)
    )
}

/// Dispatch prompt injection to a remote cluster node via GitMap cluster exec, SSH, or Supabase queue
pub async fn execute_remote_prompt_injection(
    target_node: &str,
    target_project: &str,
    prompt_text: &str,
    explicit_instance: Option<&str>,
    local_config: &supabase_sync::SupabaseConfig,
) -> String {
    use base64::Engine;
    let b64_prompt = base64::engine::general_purpose::STANDARD.encode(prompt_text.as_bytes());
    let inst_flag = explicit_instance
        .map(|i| format!(" --instance {}", i.trim()))
        .unwrap_or_default();
    let target_proj = if target_project.trim().is_empty() {
        "default"
    } else {
        target_project.trim()
    };
    let is_windows = {
        let ssh_nodes = crate::modules::ssh_manager::load_ssh_connections().unwrap_or_default();
        ssh_nodes
            .iter()
            .find(|n| {
                n.alias.eq_ignore_ascii_case(target_node)
                    || n.ip_address.eq_ignore_ascii_case(target_node)
            })
            .map(|n| {
                n.os.to_lowercase().contains("win")
                    || n.username.eq_ignore_ascii_case("administrator")
                    || n.alias.to_lowercase().starts_with('w')
            })
            .unwrap_or_else(|| {
                target_node.to_lowercase().starts_with('w')
                    || target_node.to_lowercase().contains("win")
            })
    };
    let agm_bin = if is_windows { "agm.exe" } else { "agm" };
    let remote_cmd = format!("{agm_bin} prompt {target_proj}{inst_flag} --base64 \"{b64_prompt}\"");

    let mut dispatched_gitmap = false;

    // 1. First attempt: gitmap cluster exec <target_node> <remote_cmd>
    let mut gm_cmd1 = Command::new("gitmap");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        gm_cmd1.creation_flags(0x08000000);
    }
    if let Ok(out) = gm_cmd1
        .args(["cluster", "exec", target_node, &remote_cmd])
        .output()
    {
        if out.status.success() {
            dispatched_gitmap = true;
        }
    }

    // 2. Second attempt: gitmap ssh exec <target_node> <remote_cmd> / exec_via_gitmap_ssh
    if !dispatched_gitmap {
        let mut gm_cmd2 = Command::new("gitmap");
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            gm_cmd2.creation_flags(0x08000000);
        }
        if let Ok(out) = gm_cmd2
            .args(["ssh", "exec", target_node, &remote_cmd])
            .output()
        {
            if out.status.success() {
                dispatched_gitmap = true;
            }
        }
        if !dispatched_gitmap {
            if let Ok((code, out_str, _)) =
                crate::modules::ssh_manager::exec_via_gitmap_ssh(target_node, &remote_cmd)
            {
                let is_gm_success = code == 0
                    && !out_str.contains("machine is off")
                    && !out_str.to_lowercase().contains("connection timed out");
                if is_gm_success {
                    dispatched_gitmap = true;
                }
            }
        }
    }

    // 3. Third attempt: native SSH exec command
    if !dispatched_gitmap {
        if let Ok(results) = crate::modules::ssh_manager::exec_ssh_command(
            target_node,
            std::slice::from_ref(&remote_cmd),
            None,
            None,
            None,
        ) {
            if results.iter().any(|r| r.success) {
                dispatched_gitmap = true;
            }
        }
    }

    let mut enqueued_supabase = false;
    let cmd_id = uuid::Uuid::new_v4().to_string();
    for ep in &local_config.endpoints {
        if ep.is_enabled && ep.role == "secondary" {
            if let Ok(c) = SupabaseClient::new(ep) {
                let payload = json!({
                    "id": cmd_id,
                    "target_node": target_node,
                    "target_node_id": target_node,
                    "source": "telegram",
                    "project_id": target_project,
                    "instance_id": explicit_instance.unwrap_or("default"),
                    "command": "prompt",
                    "command_text": remote_cmd,
                    "prompt": prompt_text,
                    "status": "pending",
                    "created_at": Utc::now().timestamp()
                });
                if c.insert("command_queue", payload).await.is_ok() {
                    enqueued_supabase = true;
                    break;
                }
            }
        }
    }

    let dispatch_status = RemoteDispatchStatusType::resolve(dispatched_gitmap, enqueued_supabase);

    let ide_line = ide_exe_line_for(explicit_instance.unwrap_or("default")).unwrap_or_default();
    let id8 = cmd_id.get(..8).unwrap_or(&cmd_id);
    let now_utc = Utc::now();
    let utc_str = now_utc.to_rfc3339();
    let local_str = Local::now().format("%Y-%m-%d %I:%M:%S %p").to_string();
    let (_, detected_images) = repo_db::extract_image_payload_or_path(prompt_text);
    let media_count = detected_images.len();

    format!(
        "✅ <b>Prompt Dispatched & Acknowledged</b>\n\
        • <b>Dispatched At:</b> <code>{}</code> ({})\n\
        • <b>Target Node:</b> <code>{}</code>\n\
        • <b>Instance:</b> <code>{}</code> ({})\n\
        • <b>Status:</b> {}\n\
        • <b>Reference:</b> <code>{}</code>\n\
        • <b>Attached Media:</b> {} image(s)\n\
        • <b>Quick Actions:</b> <code>/active</code> · <code>/tree</code> · <code>/backup</code> · <code>/restore</code>\n\
        {}\
        <b>Your prompt:</b>\n<pre>{}</pre>",
        utc_str,
        local_str,
        clean_for_telegram_html(target_node, 40),
        clean_for_telegram_html(target_project, 40),
        clean_for_telegram_html(explicit_instance.unwrap_or("default"), 32),
        dispatch_status,
        clean_for_telegram_html(id8, 16),
        media_count,
        ide_line,
        clean_for_telegram_html(prompt_text, 800)
    )
}
