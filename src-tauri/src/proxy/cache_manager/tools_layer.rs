use super::*;

impl CacheManager {
    // ===== Layer 2: Tools Cache =====

    /// Find cached processed tools
    ///
    /// Returns Some(tools_json_string) if cache is valid, otherwise None
    pub fn lookup_tools(&self, raw_hash: &str) -> Option<String> {
        if let Ok(mut stats) = self.tools_stats.write() {
            stats.0 += 1;
        }

        let mut hit = false;
        let mut expired = false;
        let mut json = None;

        if let Some(entry) = self.tools_cache.get(raw_hash) {
            if !Self::is_expired(entry.timestamp, LAYER_12_TTL) {
                hit = true;
                json = Some(entry.tools_json.clone());
            } else {
                expired = true;
            }
        }

        if hit {
            if let Ok(mut stats) = self.tools_stats.write() {
                stats.1 += 1;
            }
            if let Some(mut e) = self.tools_cache.get_mut(raw_hash) {
                e.hit_count += 1;
            }
            tracing::debug!(
                "[CacheManager:L2-Tools] HIT hash={}",
                &raw_hash[..raw_hash.len().min(16)]
            );
            return json;
        }

        if expired {
            self.tools_cache.remove_if(raw_hash, |_, entry| {
                Self::is_expired(entry.timestamp, LAYER_12_TTL)
            });
            if let Ok(mut stats) = self.tools_stats.write() {
                stats.2 += 1;
            }
            tracing::debug!(
                "[CacheManager:L2-Tools] EXPIRED hash={}",
                &raw_hash[..raw_hash.len().min(16)]
            );
            return None;
        }

        if let Ok(mut stats) = self.tools_stats.write() {
            stats.2 += 1;
        }
        tracing::debug!(
            "[CacheManager:L2-Tools] MISS hash={}",
            &raw_hash[..raw_hash.len().min(16)]
        );
        None
    }

    /// Save to Layer 2: raw → processed tools JSON
    pub fn cache_tools(&self, raw_hash: String, tools_json: String) {
        let entry = ToolsCacheEntry {
            tools_json,
            timestamp: Instant::now(),
            hit_count: 0,
        };

        self.tools_cache.insert(raw_hash.clone(), entry);
        tracing::debug!(
            "[CacheManager:L2-Tools] INSERT hash={}",
            &raw_hash[..raw_hash.len().min(16)]
        );

        if self.tools_cache.len() > TOOLS_CACHE_LIMIT {
            let before = self.tools_cache.len();
            self.tools_cache
                .retain(|_, v| !Self::is_expired(v.timestamp, LAYER_12_TTL));
            let after = self.tools_cache.len();
            if before != after {
                tracing::debug!(
                    "[CacheManager:L2-Tools] LRU cleanup: {} → {} (limit: {})",
                    before,
                    after,
                    TOOLS_CACHE_LIMIT
                );
            }
        }
    }

    /// Calculate Layer 2 key: SHA256(raw tools JSON string)
    pub fn compute_tools_key(raw_tools_json: &str) -> String {
        Self::sha256_hex(raw_tools_json.as_bytes())
    }

    // ===== Layer 3: Prefix Tracker =====

    /// Compute stable combined prefix hash
    ///
    /// Input: JSON string of systemInstruction and tools
    /// Use independent hash combinations of Layer 1 + Layer 2 instead of raw data
    pub fn compute_prefix_hash(si_json: &str, tools_json: &str) -> String {
        let si_hash = Self::sha256_hex(si_json.as_bytes());
        let tools_hash = if tools_json.is_empty() {
            String::from("no-tools")
        } else {
            Self::sha256_hex(tools_json.as_bytes())
        };
        // Combination: SHA256(si_hash + "::" + tools_hash)
        Self::sha256_hex(format!("{}::{}", si_hash, tools_hash).as_bytes())
    }
}
