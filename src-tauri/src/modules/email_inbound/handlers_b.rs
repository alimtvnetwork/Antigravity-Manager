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
        send_ack_receipt(
            &msg.from,
            "agm status",
            &target,
            "default",
            "-",
            local_machine_name,
            local_machine_ip,
            Some(&msg.message_id),
            Some(&msg.subject),
        );

        let projects = crate::modules::repo_db::list_running_projects().unwrap_or_default();
        let prompts = crate::modules::repo_db::list_backed_up_prompts().unwrap_or_default();
        let instances = crate::modules::instance::list_instances().unwrap_or_default();
        let current_account = crate::modules::account::get_current_account().unwrap_or(None);
        let active_acc_str = current_account
            .map(|a| a.email)
            .unwrap_or_else(|| "Default".to_string());

        output_text = format!(
            "================================================================================
NODE STATUS REPORT
================================================================================
Node Name:          {}
Local IP:           {}
Active Profile:     {}
Running Projects:   {}
Registered Sandbox: {}
Prompts in Queue:   {}
================================================================================",
            local_machine_name,
            local_machine_ip,
            active_acc_str,
            projects.len(),
            instances.len(),
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
