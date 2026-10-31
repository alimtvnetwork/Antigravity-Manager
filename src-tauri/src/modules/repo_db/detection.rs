//! Repo DB: detection

use super::failed_commands::decode_uri_to_path;
use super::liveness::{
    collect_active_workspace_uris_for_instance, is_prompt_running_for_project,
    is_prompt_running_for_project_with_summaries,
};
use super::models::{
    ActivePrompt, ProjectAuditRationaleType, PromptAuditCriteriaType, RunningProject,
};
use super::schema::connect_db;
use chrono::Utc;
use once_cell::sync::Lazy;
use rusqlite::params;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Clone)]
struct CachedRunningProjects {
    projects: Vec<RunningProject>,
    cached_at: i64,
}

static RUNNING_PROJECTS_CACHE: Lazy<RwLock<HashMap<String, CachedRunningProjects>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

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

    let now = Utc::now().timestamp();
    if let Ok(guard) = RUNNING_PROJECTS_CACHE.read() {
        if let Some(cached) = guard.get(target_id) {
            if now - cached.cached_at < 15 {
                return Ok(cached.projects.clone());
            }
        }
    }
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

    // If storage dir does not exist, return empty project list early
    if !storage_dir.exists() {
        if let Ok(mut guard) = RUNNING_PROJECTS_CACHE.write() {
            guard.insert(
                target_id.to_string(),
                CachedRunningProjects {
                    projects: projects.clone(),
                    cached_at: now,
                },
            );
        }
        return Ok(projects);
    }

    let Ok(entries) = fs::read_dir(&storage_dir) else {
        if let Ok(mut guard) = RUNNING_PROJECTS_CACHE.write() {
            guard.insert(
                target_id.to_string(),
                CachedRunningProjects {
                    projects: projects.clone(),
                    cached_at: now,
                },
            );
        }
        return Ok(projects);
    };

    let active_ws_uris = collect_active_workspace_uris_for_instance(target_id);

    for entry in entries.flatten() {
        let ws_folder = entry.path();
        if !ws_folder.is_dir() {
            continue;
        }

        let ws_json_path = ws_folder.join("workspace.json");
        if !ws_json_path.exists() {
            continue;
        }

        let Ok(content) = fs::read_to_string(&ws_json_path) else {
            continue;
        };

        let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&content) else {
            continue;
        };

        let folder_uri = json_val
            .get("folder")
            .and_then(|v| v.as_str())
            .or_else(|| json_val.get("configuration").and_then(|v| v.as_str()));

        let Some(uri) = folder_uri else {
            continue;
        };

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
        let composite_id = format!("{}__{}", base_project_id, target_id);

        let is_project_active = is_instance_active
            && is_prompt_running_for_project_with_summaries(
                &raw_path,
                target_id,
                Some(&active_ws_uris),
            );

        let pid_opt = pids.first().copied();
        let criteria = PromptAuditCriteriaType::WorkspaceStorageScanAndProcessLiveness;
        let rationale =
            ProjectAuditRationaleType::from_liveness(is_project_active, is_instance_active);

        if is_project_active {
            crate::modules::logger::log_instance_prompt_audit(
                target_id,
                &instance.name,
                &repo_name,
                &raw_path,
                &ws_folder.to_string_lossy(),
                pid_opt,
                criteria.as_str(),
                is_project_active,
                rationale.as_str(),
            );
        } else {
            let line = crate::modules::logger::format_instance_prompt_audit(
                target_id,
                &instance.name,
                &repo_name,
                &raw_path,
                &ws_folder.to_string_lossy(),
                pid_opt,
                criteria.as_str(),
                is_project_active,
                rationale.as_str(),
            );
            tracing::debug!("{line}");
        }

        projects.push(RunningProject {
            id: composite_id,
            instance_id: target_id.to_string(),
            repo_name,
            repo_path: raw_path,
            workspace_storage_path: Some(ws_folder.to_string_lossy().to_string()),
            is_running: is_project_active,
            last_detected_at: now,
        });
    }

    // Persist discovered projects into repo database safely without wiping fallback entries
    if let Ok(conn) = connect_db() {
        // Prune stale projects that have not been detected within the last 120 seconds
        // Justification: persistence of discovery scan results is best-effort; the scan results are returned regardless
        crate::error::record_ignored(
            conn.execute(
                "DELETE FROM running_projects WHERE last_detected_at < (?1 - 120)
                 AND id NOT IN (SELECT project_id FROM active_prompts)",
                params![now],
            ),
            "prune stale running projects",
        );

        // Only delete corrupted entries where BOTH workspace_storage_path is NULL AND the repo_path does not exist on disk
        // Justification: persistence of discovery scan results is best-effort; the scan results are returned regardless
        crate::error::record_ignored(
            conn.execute(
                "DELETE FROM running_projects
             WHERE (instr(id, '__') = 0
                OR (workspace_storage_path IS NULL AND (repo_path IS NULL OR repo_path = '')))
               AND id NOT IN (SELECT project_id FROM active_prompts)",
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
                        "DELETE FROM running_projects WHERE id = ?1
                         AND id NOT IN (SELECT project_id FROM active_prompts)",
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

    if let Ok(mut guard) = RUNNING_PROJECTS_CACHE.write() {
        guard.insert(
            target_id.to_string(),
            CachedRunningProjects {
                projects: projects.clone(),
                cached_at: now,
            },
        );
    }

    Ok(projects)
}

/// Extract ALL image data URIs and file paths from text.
///
/// Returns `(data_uris, file_paths)`:
/// - `data_uris`: every `data:image/` URI found, verbatim (byte-identical, no
///   re-encoding), in document order, deduplicated.
/// - `file_paths`: every local image path reference, as before.
///
/// This is the multi-image successor of `extract_image_payload_or_path`,
/// which only captured the FIRST data URI and silently dropped the rest.
pub fn extract_image_uris_or_paths(content: &str) -> (Vec<String>, Vec<String>) {
    let mut data_uris: Vec<String> = Vec::new();
    let mut found_paths: Vec<String> = Vec::new();

    // Collect every data:image/ URI in document order.
    let mut search_from = 0;
    while let Some(rel) = content[search_from..].find("data:image/") {
        let start = search_from + rel;
        let tail = &content[start..];
        let end = tail
            .find('"')
            .or_else(|| tail.find('\''))
            .or_else(|| tail.find(' '))
            .or_else(|| tail.find(')'))
            .unwrap_or(tail.len());
        // "data:image/" itself is 11 bytes, so `end >= 11` and the cursor
        // always advances — no infinite loop.
        let uri = tail[..end].to_string();
        if !data_uris.contains(&uri) {
            data_uris.push(uri);
        }
        search_from = start + end;
    }

    let img_exts = [".png", ".jpg", ".jpeg", ".webp", ".gif", ".svg", ".bmp"];

    // Also scan explicit bracket attachments [label](file;<path>), @[label](file;<path>), or @[file;<path>]
    let mut search_from = 0;
    while let Some(rel) = content[search_from..].find('[') {
        let is_at_prefixed = rel > 0 && content.as_bytes()[search_from + rel - 1] == b'@';
        let start = search_from + rel + 1;
        if let Some(end_rel) = content[start..].find(']') {
            let inner = content[start..start + end_rel].trim();
            let after_bracket = content[start + end_rel + 1..].trim_start();
            if after_bracket.starts_with("(file;") || after_bracket.starts_with("(FILE;") {
                if let Some(paren_end) = after_bracket.find(')') {
                    let path_part = after_bracket[6..paren_end].trim();
                    let lower = path_part.to_lowercase();
                    if img_exts.iter().any(|ext| lower.ends_with(ext)) {
                        let p_str = path_part.strip_prefix("file:///").unwrap_or_else(|| {
                            path_part.strip_prefix("file://").unwrap_or(path_part)
                        });
                        if !found_paths.contains(&p_str.to_string()) {
                            found_paths.push(p_str.to_string());
                        }
                    }
                }
            } else if is_at_prefixed {
                let without_file = inner
                    .strip_prefix("file;")
                    .or_else(|| inner.strip_prefix("FILE;"))
                    .unwrap_or(inner)
                    .trim();
                let lower = without_file.to_lowercase();
                if img_exts.iter().any(|ext| lower.ends_with(ext)) {
                    let p_str = without_file.strip_prefix("file:///").unwrap_or_else(|| {
                        without_file.strip_prefix("file://").unwrap_or(without_file)
                    });
                    if !found_paths.contains(&p_str.to_string()) {
                        found_paths.push(p_str.to_string());
                    }
                }
            }
            search_from = start + end_rel + 1;
        } else {
            break;
        }
    }

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
                || c == '@'
        });
        // Strip canonical "file;" prefix if present
        let without_file_prefix = clean
            .strip_prefix("file;")
            .or_else(|| clean.strip_prefix("FILE;"))
            .unwrap_or(clean)
            .trim();

        let lower = without_file_prefix.to_lowercase();
        if img_exts.iter().any(|ext| lower.ends_with(ext)) {
            let p_str = without_file_prefix
                .strip_prefix("file:///")
                .unwrap_or_else(|| {
                    without_file_prefix
                        .strip_prefix("file://")
                        .unwrap_or(without_file_prefix)
                });
            if !found_paths.contains(&p_str.to_string()) {
                found_paths.push(p_str.to_string());
            }
        }
    }

    (data_uris, found_paths)
}

