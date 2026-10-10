use crate::modules::email_vault_db::{self, EmailAccount, EmailInboundAuditLog};
use base64::prelude::*;
use chrono::Utc;
use uuid::Uuid;

use super::*;

pub fn execute_inbound_action(
    msg: &RawEmailMessage,
    action: InboundAction,
    local_machine_ip: &str,
    local_machine_name: &str,
) -> Result<String, String> {
    let now = Utc::now().timestamp();

    // 1. Security Check: Sender Authorization ACL
    if !is_authorized_notifier(&msg.from) {
        let err_msg = format!("Sender '{}' not authorized in notify recipients", msg.from);
        crate::modules::logger::log_warn(&format!("[InboundEmail] Rejected: {}", err_msg));
        let audit = EmailInboundAuditLog {
            id: Uuid::new_v4().to_string(),
            message_id: msg.message_id.clone(),
            sender_email: msg.from.clone(),
            subject: msg.subject.clone(),
            action_type: "unauthorized".to_string(),
            action_payload: msg.body.chars().take(255).collect(),
            execution_status: "rejected_unauthorized_sender".to_string(),
            execution_result: err_msg.clone(),
            received_at: now,
        };
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            email_vault_db::record_inbound_audit_log(audit),
            "record_inbound_audit_log",
        );
        return Err(err_msg);
    }

    // 2. Sliding 10-Second Debounce Stack
    let action_slug = format!("{:?}", action);
    let can_proceed = check_debounce_rate_limit(&msg.from, &action_slug, local_machine_name, now);
    if !can_proceed {
        let debounced_msg = "Debounced duplicate request throttled within 10s window".to_string();
        crate::modules::logger::log_info(&format!("[InboundEmail] {}", debounced_msg));
        let audit = EmailInboundAuditLog {
            id: Uuid::new_v4().to_string(),
            message_id: msg.message_id.clone(),
            sender_email: msg.from.clone(),
            subject: msg.subject.clone(),
            action_type: "debounced".to_string(),
            action_payload: msg.body.chars().take(255).collect(),
            execution_status: "throttled".to_string(),
            execution_result: debounced_msg.clone(),
            received_at: now,
        };
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            email_vault_db::record_inbound_audit_log(audit),
            "record_inbound_audit_log",
        );
        return Ok(debounced_msg);
    }

    // 2b. SQLite 10-Minute Rate Limit for Heavy Actions
    let is_heavy_action = matches!(
        action,
        InboundAction::AccountRotate { .. }
            | InboundAction::SystemClean { .. }
            | InboundAction::UpdateExecution { .. }
    );
    if is_heavy_action {
        if let Ok(true) = email_vault_db::is_rate_limited_in_sqlite(&msg.from, &action_slug, 600) {
            let rate_limited_msg = format!(
                "Action '{:?}' rate-limited by SQLite audit guard (10-minute cooldown).",
                action
            );
            crate::modules::logger::log_info(&format!("[InboundEmail] {}", rate_limited_msg));
            let audit = EmailInboundAuditLog {
                id: Uuid::new_v4().to_string(),
                message_id: msg.message_id.clone(),
                sender_email: msg.from.clone(),
                subject: msg.subject.clone(),
                action_type: action_slug,
                action_payload: msg.body.chars().take(255).collect(),
                execution_status: "rate_limited_10m".to_string(),
                execution_result: rate_limited_msg.clone(),
                received_at: now,
            };
            // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
            crate::error::record_ignored(
                email_vault_db::record_inbound_audit_log(audit),
                "record_inbound_audit_log",
            );
            return Ok(rate_limited_msg);
        }
    }

    let mut action_str = "unknown".to_string();
    let mut status = "success".to_string();
    let mut result_summary;
    let mut output_text = String::new();
    let mut exit_code = 0;

    match action {
        InboundAction::PromptInjection {
            project_name,
            prompt_name,
            prompt,
            instance_id,
        } => {
            let o = handle_prompt_injection(
                msg,
                local_machine_ip,
                local_machine_name,
                now,
                project_name,
                prompt_name,
                prompt,
                instance_id,
            )?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
        InboundAction::GitMapExecution { target, command } => {
            let o = handle_git_map_execution(
                msg,
                local_machine_ip,
                local_machine_name,
                now,
                target,
                command,
            )?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
        InboundAction::CliExecution { target_ip, command } => {
            let o = handle_cli_execution(
                msg,
                local_machine_ip,
                local_machine_name,
                now,
                target_ip,
                command,
            )?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
        InboundAction::UpdateExecution { target, is_gitmap } => {
            let o = handle_update_execution(
                msg,
                local_machine_ip,
                local_machine_name,
                now,
                target,
                is_gitmap,
            )?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
        InboundAction::ListInstances { target } => {
            let o = handle_list_instances(msg, local_machine_ip, local_machine_name, now, target)?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
        InboundAction::ListPrompts { target, is_gitmap } => {
            let o = handle_list_prompts(
                msg,
                local_machine_ip,
                local_machine_name,
                now,
                target,
                is_gitmap,
            )?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
        InboundAction::FastForward {
            target_node,
            instance_id,
        } => {
            let o = handle_fast_forward(
                msg,
                local_machine_ip,
                local_machine_name,
                now,
                target_node,
                instance_id,
            )?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
        InboundAction::StatusQuery { target } => {
            let o = handle_status_query(msg, local_machine_ip, local_machine_name, now, target)?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
        InboundAction::DoctorDiagnostic { target } => {
            let o =
                handle_doctor_diagnostic(msg, local_machine_ip, local_machine_name, now, target)?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
        InboundAction::ListAccounts { target } => {
            let o = handle_list_accounts(msg, local_machine_ip, local_machine_name, now, target)?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
        InboundAction::AccountSwitch {
            target,
            email_query,
            instance_id,
        } => {
            let o = handle_account_switch(
                msg,
                local_machine_ip,
                local_machine_name,
                now,
                target,
                email_query,
                instance_id,
            )?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
        InboundAction::ProxyStatus { target, is_test } => {
            let o = handle_proxy_status(
                msg,
                local_machine_ip,
                local_machine_name,
                now,
                target,
                is_test,
            )?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
        InboundAction::SystemClean { target } => {
            let o = handle_system_clean(msg, local_machine_ip, local_machine_name, now, target)?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
        InboundAction::SyncState { target } => {
            let o = handle_sync_state(msg, local_machine_ip, local_machine_name, now, target)?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
        InboundAction::HelpRequest {
            target,
            instance_id,
        } => {
            let o = handle_help_request(
                msg,
                local_machine_ip,
                local_machine_name,
                now,
                target,
                instance_id,
            )?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
        InboundAction::NamedPromptExecution { prompt_query } => {
            let o = handle_named_prompt_execution(
                msg,
                local_machine_ip,
                local_machine_name,
                now,
                prompt_query,
            )?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
        InboundAction::InstanceCreate { profile_name } => {
            let o = handle_instance_create(
                msg,
                local_machine_ip,
                local_machine_name,
                now,
                profile_name,
            )?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
        InboundAction::AccountRotate {
            target,
            instance_id,
        } => {
            let o = handle_account_rotate(
                msg,
                local_machine_ip,
                local_machine_name,
                now,
                target,
                instance_id,
            )?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
        InboundAction::MultiNodeSnapshotQuery => {
            let o =
                handle_multi_node_snapshot_query(msg, local_machine_ip, local_machine_name, now)?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
        InboundAction::Ignored { reason } => {
            let o = handle_ignored(msg, local_machine_ip, local_machine_name, now, reason)?;
            action_str = o.action_str;
            status = o.status;
            result_summary = o.result_summary;
            output_text = o.output_text;
            exit_code = o.exit_code;
        }
    }

    let audit = EmailInboundAuditLog {
        id: Uuid::new_v4().to_string(),
        message_id: msg.message_id.clone(),
        sender_email: msg.from.clone(),
        subject: msg.subject.clone(),
        action_type: action_str,
        action_payload: msg.body.chars().take(255).collect(),
        execution_status: status,
        execution_result: result_summary.clone(),
        received_at: now,
    };
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        email_vault_db::record_inbound_audit_log(audit),
        "record_inbound_audit_log",
    );

    Ok(result_summary)
}
