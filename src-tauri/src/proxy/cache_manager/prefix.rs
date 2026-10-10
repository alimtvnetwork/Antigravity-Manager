use super::*;

impl CacheManager {
    /// 查找前缀跟踪条目中的 cache_name
    ///
    /// 返回 Some(cache_name) 如果存在有效缓存，否则 None
    pub fn lookup_prefix(&self, hash: &str) -> Option<String> {
        if let Ok(mut stats) = self.prefix_stats.write() {
            stats.0 += 1;
        }

        let mut hit = false;
        let mut expired = false;
        let mut cache_name = None;

        if let Some(entry) = self.prefix_tracker.get(hash) {
            if entry.expires_at > Instant::now() {
                hit = true;
                cache_name = Some(entry.cache_name.clone());
            } else {
                expired = true;
            }
        }

        if hit {
            if let Ok(mut stats) = self.prefix_stats.write() {
                stats.1 += 1;
            }
            if let Some(ref name) = cache_name {
                tracing::debug!(
                    "[CacheManager:L3-Prefix] HIT hash={} cache_name={}",
                    &hash[..hash.len().min(16)],
                    name
                );
            }
            return cache_name;
        }

        if expired {
            self.prefix_tracker
                .remove_if(hash, |_, entry| entry.expires_at <= Instant::now());
            if let Ok(mut stats) = self.prefix_stats.write() {
                stats.2 += 1;
            }
            tracing::debug!(
                "[CacheManager:L3-Prefix] EXPIRED hash={}",
                &hash[..hash.len().min(16)]
            );
            return None;
        }

        if let Ok(mut stats) = self.prefix_stats.write() {
            stats.2 += 1;
        }
        tracing::debug!(
            "[CacheManager:L3-Prefix] MISS hash={}",
            &hash[..hash.len().min(16)]
        );
        None
    }

    /// 插入或更新 Layer 3 条目
    ///
    /// cache_name: Gemini 返回的缓存资源名。目前使用 hash 本身作为 fallback。
    /// si_hash / tools_hash: 关联的 Layer 1/2 key，用于追踪
    pub fn insert_prefix(
        &self,
        hash: String,
        cache_name: String,
        si_hash: String,
        tools_hash: String,
        model: String,
        ttl_secs: Option<u64>,
    ) {
        let ttl = ttl_secs.unwrap_or(3600);
        let now = Instant::now();
        let entry = PrefixTrackingEntry {
            si_hash,
            tools_hash,
            cache_name,
            created_at: now,
            expires_at: now + Duration::from_secs(ttl),
            implicit_hit_count: 0,
            explicit_hit_count: 0,
            model,
        };
        tracing::info!(
            "[CacheManager:L3-Prefix] INSERT hash={} ttl={}s",
            &hash[..hash.len().min(16)],
            ttl
        );
        self.prefix_tracker.insert(hash, entry);

        if self.prefix_tracker.len() > PREFIX_TRACKER_LIMIT {
            let before = self.prefix_tracker.len();
            let now = Instant::now();
            self.prefix_tracker.retain(|_, v| v.expires_at > now);
            let after = self.prefix_tracker.len();
            if before != after {
                tracing::debug!(
                    "[CacheManager:L3-Prefix] LRU cleanup: {} → {} (limit: {})",
                    before,
                    after,
                    PREFIX_TRACKER_LIMIT
                );
            }
        }
    }

    /// 记录一次隐式缓存命中（来自响应的 cachedContentTokenCount > 0）
    pub fn record_implicit_hit(&self, hash: &str) {
        if let Some(mut entry) = self.prefix_tracker.get_mut(hash) {
            entry.implicit_hit_count += 1;
            tracing::debug!(
                "[CacheManager:L3-Prefix] implicit hit #{} for hash={}",
                entry.implicit_hit_count,
                &hash[..hash.len().min(16)]
            );
        }
    }

    /// 记录一次显式缓存命中
    pub fn record_explicit_hit(&self, hash: &str) {
        if let Some(mut entry) = self.prefix_tracker.get_mut(hash) {
            entry.explicit_hit_count += 1;
        }
    }
}