/// Serialize extracted data URIs for the single-TEXT `image_payload` /
/// `images_payload` columns.
///
/// - 0 URIs → `None`
/// - 1 URI → the URI verbatim (legacy single-string format, byte-identical)
/// - 2+ URIs → JSON array string, so no URI is silently dropped
pub fn serialize_image_uris(uris: &[String]) -> Option<String> {
    match uris.len() {
        0 => None,
        1 => Some(uris[0].clone()),
        _ => serde_json::to_string(uris).ok(),
    }
}

/// Parse a stored `image_payload` / `images_payload` value back into URIs.
///
/// Handles both the legacy verbatim single-URI format and the JSON array
/// format produced by `serialize_image_uris` for multi-image prompts.
pub fn parse_image_uris(payload: &str) -> Vec<String> {
    let trimmed = payload.trim();
    if trimmed.starts_with('[') {
        if let Ok(uris) = serde_json::from_str::<Vec<String>>(trimmed) {
            return uris;
        }
    }
    if trimmed.is_empty() {
        Vec::new()
    } else {
        vec![trimmed.to_string()]
    }
}

/// Resolve the effective image URIs for a prompt: the stored payload wins
/// (parsed back into URIs); otherwise the freshly extracted URIs are used.
/// File paths travel in the separate `image_paths` channel and are not
/// included here.
pub fn effective_image_uris(
    stored_payload: Option<&str>,
    extracted_uris: Vec<String>,
) -> Vec<String> {
    match stored_payload {
        Some(s) => {
            let parsed = parse_image_uris(s);
            if parsed.is_empty() {
                extracted_uris
            } else {
                parsed
            }
        }
        None => extracted_uris,
    }
}

