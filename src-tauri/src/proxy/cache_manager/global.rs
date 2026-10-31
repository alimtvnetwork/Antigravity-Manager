use super::*;

use std::sync::LazyLock;
static GLOBAL_CACHE_MANAGER: LazyLock<CacheManager> = LazyLock::new(CacheManager::new);

/// Get the global CacheManager singleton
pub fn global_cache_manager() -> &'static CacheManager {
    &GLOBAL_CACHE_MANAGER
}

// ===== Tests =====
