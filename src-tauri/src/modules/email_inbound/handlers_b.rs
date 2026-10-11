use base64::prelude::*;

use super::*;

pub(crate) fn handle_list_instances(
    msg: &RawEmailMessage,
    local_machine_ip: &str,
    local_machine_name: &str,
    now: i64,
    target: String,
) -> Result<ActionOutcome, String> {
    let mut action_str = "unknown".to_string();
    let mut status = "success".to_string();
    let mut result_summary = String::new();
    let mut output_text = String::new();
    let mut exit_code: i32 = 0;
    action_str = "list_instances".to_string();
    if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
        status = "skipped".to_string();
        result_summary = format!("Target mismatch: {}", target);
    } else {
        send_ack_receipt(
            &msg.from,
            "agm instances",
            &target,
            "default",
            "-",
            local_machine_name,
            local_machine_ip,
            Some(&msg.message_id),
            Some(&msg.subject),
        );

        let instances = crate::modules::instance::list_instances().unwrap_or_default();
        let mut table = format!(
            "{:<16} {:<20} {:<16} {:<24} {}\n",
            "ID", "NAME", "STATUS", "BOUND ACCOUNT", "DATA DIR"
        );
        table.push_str(&"-".repeat(95));
        table.push('\n');
        for inst in instances {
            let status_str = if inst.is_running {
                format!("Running (PID: {})", inst.pid.unwrap_or(0))
            } else {
                "Idle".to_string()
            };
            let email = inst.config.bound_email.unwrap_or_else(|| "-".to_string());
            table.push_str(&format!(
                "{:<16} {:<20} {:<16} {:<24} {}\n",
                inst.config.id, inst.config.name, status_str, email, inst.config.data_dir
            ));
        }
        output_text = table;
        result_summary = "Listed sandbox profiles".to_string();

        send_result_receipt(
            &msg.from,
            "agm instances",
            "SUCCESS",
            0,
            &output_text,
            local_machine_name,
            local_machine_ip,
            "default",
            Some(&msg.message_id),
            Some(&msg.subject),
        );
    }
    Ok(ActionOutcome {
        action_str,
        status,
        result_summary,
        output_text,
        exit_code,
    })
}

pub(crate) fn handle_list_prompts(
    msg: &RawEmailMessage,
    local_machine_ip: &str,
    local_machine_name: &str,
    now: i64,
    target: String,
    is_gitmap: bool,
) -> Result<ActionOutcome, String> {
    let mut action_str = "unknown".to_string();
    let mut status = "success".to_string();
    let mut result_summary = String::new();
    let mut output_text = String::new();
    let mut exit_code: i32 = 0;
    action_str = "list_prompts".to_string();
    if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
        status = "skipped".to_string();
        result_summary = format!("Target mismatch: {}", target);
    } else {
        let cmd_label = if is_gitmap {
            "gitmap prompts ls"
        } else {
            "agy prompts ls"
        };
        send_ack_receipt(
            &msg.from,
            cmd_label,
            &target,
            "default",
            "-",
            local_machine_name,
            local_machine_ip,
            Some(&msg.message_id),
            Some(&msg.subject),
        );

        if is_gitmap {
            output_text = execute_safe_cli_command("gitmap prompts ls")
                .unwrap_or_else(|e| format!("Failed to list gitmap prompts: {}", e));
        } else {
            let all_prompts = crate::modules::repo_db::list_all_prompts().unwrap_or_default();
            let mut list = format!("Total Backed-Up Prompts: {}\n\n", all_prompts.len());
            for p in all_prompts.iter().take(15) {
                let short_id = &p.id[..8.min(p.id.len())];
                let short_content: String = p.prompt_content.chars().take(60).collect();
                list.push_str(&format!("- [{}] {}\n", short_id, short_content));
            }
            output_text = list;
        }
        result_summary = "Listed prompts".to_string();

        send_result_receipt(
            &msg.from,
            cmd_label,
            "SUCCESS",
            0,
            &output_text,
            local_machine_name,
            local_machine_ip,
            "default",
            Some(&msg.message_id),
            Some(&msg.subject),
        );
    }
    Ok(ActionOutcome {
        action_str,
        status,
        result_summary,
        output_text,
        exit_code,
    })
}

pub(crate) fn handle_fast_forward(
    msg: &RawEmailMessage,
    local_machine_ip: &str,
    local_machine_name: &str,
    now: i64,
    target_node: String,
    instance_id: Option<String>,
) -> Result<ActionOutcome, String> {
    let mut action_str = "unknown".to_string();
    let mut status = "success".to_string();
    let mut result_summary = String::new();
    let mut output_text = String::new();
    let mut exit_code: i32 = 0;
    action_str = "fast_forward".to_string();
    if !matches_target_node_or_ip(&target_node, local_machine_ip, local_machine_name) {
        status = "skipped".to_string();
        result_summary = format!(
            "Fast Forward target mismatch: {} != {}",
            target_node, local_machine_name
        );
    } else {
        let inst_label = instance_id.as_deref().unwrap_or("default");
        send_ack_receipt(
            &msg.from,
            "agm ff",
            &target_node,
            inst_label,
            "-",
            local_machine_name,
            local_machine_ip,
            Some(&msg.message_id),
            Some(&msg.subject),
        );

        let rt = tokio::runtime::Runtime::new().ok();
        let ff_result = if let Some(r) = rt {
            r.block_on(
                crate::modules::auto_switcher::trigger_manual_rotation_for_instance(
                    instance_id.as_deref(),
                ),
            )
        } else {
            Err("Failed to start runtime".to_string())
        };

        let current_account = crate::modules::account::get_current_account().unwrap_or(None);
        let current_email = current_account
            .map(|a| a.email)
            .unwrap_or_else(|| "Default".to_string());

        match ff_result {
            Ok(msg_txt) => {
                output_text = format!(
                    "Fast-forward successfully executed.\nDetails: {}\nActive Account: {}",
                    msg_txt, current_email
                );
                result_summary = format!("Fast-forward completed. Active: {}", current_email);
            }
            Err(e) => {
                exit_code = 1;
                status = "error".to_string();
                output_text = format!("Fast-forward error: {}", e);
                result_summary = format!("Fast-forward failed: {}", e);
            }
        }

        send_result_receipt(
            &msg.from,
            "agm ff",
            &status.to_uppercase(),
            exit_code,
            &output_text,
            local_machine_name,
            local_machine_ip,
            inst_label,
            Some(&msg.message_id),
            Some(&msg.subject),
        );
    }
    Ok(ActionOutcome {
        action_str,
        status,
        result_summary,
        output_text,
        exit_code,
    })
}

