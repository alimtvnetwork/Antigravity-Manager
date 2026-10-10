use super::*;

use std::sync::LazyLock;
static GLOBAL_CACHE_MANAGER: LazyLock<CacheManager> = LazyLock::new(CacheManager::new);

/// 获取全局 CacheManager 单例
pub fn global_cache_manager() -> &'static CacheManager {
    &GLOBAL_CACHE_MANAGER
}

// ===== Tests =====
