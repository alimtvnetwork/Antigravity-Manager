//! Repo DB: tree format

use super::agy::spawn_prompt_via_agy;
use super::models::{ActivePrompt, AgmConversationNode, AgmProjectTreeNode, AgmSequenceResolution};
use super::prompts_crud::save_or_requeue_prompt;
use super::schema::connect_db;
use super::text_utils::extract_smart_prompt_summary;
use super::tree::{get_project_conversation_tree, get_project_conversation_tree_cached};
use chrono::Utc;
use rusqlite::params;
use std::collections::HashSet;
use std::process::{Command, Stdio};
use uuid::Uuid;

/// Compact grouping of conversation nodes with identical prompts into a single parent node with repeat badge
pub fn group_identical_conversation_runs(
    convs: Vec<AgmConversationNode>,
) -> Vec<AgmConversationNode> {
    if convs.is_empty() {
        return Vec::new();
    }
    let mut grouped: Vec<AgmConversationNode> = Vec::new();
    let mut seen_indices: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for i in 0..convs.len() {
        if seen_indices.contains(&i) {
            continue;
        }
        let current = &convs[i];
        let p_key = current.prompt_preview_200w.trim();
        let t_key = current.title.trim();

        let mut matches = vec![current.clone()];
        seen_indices.insert(i);

        for j in (i + 1)..convs.len() {
            if seen_indices.contains(&j) {
                continue;
            }
            let next = &convs[j];
            let next_p = next.prompt_preview_200w.trim();
            let next_t = next.title.trim();

            let is_match = (!p_key.is_empty() && p_key == next_p)
                || (!t_key.is_empty() && t_key != "Untitled Conversation" && t_key == next_t);

            if is_match {
                matches.push(next.clone());
                seen_indices.insert(j);
            }
        }

        if matches.len() > 1 {
            let count = matches.len();
            let mut parent = matches[0].clone();
            let any_running = matches.iter().any(|m| m.is_running);
            let any_queued = matches.iter().any(|m| m.is_queued);
            if any_running {
                parent.is_running = true;
                parent.status = "RUNNING".to_string();
            } else if any_queued {
                parent.is_queued = true;
                parent.status = "QUEUED".to_string();
            }
            parent.byte_size = Some(parent.prompt_preview_200w.len());
            parent.repeat_count = Some(count);
            parent.repeat_badge = Some(format!("x{}", count));
            parent.sub_runs = matches;
            grouped.push(parent);
        } else {
            let mut single = current.clone();
            single.byte_size = Some(single.prompt_preview_200w.len());
            single.repeat_count = None;
            single.repeat_badge = None;
            single.sub_runs = Vec::new();
            grouped.push(single);
        }
    }
    grouped
}

/// Helper method for prompt tree retrieval scoped to instance with optional repo filter
pub fn get_project_conversation_tree_for_instance(
    instance_id: &str,
    repo_filter: Option<&str>,
) -> Result<Vec<AgmProjectTreeNode>, String> {
    let mut tree = get_project_conversation_tree_cached(Some(instance_id), 200, false, true);
    if let Some(filter) = repo_filter {
        let f_clean = filter.trim().to_lowercase().replace('\\', "/");
        tree.retain(|p| {
            let p_repo = p.repo_path.to_lowercase().replace('\\', "/");
            let p_name = p.repo_name.to_lowercase();
            p_repo == f_clean
                || p_repo.contains(&f_clean)
                || p_name == f_clean
                || p_name.contains(&f_clean)
        });
    }
    Ok(tree)
}

/// Helper method to list actively running or in-flight prompts for an instance
pub fn list_running_prompts_for_instance(instance_id: &str) -> Result<Vec<ActivePrompt>, String> {
    let conn = connect_db()?;
    let norm = if instance_id == "default" || instance_id == "__default__" {
        "default".to_string()
    } else {
        crate::modules::instance::resolve_instance_id(instance_id)
            .unwrap_or_else(|_| instance_id.to_string())
    };
    let mut stmt = conn.prepare(
        "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload
         FROM active_prompts
         WHERE (instance_id = ?1 OR (instance_id = '__default__' AND ?1 = 'default'))
           AND status IN ('running', 'in_flight', 'dispatched')
         ORDER BY updated_at DESC"
    ).map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![&norm], |row| {
            Ok(ActivePrompt {
                id: row.get(0)?,
                project_id: row.get(1)?,
                instance_id: row.get(2)?,
                repo_path: row.get(3)?,
                prompt_content: row.get(4)?,
                model: row.get(5)?,
                session_id: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                image_payload: row.get(10).ok(),
            })
        })
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();
    Ok(rows)
}

