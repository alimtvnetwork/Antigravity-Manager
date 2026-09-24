//! Machine Training REST API Engine
//!
//! Provides external endpoints for seeking into node telemetry, ingesting training/learning signals,
//! and remotely modifying machine and model routing states. Controlled via settings toggle.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

/// Database file path for persisting training logs and reinforcement learning events
pub fn get_training_db_path() -> Result<PathBuf, String> {
    let data_dir = crate::modules::account::get_data_dir()
        .map_err(|e| format!("Failed to get data dir: {}", e))?;
    Ok(data_dir.join("training_vault.db"))
}

/// Connect to the training database with WAL mode and 5000ms busy timeout
pub fn connect_training_db() -> Result<Connection, String> {
    let path = get_training_db_path()?;
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let conn =
        Connection::open(&path).map_err(|e| format!("Failed to open training database: {}", e))?;
    let _ = conn.pragma_update(None, "journal_mode", "WAL");
    let _ = conn.pragma_update(None, "busy_timeout", 5000);
    init_tables(&conn)?;
    Ok(conn)
}

fn init_tables(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS training_logs (
            id TEXT PRIMARY KEY,
            session_id TEXT,
            model TEXT,
            prompt_type TEXT,
            input_tokens INTEGER,
            output_tokens INTEGER,
            latency_ms INTEGER,
            success INTEGER NOT NULL DEFAULT 1,
            score REAL,
            feedback TEXT,
            adjust_routing INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create training_logs table: {}", e))?;

    Ok(())
}

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

fn guard_training_api() -> Result<(), Response> {
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

fn load_all_accounts_list() -> Vec<crate::models::Account> {
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

// ============================================================================
// Learning & Feedback Ingestion
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnRequest {
    pub session_id: Option<String>,
    pub model: Option<String>,
    pub prompt_type: Option<String>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub latency_ms: Option<i64>,
    pub success: Option<bool>,
    pub score: Option<f64>,
    pub feedback: Option<String>,
    pub adjust_routing: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LearnResponse {
    pub status: String,
    pub log_id: String,
    pub timestamp: i64,
    pub message: String,
    pub model: Option<String>,
    pub updated_routing: bool,
}

pub fn ingest_learning(req: LearnRequest) -> Result<LearnResponse, String> {
    let conn = connect_training_db()?;
    let log_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();
    let success_int = if req.success.unwrap_or(true) { 1 } else { 0 };
    let adjust_int = if req.adjust_routing.unwrap_or(false) {
        1
    } else {
        0
    };

    conn.execute(
        "INSERT INTO training_logs (
            id, session_id, model, prompt_type, input_tokens, output_tokens,
            latency_ms, success, score, feedback, adjust_routing, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            log_id,
            req.session_id,
            req.model,
            req.prompt_type,
            req.input_tokens,
            req.output_tokens,
            req.latency_ms,
            success_int,
            req.score,
            req.feedback,
            adjust_int,
            now
        ],
    )
    .map_err(|e| format!("Failed to persist training log: {}", e))?;

    let mut updated_routing = false;
    if req.adjust_routing.unwrap_or(false) {
        if let Some(ref target_model) = req.model {
            if let Ok(mut app_cfg) = crate::modules::config::load_app_config() {
                app_cfg.auto_profile_switcher.target_model = target_model.clone();
                let _ = crate::modules::config::save_app_config(&app_cfg);
                updated_routing = true;
                crate::modules::logger::log_info(&format!(
                    "[TrainingAPI] Feedback dynamically updated auto-switcher target model to '{}'",
                    target_model
                ));
            }
        }
    }

    Ok(LearnResponse {
        status: "ok".to_string(),
        log_id,
        timestamp: now,
        message: "Training signal ingested and logged successfully".to_string(),
        model: req.model,
        updated_routing,
    })
}

/// Axum POST /api/v1/training/learn
pub async fn handle_training_learn(Json(payload): Json<LearnRequest>) -> Response {
    if let Err(resp) = guard_training_api() {
        return resp;
    }

    match ingest_learning(payload) {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": format!("Failed to ingest learning: {}", e),
                "status": 500
            })),
        )
            .into_response(),
    }
}

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