/// Compose the storable single-TEXT `image_payload` value from a stored
/// payload, freshly extracted URIs, and file paths.
///
/// Precedence: stored payload → extracted URIs → legacy joined-paths fallback
/// (preserves the historical behavior where paths-only prompts stored
/// `"p1;p2"` in `image_payload`) → `None`.
pub fn compose_image_payload(
    stored: Option<&str>,
    extracted_uris: Vec<String>,
    paths: &[String],
) -> Option<String> {
    let uris = effective_image_uris(stored, extracted_uris);
    if let Some(ser) = serialize_image_uris(&uris) {
        return Some(ser);
    }
    if paths.is_empty() {
        None
    } else {
        Some(paths.join(";"))
    }
}

/// Extract image data URI, file path, or markdown image reference from text.
///
/// Backward-compatible wrapper: returns the FIRST `data:image/` URI (legacy
/// behavior) plus all file paths. New code that needs every image should use
/// `extract_image_uris_or_paths` instead.
pub fn extract_image_payload_or_path(content: &str) -> (Option<String>, Vec<String>) {
    let (uris, found_paths) = extract_image_uris_or_paths(content);
    let mut raw_payload = uris.into_iter().next();

    if raw_payload.is_none() && !found_paths.is_empty() {
        raw_payload = Some(found_paths.join(";"));
    }

    if let Some(ss_token) = extract_screenshot_id_token(content) {
        if let Some(ref mut payload) = raw_payload {
            if !payload.contains(&ss_token) {
                payload.push_str(&format!(";screenshot_id={}", ss_token));
            }
        } else {
            raw_payload = Some(format!("screenshot_id={}", ss_token));
        }
    }

    (raw_payload, found_paths)
}

