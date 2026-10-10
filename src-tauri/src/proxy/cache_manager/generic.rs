use super::*;

impl CacheManager {
    // ===== 向后兼容 API =====

    /// 查找前缀: 兼容旧 API，等同于 lookup_prefix
    #[inline]
    pub fn lookup(&self, hash: &str) -> Option<String> {
        self.lookup_prefix(hash)
    }

    /// 插入前缀: 兼容旧 API
    #[inline]
    pub fn insert(&self, hash: String, cache_name: String, ttl_secs: Option<u64>) {
        self.insert_prefix(
            hash,
            cache_name,
            String::new(), // si_hash unknown in legacy path
            String::new(), // tools_hash unknown in legacy path
            String::new(), // model unknown in legacy path
            ttl_secs,
        );
    }

    // ===== Stats & Management =====

    /// 获取分层统计
    pub fn get_layer_stats(&self) -> LayerStats {
        let si = self.si_stats.read().unwrap();
        let tools = self.tools_stats.read().unwrap();
        let prefix = self.prefix_stats.read().unwrap();

        let total_implicit = self
            .prefix_tracker
            .iter()
            .map(|e| e.implicit_hit_count)
            .sum();
        let total_explicit = self
            .prefix_tracker
            .iter()
            .map(|e| e.explicit_hit_count)
            .sum();

        LayerStats {
            si_total: si.0,
            si_hits: si.1,
            si_misses: si.2,
            tools_total: tools.0,
            tools_hits: tools.1,
            tools_misses: tools.2,
            prefix_total: prefix.0,
            prefix_hits: prefix.1,
            prefix_misses: prefix.2,
            total_implicit_hits: total_implicit,
            total_explicit_hits: total_explicit,
            active_si_entries: self.si_cache.len(),
            active_tools_entries: self.tools_cache.len(),
            active_prefix_entries: self.prefix_tracker.len(),
        }
    }

    /// 淘汰所有过期条目
    pub fn evict_expired(&self) -> usize {
        let si_before = self.si_cache.len();
        self.si_cache
            .retain(|_, v| !Self::is_expired(v.timestamp, LAYER_12_TTL));
        let tools_before = self.tools_cache.len();
        self.tools_cache
            .retain(|_, v| !Self::is_expired(v.timestamp, LAYER_12_TTL));
        let prefix_before = self.prefix_tracker.len();
        let now = Instant::now();
        self.prefix_tracker.retain(|_, v| v.expires_at > now);

        let total = (si_before - self.si_cache.len())
            + (tools_before - self.tools_cache.len())
            + (prefix_before - self.prefix_tracker.len());

        if total > 0 {
            tracing::debug!(
                "[CacheManager] Evicted {} expired entries across all layers",
                total
            );
        }
        total
    }

    /// 清空所有层缓存和统计
    pub fn clear(&self) {
        self.si_cache.clear();
        self.tools_cache.clear();
        self.prefix_tracker.clear();
        if let Ok(mut s) = self.si_stats.write() {
            *s = (0, 0, 0);
        }
        if let Ok(mut s) = self.tools_stats.write() {
            *s = (0, 0, 0);
        }
        if let Ok(mut s) = self.prefix_stats.write() {
            *s = (0, 0, 0);
        }
        tracing::info!("[CacheManager] All layers cleared");
    }
}
