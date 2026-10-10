//! Instance management REST admin handlers.
use super::audit::log_admin_audit;
use super::dto::ErrorResponse;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;
use serde_json::json;

// ── Instance Handlers ──

pub(crate) async fn admin_list_instances(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit("/api/instances", "GET", 200);
    match crate::modules::instance::list_instances() {
        Ok(instances) => {
            let enriched: Vec<serde_json::Value> = instances
                .iter()
                .map(|inst| {
                    let status_str = if inst.is_running { "running" } else { "idle" };
                    serde_json::json!({
                        "id": inst.config.id,
                        "name": inst.config.name,
                        "status": status_str,
                        "data_dir": inst.config.data_dir,
                        "is_running": inst.is_running,
                        "pid": inst.pid,
                        "bound_email": inst.config.bound_email,
                        "config": inst.config,
                    })
                })
                .collect();
            Ok(Json(serde_json::json!({
                "success": true,
                "data": enriched,
                "error": null,
            })))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
    }
}

#[derive(Deserialize, Default)]
struct CreateInstancePayload {
    name: String,
    account: Option<String>,
    from: Option<String>,
}

pub(crate) async fn admin_create_instance(
    Json(payload): Json<CreateInstancePayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit("/api/instances", "POST", 201);
    let result = if let Some(ref source) = payload.from {
        let resolved_src = crate::modules::instance::resolve_instance_id(source)
            .unwrap_or_else(|_| source.clone());
        crate::modules::instance::copy_instance(&resolved_src, payload.name, Some("full"))
    } else {
        crate::modules::instance::create_instance_with_account(
            payload.name,
            payload.account.as_deref(),
        )
    };

    match result {
        Ok(cfg) => Ok((
            StatusCode::CREATED,
            Json(serde_json::json!({
                "success": true,
                "data": cfg,
                "error": null,
            })),
        )),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e }))),
    }
}

pub(crate) async fn admin_get_instance_detail(
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit(&format!("/api/instances/{}", id), "GET", 200);
    let resolved =
        crate::modules::instance::resolve_instance_id(&id).unwrap_or_else(|_| id.clone());
    let instances = crate::modules::instance::list_instances().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    if let Some(target) = instances
        .into_iter()
        .find(|i| i.config.id == resolved || i.config.name == resolved)
    {
        Ok(Json(
            serde_json::json!({ "success": true, "data": target, "error": null }),
        ))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Instance '{}' not found", id),
            }),
        ))
    }
}

pub(crate) async fn admin_delete_instance(
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit(&format!("/api/instances/{}", id), "DELETE", 200);
    let resolved =
        crate::modules::instance::resolve_instance_id(&id).unwrap_or_else(|_| id.clone());
    match crate::modules::instance::delete_instance(&resolved) {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "data": { "id": resolved, "deleted": true },
            "error": null,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
    }
}

pub(crate) async fn admin_get_instance_status(
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit(&format!("/api/instances/{}/status", id), "GET", 200);
    let resolved =
        crate::modules::instance::resolve_instance_id(&id).unwrap_or_else(|_| id.clone());
    let instances = crate::modules::instance::list_instances().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;

    if let Some(target) = instances
        .into_iter()
        .find(|i| i.config.id == resolved || i.config.name == resolved)
    {
        Ok(Json(serde_json::json!({
            "success": true,
            "data": {
                "id": target.config.id,
                "name": target.config.name,
                "is_running": target.is_running,
                "pid": target.pid,
                "data_dir": target.config.data_dir,
            },
            "error": null,
        })))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Instance '{}' not found", id),
            }),
        ))
    }
}

#[derive(Deserialize, Default)]
struct LaunchInstancePayload {
    #[serde(rename = "repoPath")]
    repo_path: Option<String>,
}

