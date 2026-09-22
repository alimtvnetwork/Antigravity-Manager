//! Antigravity Conversation Pruning, Cache Cleaning, and Undo Engine.
//!
//! Provides cross-platform discovery of Antigravity conversation databases and
//! application caches, recency-based retention pruning, safe staging in the OS
//! temporary directory for undo recovery, and transaction rollback.

use crate::modules::config;
use crate::modules::logger;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Summary metadata for a single Antigravity conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationItem {
    pub conversation_id: String,
    pub title: String,
    pub preview: String,
    pub step_count: i64,
    pub workspace_uris: String,
    pub last_modified_time: String,
    pub db_path: String,
    pub file_size: u64,
    pub is_preserved: bool,
}

/// Pre-flight report for dry-run inspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightReport {
    pub total_conversations: usize,
    pub keep_count: usize,
    pub preserved_count: usize,
    pub pruned_count: usize,
    pub total_conversation_bytes: u64,
    pub projected_reclaimed_bytes: u64,
    pub cache_paths_count: usize,
    pub cache_bytes: u64,
    pub conversations_to_preserve: Vec<ConversationItem>,
    pub conversations_to_prune: Vec<ConversationItem>,
    pub cache_targets: Vec<String>,
    pub staging_dir: String,
}

/// Result of an applied pruning operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PruneResult {
    pub transaction_id: String,
    pub keep_count: usize,
    pub preserved_count: usize,
    pub pruned_count: usize,
    pub pruned_bytes: u64,
    pub cache_cleared_bytes: u64,
    pub total_freed_bytes: u64,
    pub staging_dir: String,
    pub errors: Vec<String>,
}

/// Result of an undo restoration operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoResult {
    pub transaction_id: String,
    pub restored_conversations: usize,
    pub restored_bytes: u64,
    pub errors: Vec<String>,
}

/// Staged item record in transaction manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedItem {
    pub conversation_id: String,
    pub db_staged_path: String,
    pub db_original_path: String,
    pub brain_staged_path: Option<String>,
    pub brain_original_path: Option<String>,
    pub size: u64,
}

/// Manifest saved in the temp staging directory for rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionManifest {
    pub transaction_id: String,
    pub timestamp: String,
    pub keep_count: usize,
    pub items: Vec<StagedItem>,
}

/// Get the base Antigravity user data directory (~/.gemini/antigravity)
pub fn get_gemini_base_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".gemini").join("antigravity"))
}

/// Get the temporary staging directory for recoverable backups
pub fn get_temp_staging_dir() -> PathBuf {
    std::env::temp_dir().join("antigravity-cleaner-backup")
}

/// Safely copy or move a file/directory
fn safe_move_path(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    if let Some(parent) = dst.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    if let Err(_rename_err) = fs::rename(src, dst) {
        // Fallback to copy + remove for cross-device moves
        if src.is_dir() {
            copy_dir_all(src, dst)?;
            fs::remove_dir_all(src)?;
        } else {
            fs::copy(src, dst)?;
            fs::remove_file(src)?;
        }
    }

    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let dest_child = dst.join(entry.file_name());
        if entry_type.is_dir() {
            copy_dir_all(&entry.path(), &dest_child)?;
        } else {
            fs::copy(entry.path(), dest_child)?;
        }
    }
    Ok(())
}

/// Calculate directory size recursively
pub fn get_dir_size_bytes(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    if path.is_file() {
        return fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    }
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let child = entry.path();
            if child.is_file() {
                total += fs::metadata(&child).map(|m| m.len()).unwrap_or(0);
            } else if child.is_dir() {
                total += get_dir_size_bytes(&child);
            }
        }
    }
    total
}

