//! Dedicated backend module for multi-round prompt verification scheduling.
//!
//! Implements strict FIFO queue insertion (ORDER BY created_at ASC, id ASC)
//! and 500ms safety throttling between sequential rounds to prevent
//! .antigravity_resume_task.json write collisions.

use super::detection::{extract_image_payload_or_path, resume_task_document};
use super::models::ActivePrompt;
use super::prompts_crud::save_or_requeue_prompt;
use super::schema::connect_db;
use super::tree::invalidate_prompt_tree_cache;
use crate::error::AppError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationEnqueueRequest {
    pub instance_id: String,
    pub repo_path: String,
    pub conversation_id: String,
    pub style: String,
    pub round_prompts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationEnqueueResult {
    pub enqueued_count: usize,
    pub conversation_id: String,
    pub message: String,
}

/// Enqueue multiple verification rounds for an instance and optional session.
///
/// Enforces Invariant I1: strict FIFO queue insertion (`ORDER BY created_at ASC, id ASC`).
/// Enforces Invariant I6/I2: 500ms safety sleep between repeated rounds (`tokio::time::sleep`)
/// to eliminate `.antigravity_resume_task.json` write collisions.
/// Calls `invalidate_prompt_tree_cache(Some(instance_id))`.
pub async fn enqueue_verification_rounds(
    instance_id: &str,
    prompts: Vec<String>,
    session_id: Option<String>,
) -> Result<Vec<String>, String> {
    if prompts.is_empty() {
        return Err("No verification round prompts provided".to_string());
    }

    let clean_inst = if instance_id.trim().is_empty() || instance_id == "__default__" {
        "default"
    } else {
        instance_id.trim()
    };
    let canonical_inst = crate::modules::instance::resolve_instance_id(clean_inst)
        .unwrap_or_else(|_| clean_inst.to_string());

    // Resolve repo_path from database if possible
    let resolved_repo_path = {
        let conn = connect_db()?;
        let sid = session_id.as_deref().unwrap_or("");
        let path_by_session: Option<String> = conn
            .query_row(
                "SELECT repo_path FROM active_prompts WHERE session_id = ?1 OR id = ?1 LIMIT 1",
                rusqlite::params![sid],
                |r| r.get(0),
            )
            .ok();
        path_by_session.or_else(|| {
            conn.query_row(
                "SELECT repo_path FROM running_projects WHERE instance_id = ?1 ORDER BY updated_at DESC LIMIT 1",
                rusqlite::params![&canonical_inst],
                |r| r.get(0),
            )
            .ok()
        }).unwrap_or_default()
    };

    let total_rounds = prompts.len();
    let mut enqueued_ids = Vec::with_capacity(total_rounds);
    let base_now = Utc::now().timestamp();

    for (idx, prompt_body) in prompts.into_iter().enumerate() {
        let round_num = idx + 1;
        let prompt_clone = prompt_body.clone();
        let inst_clone = canonical_inst.clone();
        let session_clone = session_id.clone();
        let r_path = resolved_repo_path.clone();
        let current_created_at = base_now + (idx as i64);

        let prompt_id = format!(
            "verify-{}-{}-r{}",
            session_clone.as_deref().unwrap_or("anon"),
            base_now,
            round_num
        );
        let pid_clone = prompt_id.clone();

        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let clean_repo_name = Path::new(&r_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "project".to_string());

            let active_prompt = ActivePrompt {
                id: pid_clone.clone(),
                project_id: clean_repo_name,
                instance_id: inst_clone.clone(),
                repo_path: r_path.clone(),
                prompt_content: prompt_clone.clone(),
                model: Some("gemini-2.5-pro".to_string()),
                session_id: session_clone.clone(),
                status: "queued".to_string(),
                created_at: current_created_at,
                updated_at: current_created_at,
                image_payload: None,
            };

            save_or_requeue_prompt(&active_prompt)?;

            // If repo_path exists, safely write .antigravity_resume_task.json
            if !r_path.trim().is_empty() {
                let ws_dir = PathBuf::from(&r_path);
                if ws_dir.exists() {
                    let (extracted_img, img_paths) = extract_image_payload_or_path(&prompt_clone);
                    let mut payload = resume_task_document(
                        &active_prompt,
                        "queued",
                        current_created_at,
                        &img_paths,
                    );
                    if let Some(img) = extracted_img {
                        payload["image_payload"] = serde_json::json!(img);
                    }
                    payload["auto_boot"] = serde_json::json!(false);
                    let serialized = serde_json::to_string_pretty(&payload).unwrap_or_default();
                    // Justification: resume snapshot is a convenience file; state is already recorded in active_prompts
                    crate::error::record_ignored(
                        fs::write(ws_dir.join(".antigravity_resume_task.json"), &serialized),
                        "write verification resume task snapshot",
                    );
                    // Justification: resume snapshot is a convenience file; state is already recorded in active_prompts
                    crate::error::record_ignored(
                        fs::write(
                            ws_dir.join(format!(".antigravity_resume_task.{}.json", inst_clone)),
                            &serialized,
                        ),
                        "write per-instance verification resume task snapshot",
                    );
                }
            }

            Ok(())
        })
        .await
        .map_err(|e| format!("Tokio blocking task join failure: {e}"))??;

        enqueued_ids.push(prompt_id);
        info!(
            "Enqueued verification round {}/{} for instance {} session {:?}",
            round_num, total_rounds, canonical_inst, session_id
        );

        // Enforce Invariant I6/I2: 500ms safety sleep between repeated rounds (skip after last round)
        if round_num < total_rounds {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    // Call invalidate_prompt_tree_cache(Some(instance_id))
    let inst_for_inv = canonical_inst.clone();
    tokio::task::spawn_blocking(move || {
        invalidate_prompt_tree_cache(Some(&inst_for_inv));
    })
    .await
    .map_err(|e| format!("Cache invalidation task join failure: {e}"))?;

    Ok(enqueued_ids)
}

/// Enqueue multiple verification rounds from a structured VerificationEnqueueRequest.
pub async fn enqueue_verification_rounds_request(
    req: VerificationEnqueueRequest,
) -> Result<VerificationEnqueueResult, AppError> {
    if req.round_prompts.is_empty() {
        return Err(AppError::Config(
            "No verification round prompts provided".into(),
        ));
    }
    let conv_id = req.conversation_id.clone();
    let prompt_ids = enqueue_verification_rounds(
        &req.instance_id,
        req.round_prompts,
        Some(req.conversation_id),
    )
    .await
    .map_err(|e| AppError::Process(e))?;

    Ok(VerificationEnqueueResult {
        enqueued_count: prompt_ids.len(),
        conversation_id: conv_id,
        message: format!(
            "Successfully enqueued {} verification round(s)",
            prompt_ids.len()
        ),
    })
}

/// Alias for `enqueue_verification_rounds_request`.
pub async fn enqueue_verification_rounds_req(
    req: VerificationEnqueueRequest,
) -> Result<VerificationEnqueueResult, AppError> {
    enqueue_verification_rounds_request(req).await
}
