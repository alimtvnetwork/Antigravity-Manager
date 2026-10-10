//! Repo DB: detection

use super::failed_commands::decode_uri_to_path;
use super::liveness::is_prompt_running_for_project;
use super::models::{ActivePrompt, RunningProject};
use super::schema::connect_db;
use super::tree::invalidate_prompt_tree_cache;
use chrono::Utc;
use rusqlite::params;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// Scan active workspace storage for an instance and discover projects
pub fn detect_running_projects(instance_id: &str) -> Result<Vec<RunningProject>, String> {
    let resolved_id =
        crate::modules::instance::resolve_instance_id(instance_id).unwrap_or_else(|_| {
            if instance_id == "__default__" || instance_id.is_empty() {
                "default".to_string()
            } else {
                instance_id.to_string()
            }
        });
    let target_id = resolved_id.as_str();
    let registry = crate::modules::instance::load_registry()?;
    let instance = registry
        .instances
        .iter()
        .find(|i| i.id == target_id || (target_id == "default" && i.is_default))
        .ok_or_else(|| {
            format!(
                "Instance '{}' (resolved: '{}') not found",
                instance_id, target_id
            )
        })?;

    let pids =
        crate::modules::instance::find_pids_for_data_dir(&instance.data_dir, instance.is_default);
    let is_instance_active = crate::modules::instance::is_instance_running(
        &instance.id,
        &instance.data_dir,
        instance.pid,
    ) && !pids.is_empty();

    let storage_dir = PathBuf::from(&instance.data_dir)
        .join("User")
        .join("workspaceStorage");

    let mut projects = Vec::new();
    let now = Utc::now().timestamp();

    if storage_dir.exists() {
        if let Ok(entries) = fs::read_dir(&storage_dir) {
            for entry in entries.flatten() {
                let ws_folder = entry.path();
                if ws_folder.is_dir() {
                    let ws_json_path = ws_folder.join("workspace.json");
                    if ws_json_path.exists() {
                        if let Ok(content) = fs::read_to_string(&ws_json_path) {
                            if let Ok(json_val) =
                                serde_json::from_str::<serde_json::Value>(&content)
                            {
                                let folder_uri =
                                    json_val.get("folder").and_then(|v| v.as_str()).or_else(|| {
                                        json_val.get("configuration").and_then(|v| v.as_str())
                                    });

                                if let Some(uri) = folder_uri {
                                    let raw_path = decode_uri_to_path(uri);
                                    let project_path = Path::new(&raw_path);
                                    let repo_name = project_path
                                        .file_name()
                                        .map(|n| n.to_string_lossy().to_string())
                                        .unwrap_or_else(|| "unnamed-project".to_string());

                                    let base_project_id = format!(
                                        "{}-{}",
                                        repo_name.to_lowercase(),
                                        entry.file_name().to_string_lossy()
                                    );
                                    let composite_id =
                                        format!("{}__{}", base_project_id, target_id);

                                    let is_project_active = is_instance_active
                                        && is_prompt_running_for_project(&raw_path, target_id);

                                    let pid_opt = pids.first().copied();
                                    crate::modules::logger::log_instance_prompt_audit(
                                        target_id,
                                        &instance.name,
                                        &repo_name,
                                        &raw_path,
                                        &ws_folder.to_string_lossy(),
                                        pid_opt,
                                        "WorkspaceStorageScan + ProcessLiveness",
                                        is_project_active,
                                        if is_project_active {
                                            "ACTIVE_IN_FLIGHT_TASKS"
                                        } else if !is_instance_active {
                                            "INSTANCE_PROCESS_DEAD"
                                        } else {
                                            "IDLE_NO_ACTIVE_TASKS"
                                        },
                                    );

                                    projects.push(RunningProject {
                                        id: composite_id,
                                        instance_id: target_id.to_string(),
                                        repo_name,
                                        repo_path: raw_path,
                                        workspace_storage_path: Some(
                                            ws_folder.to_string_lossy().to_string(),
                                        ),
                                        is_running: is_project_active,
                                        last_detected_at: now,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Persist discovered projects into repo database safely without wiping fallback entries
    if let Ok(conn) = connect_db() {
        // Prune stale projects that have not been detected within the last 120 seconds
        // Justification: persistence of discovery scan results is best-effort; the scan results are returned regardless
        crate::error::record_ignored(
            conn.execute(
                "DELETE FROM running_projects WHERE last_detected_at < (?1 - 120)",
                params![now],
            ),
            "prune stale running projects",
        );

        // Only delete corrupted entries where BOTH workspace_storage_path is NULL AND the repo_path does not exist on disk
        // Justification: persistence of discovery scan results is best-effort; the scan results are returned regardless
        crate::error::record_ignored(
            conn.execute(
                "DELETE FROM running_projects
             WHERE instr(id, '__') = 0
                OR (workspace_storage_path IS NULL AND (repo_path IS NULL OR repo_path = ''))",
                [],
            ),
            "delete corrupted running projects",
        );

        let discovered_ids: std::collections::HashSet<String> =
            projects.iter().map(|p| p.id.clone()).collect();
        if let Ok(mut stmt) = conn.prepare(
            "SELECT id, repo_path, workspace_storage_path FROM running_projects WHERE instance_id = ?1",
        ) {
            let existing_rows: Vec<(String, String, Option<String>)> = stmt
                .query_map(params![target_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .map(|iter| iter.flatten().collect())
                .unwrap_or_default();
            for (old_id, repo_path, wspath_opt) in existing_rows {
                let ws_exists = wspath_opt
                    .as_deref()
                    .map(|p| Path::new(p).exists())
                    .unwrap_or(false);
                let repo_exists = Path::new(&repo_path).exists();
                let is_valid_fallback = wspath_opt.is_none() && repo_exists;
                if !discovered_ids.contains(&old_id) && !ws_exists && !is_valid_fallback {
                    // Justification: persistence of discovery scan results is best-effort; the scan results are returned regardless
                    crate::error::record_ignored(
                        conn.execute(
                        "DELETE FROM running_projects WHERE id = ?1",
                        params![&old_id],
                    ),
                        "delete undiscovered running project",
                    );
                }
            }
        }
        for p in &projects {
            let running_int = if p.is_running { 1 } else { 0 };
            // Justification: persistence of discovery scan results is best-effort; the scan results are returned regardless
            crate::error::record_ignored(
                conn.execute(
                "INSERT INTO running_projects
                 (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(id) DO UPDATE SET
                    instance_id = excluded.instance_id,
                    repo_name = excluded.repo_name,
                    repo_path = excluded.repo_path,
                    workspace_storage_path = COALESCE(excluded.workspace_storage_path, running_projects.workspace_storage_path),
                    is_running = excluded.is_running,
                    last_detected_at = excluded.last_detected_at,
                    updated_at = excluded.updated_at",
                params![
                    &p.id,
                    &p.instance_id,
                    &p.repo_name,
                    &p.repo_path,
                    &p.workspace_storage_path,
                    running_int,
                    p.last_detected_at,
                    now,
                ],
            ),
                "upsert discovered running project",
            );
            if !p.is_running {
                // Justification: persistence of discovery scan results is best-effort; the scan results are returned regardless
                crate::error::record_ignored(
                    conn.execute(
                        "UPDATE running_projects SET is_running = 0, updated_at = ?1 WHERE id = ?2",
                        params![now, &p.id],
                    ),
                    "mark non-running project",
                );
            }
        }

        // If instance is not active, ensure all projects belonging to this instance are marked is_running = 0
        if !is_instance_active {
            // Justification: persistence of discovery scan results is best-effort; the scan results are returned regardless
            crate::error::record_ignored(
                conn.execute(
                "UPDATE running_projects SET is_running = 0, updated_at = ?1 WHERE instance_id = ?2",
                params![now, target_id],
            ),
                "mark instance projects not running",
            );
        }
    }

    if !is_instance_active {
        for p in &mut projects {
            p.is_running = false;
        }
    }

    invalidate_prompt_tree_cache(Some(target_id));
    Ok(projects)
}

/// Extract image data URI, file path, or markdown image reference from text
pub fn extract_image_payload_or_path(content: &str) -> (Option<String>, Vec<String>) {
    let mut found_paths: Vec<String> = Vec::new();
    let mut raw_payload = None;

    if content.contains("data:image/") {
        if let Some(start) = content.find("data:image/") {
            let tail = &content[start..];
            let end = tail
                .find('"')
                .or_else(|| tail.find('\''))
                .or_else(|| tail.find(' '))
                .or_else(|| tail.find(')'))
                .unwrap_or(tail.len());
            raw_payload = Some(tail[..end].to_string());
        }
    }

    let img_exts = [".png", ".jpg", ".jpeg", ".webp", ".gif", ".svg", ".bmp"];
    for token in content.split_whitespace() {
        let clean = token.trim_matches(|c| {
            c == '"'
                || c == '\''
                || c == '('
                || c == ')'
                || c == '['
                || c == ']'
                || c == '<'
                || c == '>'
        });
        let lower = clean.to_lowercase();
        if img_exts.iter().any(|ext| lower.ends_with(ext)) {
            let p_str = clean
                .strip_prefix("file:///")
                .unwrap_or_else(|| clean.strip_prefix("file://").unwrap_or(clean));
            if !found_paths.contains(&p_str.to_string()) {
                found_paths.push(p_str.to_string());
            }
        }
    }

    if raw_payload.is_none() && !found_paths.is_empty() {
        raw_payload = Some(found_paths.join(";"));
    }

    (raw_payload, found_paths)
}

/// Text to store for a live conversation. The transcript's latest USER_INPUT wins.
/// When that line is missing, the conversation preview is the prompt that was running.
pub fn live_prompt_text(preview: &str, transcript: &str) -> Option<String> {
    for line in transcript.lines().rev() {
        if !line.contains("USER_INPUT") {
            continue;
        }
        let Ok(val) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if val.get("type").and_then(|t| t.as_str()) != Some("USER_INPUT") {
            continue;
        }
        if let Some(txt) = val.get("content").and_then(|c| c.as_str()) {
            if !txt.trim().is_empty() {
                return Some(txt.to_string());
            }
        }
    }
    let trimmed = preview.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Resume file the IDE reads after a switch. `session_id` is the same conversation.
pub fn resume_task_document(
    prompt: &ActivePrompt,
    status: &str,
    at: i64,
    image_paths: &[String],
) -> serde_json::Value {
    let has_image = prompt.image_payload.is_some() || !image_paths.is_empty();
    serde_json::json!({
        "prompt_id": prompt.id,
        "project_id": prompt.project_id,
        "instance_id": prompt.instance_id,
        "repo_path": prompt.repo_path,
        "prompt_content": prompt.prompt_content,
        "model": prompt.model,
        "session_id": prompt.session_id,
        "conversation_id": prompt.session_id,
        "image_payload": prompt.image_payload,
        "image_paths": image_paths,
        "has_image": has_image,
        "auto_boot": true,
        "status": status,
        "backed_up_at": at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_prompt_uses_preview_when_the_transcript_has_no_user_input() {
        let preview = live_prompt_text("keep the same chat", "{\"type\":\"MODEL\"}\n").unwrap();
        assert_eq!(preview, "keep the same chat");
        let user = live_prompt_text(
            "preview",
            "{\"type\":\"USER_INPUT\",\"content\":\"real prompt\"}",
        )
        .unwrap();
        assert_eq!(user, "real prompt");
        assert!(live_prompt_text("  ", "").is_none());
    }

    #[test]
    fn resume_document_keeps_the_same_conversation_id() {
        let prompt = ActivePrompt {
            id: "prompt-conv-9".to_string(),
            project_id: "proj".to_string(),
            instance_id: "default".to_string(),
            repo_path: "D:/work/app".to_string(),
            prompt_content: "keep going".to_string(),
            model: None,
            session_id: Some("conv-9".to_string()),
            status: "backed_up".to_string(),
            created_at: 10,
            updated_at: 10,
            image_payload: None,
        };
        let doc = resume_task_document(&prompt, "backed_up", 10, &[]);
        assert_eq!(doc["session_id"], "conv-9");
        assert_eq!(doc["conversation_id"], "conv-9");
        assert_eq!(doc["prompt_content"], "keep going");
    }

    #[test]
    fn test_resume_task_document_canonical_keys() {
        let prompt = ActivePrompt {
            id: "prompt-123".to_string(),
            project_id: "test-proj".to_string(),
            instance_id: "default".to_string(),
            repo_path: "/test/repo".to_string(),
            prompt_content: "test prompt content".to_string(),
            model: Some("gemini-2.5-pro".to_string()),
            session_id: Some("conv-abc-456".to_string()),
            status: "dispatched".to_string(),
            created_at: 1000,
            updated_at: 1000,
            image_payload: None,
        };
        let doc = resume_task_document(&prompt, "dispatched", 1000, &[]);
        assert_eq!(doc["prompt_id"], "prompt-123");
        assert_eq!(doc["project_id"], "test-proj");
        assert_eq!(doc["instance_id"], "default");
        assert_eq!(doc["repo_path"], "/test/repo");
        assert_eq!(doc["prompt_content"], "test prompt content");
        assert_eq!(doc["model"], "gemini-2.5-pro");
        assert_eq!(doc["session_id"], "conv-abc-456");
        assert_eq!(doc["conversation_id"], "conv-abc-456");
        assert_eq!(doc["auto_boot"], true);
        assert_eq!(doc["status"], "dispatched");
        assert_eq!(doc["backed_up_at"], 1000);
    }
}
