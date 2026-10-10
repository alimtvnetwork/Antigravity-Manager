use crate::models::Account;
use crate::modules::{account, config, instance, logger};
use std::time::Duration;

use super::*;

/// List instances that are either currently running or marked as active
pub fn list_running_or_active_instances() -> Result<Vec<crate::models::InstanceConfig>, String> {
    let registry = instance::load_registry()?;
    let active_id = registry.active_instance_id.clone();
    let mut result = Vec::new();

    for inst in registry.instances {
        let is_active = inst.id == active_id;
        let is_running = instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid);

        if is_active {
            result.push(inst);
        } else if is_running {
            result.push(inst);
        }
    }

    Ok(result)
}

/// Check whether an instance binding is stale (no ping/activity for > `stale_binding_timeout_hours`, default 6h)
/// or has 0% credits remaining (not working / exhausted).
pub fn is_instance_binding_stale_or_exhausted(
    inst: &crate::models::InstanceConfig,
    is_running: bool,
    stale_timeout_hours: u32,
    target_model: &str,
    now_sec: i64,
) -> bool {
    let stale_timeout_secs = (stale_timeout_hours.clamp(1, 24) as i64) * 3600;
    let is_stale_time = inst.last_used > 0 && (now_sec - inst.last_used) > stale_timeout_secs;

    // If not actively running and last activity exceeded stale_binding_timeout_hours (6h-10h), mark inactive
    if !is_running && is_stale_time {
        return true;
    }

    // If bound account has 0% credits remaining (no credits), mark as not working / exhausted
    if let Some(ref acc_id) = inst.bound_account_id {
        if let Ok(acc) = account::load_account(acc_id) {
            let q_4h = calculate_4h_window_quota(&acc, target_model).unwrap_or(100.0);
            if q_4h <= 0.0 {
                return true;
            }
        }
    }

    false
}

/// Get account IDs currently bound to any running or active non-stale instance
pub fn get_active_in_use_account_ids() -> Vec<String> {
    let instances = list_running_or_active_instances().unwrap_or_default();
    let switcher_cfg = config::load_app_config()
        .map(|c| c.auto_profile_switcher)
        .unwrap_or_default();
    let now_sec = chrono::Utc::now().timestamp();
    let mut in_use = Vec::new();
    for inst in instances {
        let is_running = instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid);
        if is_instance_binding_stale_or_exhausted(
            &inst,
            is_running,
            switcher_cfg.stale_binding_timeout_hours,
            &switcher_cfg.target_model,
            now_sec,
        ) {
            continue;
        }
        let acc_id_opt = inst
            .bound_account_id
            .clone()
            .or_else(|| account::get_current_account_id().ok().flatten());
        if let Some(acc_id) = acc_id_opt {
            if !in_use.contains(&acc_id) {
                in_use.push(acc_id);
            }
        }
    }
    if let Ok(Some(current_id)) = account::get_current_account_id() {
        if let Ok(cur_acc) = account::load_account(&current_id) {
            let cur_q =
                calculate_4h_window_quota(&cur_acc, &switcher_cfg.target_model).unwrap_or(100.0);
            let is_cur_stale = cur_acc.last_used > 0
                && (now_sec - cur_acc.last_used)
                    > (switcher_cfg.stale_binding_timeout_hours.clamp(1, 24) as i64) * 3600;
            if cur_q > 0.0 && !is_cur_stale && !in_use.contains(&current_id) {
                in_use.push(current_id);
            }
        } else if !in_use.contains(&current_id) {
            in_use.push(current_id);
        }
    }
    in_use
}

/// Check if Antigravity IDE, isolated instances, active projects, or proxy services are currently running
pub fn is_antigravity_or_instance_running(instance_id: Option<&str>) -> bool {
    // 1. Check running projects in repo_db
    if let Ok(projects) = crate::modules::repo_db::list_running_projects() {
        if projects.into_iter().any(|p| p.is_running) {
            return true;
        }
    }

    // 2. Check active running prompts in repo_db
    if let Ok(prompts) = crate::modules::repo_db::list_backed_up_prompts() {
        if !prompts.is_empty() {
            return true;
        }
    }

    // 3. Check specific instance if provided
    if let Some(inst_id) = instance_id {
        if let Ok(registry) = crate::modules::instance::load_registry() {
            if let Some(inst) = registry.instances.iter().find(|i| i.id == inst_id) {
                return crate::modules::instance::is_instance_running(
                    &inst.id,
                    &inst.data_dir,
                    inst.pid,
                );
            }
        }
    }

    // 4. Check if any registered instance is currently running
    if let Ok(registry) = crate::modules::instance::load_registry() {
        for inst in registry.instances {
            if crate::modules::instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid) {
                return true;
            }
        }
    }

    // 5. Fallback check for running Antigravity IDE processes
    if crate::modules::process::is_process_running_by_name("antigravity")
        || crate::modules::process::is_process_running_by_name("agy")
    {
        return true;
    }

    // 6. Check if proxy port 8045 is active and responding
    if std::net::TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], 8045)),
        std::time::Duration::from_millis(50),
    )
    .is_ok()
    {
        return true;
    }

    false
}

/// Check whether a given account is currently in active use
pub fn is_account_in_use(acc: &Account) -> bool {
    let cur_id = account::get_current_account_id().ok().flatten();
    let cur_acc = account::get_current_account().ok().flatten();
    let cur_email = cur_acc.as_ref().map(|a| a.email.as_str());
    let in_use_ids = get_active_in_use_account_ids();
    let bound_pairs = list_running_or_active_instances()
        .map(|list| {
            list.into_iter()
                .map(|i| (i.bound_account_id, i.bound_email))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    is_account_in_use_core(
        &acc.id,
        &acc.email,
        cur_id
            .as_deref()
            .or(cur_acc.as_ref().map(|a| a.id.as_str())),
        cur_email,
        &in_use_ids,
        &bound_pairs,
    )
}

/// Core evaluation helper for account in-use checks, decoupled from disk I/O
pub fn is_account_in_use_core(
    acc_id: &str,
    acc_email: &str,
    cur_id: Option<&str>,
    cur_email: Option<&str>,
    in_use_ids: &[String],
    bound_pairs: &[(Option<String>, Option<String>)],
) -> bool {
    let lower_email = acc_email.trim().to_lowercase();
    if let Some(cid) = cur_id {
        if acc_id == cid {
            return true;
        }
    }
    if let Some(cemail) = cur_email {
        if lower_email == cemail.trim().to_lowercase() {
            return true;
        }
    }
    if in_use_ids.iter().any(|id| id == acc_id) {
        return true;
    }
    for (bid, bemail) in bound_pairs {
        if let Some(ref b) = bid {
            if b == acc_id {
                return true;
            }
        }
        if let Some(ref e) = bemail {
            if e.trim().to_lowercase() == lower_email {
                return true;
            }
        }
    }
    false
}
