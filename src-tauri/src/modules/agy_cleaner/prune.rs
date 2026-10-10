use crate::modules::logger;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

use super::*;

/// Execute conversation pruning with safe temporary staging for undo
pub fn prune_and_clean(keep_count: usize) -> Result<PruneResult, String> {
    prune_internal(keep_count, true)
}

/// Execute conversation-only pruning without scrubbing application caches
pub fn prune_conversations_only(keep_count: usize) -> Result<PruneResult, String> {
    prune_internal(keep_count, false)
}

pub(crate) fn prune_internal(keep_count: usize, clear_caches: bool) -> Result<PruneResult, String> {
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
                let conv_base_opt = db_src.parent().and_then(|p| p.parent());
                let brain_src_opt =
                    conv_base_opt.map(|b| b.join("brain").join(&conv.conversation_id));
                let mut brain_staged_str = None;
                let mut brain_orig_str = None;

                if let Some(brain_src) = brain_src_opt {
                    if brain_src.is_dir() {
                        let brain_dest = tx_brain_dir.join(&conv.conversation_id);
                        let bsz = get_dir_size_bytes(&brain_src);
                        if safe_move_path(&brain_src, &brain_dest).is_ok() {
                            pruned_bytes += bsz;
                            brain_staged_str = Some(brain_dest.to_string_lossy().to_string());
                            brain_orig_str = Some(brain_src.to_string_lossy().to_string());
                        }
                    }
                }

                // Also clean from conversation_summaries.db if present
                if let Some(conv_base) = conv_base_opt {
                    let summaries_db = conv_base.join("conversation_summaries.db");
                    if summaries_db.is_file() {
                        if let Ok(conn) = Connection::open(&summaries_db) {
                            // Justification: best-effort DB statement (idempotent schema/cleanup write); failure logged
                            crate::error::record_ignored(
                                conn.execute(
                                    "DELETE FROM conversation_summaries WHERE conversation_id = ?",
                                    [&conv.conversation_id],
                                ),
                                "db execute",
                            );
                        }
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

    let cache_cleared_bytes = if clear_caches {
        // Clear application caches
        let cache_result = crate::modules::cache::clear_antigravity_cache(None);
        let bytes = match cache_result {
            Ok(res) => res.total_size_freed,
            Err(e) => {
                errors.push(format!("Cache clear warning: {}", e));
                0
            }
        };

        // Clean ephemeral brain subfolders across all candidate directories
        for base_dir in &get_gemini_candidate_dirs() {
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
                            // Justification: best-effort conditional filesystem cleanup; a leftover file is harmless
                            crate::error::record_ignored(
                                if path.is_dir() {
                                    fs::remove_dir_all(&path)
                                } else {
                                    fs::remove_file(&path)
                                },
                                "conditional fs remove",
                            );
                        }
                    }
                }
            }
        }
        bytes
    } else {
        0
    };

    // Write manifest for reversible undo
    let manifest = TransactionManifest {
        transaction_id: tx_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        keep_count,
        items: staged_items,
    };

    if let Ok(manifest_json) = serde_json::to_string_pretty(&manifest) {
        let manifest_path = tx_dir.join("manifest.json");
        // Justification: best-effort file write; failure is logged and surfaces on the next read
        crate::error::record_ignored(fs::write(manifest_path, manifest_json), "fs::write");
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
                // Justification: best-effort file move; logged for diagnosis
                crate::error::record_ignored(
                    safe_move_path(&b_staged_p, &b_orig_p),
                    "safe_move_path",
                );
            }
        }
    }

    // Rename manifest to mark transaction as reverted
    let reverted_manifest_path = target_dir.join("manifest.json.reverted");
    // Justification: best-effort file move; logged for diagnosis
    crate::error::record_ignored(fs::rename(&manifest_path, reverted_manifest_path), "rename");

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