pub(crate) fn handle_status_query(
    msg: &RawEmailMessage,
    local_machine_ip: &str,
    local_machine_name: &str,
    now: i64,
    target: String,
) -> Result<ActionOutcome, String> {
    let mut action_str = "unknown".to_string();
    let mut status = "success".to_string();
    let mut result_summary = String::new();
    let mut output_text = String::new();
    let mut exit_code: i32 = 0;
    action_str = "status_query".to_string();
    if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
        status = "skipped".to_string();
        result_summary = format!("Status target mismatch: {}", target);
    } else {
        let reg = crate::modules::instance::load_registry().unwrap_or_default();
        let active_inst = reg
            .instances
            .iter()
            .find(|i| i.id == reg.active_instance_id || i.is_default);
        let active_inst_name = active_inst.map(|i| i.name.as_str()).unwrap_or("default");
        let active_inst_id = active_inst.map(|i| i.id.as_str()).unwrap_or("default");
        let active_bound_email = active_inst
            .and_then(|i| i.bound_email.as_deref())
            .unwrap_or("none");

        let pids = active_inst
            .map(|i| crate::modules::instance::find_pids_for_data_dir(&i.data_dir, i.is_default))
            .unwrap_or_default();
        let is_running = !pids.is_empty();
        let state_label = if is_running { "RUNNING" } else { "IDLE" };
        let pid_str = if pids.is_empty() {
            "-".to_string()
        } else {
            pids.iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };

        let accounts = crate::modules::account::list_accounts().unwrap_or_default();
        let bound_acc = accounts
            .iter()
            .find(|a| a.email.eq_ignore_ascii_case(active_bound_email))
            .or_else(|| {
                let curr_id = crate::modules::account::get_current_account_id().unwrap_or_default();
                accounts.iter().find(|a| Some(&a.id) == curr_id.as_ref())
            });

        let (tier_str, quota_str) = if let Some(acc) = bound_acc {
            let t = acc
                .quota
                .as_ref()
                .and_then(|q| q.subscription_tier.as_deref())
                .unwrap_or("PRO");
            let (q_4h, q_weekly) =
                crate::modules::auto_switcher::extract_dual_window_quotas(acc, "gemini-2.5-pro");
            let q_formatted = match (q_4h, q_weekly) {
                (Some(h), Some(w)) => format!("4h: {:.1}% | Weekly: {:.1}%", h, w),
                (Some(h), None) => format!("4h: {:.1}%", h),
                (None, Some(w)) => format!("Weekly: {:.1}%", w),
                _ => "Active".to_string(),
            };
            (t.to_string(), q_formatted)
        } else {
            ("Standard".to_string(), "Active".to_string())
        };

        send_ack_receipt(
            &msg.from,
            "agm status",
            &target,
            active_inst_id,
            "-",
            local_machine_name,
            local_machine_ip,
            Some(&msg.message_id),
            Some(&msg.subject),
        );

        let projects = crate::modules::repo_db::list_running_projects().unwrap_or_default();
        let prompts = crate::modules::repo_db::list_backed_up_prompts().unwrap_or_default();
        let total_instances = reg.instances.len();
        let total_accounts = accounts.len();

        output_text = format!(
            "================================================================================
TELEMETRY FLEET & NODE STATUS REPORT
================================================================================
Machine:          {} ({})
OS / Arch:        {}/{}
Active Instance:  {} (ID: {})
State:            {} (PIDs: {})
Bound Account:    {}
Tier / Quotas:    {} | {}
Inventory:        {} sandboxes | {} accounts
Running Projects: {}
Prompts in Queue: {}
================================================================================",
            local_machine_name,
            local_machine_ip,
            std::env::consts::OS,
            std::env::consts::ARCH,
            active_inst_name,
            active_inst_id,
            state_label,
            pid_str,
            active_bound_email,
            tier_str,
            quota_str,
            total_instances,
            total_accounts,
            projects.len(),
            prompts.len()
        );
        result_summary = format!("Status reported to '{}'", msg.from);

        send_result_receipt(
            &msg.from,
            "agm status",
            "SUCCESS",
            0,
            &output_text,
            local_machine_name,
            local_machine_ip,
            active_inst_id,
            Some(&msg.message_id),
            Some(&msg.subject),
        );
    }
    Ok(ActionOutcome {
        action_str,
        status,
        result_summary,
        output_text,
        exit_code,
    })
}
