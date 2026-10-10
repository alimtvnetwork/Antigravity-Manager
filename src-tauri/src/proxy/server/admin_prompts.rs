//! Prompt lifecycle and system-diagnostics admin handlers.
use super::audit::log_admin_audit;
use super::dto::ErrorResponse;
use crate::modules::account;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;
use serde_json::json;

// ── Prompt Lifecycle Handlers ──

#[derive(Deserialize, Default)]
struct PromptQueryParameters {
    #[serde(rename = "instanceId")]
    instance_id: Option<String>,
    repo: Option<String>,
}

pub(crate) async fn admin_list_prompts(
    Query(params): Query<PromptQueryParameters>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit("/api/prompts", "GET", 200);
    let prompts = if let Some(ref inst) = params.instance_id {
        crate::modules::repo_db::list_running_prompts_for_instance(inst)
    } else {
        crate::modules::repo_db::list_all_prompts()
    };
    match prompts {
        Ok(list) => Ok(Json(serde_json::json!({
            "success": true,
            "data": list,
            "error": null,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
    }
}

pub(crate) async fn admin_get_prompt_tree(
    Query(params): Query<PromptQueryParameters>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit("/api/prompts/tree", "GET", 200);
    let target = params.instance_id.as_deref().unwrap_or("default");
    match crate::modules::repo_db::get_project_conversation_tree_for_instance(
        target,
        params.repo.as_deref(),
    ) {
        Ok(tree) => Ok(Json(serde_json::json!({
            "success": true,
            "data": tree,
            "error": null,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
    }
}

pub(crate) async fn admin_get_running_prompts(
    Query(params): Query<PromptQueryParameters>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit("/api/prompts/running", "GET", 200);
    let target = params.instance_id.as_deref().unwrap_or("default");
    match crate::modules::repo_db::list_running_prompts_for_instance(target) {
        Ok(running) => Ok(Json(serde_json::json!({
            "success": true,
            "data": running,
            "error": null,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
    }
}

#[derive(Deserialize)]
struct DispatchPromptPayload {
    #[serde(rename = "instanceId")]
    instance_id: Option<String>,
    #[serde(rename = "repoPath")]
    repo_path: Option<String>,
    content: String,
    model: Option<String>,
}

pub(crate) async fn admin_dispatch_prompt(
    Json(payload): Json<DispatchPromptPayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit("/api/prompts/dispatch", "POST", 200);
    let inst = payload.instance_id.unwrap_or_else(|| "default".to_string());
    let repo = payload
        .repo_path
        .unwrap_or_else(|| "scratch/test-repo".to_string());
    let now = chrono::Utc::now().timestamp();
    let prompt_id = format!("prompt-{}", uuid::Uuid::new_v4());
    let active_prompt = crate::modules::repo_db::ActivePrompt {
        id: prompt_id.clone(),
        project_id: repo.clone(),
        instance_id: inst.clone(),
        repo_path: repo.clone(),
        prompt_content: payload.content.clone(),
        model: payload.model,
        session_id: Some(format!("conv-{}", uuid::Uuid::new_v4())),
        status: "in_flight".to_string(),
        created_at: now,
        updated_at: now,
        image_payload: None,
    };

    match crate::modules::repo_db::insert_active_prompt(&active_prompt) {
        Ok(_) => Ok(Json(serde_json::json!({
            "success": true,
            "data": active_prompt,
            "error": null,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
    }
}

pub(crate) async fn admin_tick_prompt_queue(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit("/api/prompts/queue/tick", "POST", 200);
    match crate::modules::repo_db::check_and_dispatch_enqueued_prompts(None) {
        Ok(dispatched) => Ok(Json(serde_json::json!({
            "success": true,
            "data": { "dispatched_count": dispatched },
            "error": null,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
    }
}

#[derive(Deserialize, Default)]
struct PromptScopePayload {
    #[serde(rename = "instanceId")]
    instance_id: Option<String>,
    keep: Option<usize>,
}

pub(crate) async fn admin_backup_prompts(
    Json(payload): Json<Option<PromptScopePayload>>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit("/api/prompts/backup", "POST", 200);
    let target = payload
        .as_ref()
        .and_then(|p| p.instance_id.as_deref())
        .unwrap_or("default");
    match crate::modules::repo_db::backup_running_prompts(target) {
        Ok(backed) => Ok(Json(serde_json::json!({
            "success": true,
            "data": { "backed_up_count": backed },
            "error": null,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
    }
}

pub(crate) async fn admin_restore_prompts(
    Json(payload): Json<Option<PromptScopePayload>>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit("/api/prompts/restore", "POST", 200);
    let target = payload
        .as_ref()
        .and_then(|p| p.instance_id.as_deref())
        .unwrap_or("default");
    match crate::modules::repo_db::dispatch_running_prompts(target) {
        Ok(restored) => Ok(Json(serde_json::json!({
            "success": true,
            "data": { "restored_count": restored },
            "error": null,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
    }
}

pub(crate) async fn admin_purge_prompts(
    Json(payload): Json<Option<PromptScopePayload>>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit("/api/prompts/purge", "POST", 200);
    let keep_count = payload.and_then(|p| p.keep).unwrap_or(10);
    match crate::modules::agy_cleaner::prune_conversations_only(keep_count) {
        Ok(freed) => Ok(Json(serde_json::json!({
            "success": true,
            "data": { "pruned_bytes": freed.total_freed_bytes, "kept": freed.preserved_count },
            "error": null,
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )),
    }
}

pub(crate) async fn admin_get_prompt_detail(
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit(&format!("/api/prompts/{}", id), "GET", 200);
    let conn = crate::modules::repo_db::connect_db().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
    })?;
    let found = conn.query_row(
        "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload
         FROM active_prompts WHERE id = ?1",
        [&id],
        |row| {
            Ok(crate::modules::repo_db::ActivePrompt {
                id: row.get(0)?,
                project_id: row.get(1)?,
                instance_id: row.get(2)?,
                repo_path: row.get(3)?,
                prompt_content: row.get(4)?,
                model: row.get(5)?,
                session_id: row.get(6)?,
                status: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
                image_payload: row.get(10).ok(),
            })
        },
    ).ok();

    if let Some(prompt) = found {
        Ok(Json(
            serde_json::json!({ "success": true, "data": prompt, "error": null }),
        ))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Prompt '{}' not found", id),
            }),
        ))
    }
}

// ── System Diagnostics Handlers ──

pub(crate) async fn admin_get_db_stats(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit("/api/system/db-stats", "GET", 200);
    let data_dir = crate::modules::account::get_data_dir().unwrap_or_default();
    let repo_db_p = data_dir.join("repo_prompts.db");
    let security_db_p = data_dir.join("security.db");
    let accounts_db_p = data_dir.join("account.db");

    let repo_size = std::fs::metadata(&repo_db_p).map(|m| m.len()).unwrap_or(0);
    let security_size = std::fs::metadata(&security_db_p)
        .map(|m| m.len())
        .unwrap_or(0);
    let accounts_size = std::fs::metadata(&accounts_db_p)
        .map(|m| m.len())
        .unwrap_or(0);

    Ok(Json(serde_json::json!({
        "success": true,
        "data": {
            "repo_prompts_bytes": repo_size,
            "security_db_bytes": security_size,
            "accounts_db_bytes": accounts_size,
            "total_bytes": repo_size + security_size + accounts_size,
        },
        "error": null,
    })))
}

pub(crate) async fn admin_vacuum_system(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    log_admin_audit("/api/system/vacuum", "POST", 200);
    let mut vacuumed_count = 0;
    if let Ok(conn) = crate::modules::repo_db::connect_db() {
        if conn.execute("VACUUM", []).is_ok() {
            vacuumed_count += 1;
        }
    }
    if let Ok(data_dir) = crate::modules::account::get_data_dir() {
        let sec_path = data_dir.join("security.db");
        if sec_path.exists() {
            if let Ok(conn) = rusqlite::Connection::open(&sec_path) {
                if conn.execute("VACUUM", []).is_ok() {
                    vacuumed_count += 1;
                }
            }
        }
    }
    Ok(Json(serde_json::json!({
        "success": true,
        "data": { "vacuumed_databases": vacuumed_count },
        "error": null,
    })))
}
