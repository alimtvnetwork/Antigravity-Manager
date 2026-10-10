use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::*;

/// Check if the training REST API is enabled in AppConfig
pub fn is_training_api_enabled() -> bool {
    crate::modules::config::load_app_config()
        .map(|cfg| cfg.training_api_enabled)
        .unwrap_or(false)
}

/// Enable or disable training REST API in AppConfig
pub fn set_training_api_enabled(enabled: bool) -> Result<(), String> {
    let mut cfg = crate::modules::config::load_app_config().map_err(|e| e.to_string())?;
    cfg.training_api_enabled = enabled;
    crate::modules::config::save_app_config(&cfg).map_err(|e| e.to_string())?;
    crate::modules::logger::log_info(&format!(
        "[TrainingAPI] Service status updated to enabled={}",
        enabled
    ));
    Ok(())
}

pub(crate) fn guard_training_api() -> Result<(), Response> {
    if !is_training_api_enabled() {
        let err_body = json!({
            "error": "Training REST API is disabled. Enable it in Antigravity-Manager Settings.",
            "code": "TRAINING_API_DISABLED",
            "status": 403
        });
        return Err((StatusCode::FORBIDDEN, Json(err_body)).into_response());
    }
    Ok(())
}

// ============================================================================
// Telemetry Models & Handler
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountTelemetry {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub tier: String,
    pub is_active: bool,
    pub quota_percent: f64,
    pub last_used: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstanceTelemetry {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub bound_email: Option<String>,
    pub pid: Option<u32>,
    pub is_running: bool,
    pub last_used: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoSwitcherTelemetry {
    pub is_enabled: bool,
    pub target_model: String,
    pub low_quota_threshold_percent: f64,
    pub critical_threshold_percent: f64,
    pub active_instance_id: String,
    pub current_quota_percent: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeTelemetry {
    pub node_name: String,
    pub local_ip: String,
    pub version: String,
    pub os: String,
    pub arch: String,
    pub training_api_enabled: bool,
    pub timestamp: i64,
    pub active_account: Option<AccountTelemetry>,
    pub accounts_summary: serde_json::Value,
    pub instances: Vec<InstanceTelemetry>,
    pub auto_switcher: AutoSwitcherTelemetry,
    pub active_prompts_running: usize,
    pub active_prompts_total: usize,
}

pub(crate) fn load_all_accounts_list() -> Vec<crate::models::Account> {
    if let Ok(index) = crate::modules::account::load_account_index() {
        index
            .accounts
            .iter()
            .filter_map(|summary| crate::modules::account::load_account(&summary.id).ok())
            .collect()
    } else {
        Vec::new()
    }
}

/// Gather full machine telemetry
pub fn gather_telemetry() -> Result<NodeTelemetry, String> {
    let node_name = crate::modules::email_watcher::detect_machine_name();
    let local_ip = crate::modules::email_watcher::detect_local_ip();
    let now = chrono::Utc::now().timestamp();

    // Accounts
    let all_accounts = load_all_accounts_list();
    let current_acc_opt = crate::modules::account::get_current_account().unwrap_or(None);

    let active_account = current_acc_opt.as_ref().map(|acc| {
        let quota_pct = crate::modules::auto_switcher::calculate_account_quota(acc, "gemini-flash")
            .unwrap_or(100.0);
        let tier = acc
            .quota
            .as_ref()
            .and_then(|q| q.subscription_tier.clone())
            .unwrap_or_else(|| "PRO".to_string());
        let last_used_str = if acc.last_used > 0 {
            chrono::DateTime::from_timestamp(acc.last_used, 0).map(|dt| dt.to_rfc3339())
        } else {
            None
        };
        AccountTelemetry {
            id: acc.id.clone(),
            email: acc.email.clone(),
            name: acc.name.clone(),
            tier,
            is_active: true,
            quota_percent: quota_pct,
            last_used: last_used_str,
        }
    });

    let mut healthy_count = 0;
    let mut depleted_count = 0;
    for acc in &all_accounts {
        let q = crate::modules::auto_switcher::calculate_account_quota(acc, "gemini-flash")
            .unwrap_or(100.0);
        if q <= 15.0 {
            depleted_count += 1;
        } else {
            healthy_count += 1;
        }
    }

    let accounts_summary = json!({
        "total": all_accounts.len(),
        "healthy": healthy_count,
        "depleted": depleted_count,
        "active_email": current_acc_opt.as_ref().map(|a| a.email.clone()),
    });

    // Instances
    let registry = crate::modules::instance::load_registry().unwrap_or_default();
    let instances: Vec<InstanceTelemetry> = registry
        .instances
        .iter()
        .map(|inst| {
            let running =
                crate::modules::instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid);
            InstanceTelemetry {
                id: inst.id.clone(),
                name: inst.name.clone(),
                is_default: inst.is_default,
                bound_email: inst.bound_email.clone(),
                pid: inst.pid,
                is_running: running,
                last_used: inst.last_used,
            }
        })
        .collect();

    // Auto-switcher
    let app_cfg = crate::modules::config::load_app_config().unwrap_or_default();
    let switcher_status = crate::modules::auto_switcher::get_status();
    let auto_switcher = AutoSwitcherTelemetry {
        is_enabled: app_cfg.auto_profile_switcher.is_enabled,
        target_model: app_cfg.auto_profile_switcher.target_model.clone(),
        low_quota_threshold_percent: app_cfg.auto_profile_switcher.low_quota_threshold_percent,
        critical_threshold_percent: app_cfg.auto_profile_switcher.critical_threshold_percent,
        active_instance_id: switcher_status.active_instance_id,
        current_quota_percent: switcher_status.current_quota_percent,
    };

    // Active Prompts
    let mut running_prompts = 0;
    let mut total_prompts = 0;
    if let Ok(conn) = crate::modules::repo_db::connect_db() {
        if let Ok(mut stmt) =
            conn.prepare("SELECT count(*) FROM active_prompts WHERE status = 'running'")
        {
            if let Ok(cnt) = stmt.query_row([], |row| row.get::<_, usize>(0)) {
                running_prompts = cnt;
            }
        }
        if let Ok(mut stmt) = conn.prepare("SELECT count(*) FROM active_prompts") {
            if let Ok(cnt) = stmt.query_row([], |row| row.get::<_, usize>(0)) {
                total_prompts = cnt;
            }
        }
    }

    Ok(NodeTelemetry {
        node_name,
        local_ip,
        version: env!("CARGO_PKG_VERSION").to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        training_api_enabled: app_cfg.training_api_enabled,
        timestamp: now,
        active_account,
        accounts_summary,
        instances,
        auto_switcher,
        active_prompts_running: running_prompts,
        active_prompts_total: total_prompts,
    })
}

/// Axum GET /api/v1/training/telemetry
pub async fn handle_training_telemetry() -> Response {
    if let Err(resp) = guard_training_api() {
        return resp;
    }

    match gather_telemetry() {
        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Failed to gather telemetry: {}", e),
                "status": 500
            })),
        )
            .into_response(),
    }
}
