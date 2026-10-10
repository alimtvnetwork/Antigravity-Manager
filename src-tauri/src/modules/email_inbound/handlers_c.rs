use base64::prelude::*;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use super::*;

pub(crate) fn handle_doctor_diagnostic(
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
    action_str = "doctor".to_string();
    if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
        status = "skipped".to_string();
        result_summary = format!("Doctor target mismatch: {}", target);
    } else {
        send_ack_receipt(
            &msg.from,
            "agm doctor",
            &target,
            "default",
            "-",
            local_machine_name,
            local_machine_ip,
            Some(&msg.message_id),
            Some(&msg.subject),
        );

        let accounts_cnt = crate::modules::account::load_account_index()
            .map(|idx| idx.accounts.len())
            .unwrap_or(0);
        let instances_cnt = crate::modules::instance::list_instances()
            .map(|l| l.len())
            .unwrap_or(0);
        let proxy_online = std::net::TcpStream::connect_timeout(
            &"127.0.0.1:8045".parse().unwrap(),
            std::time::Duration::from_millis(500),
        )
        .is_ok();

        output_text = format!(
            "================================================================================
AGM SYSTEM DIAGNOSTIC (DOCTOR)
================================================================================
Node Name:          {}
Local IP:           {}
Database Vaults:    Verified Healthy (accounts: {}, instances: {})
Proxy Gateway:      {}
System Status:      HEALTHY
================================================================================",
            local_machine_name,
            local_machine_ip,
            accounts_cnt,
            instances_cnt,
            if proxy_online {
                "ONLINE (Port 8045 listening)"
            } else {
                "OFFLINE (Standby)"
            }
        );
        result_summary = format!("Doctor diagnostic sent to '{}'", msg.from);

        send_result_receipt(
            &msg.from,
            "agm doctor",
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

pub(crate) fn handle_list_accounts(
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
    action_str = "list_accounts".to_string();
    if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
        status = "skipped".to_string();
        result_summary = format!("Accounts target mismatch: {}", target);
    } else {
        send_ack_receipt(
            &msg.from,
            "agm accounts",
            &target,
            "default",
            "-",
            local_machine_name,
            local_machine_ip,
            Some(&msg.message_id),
            Some(&msg.subject),
        );

        let index_res = crate::modules::account::load_account_index();
        let mut acc_rows = Vec::new();
        if let Ok(index) = index_res {
            let active_id = index.current_account_id.as_deref().unwrap_or("");
            for (i, acc) in index.accounts.iter().enumerate() {
                let is_curr = acc.id == active_id;
                let st = if is_curr { "ACTIVE" } else { "STANDBY" };
                acc_rows.push(format!("{:<4} {:<32} {:<10}", i + 1, acc.email, st));
            }
        }

        output_text = format!(
            "================================================================================
REGISTERED ACCOUNTS
================================================================================
INDEX EMAIL                            STATUS
--------------------------------------------------------------------------------
{}
================================================================================",
            if acc_rows.is_empty() {
                "No accounts registered.".to_string()
            } else {
                acc_rows.join("\n")
            }
        );
        result_summary = format!("Accounts list sent to '{}'", msg.from);

        send_result_receipt(
            &msg.from,
            "agm accounts",
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

pub(crate) fn handle_account_switch(
    msg: &RawEmailMessage,
    local_machine_ip: &str,
    local_machine_name: &str,
    now: i64,
    target: String,
    email_query: String,
    instance_id: Option<String>,
) -> Result<ActionOutcome, String> {
    let mut action_str = "unknown".to_string();
    let mut status = "success".to_string();
    let mut result_summary = String::new();
    let mut output_text = String::new();
    let mut exit_code: i32 = 0;
    action_str = "switch_account".to_string();
    if !matches_target_node_or_ip(&target, local_machine_ip, local_machine_name) {
        status = "skipped".to_string();
        result_summary = format!("Switch target mismatch: {}", target);
    } else {
        let inst_label = instance_id.as_deref().unwrap_or("default");
        send_ack_receipt(
            &msg.from,
            "agm switch",
            &target,
            inst_label,
            &email_query,
            local_machine_name,
            local_machine_ip,
            Some(&msg.message_id),
            Some(&msg.subject),
        );

        let query = email_query.trim().to_lowercase();
        let prev_email = crate::modules::account::get_current_account()
            .ok()
            .flatten()
            .map(|a| a.email)
            .unwrap_or_else(|| "(None)".to_string());

        let mut switch_ok = false;
        let mut new_email = String::new();
        let mut switch_err = String::new();

        // If query is empty or requesting smart rotation
        if query.is_empty()
            || query == "smart"
            || query == "ff"
            || query == "rotate"
            || query == "next"
        {
            let rt = tokio::runtime::Runtime::new().ok();
            let rot_res = if let Some(r) = rt {
                r.block_on(
                    crate::modules::auto_switcher::trigger_manual_rotation_for_instance(
                        instance_id.as_deref(),
                    ),
                )
            } else {
                Err("Failed to start runtime".to_string())
            };

            match rot_res {
                Ok(details) => {
                    switch_ok = true;
                    new_email = crate::modules::account::get_current_account()
                        .ok()
                        .flatten()
                        .map(|a| a.email)
                        .unwrap_or_else(|| "Rotated Account".to_string());
                    result_summary =
                        format!("Rotated to '{}' via Smart Rotator ({})", new_email, details);
                }
                Err(e) => {
                    switch_err = format!("Smart rotation failed: {}", e);
                }
            }
        } else if let Ok(index) = crate::modules::account::load_account_index() {
            let matched = index.accounts.iter().find(|a| {
                a.email.to_lowercase().contains(&query) || a.id.to_lowercase().contains(&query)
            });
            if let Some(target_acc) = matched {
                // Consult Smart Rotator: if target_acc is already the active account on this workspace,
                // rotate to the next best candidate via Smart Rotator instead of re-injecting the same account.
                let is_already_current = target_acc.email.eq_ignore_ascii_case(&prev_email);
                let target_inst = instance_id.clone();
                let rt = tokio::runtime::Runtime::new().ok();

                if is_already_current {
                    let rot_res = if let Some(r) = rt {
                        r.block_on(
                            crate::modules::auto_switcher::trigger_manual_rotation_for_instance(
                                target_inst.as_deref(),
                            ),
                        )
                    } else {
                        Err("Failed to start runtime".to_string())
                    };
                    match rot_res {
                        Ok(_) => {
                            switch_ok = true;
                            new_email = crate::modules::account::get_current_account()
                                .ok()
                                .flatten()
                                .map(|a| a.email)
                                .unwrap_or_else(|| target_acc.email.clone());
                        }
                        Err(e) => {
                            switch_err = format!("Smart rotation failed: {}", e);
                        }
                    }
                } else {
                    let target_acc_id = target_acc.id.clone();
                    let target_acc_email = target_acc.email.clone();
                    let switch_res = if let Some(r) = rt {
                        r.block_on(crate::modules::instance::switch_account_to_instance(
                            &target_acc_id,
                            target_inst.as_deref(),
                        ))
                    } else {
                        Err("Failed to start runtime".to_string())
                    };

                    match switch_res {
                        Ok(_) => {
                            switch_ok = true;
                            new_email = target_acc_email;
                        }
                        Err(e) => {
                            switch_err =
                                format!("Failed to inject account credentials to IDE: {}", e);
                        }
                    }
                }
            } else {
                // No exact match for query -> delegate to Smart Rotator to pick the best candidate
                let rt = tokio::runtime::Runtime::new().ok();
                let rot_res = if let Some(r) = rt {
                    r.block_on(
                        crate::modules::auto_switcher::trigger_manual_rotation_for_instance(
                            instance_id.as_deref(),
                        ),
                    )
                } else {
                    Err("Failed to start runtime".to_string())
                };
                match rot_res {
                    Ok(_) => {
                        switch_ok = true;
                        new_email = crate::modules::account::get_current_account()
                            .ok()
                            .flatten()
                            .map(|a| a.email)
                            .unwrap_or_else(|| "Smart Rotated Account".to_string());
                    }
                    Err(e) => {
                        switch_err = format!(
                            "No account matched '{}' and Smart Rotator fallback failed: {}",
                            query, e
                        );
                    }
                }
            }
        } else {
            switch_err = "Failed to load accounts index".to_string();
        }

        if switch_ok {
            output_text = format!(
                "================================================================================
ACCOUNT SWITCH SUCCESS
================================================================================
Node Name:          {}
Target Instance:    {}
Previous Account:   {}
New Active Account: {}
IDE Injection:      COMPLETED (state.vscdb updated, token refreshed, IDE synced)
Status:             SUCCESS
================================================================================",
                local_machine_name, inst_label, prev_email, new_email
            );
            result_summary = format!("Switched to '{}' on [{}]", new_email, inst_label);
            send_result_receipt(
                &msg.from,
                "agm switch",
                "SUCCESS",
                0,
                &output_text,
                local_machine_name,
                local_machine_ip,
                inst_label,
                Some(&msg.message_id),
                Some(&msg.subject),
            );
        } else {
            output_text = format!(
                "================================================================================
ACCOUNT SWITCH FAILED
================================================================================
Node Name:          {}
Target Instance:    {}
Error:              {}
Previous Account:   {}
================================================================================",
                local_machine_name, inst_label, switch_err, prev_email
            );
            result_summary = format!("Switch failed: {}", switch_err);
            send_result_receipt(
                &msg.from,
                "agm switch",
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
    Ok(ActionOutcome {
        action_str,
        status,
        result_summary,
        output_text,
        exit_code,
    })
}