pub(crate) async fn admin_start_instance(
    Path(id): Path<String>,
    Json(payload): Json<Option<LaunchInstancePayload>>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit(&format!("/api/instances/{}/start", id), "POST", 200);
    let resolved =
        crate::modules::instance::resolve_instance_id(&id).unwrap_or_else(|_| id.clone());
    let repo_opt = payload.and_then(|p| p.repo_path);
    let extra = repo_opt.as_ref().map(|r| vec![r.clone()]);
    match crate::modules::instance::launch_instance_with_workspaces(
        &resolved,
        extra.as_deref(),
        true,
    ) {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "data": { "id": resolved, "status": "running" },
            "error": null,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

pub(crate) async fn admin_stop_instance(
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit(&format!("/api/instances/{}/stop", id), "POST", 200);
    let resolved =
        crate::modules::instance::resolve_instance_id(&id).unwrap_or_else(|_| id.clone());
    match crate::modules::instance::stop_instance(&resolved) {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "data": { "id": resolved, "status": "stopped" },
            "error": null,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
    }
}

pub(crate) async fn admin_restart_instance(
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit(&format!("/api/instances/{}/restart", id), "POST", 200);
    let resolved =
        crate::modules::instance::resolve_instance_id(&id).unwrap_or_else(|_| id.clone());
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        crate::modules::instance::stop_instance(&resolved),
        "stop_instance",
    );
    match crate::modules::instance::launch_instance(&resolved) {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "data": { "id": resolved, "status": "restarted" },
            "error": null,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

#[derive(Deserialize)]
struct SwitchInstanceAccountPayload {
    account: Option<String>,
    #[serde(rename = "accountId")]
    account_id: Option<String>,
}

pub(crate) async fn admin_switch_instance_account(
    Path(id): Path<String>,
    Json(payload): Json<SwitchInstanceAccountPayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit(&format!("/api/instances/{}/switch", id), "POST", 200);
    let resolved =
        crate::modules::instance::resolve_instance_id(&id).unwrap_or_else(|_| id.clone());
    let target_acc = payload.account.or(payload.account_id).unwrap_or_default();
    if target_acc.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Missing account query parameter".to_string(),
            }),
        ));
    }
    match crate::modules::instance::switch_account_to_instance(&target_acc, Some(&resolved)).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "data": { "instance_id": resolved, "account": target_acc },
            "error": null,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
    }
}

#[derive(Deserialize)]
struct CloneInstancePayload {
    name: String,
}

pub(crate) async fn admin_clone_instance(
    Path(id): Path<String>,
    Json(payload): Json<CloneInstancePayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit(&format!("/api/instances/{}/clone", id), "POST", 201);
    let resolved =
        crate::modules::instance::resolve_instance_id(&id).unwrap_or_else(|_| id.clone());
    match crate::modules::instance::copy_instance(&resolved, payload.name, Some("full")) {
        Ok(cfg) => Ok((
            StatusCode::CREATED,
            Json(serde_json::json!({
                "success": true,
                "data": cfg,
                "error": null,
            })),
        )),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e }))),
    }
}

pub(crate) async fn admin_get_instance_logs(
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit(&format!("/api/instances/{}/logs", id), "GET", 200);
    let resolved =
        crate::modules::instance::resolve_instance_id(&id).unwrap_or_else(|_| id.clone());
    let reg = crate::modules::instance::load_registry().unwrap_or_default();
    let data_dir = reg
        .instances
        .iter()
        .find(|i| i.id == resolved)
        .map(|i| i.data_dir.clone())
        .unwrap_or_else(|| {
            crate::modules::instance::get_default_antigravity_data_dir()
                .to_string_lossy()
                .to_string()
        });
    let log_file = std::path::Path::new(&data_dir).join("antigravity_startup.log");
    let logs = if log_file.exists() {
        std::fs::read_to_string(&log_file).unwrap_or_default()
    } else {
        String::new()
    };
    Ok(Json(serde_json::json!({
        "success": true,
        "data": { "instance_id": resolved, "logs": logs },
        "error": null,
    })))
}