/// Resolve an AGM or GitMap Sequence ID (`P001`, `AGM:P001`, `GM:#1`, `C001`, `AGM:C001`, `GM:<cid>`, or conversation UUID prefix)
/// to its target project, instance, and optional conversation.
pub fn resolve_agm_sequence_target(target_token: &str) -> Option<AgmSequenceResolution> {
    let tree = get_project_conversation_tree(200, false);
    let mut clean = target_token
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim();
    if let Some(stripped) = clean
        .strip_prefix("AGM:")
        .or_else(|| clean.strip_prefix("agm:"))
        .or_else(|| clean.strip_prefix("GM:"))
        .or_else(|| clean.strip_prefix("gm:"))
    {
        clean = stripped.trim();
    }
    let clean = clean.trim_start_matches('#');
    if clean.is_empty() {
        return None;
    }

    let upper = clean.to_uppercase();

    if let Some(num_str) = upper.strip_prefix('P') {
        if let Ok(seq_num) = num_str.parse::<i64>() {
            if let Some(proj) = tree.iter().find(|p| p.seq_id == seq_num) {
                let first_conv = proj.conversations.first();
                return Some(AgmSequenceResolution {
                    seq_code: proj.seq_code.clone(),
                    gitmap_seq_code: proj.gitmap_seq_code.clone(),
                    project_id: proj.project_id.clone(),
                    repo_name: proj.repo_name.clone(),
                    repo_path: proj.repo_path.clone(),
                    instance_id: proj.instance_id.clone(),
                    conversation_id: first_conv.map(|c| c.conversation_id.clone()),
                    conversation_title: first_conv.map(|c| c.title.clone()),
                });
            }
        }
    }

    if let Some(num_str) = upper.strip_prefix('C') {
        if let Ok(seq_num) = num_str.parse::<i64>() {
            for proj in &tree {
                if let Some(conv) = proj.conversations.iter().find(|c| c.seq_id == seq_num) {
                    return Some(AgmSequenceResolution {
                        seq_code: conv.seq_code.clone(),
                        gitmap_seq_code: conv.gitmap_seq_code.clone(),
                        project_id: proj.project_id.clone(),
                        repo_name: proj.repo_name.clone(),
                        repo_path: proj.repo_path.clone(),
                        instance_id: conv.instance_id.clone(),
                        conversation_id: Some(conv.conversation_id.clone()),
                        conversation_title: Some(conv.title.clone()),
                    });
                }
            }
        }
    }

    if let Ok(seq_num) = clean.parse::<i64>() {
        if let Some(proj) = tree.iter().find(|p| p.seq_id == seq_num) {
            let first_conv = proj.conversations.first();
            return Some(AgmSequenceResolution {
                seq_code: proj.seq_code.clone(),
                gitmap_seq_code: proj.gitmap_seq_code.clone(),
                project_id: proj.project_id.clone(),
                repo_name: proj.repo_name.clone(),
                repo_path: proj.repo_path.clone(),
                instance_id: proj.instance_id.clone(),
                conversation_id: first_conv.map(|c| c.conversation_id.clone()),
                conversation_title: first_conv.map(|c| c.title.clone()),
            });
        }
        for proj in &tree {
            if let Some(conv) = proj.conversations.iter().find(|c| c.seq_id == seq_num) {
                return Some(AgmSequenceResolution {
                    seq_code: conv.seq_code.clone(),
                    gitmap_seq_code: conv.gitmap_seq_code.clone(),
                    project_id: proj.project_id.clone(),
                    repo_name: proj.repo_name.clone(),
                    repo_path: proj.repo_path.clone(),
                    instance_id: conv.instance_id.clone(),
                    conversation_id: Some(conv.conversation_id.clone()),
                    conversation_title: Some(conv.title.clone()),
                });
            }
        }
    }

    let lower = clean.to_lowercase();
    if lower.len() >= 4 {
        for proj in &tree {
            if let Some(conv) = proj
                .conversations
                .iter()
                .find(|c| c.conversation_id.to_lowercase().starts_with(&lower))
            {
                return Some(AgmSequenceResolution {
                    seq_code: conv.seq_code.clone(),
                    gitmap_seq_code: conv.gitmap_seq_code.clone(),
                    project_id: proj.project_id.clone(),
                    repo_name: proj.repo_name.clone(),
                    repo_path: proj.repo_path.clone(),
                    instance_id: conv.instance_id.clone(),
                    conversation_id: Some(conv.conversation_id.clone()),
                    conversation_title: Some(conv.title.clone()),
                });
            }
        }
        if let Some(proj) = tree.iter().find(|p| {
            p.project_id.to_lowercase().contains(&lower)
                || p.repo_name.to_lowercase().contains(&lower)
        }) {
            let first_conv = proj.conversations.first();
            return Some(AgmSequenceResolution {
                seq_code: proj.seq_code.clone(),
                gitmap_seq_code: proj.gitmap_seq_code.clone(),
                project_id: proj.project_id.clone(),
                repo_name: proj.repo_name.clone(),
                repo_path: proj.repo_path.clone(),
                instance_id: proj.instance_id.clone(),
                conversation_id: first_conv.map(|c| c.conversation_id.clone()),
                conversation_title: first_conv.map(|c| c.title.clone()),
            });
        }
    }

    None
}

