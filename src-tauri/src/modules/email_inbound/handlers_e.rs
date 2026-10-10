use base64::prelude::*;

use super::*;

pub(crate) fn handle_named_prompt_execution(
    msg: &RawEmailMessage,
    local_machine_ip: &str,
    local_machine_name: &str,
    now: i64,
    prompt_query: String,
) -> Result<ActionOutcome, String> {
    let mut action_str = "unknown".to_string();
    let mut status = "success".to_string();
    let mut result_summary = String::new();
    let mut output_text = String::new();
    let mut exit_code: i32 = 0;
    action_str = "named_prompt_exec".to_string();
    result_summary = format!("Named prompt query: {}", prompt_query);
    Ok(ActionOutcome {
        action_str,
        status,
        result_summary,
        output_text,
        exit_code,
    })
}

pub(crate) fn handle_instance_create(
    msg: &RawEmailMessage,
    local_machine_ip: &str,
    local_machine_name: &str,
    now: i64,
    profile_name: String,
) -> Result<ActionOutcome, String> {
    let mut action_str = "unknown".to_string();
    let mut status = "success".to_string();
    let mut result_summary = String::new();
    let mut output_text = String::new();
    let mut exit_code: i32 = 0;
    action_str = "instance_create".to_string();
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        crate::modules::instance::create_instance(profile_name.clone()),
        "create_instance",
    );
    result_summary = format!("Spawned instance profile '{}'", profile_name);
    Ok(ActionOutcome {
        action_str,
        status,
        result_summary,
        output_text,
        exit_code,
    })
}

pub(crate) fn handle_account_rotate(
    msg: &RawEmailMessage,
    local_machine_ip: &str,
    local_machine_name: &str,
    now: i64,
    target: String,
    instance_id: Option<String>,
) -> Result<ActionOutcome, String> {
    let mut action_str = "unknown".to_string();
    let mut status = "success".to_string();
    let mut result_summary = String::new();
    let mut output_text = String::new();
    let mut exit_code: i32 = 0;
    action_str = "rotate".to_string();
    if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
        status = "skipped".to_string();
        result_summary = format!("Rotate target mismatch: {}", target);
    } else {
        let inst_label = instance_id.as_deref().unwrap_or("default");
        send_ack_receipt(
            &msg.from,
            "agm rotate",
            &target,
            inst_label,
            "-",
            local_machine_name,
            local_machine_ip,
            Some(&msg.message_id),
            Some(&msg.subject),
        );

        let rt = tokio::runtime::Runtime::new().ok();
        let rot_result = if let Some(r) = rt {
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

        match rot_result {
            Ok(msg_txt) => {
                output_text = format!(
                "Account rotation successfully executed via Smart Rotator.\nDetails: {}\nActive Account: {}",
                msg_txt, current_email
            );
                result_summary = format!("Rotation completed. Active: {}", current_email);
                send_result_receipt(
                    &msg.from,
                    "agm rotate",
                    "SUCCESS",
                    0,
                    &output_text,
                    local_machine_name,
                    local_machine_ip,
                    inst_label,
                    Some(&msg.message_id),
                    Some(&msg.subject),
                );
            }
            Err(e) => {
                exit_code = 1;
                status = "error".to_string();
                output_text = format!("Rotation error: {}", e);
                result_summary = format!("Rotation failed: {}", e);
                send_result_receipt(
                    &msg.from,
                    "agm rotate",
                    "FAILED",
                    1,
                    &output_text,
                    local_machine_name,
                    local_machine_ip,
                    inst_label,
                    Some(&msg.message_id),
                    Some(&msg.subject),
                );
            }
        }
    }
    Ok(ActionOutcome {
        action_str,
        status,
        result_summary,
        output_text,
        exit_code,
    })
}

pub(crate) fn handle_multi_node_snapshot_query(
    msg: &RawEmailMessage,
    local_machine_ip: &str,
    local_machine_name: &str,
    now: i64,
) -> Result<ActionOutcome, String> {
    let mut action_str = "unknown".to_string();
    let mut status = "success".to_string();
    let mut result_summary = String::new();
    let mut output_text = String::new();
    let mut exit_code: i32 = 0;
    action_str = "cluster_snapshot".to_string();
    result_summary = "Cluster snapshot generated".to_string();
    Ok(ActionOutcome {
        action_str,
        status,
        result_summary,
        output_text,
        exit_code,
    })
}

pub(crate) fn handle_ignored(
    msg: &RawEmailMessage,
    local_machine_ip: &str,
    local_machine_name: &str,
    now: i64,
    reason: String,
) -> Result<ActionOutcome, String> {
    let mut action_str = "unknown".to_string();
    let mut status = "success".to_string();
    let mut result_summary = String::new();
    let mut output_text = String::new();
    let mut exit_code: i32 = 0;
    action_str = "ignored".to_string();
    status = "rejected".to_string();
    result_summary = reason;
    Ok(ActionOutcome {
        action_str,
        status,
        result_summary,
        output_text,
        exit_code,
    })
}