/// Scan all conversations and sort by recency (newest first)
pub fn scan_conversations(keep_count: usize) -> Vec<ConversationItem> {
    let base_dir = match get_gemini_base_dir() {
        Some(dir) => dir,
        None => return Vec::new(),
    };

    let conv_dir = base_dir.join("conversations");
    if !conv_dir.is_dir() {
        return Vec::new();
    }

    // Load indexed metadata from conversation_summaries.db if present
    let summaries_db_path = base_dir.join("conversation_summaries.db");
    let mut summaries_map: std::collections::HashMap<
        String,
        (String, String, i64, String, String),
    > = std::collections::HashMap::new();

    if summaries_db_path.is_file() {
        if let Ok(conn) = Connection::open_with_flags(
            &summaries_db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        ) {
            let query = "SELECT conversation_id, title, preview, step_count, workspace_uris, last_modified_time FROM conversation_summaries";
            if let Ok(mut stmt) = conn.prepare(query) {
                let rows = stmt.query_map([], |row| {
                    let cid: String = row.get(0)?;
                    let title: String = row.get(1).unwrap_or_default();
                    let preview: String = row.get(2).unwrap_or_default();
                    let step_count: i64 = row.get(3).unwrap_or(0);
                    let uris: String = row.get(4).unwrap_or_default();
                    let last_mod: String = row.get(5).unwrap_or_default();
                    Ok((cid, title, preview, step_count, uris, last_mod))
                });

                if let Ok(items) = rows {
                    for item in items.flatten() {
                        summaries_map.insert(item.0, (item.1, item.2, item.3, item.4, item.5));
                    }
                }
            }
        }
    }

    let mut conversations = Vec::new();
    if let Ok(entries) = fs::read_dir(&conv_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let is_db = path.extension().and_then(|s| s.to_str()) == Some("db");
            if !is_db {
                continue;
            }

            let cid = match path.file_stem().and_then(|s| s.to_str()) {
                Some(stem) => stem.to_string(),
                None => continue,
            };

            let metadata = fs::metadata(&path).ok();
            let file_size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            let mtime_secs = metadata
                .as_ref()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);

            let (title, preview, step_count, workspace_uris, last_mod) =
                if let Some(entry_meta) = summaries_map.get(&cid) {
                    (
                        entry_meta.0.clone(),
                        entry_meta.1.clone(),
                        entry_meta.2,
                        entry_meta.3.clone(),
                        entry_meta.4.clone(),
                    )
                } else {
                    (
                        String::new(),
                        String::new(),
                        0,
                        String::new(),
                        mtime_secs.to_string(),
                    )
                };

            conversations.push(ConversationItem {
                conversation_id: cid,
                title,
                preview,
                step_count,
                workspace_uris,
                last_modified_time: last_mod,
                db_path: path.to_string_lossy().to_string(),
                file_size,
                is_preserved: false,
            });
        }
    }

    // Sort descending by recency
    conversations.sort_by(|a, b| {
        b.last_modified_time
            .cmp(&a.last_modified_time)
            .then_with(|| b.file_size.cmp(&a.file_size))
    });

    // Mark top keep_count as preserved
    for (idx, conv) in conversations.iter_mut().enumerate() {
        let is_within_keep = idx < keep_count;
        if is_within_keep {
            conv.is_preserved = true;
        }
    }

    conversations
}

/// Perform a dry-run pre-flight check
pub fn preflight_check(keep_count: usize) -> PreflightReport {
    let convs = scan_conversations(keep_count);
    let mut preserved = Vec::new();
    let mut pruned = Vec::new();

    let mut total_bytes = 0u64;
    let mut projected_reclaimed = 0u64;

    for conv in convs {
        total_bytes += conv.file_size;
        if conv.is_preserved {
            preserved.push(conv);
        } else {
            projected_reclaimed += conv.file_size;
            pruned.push(conv);
        }
    }

    let existing_cache_paths = crate::modules::cache::get_existing_cache_paths();
    let cache_paths_count = existing_cache_paths.len();
    let mut cache_bytes = 0u64;
    let mut cache_targets = Vec::new();

    for p in &existing_cache_paths {
        let sz = get_dir_size_bytes(p);
        cache_bytes += sz;
        cache_targets.push(format!(
            "{} ({:.2} MB)",
            p.display(),
            sz as f64 / 1024.0 / 1024.0
        ));
    }

    let staging_dir = get_temp_staging_dir().to_string_lossy().to_string();

    PreflightReport {
        total_conversations: preserved.len() + pruned.len(),
        keep_count,
        preserved_count: preserved.len(),
        pruned_count: pruned.len(),
        total_conversation_bytes: total_bytes,
        projected_reclaimed_bytes: projected_reclaimed,
        cache_paths_count,
        cache_bytes,
        conversations_to_preserve: preserved,
        conversations_to_prune: pruned,
        cache_targets,
        staging_dir,
    }
}