/// Inject a prompt into a specific project or conversation by AGM/GitMap Sequence ID (`C001`, `P001`, `AGM:C001`, `GM:#1`),
/// with optional `--instance <id|#seq|name>` and remote machine `--node <node>` scoping.
pub fn prompt_target_by_sequence_scoped(
    target_token: &str,
    prompt_text: &str,
    instance_override: Option<&str>,
    node_override: Option<&str>,
) -> Result<String, String> {
    let clean_prompt = prompt_text.trim();
    if clean_prompt.is_empty() {
        return Err("Prompt text cannot be empty.".to_string());
    }

    // Remote SSH Node Delegation if `node_override` is provided and not "local"
    if let Some(node) = node_override
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && *s != "local")
    {
        let inst_flag = instance_override
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|i| format!(" --instance {}", i))
            .unwrap_or_default();
        let escaped_prompt = clean_prompt.replace('"', "\\\"");
        let remote_cmd = format!(
            "agm prompt {}{} \"{}\"",
            target_token.trim(),
            inst_flag,
            escaped_prompt
        );
        let output = std::process::Command::new("gitmap")
            .args(["ssh", "exec", &remote_cmd, "--node", node])
            .output()
            .or_else(|_| {
                std::process::Command::new("gitmap")
                    .args(["ssh", "exec", &remote_cmd])
                    .output()
            })
            .map_err(|e| format!("Failed to delegate prompt to SSH node '{}': {}", node, e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if output.status.success() {
            return Ok(format!(
                "🌐 Delegated prompt to remote node '{}' for target [{}]:\n{}",
                node,
                target_token.trim(),
                if stdout.is_empty() { "OK" } else { &stdout }
            ));
        } else {
            return Err(format!(
                "Remote SSH node '{}' returned error: {} {}",
                node, stdout, stderr
            ));
        }
    }

    let resolved = resolve_agm_sequence_target(target_token).ok_or_else(|| {
        format!(
            "Target '{}' not found in AGM/GitMap Tree. Run `agm tree` or `/tree` to view valid [AGM:P001 | GM:#1] and [AGM:C001 | GM:<cid>] sequence codes.",
            target_token
        )
    })?;

    // Resolve optional instance override (#1, #2, instance name, or UUID)
    let effective_instance_id = if let Some(inst_raw) = instance_override
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        if inst_raw.eq_ignore_ascii_case("default") || inst_raw == "#1" || inst_raw == "1" {
            "default".to_string()
        } else if let Ok(reg) = crate::modules::instance::load_registry() {
            let clean_seq = inst_raw.trim_start_matches('#').parse::<u32>().ok();
            if let Some(found) = reg.instances.iter().find(|i| {
                i.id.eq_ignore_ascii_case(inst_raw)
                    || i.name.eq_ignore_ascii_case(inst_raw)
                    || (clean_seq.is_some() && i.seq_num == clean_seq)
            }) {
                found.id.clone()
            } else {
                inst_raw.to_string()
            }
        } else {
            inst_raw.to_string()
        }
    } else {
        resolved.instance_id.clone()
    };

    let now = Utc::now().timestamp();
    let prompt_id = Uuid::new_v4().to_string();
    let active_prompt = ActivePrompt {
        id: prompt_id.clone(),
        project_id: resolved.project_id.clone(),
        instance_id: effective_instance_id.clone(),
        repo_path: resolved.repo_path.clone(),
        prompt_content: clean_prompt.to_string(),
        model: Some("gemini-pro".to_string()),
        session_id: resolved
            .conversation_id
            .clone()
            .or_else(|| Some(resolved.project_id.clone())),
        status: "running".to_string(),
        created_at: now,
        updated_at: now,
        image_payload: None,
    };

    // Justification: prompt tracking save is auxiliary; the agy spawn result below drives the return
    crate::error::record_ignored(
        save_or_requeue_prompt(&active_prompt),
        "save prompt before agy spawn",
    );
    let spawned = spawn_prompt_via_agy(&active_prompt);

    let conv_label = resolved
        .conversation_id
        .as_deref()
        .map(|cid| {
            let short = if cid.len() >= 8 { &cid[..8] } else { cid };
            format!("conv:{}", short)
        })
        .unwrap_or_else(|| "new/latest conv".to_string());

    Ok(format!(
        "✅ Dispatched prompt to [AGM:{} | {}] ({}) on instance '{}' (spawned={}): \"{}\"",
        resolved.seq_code,
        resolved.gitmap_seq_code,
        conv_label,
        effective_instance_id,
        spawned,
        extract_smart_prompt_summary(clean_prompt, 80)
    ))
}
