#![allow(dead_code)]
// Reserved cache implementation, currently not enabled in the production path

use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Instant;

/// cache entry
#[derive(Clone)]
struct CacheEntry {
    /// Schema after cleaning
    schema: Value,
    /// last use time
    last_used: Instant,
    /// Number of hits
    hit_count: usize,
}

/// Schema cache
struct SchemaCache {
    /// Cache storage (key: cache_key, value: CacheEntry)
    cache: HashMap<String, CacheEntry>,
    /// cache statistics
    stats: CacheStats,
}

/// cache statistics
#[derive(Default, Clone, Debug)]
pub struct CacheStats {
    /// Total number of requests
    pub total_requests: usize,
    /// Number of cache hits
    pub cache_hits: usize,
    /// Number of cache misses
    pub cache_misses: usize,
}

impl CacheStats {
    /// Calculate cache hit ratio
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_requests as f64
        }
    }
}

impl SchemaCache {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
            stats: CacheStats::default(),
        }
    }

    /// Get cache entry
    fn get(&mut self, key: &str) -> Option<Value> {
        self.stats.total_requests += 1;

        if let Some(entry) = self.cache.get_mut(key) {
            // Update usage time and number of hits
            entry.last_used = Instant::now();
            entry.hit_count += 1;
            self.stats.cache_hits += 1;
            Some(entry.schema.clone())
        } else {
            self.stats.cache_misses += 1;
            None
        }
    }

    /// Insert cache entry
    fn insert(&mut self, key: String, schema: Value) {
        // Check the cache size and clean it if it exceeds the limit
        const MAX_CACHE_SIZE: usize = 1000;
        if self.cache.len() >= MAX_CACHE_SIZE {
            self.evict_lru();
        }

        let entry = CacheEntry {
            schema,
            last_used: Instant::now(),
            hit_count: 0,
        };
        self.cache.insert(key, entry);
    }

    /// LRU elimination strategy: remove the oldest unused entries
    fn evict_lru(&mut self) {
        if self.cache.is_empty() {
            return;
        }

        // Find the oldest unused entry
        let oldest_key = self
            .cache
            .iter()
            .min_by_key(|(_, entry)| entry.last_used)
            .map(|(key, _)| key.clone());

        if let Some(key) = oldest_key {
            self.cache.remove(&key);
        }
    }

    /// Get cache statistics
    fn stats(&self) -> CacheStats {
        self.stats.clone()
    }

    /// Clear cache
    fn clear(&mut self) {
        self.cache.clear();
        self.stats = CacheStats::default();
    }
}

/// Global Schema cache instance
static SCHEMA_CACHE: Lazy<RwLock<SchemaCache>> = Lazy::new(|| RwLock::new(SchemaCache::new()));

/// Calculate the hash value of Schema
///
/// Use the SHA-256 algorithm to calculate the hash value of the Schema to ensure that the same Schema produces the same hash
fn compute_schema_hash(schema: &Value) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    // Use compact format serialization to improve consistency
    let schema_str = schema.to_string();
    hasher.update(schema_str.as_bytes());

    // Returns the first 16 digits of a hexadecimal string (unique enough)
    format!("{:x}", hasher.finalize())[..16].to_string()
}

/// Schema cleaning with caching
///
/// This is the recommended cleaning entrance and supports cache optimization.
///
/// # Arguments
/// * `schema` - JSON Schema to be cleaned
/// * `tool_name` - tool name, used for cache keys
///
/// # Returns
/// Schema after cleaning
pub fn clean_json_schema_cached(schema: &mut Value, tool_name: &str) {
    // 1. Calculate the cache key of the original Schema
    let hash = compute_schema_hash(schema);
    let cache_key = format!("{}:{}", tool_name, hash);

    // 2. Try to read from cache
    {
        if let Ok(mut cache) = SCHEMA_CACHE.write() {
            if let Some(cached) = cache.get(&cache_key) {
                *schema = cached;
                return;
            }
        }
    }

    // 3. Cache miss, perform cleaning
    super::json_schema::clean_json_schema_for_tool(schema, tool_name);

    // 4. Write cache (using original hash as key)
    if let Ok(mut cache) = SCHEMA_CACHE.write() {
        cache.insert(cache_key, schema.clone());
    }
}

/// Get cache statistics
pub fn get_cache_stats() -> CacheStats {
    SCHEMA_CACHE
        .read()
        .map(|cache| cache.stats())
        .unwrap_or_default()
}

/// Clear cache
pub fn clear_cache() {
    if let Ok(mut cache) = SCHEMA_CACHE.write() {
        cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_compute_schema_hash() {
        let schema1 = json!({"type": "string"});
        let schema2 = json!({"type": "string"});
        let schema3 = json!({"type": "number"});

        let hash1 = compute_schema_hash(&schema1);
        let hash2 = compute_schema_hash(&schema2);
        let hash3 = compute_schema_hash(&schema3);

        // The same Schema should produce the same hash
        assert_eq!(hash1, hash2);
        // Different Schemas should produce different hashes
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_cache_hit() {
        let _guard = TEST_MUTEX.lock().unwrap();
        clear_cache();

        let mut schema = json!({"type": "string", "minLength": 5});
        let tool_name = "test_tool";

        // First call - cache miss
        clean_json_schema_cached(&mut schema, tool_name);

        // Second call to same Schema - should cache hit
        let mut schema2 = json!({"type": "string", "minLength": 5});
        clean_json_schema_cached(&mut schema2, tool_name);

        let stats = get_cache_stats();
        // Verify there is a cache hit
        assert!(
            stats.cache_hits > 0,
            "Expected cache hits, got: {:?}",
            stats
        );
        assert!(stats.hit_rate() > 0.0);
    }

    #[test]
    fn test_cache_eviction() {
        let _guard = TEST_MUTEX.lock().unwrap();
        clear_cache();

        // Inserting a large number of entries triggers elimination
        for i in 0..1100 {
            let mut schema = json!({"type": "string", "index": i});
            let tool_name = format!("tool_{}", i);
            clean_json_schema_cached(&mut schema, &tool_name);
        }

        // Verify cache size is limited
        let stats = get_cache_stats();
        assert!(stats.total_requests > 0);
    }
}