/// Execute conversation pruning with safe temporary staging for undo
pub fn prune_and_clean(keep_count: usize) -> Result<PruneResult, String> {
    let base_dir = get_gemini_base_dir()
        .ok_or_else(|| "Failed to determine Antigravity base directory".to_string())?;

    let staging_root = get_temp_staging_dir();
    let tx_id = format!("tx_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
    let tx_dir = staging_root.join(&tx_id);
    let tx_conv_dir = tx_dir.join("conversations");
    let tx_brain_dir = tx_dir.join("brain");

    fs::create_dir_all(&tx_conv_dir)
        .map_err(|e| format!("Failed to create temp staging directory: {}", e))?;
    fs::create_dir_all(&tx_brain_dir)
        .map_err(|e| format!("Failed to create temp staging brain directory: {}", e))?;

    let convs = scan_conversations(keep_count);
    let mut staged_items = Vec::new();
    let mut pruned_bytes = 0u64;
    let mut errors = Vec::new();

    let brain_root = base_dir.join("brain");

    for conv in &convs {
        if conv.is_preserved {
            continue;
        }

        let db_src = PathBuf::from(&conv.db_path);
        let db_filename = match db_src.file_name() {
            Some(name) => name,
            None => continue,
        };
        let db_dest = tx_conv_dir.join(db_filename);

        let sz = conv.file_size;
        match safe_move_path(&db_src, &db_dest) {
            Ok(_) => {
                pruned_bytes += sz;
                let brain_src = brain_root.join(&conv.conversation_id);
                let brain_dest = tx_brain_dir.join(&conv.conversation_id);
                let mut brain_staged_str = None;
                let mut brain_orig_str = None;

                if brain_src.is_dir() {
                    let bsz = get_dir_size_bytes(&brain_src);
                    if let Ok(_) = safe_move_path(&brain_src, &brain_dest) {
                        pruned_bytes += bsz;
                        brain_staged_str = Some(brain_dest.to_string_lossy().to_string());
                        brain_orig_str = Some(brain_src.to_string_lossy().to_string());
                    }
                }

                staged_items.push(StagedItem {
                    conversation_id: conv.conversation_id.clone(),
                    db_staged_path: db_dest.to_string_lossy().to_string(),
                    db_original_path: db_src.to_string_lossy().to_string(),
                    brain_staged_path: brain_staged_str,
                    brain_original_path: brain_orig_str,
                    size: sz,
                });
            }
            Err(e) => {
                errors.push(format!("Failed to stage {}: {}", conv.conversation_id, e));
            }
        }
    }

    // Clear application caches
    let cache_result = crate::modules::cache::clear_antigravity_cache(None);
    let cache_cleared_bytes = match cache_result {
        Ok(res) => res.total_size_freed,
        Err(e) => {
            errors.push(format!("Cache clear warning: {}", e));
            0
        }
    };

    // Clean ephemeral brain subfolders (crashes, logs, tempmediaStorage)
    let static_ephemeral = [
        base_dir.join("crashes"),
        base_dir.join("log"),
        base_dir.join("brain").join("tempmediaStorage"),
        base_dir.join("brain").join("cache"),
    ];

    for eph in &static_ephemeral {
        if eph.is_dir() {
            if let Ok(entries) = fs::read_dir(eph) {
                for item in entries.flatten() {
                    let path = item.path();
                    let _ = if path.is_dir() {
                        fs::remove_dir_all(&path)
                    } else {
                        fs::remove_file(&path)
                    };
                }
            }
        }
    }

    // Write manifest for reversible undo
    let manifest = TransactionManifest {
        transaction_id: tx_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        keep_count,
        items: staged_items,
    };

    if let Ok(manifest_json) = serde_json::to_string_pretty(&manifest) {
        let manifest_path = tx_dir.join("manifest.json");
        let _ = fs::write(manifest_path, manifest_json);
    }

    let preserved_count = convs.iter().filter(|c| c.is_preserved).count();
    let pruned_count = manifest.items.len();
    let total_freed = pruned_bytes + cache_cleared_bytes;

    logger::log_info(&format!(
        "[AgyCleaner] Pruned {} conversations ({:.2} MB), kept {}, freed {:.2} MB total. Staged at {}",
        pruned_count,
        pruned_bytes as f64 / 1024.0 / 1024.0,
        preserved_count,
        total_freed as f64 / 1024.0 / 1024.0,
        tx_dir.display()
    ));

    Ok(PruneResult {
        transaction_id: tx_id,
        keep_count,
        preserved_count,
        pruned_count,
        pruned_bytes,
        cache_cleared_bytes,
        total_freed_bytes: total_freed,
        staging_dir: tx_dir.to_string_lossy().to_string(),
        errors,
    })
}

/// Undo a specific or the most recent pruning transaction from temp staging
pub fn undo_prune(target_tx_id: Option<&str>) -> Result<UndoResult, String> {
    let staging_root = get_temp_staging_dir();
    if !staging_root.is_dir() {
        return Err("No temporary staging directory found to restore from.".to_string());
    }

    let mut candidate_tx_dirs = Vec::new();
    if let Ok(entries) = fs::read_dir(&staging_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            let is_tx = path.is_dir()
                && path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .map(|s| s.starts_with("tx_"))
                    .unwrap_or(false);
            if is_tx {
                candidate_tx_dirs.push(path);
            }
        }
    }

    candidate_tx_dirs.sort_by(|a, b| b.cmp(a));
    if candidate_tx_dirs.is_empty() {
        return Err("No backup transactions found in temporary storage.".to_string());
    }

    let target_dir = match target_tx_id {
        Some(id) => candidate_tx_dirs
            .into_iter()
            .find(|d| d.file_name().and_then(|s| s.to_str()) == Some(id))
            .ok_or_else(|| format!("Transaction '{}' not found", id))?,
        None => candidate_tx_dirs.remove(0),
    };

    let manifest_path = target_dir.join("manifest.json");
    if !manifest_path.is_file() {
        return Err(format!(
            "Transaction manifest not found or already reverted in {}",
            target_dir.display()
        ));
    }

    let manifest_str = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;
    let manifest: TransactionManifest = serde_json::from_str(&manifest_str)
        .map_err(|e| format!("Failed to parse manifest: {}", e))?;

    let mut restored_count = 0usize;
    let mut restored_bytes = 0u64;
    let mut errors = Vec::new();

    for item in &manifest.items {
        let staged_db = PathBuf::from(&item.db_staged_path);
        let orig_db = PathBuf::from(&item.db_original_path);

        if staged_db.exists() {
            match safe_move_path(&staged_db, &orig_db) {
                Ok(_) => {
                    restored_count += 1;
                    restored_bytes += item.size;
                }
                Err(e) => {
                    errors.push(format!(
                        "Failed to restore DB {}: {}",
                        item.conversation_id, e
                    ));
                }
            }
        }

        if let (Some(ref b_staged), Some(ref b_orig)) =
            (&item.brain_staged_path, &item.brain_original_path)
        {
            let b_staged_p = PathBuf::from(b_staged);
            let b_orig_p = PathBuf::from(b_orig);
            if b_staged_p.exists() {
                let _ = safe_move_path(&b_staged_p, &b_orig_p);
            }
        }
    }

    // Rename manifest to mark transaction as reverted
    let reverted_manifest_path = target_dir.join("manifest.json.reverted");
    let _ = fs::rename(&manifest_path, reverted_manifest_path);

    logger::log_info(&format!(
        "[AgyCleaner] Reverted transaction {}: restored {} conversations ({:.2} MB)",
        manifest.transaction_id,
        restored_count,
        restored_bytes as f64 / 1024.0 / 1024.0
    ));

    Ok(UndoResult {
        transaction_id: manifest.transaction_id,
        restored_conversations: restored_count,
        restored_bytes,
        errors,
    })
}