/// Extract screenshot ID token (e.g. SS_ID_987654321, screenshot_id=..., screenshot_id:...) from text
pub fn extract_screenshot_id_token(content: &str) -> Option<String> {
    for part in content.split_whitespace() {
        for token in part.split(';') {
            let clean = token.trim_matches(|c: char| {
                c == '"'
                    || c == '\''
                    || c == '('
                    || c == ')'
                    || c == '['
                    || c == ']'
                    || c == '{'
                    || c == '}'
                    || c == '<'
                    || c == '>'
                    || c == ','
                    || c == ';'
            });
            if clean.starts_with("SS_ID_") || clean.starts_with("ss_id_") {
                return Some(clean.to_string());
            }
            if clean.to_lowercase().starts_with("screenshot_id=") {
                let val = clean[14..].trim();
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
            if clean.to_lowercase().starts_with("screenshot_id:") {
                let val = clean[14..].trim();
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
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
    let seq_code = prompt
        .session_id
        .as_deref()
        .and_then(super::sequences::get_sequence_code_for_conversation);
    serde_json::json!({
        "prompt_id": prompt.id,
        "project_id": prompt.project_id,
        "workspace_id": prompt.project_id,
        "sequence_code": seq_code,
        "seq_code": seq_code,
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
        assert_eq!(doc["workspace_id"], "test-proj");
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

    #[test]
    fn test_extract_screenshot_id_token() {
        assert_eq!(
            extract_screenshot_id_token("Test prompt with SS_ID_12345"),
            Some("SS_ID_12345".to_string())
        );
        assert_eq!(
            extract_screenshot_id_token("E2E verification with [SS_ID_12345] in brackets"),
            Some("SS_ID_12345".to_string())
        );
        assert_eq!(
            extract_screenshot_id_token("multimodal payload;screenshot_id=SS_ID_12345;format=png"),
            Some("SS_ID_12345".to_string())
        );
        assert_eq!(
            extract_screenshot_id_token("prompt screenshot_id:SS_ID_12345 done"),
            Some("SS_ID_12345".to_string())
        );
        assert_eq!(
            extract_screenshot_id_token("prompt with no screenshot token"),
            None
        );
    }

    #[test]
    fn test_extract_bracketed_image_attachment() {
        let content = "Please check this bug:\nAttachments:\n1. @[file;C:\\path\\to\\image.png]\n2. @[file;file:///path/to/test.webp]\n3. @[file;relative/shot.png]\n4. @[legacy/path.png]";
        let (data_uris, paths) = extract_image_uris_or_paths(content);
        assert!(data_uris.is_empty());
        assert_eq!(
            paths,
            vec![
                "C:\\path\\to\\image.png".to_string(),
                "path/to/test.webp".to_string(),
                "relative/shot.png".to_string(),
                "legacy/path.png".to_string(),
            ]
        );
    }

    #[test]
    fn test_extract_canonical_bracketed_image_attachment() {
        let content = "Please check this bug:\nAttachments:\n1. @[file;fixtures/screenshots/image.png]\n2. @[file;file:///path/to/test.webp]\n3. @[file;relative/shot.png]";
        let (data_uris, paths) = extract_image_uris_or_paths(content);
        assert!(data_uris.is_empty());
        assert_eq!(
            paths,
            vec![
                "fixtures/screenshots/image.png".to_string(),
                "path/to/test.webp".to_string(),
                "relative/shot.png".to_string()
            ]
        );
    }

    #[test]
    fn test_extract_mixed_bracketed_image_attachments() {
        let content = "Check:\n1. @[file;assets/canonical.png]\n2. @[assets/legacy.png]";
        let (data_uris, paths) = extract_image_uris_or_paths(content);
        assert!(data_uris.is_empty());
        assert_eq!(
            paths,
            vec![
                "assets/canonical.png".to_string(),
                "assets/legacy.png".to_string()
            ]
        );
    }
}
