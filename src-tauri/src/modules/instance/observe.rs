//! Instance state observation and id resolution.
use super::*;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObservedInstanceState {
    pub instance_id: String,
    pub name: String,
    pub data_dir: String,
    pub is_running: bool,
    pub pids: Vec<u32>,
    pub bound_account_id: Option<String>,
    pub bound_account_email: Option<String>,
    pub injected_email_in_db: Option<String>,
    pub workspace_folders: Vec<String>,
    pub active_prompts_count: usize,
    pub prompt_goal_running: bool,
    pub last_heartbeat_timestamp: Option<String>,
    pub last_heartbeat_line: Option<String>,
}

/// Observe an instance's complete operational state: running status, conscious PIDs, bound vs injected credentials, active prompt queue, and prompt goal heartbeats
pub fn observe_instance(instance_id: &str) -> Result<ObservedInstanceState, String> {
    let registry = load_registry()?;
    let inst = registry
        .instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance '{}' not found", instance_id))?;

    let is_running = is_instance_running(&inst.id, &inst.data_dir, inst.pid);
    let pids = find_pids_for_data_dir(&inst.data_dir, inst.id == "default");
    let workspaces = get_instance_workspace_folders(&inst.id, &inst.data_dir);

    let target_data_path = PathBuf::from(&inst.data_dir);
    let db_path = target_data_path
        .join("User")
        .join("globalStorage")
        .join("state.vscdb");
    let injected_email = crate::modules::db::read_injected_email(&db_path);

    let (goal_running, _hb_file, last_line) =
        crate::modules::repo_db::inspect_prompt_goal_status(&inst.id, &workspaces);

    let active_prompts = crate::modules::repo_db::list_all_prompts()
        .map(|list| list.iter().filter(|p| p.instance_id == inst.id).count())
        .unwrap_or(0);

    let now_str = if goal_running {
        Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string())
    } else {
        None
    };

    Ok(ObservedInstanceState {
        instance_id: inst.id.clone(),
        name: inst.name.clone(),
        data_dir: inst.data_dir.clone(),
        is_running,
        pids,
        bound_account_id: inst.bound_account_id.clone(),
        bound_account_email: inst.bound_email.clone(),
        injected_email_in_db: injected_email,
        workspace_folders: workspaces,
        active_prompts_count: active_prompts,
        prompt_goal_running: goal_running,
        last_heartbeat_timestamp: now_str,
        last_heartbeat_line: last_line,
    })
}

/// Resolve an instance query string (seq_num like "1", ID like "inst-xyz", name like "Instance 1", or "default"/"active")
/// to a valid concrete instance ID.
pub fn resolve_instance_id(specifier: &str) -> Result<String, String> {
    let registry = load_registry()?;
    let clean = specifier.trim();
    if clean.is_empty() || clean.eq_ignore_ascii_case("active") {
        return get_active_instance_id();
    }
    if clean.eq_ignore_ascii_case("default") {
        if let Some(def) = registry
            .instances
            .iter()
            .find(|i| i.is_default || i.id == "default")
        {
            return Ok(def.id.clone());
        }
        return Ok("default".to_string());
    }
    // Check if numeric seq_num (e.g. "1")
    if let Ok(num) = clean.parse::<u32>() {
        if let Some(inst) = registry.instances.iter().find(|i| i.seq_num == Some(num)) {
            return Ok(inst.id.clone());
        }
    }
    // Check clean number prefix like "ins-1", "instance-1", "#1"
    let clean_num = clean
        .trim_start_matches("ins-")
        .trim_start_matches("instance-")
        .trim_start_matches('#');
    if let Ok(num) = clean_num.parse::<u32>() {
        if let Some(inst) = registry.instances.iter().find(|i| i.seq_num == Some(num)) {
            return Ok(inst.id.clone());
        }
    }
    // Check exact id or name match
    if let Some(inst) = registry
        .instances
        .iter()
        .find(|i| i.id.eq_ignore_ascii_case(clean) || i.name.eq_ignore_ascii_case(clean))
    {
        return Ok(inst.id.clone());
    }

    // Check suffix matching for IDs like "8159", "-8159", or "inst-8159"
    let trimmed_suffix = clean
        .trim_start_matches("ins-")
        .trim_start_matches("instance-")
        .trim_start_matches("inst-")
        .trim_start_matches('-');

    let suffix_matches: Vec<&InstanceConfig> = registry
        .instances
        .iter()
        .filter(|i| {
            i.id.ends_with(&format!("-{}", clean))
                || i.name.ends_with(&format!("-{}", clean))
                || i.id.ends_with(clean)
                || (!trimmed_suffix.is_empty()
                    && (i.id.ends_with(&format!("-{}", trimmed_suffix))
                        || i.name.ends_with(&format!("-{}", trimmed_suffix))
                        || i.id.ends_with(trimmed_suffix)))
        })
        .collect();

    if suffix_matches.len() == 1 {
        return Ok(suffix_matches[0].id.clone());
    } else if suffix_matches.len() > 1 {
        if let Some(hyphen_match) = suffix_matches.iter().find(|i| {
            i.id.ends_with(&format!("-{}", clean))
                || (!trimmed_suffix.is_empty() && i.id.ends_with(&format!("-{}", trimmed_suffix)))
        }) {
            return Ok(hyphen_match.id.clone());
        }
        return Ok(suffix_matches[0].id.clone());
    }

    if clean.is_empty() {
        return Ok(registry.active_instance_id);
    }
    Err(format!("Instance '{}' not found", clean))
}
