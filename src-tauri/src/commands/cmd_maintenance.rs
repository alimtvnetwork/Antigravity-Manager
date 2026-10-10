use super::*;

#[tauri::command]
pub async fn clear_log_cache() -> Result<(), String> {
    modules::logger::clear_logs()
}

/// 清理 Antigravity 应用缓存
/// 用于解决登录失败、版本验证错误等问题
#[tauri::command]
pub async fn clear_antigravity_cache() -> Result<modules::cache::ClearResult, String> {
    modules::cache::clear_antigravity_cache(None)
}

/// 获取 Antigravity 缓存路径列表（用于预览）
#[tauri::command]
pub async fn get_antigravity_cache_paths() -> Result<Vec<String>, String> {
    Ok(modules::cache::get_existing_cache_paths()
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}

/// Pre-flight simulation for Antigravity conversation and cache cleanup
#[tauri::command]
pub async fn preflight_antigravity_clean(
    keep_count: Option<usize>,
) -> Result<modules::agy_cleaner::PreflightReport, String> {
    let keep = keep_count.unwrap_or(10);
    Ok(modules::agy_cleaner::preflight_check(keep))
}

/// Prune older Antigravity conversations and scrub ephemeral cache with temp staging
#[tauri::command]
pub async fn prune_antigravity_conversations(
    keep_count: Option<usize>,
) -> Result<modules::agy_cleaner::PruneResult, String> {
    let keep = keep_count.unwrap_or(10);
    modules::agy_cleaner::prune_and_clean(keep)
}

/// Prune older Antigravity conversations only (preserving caches) with temp staging
#[tauri::command]
pub async fn prune_antigravity_conversations_only(
    keep_count: Option<usize>,
) -> Result<modules::agy_cleaner::PruneResult, String> {
    let keep = keep_count.unwrap_or(10);
    modules::agy_cleaner::prune_conversations_only(keep)
}

/// Undo the last or specific Antigravity conversation pruning transaction
#[tauri::command]
pub async fn undo_antigravity_prune(
    transaction_id: Option<String>,
) -> Result<modules::agy_cleaner::UndoResult, String> {
    modules::agy_cleaner::undo_prune(transaction_id.as_deref())
}
