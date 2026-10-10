use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::*;

// ============================================================================
// Machine Remote Modification
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineModifyRequest {
    pub action: String, // "switch_account", "bind_instance", "set_target_model", "adjust_threshold", "trigger_rotation"
    pub account_email_or_id: Option<String>,
    pub instance_id: Option<String>,
    pub target_model: Option<String>,
    pub low_quota_threshold: Option<f64>,
    pub critical_quota_threshold: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MachineModifyResponse {
    pub success: bool,
    pub action: String,
    pub message: String,
    pub current_active_account: Option<String>,
    pub current_target_model: String,
}

pub async fn execute_machine_modify(
    req: MachineModifyRequest,
) -> Result<MachineModifyResponse, String> {
    let mut app_cfg = crate::modules::config::load_app_config().map_err(|e| e.to_string())?;

    match req.action.as_str() {
        "switch_account" => {
            let query = req
                .account_email_or_id
                .ok_or_else(|| "account_email_or_id is required for switch_account".to_string())?;
            let accounts = load_all_accounts_list();
            let target = accounts
                .into_iter()
                .find(|a| a.email.eq_ignore_ascii_case(&query) || a.id == query)
                .ok_or_else(|| format!("Account matching '{}' not found", query))?;

            let inst_id = req.instance_id.unwrap_or_else(|| "default".to_string());
            crate::modules::instance::switch_account_to_instance(&target.id, Some(&inst_id))
                .await
                .map_err(|e| format!("Failed to switch account to instance: {}", e))?;

            Ok(MachineModifyResponse {
                success: true,
                action: req.action,
                message: format!(
                    "Successfully switched active account to {} on instance '{}'",
                    target.email, inst_id
                ),
                current_active_account: Some(target.email),
                current_target_model: app_cfg.auto_profile_switcher.target_model,
            })
        }
        "bind_instance" => {
            let inst_id = req
                .instance_id
                .ok_or_else(|| "instance_id is required for bind_instance".to_string())?;
            let query = req
                .account_email_or_id
                .ok_or_else(|| "account_email_or_id is required for bind_instance".to_string())?;
            let accounts = load_all_accounts_list();
            let target = accounts
                .into_iter()
                .find(|a| a.email.eq_ignore_ascii_case(&query) || a.id == query)
                .ok_or_else(|| format!("Account matching '{}' not found", query))?;

            crate::modules::instance::bind_account_to_instance(
                &inst_id,
                &target.id,
                &target.email,
            )?;

            Ok(MachineModifyResponse {
                success: true,
                action: req.action,
                message: format!("Bound account {} to instance {}", target.email, inst_id),
                current_active_account: Some(target.email),
                current_target_model: app_cfg.auto_profile_switcher.target_model,
            })
        }
        "set_target_model" => {
            let model = req
                .target_model
                .ok_or_else(|| "target_model is required for set_target_model".to_string())?;
            app_cfg.auto_profile_switcher.target_model = model.clone();
            crate::modules::config::save_app_config(&app_cfg).map_err(|e| e.to_string())?;

            let active_acc = crate::modules::account::get_current_account()
                .ok()
                .flatten()
                .map(|a| a.email);

            Ok(MachineModifyResponse {
                success: true,
                action: req.action,
                message: format!("Auto-switcher target model updated to '{}'", model),
                current_active_account: active_acc,
                current_target_model: model,
            })
        }
        "adjust_threshold" => {
            if let Some(low) = req.low_quota_threshold {
                app_cfg.auto_profile_switcher.low_quota_threshold_percent = low;
            }
            if let Some(crit) = req.critical_quota_threshold {
                app_cfg.auto_profile_switcher.critical_threshold_percent = crit;
            }
            crate::modules::config::save_app_config(&app_cfg).map_err(|e| e.to_string())?;

            let active_acc = crate::modules::account::get_current_account()
                .ok()
                .flatten()
                .map(|a| a.email);

            Ok(MachineModifyResponse {
                success: true,
                action: req.action,
                message: format!(
                    "Quota thresholds updated: low={:.1}%, critical={:.1}%",
                    app_cfg.auto_profile_switcher.low_quota_threshold_percent,
                    app_cfg.auto_profile_switcher.critical_threshold_percent
                ),
                current_active_account: active_acc,
                current_target_model: app_cfg.auto_profile_switcher.target_model,
            })
        }
        "trigger_rotation" => {
            let result = crate::modules::auto_switcher::trigger_manual_rotation_for_instance(
                req.instance_id.as_deref(),
            )
            .await?;
            let active_acc = crate::modules::account::get_current_account()
                .ok()
                .flatten()
                .map(|a| a.email);

            Ok(MachineModifyResponse {
                success: true,
                action: req.action,
                message: format!("Auto-rotation triggered: {}", result),
                current_active_account: active_acc,
                current_target_model: app_cfg.auto_profile_switcher.target_model,
            })
        }
        other => Err(format!(
            "Unsupported machine modification action: '{}'. Supported: switch_account, bind_instance, set_target_model, adjust_threshold, trigger_rotation",
            other
        )),
    }
}

/// Axum POST /api/v1/training/machines
pub async fn handle_training_machines(Json(payload): Json<MachineModifyRequest>) -> Response {
    if let Err(resp) = guard_training_api() {
        return resp;
    }

    match execute_machine_modify(payload).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": format!("Failed to execute machine modification: {}", e),
                "status": 400
            })),
        )
            .into_response(),
    }
}
