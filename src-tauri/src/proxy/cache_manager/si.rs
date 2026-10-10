use super::*;

impl CacheManager {
    /// 创建新的 CacheManager
    pub fn new() -> Self {
        Self {
            si_cache: DashMap::new(),
            si_stats: std::sync::RwLock::new((0, 0, 0)),
            tools_cache: DashMap::new(),
            tools_stats: std::sync::RwLock::new((0, 0, 0)),
            prefix_tracker: DashMap::new(),
            prefix_stats: std::sync::RwLock::new((0, 0, 0)),
        }
    }

    // ===== Shared Utilities =====

    /// SHA256 快速哈希
    fn sha256_hex(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// 检查 Instant 是否已过期
    fn is_expired(timestamp: Instant, ttl: Duration) -> bool {
        timestamp.elapsed() > ttl
    }

    // ===== Layer 1: System Instruction Cache =====

    /// 查找已缓存的 sanitized system instruction
    ///
    /// 返回 Some(sanitized_text) 如果缓存有效，否则 None
    pub fn lookup_si(&self, raw_hash: &str) -> Option<String> {
        if let Ok(mut stats) = self.si_stats.write() {
            stats.0 += 1; // total
        }

        let mut hit = false;
        let mut expired = false;
        let mut text = None;

        if let Some(entry) = self.si_cache.get(raw_hash) {
            if !Self::is_expired(entry.timestamp, LAYER_12_TTL) {
                hit = true;
                text = Some(entry.sanitized_text.clone());
            } else {
                expired = true;
            }
        }

        if hit {
            if let Ok(mut stats) = self.si_stats.write() {
                stats.1 += 1; // hits
            }
            let mut hit_count = 0;
            if let Some(mut e) = self.si_cache.get_mut(raw_hash) {
                e.hit_count += 1;
                hit_count = e.hit_count;
            }
            tracing::debug!(
                "[CacheManager:L1-SI] HIT hash={} hit_count={}",
                &raw_hash[..raw_hash.len().min(16)],
                hit_count
            );
            return text;
        }

        if expired {
            self.si_cache.remove_if(raw_hash, |_, entry| {
                Self::is_expired(entry.timestamp, LAYER_12_TTL)
            });
            if let Ok(mut stats) = self.si_stats.write() {
                stats.2 += 1; // misses
            }
            tracing::debug!(
                "[CacheManager:L1-SI] EXPIRED hash={}",
                &raw_hash[..raw_hash.len().min(16)]
            );
            return None;
        }

        if let Ok(mut stats) = self.si_stats.write() {
            stats.2 += 1; // misses
        }
        tracing::debug!(
            "[CacheManager:L1-SI] MISS hash={}",
            &raw_hash[..raw_hash.len().min(16)]
        );
        None
    }

    /// 存入 Layer 1: raw → sanitized
    pub fn cache_si(&self, raw_hash: String, sanitized_text: String) {
        let entry = SiCacheEntry {
            sanitized_text,
            timestamp: Instant::now(),
            hit_count: 0,
        };

        self.si_cache.insert(raw_hash.clone(), entry);
        tracing::debug!(
            "[CacheManager:L1-SI] INSERT hash={}",
            &raw_hash[..raw_hash.len().min(16)]
        );

        // LRU 淘汰: 超出限制时清除过期条目
        if self.si_cache.len() > SI_CACHE_LIMIT {
            let before = self.si_cache.len();
            self.si_cache
                .retain(|_, v| !Self::is_expired(v.timestamp, LAYER_12_TTL));
            let after = self.si_cache.len();
            if before != after {
                tracing::debug!(
                    "[CacheManager:L1-SI] LRU cleanup: {} → {} (limit: {})",
                    before,
                    after,
                    SI_CACHE_LIMIT
                );
            }
        }
    }

    /// 计算 Layer 1 key: SHA256(raw instructions text)
    pub fn compute_si_key(raw_instructions: &str) -> String {
        Self::sha256_hex(raw_instructions.as_bytes())
    }
}