/// Start background conversation cleanup ticker (runs every N hours if enabled)
pub fn start_cleanup_daemon() {
    tauri::async_runtime::spawn(async move {
        logger::log_info("[AgyCleaner] Background cleanup daemon initialized.");
        loop {
            let app_config = config::load_app_config().unwrap_or_default();
            let cleanup_cfg = app_config.conversation_cleanup;

            let interval_hours = if cleanup_cfg.interval_hours == 0 {
                1
            } else {
                cleanup_cfg.interval_hours
            };

            tokio::time::sleep(std::time::Duration::from_secs(interval_hours as u64 * 3600)).await;

            let current_config = config::load_app_config().unwrap_or_default();
            if current_config.conversation_cleanup.is_enabled {
                let keep_count = current_config.conversation_cleanup.keep_count;
                logger::log_info(&format!(
                    "[AgyCleaner] Periodic auto-cleanup triggered (keeping top {} conversations)",
                    keep_count
                ));
                match prune_and_clean(keep_count) {
                    Ok(report) => {
                        logger::log_info(&format!(
                            "[AgyCleaner] Periodic cleanup finished: pruned {} conversations, freed {} bytes",
                            report.pruned_count, report.total_freed_bytes
                        ));
                    }
                    Err(e) => {
                        logger::log_warn(&format!("[AgyCleaner] Periodic cleanup error: {}", e));
                    }
                }
            }
        }
    });
}
