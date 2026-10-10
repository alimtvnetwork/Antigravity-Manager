use super::*;

mod tests {
    use super::*;
    use std::thread;

    // ===== Layer 1 Tests =====

    #[test]
    fn test_si_cache_insert_and_lookup() {
        let cm = CacheManager::new();
        let raw = "Current time: 2024-01-01\nYou are a helpful assistant.";
        let key = CacheManager::compute_si_key(raw);

        // Initially miss
        assert!(cm.lookup_si(&key).is_none());

        // Insert and hit
        cm.cache_si(key.clone(), "sanitized text".to_string());
        assert_eq!(cm.lookup_si(&key), Some("sanitized text".to_string()));

        // Second hit should work
        assert_eq!(cm.lookup_si(&key), Some("sanitized text".to_string()));
    }

    #[test]
    fn test_si_cache_hit_count() {
        let cm = CacheManager::new();
        let key = CacheManager::compute_si_key("test prompt");
        cm.cache_si(key.clone(), "cached".to_string());

        cm.lookup_si(&key);
        cm.lookup_si(&key);

        let stats = cm.get_layer_stats();
        assert_eq!(stats.si_total, 2); // 2 lookups
        assert_eq!(stats.si_hits, 2);
        assert_eq!(stats.active_si_entries, 1);
    }

    #[test]
    fn test_si_cache_different_inputs() {
        let cm = CacheManager::new();
        let key_a = CacheManager::compute_si_key("prompt A");
        let key_b = CacheManager::compute_si_key("prompt B");

        cm.cache_si(key_a.clone(), "sanitized A".to_string());
        cm.cache_si(key_b.clone(), "sanitized B".to_string());

        assert_eq!(cm.lookup_si(&key_a), Some("sanitized A".to_string()));
        assert_eq!(cm.lookup_si(&key_b), Some("sanitized B".to_string()));
        assert_ne!(key_a, key_b); // different raw prompts → different keys

        let stats = cm.get_layer_stats();
        assert_eq!(stats.active_si_entries, 2);
    }

    // ===== Layer 2 Tests =====

    #[test]
    fn test_tools_cache_insert_and_lookup() {
        let cm = CacheManager::new();
        let raw = r#"[{"functionDeclarations":[{"name":"tool_a"}]}]"#;
        let key = CacheManager::compute_tools_key(raw);

        assert!(cm.lookup_tools(&key).is_none());

        cm.cache_tools(key.clone(), raw.to_string());
        assert_eq!(cm.lookup_tools(&key), Some(raw.to_string()));
    }

    #[test]
    fn test_tools_cache_hit_count() {
        let cm = CacheManager::new();
        let key = CacheManager::compute_tools_key("[{}]");
        cm.cache_tools(key.clone(), "[{}]".to_string());

        cm.lookup_tools(&key);
        cm.lookup_tools(&key);
        cm.lookup_tools(&key);

        let stats = cm.get_layer_stats();
        assert_eq!(stats.tools_total, 3);
        assert_eq!(stats.tools_hits, 3);
    }

    #[test]
    fn test_tools_cache_isolated_from_si() {
        let cm = CacheManager::new();

        let si_key = CacheManager::compute_si_key("prompt");
        let tools_key = CacheManager::compute_tools_key("tools");

        cm.cache_si(si_key.clone(), "cached prompt".to_string());
        cm.cache_tools(tools_key.clone(), "cached tools".to_string());

        // SI lookup shouldn't affect Tools stats
        cm.lookup_si(&si_key);
        cm.lookup_si(&si_key);
        cm.lookup_tools(&tools_key);

        let stats = cm.get_layer_stats();
        assert_eq!(stats.si_total, 2);
        assert_eq!(stats.si_hits, 2);
        assert_eq!(stats.tools_total, 1);
        assert_eq!(stats.tools_hits, 1);
    }

    // ===== Layer 3 Tests =====

    #[test]
    fn test_compute_prefix_hash_deterministic() {
        let si1 = r#"{"role":"user","parts":[{"text":"system prompt"}]}"#;
        let tools1 = r#"[{"functionDeclarations":[{"name":"test_tool"}]}]"#;

        let hash1 = CacheManager::compute_prefix_hash(si1, tools1);
        let hash2 = CacheManager::compute_prefix_hash(si1, tools1);
        assert_eq!(hash1, hash2, "Same inputs must produce same hash");
    }

    #[test]
    fn test_compute_prefix_hash_different_si() {
        let si_a = r#"{"role":"user","parts":[{"text":"system A"}]}"#;
        let si_b = r#"{"role":"user","parts":[{"text":"system B"}]}"#;
        let tools = r#"[{"functionDeclarations":[]}]"#;

        let hash_a = CacheManager::compute_prefix_hash(si_a, tools);
        let hash_b = CacheManager::compute_prefix_hash(si_b, tools);
        assert_ne!(hash_a, hash_b, "Different SI must produce different hash");
    }

    #[test]
    fn test_compute_prefix_hash_different_tools() {
        let si = r#"{"role":"user","parts":[{"text":"system"}]}"#;
        let tools_a = r#"[{"functionDeclarations":[{"name":"a"}]}]"#;
        let tools_b = r#"[{"functionDeclarations":[{"name":"b"}]}]"#;

        let hash_a = CacheManager::compute_prefix_hash(si, tools_a);
        let hash_b = CacheManager::compute_prefix_hash(si, tools_b);
        assert_ne!(
            hash_a, hash_b,
            "Different tools must produce different hash"
        );
    }

    #[test]
    fn test_compute_prefix_hash_empty_tools() {
        let si = "some si";
        let tools_empty = "";

        let hash = CacheManager::compute_prefix_hash(si, tools_empty);
        // Should not panic with empty tools
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_prefix_lookup_miss() {
        let cm = CacheManager::new();
        let result = cm.lookup_prefix("nonexistent_hash");
        assert!(result.is_none());
    }

    #[test]
    fn test_prefix_insert_and_lookup() {
        let cm = CacheManager::new();
        let hash = "test_hash_prefix_123".to_string();
        let cache_name = "cachedContents/test123".to_string();

        cm.insert_prefix(
            hash.clone(),
            cache_name.clone(),
            "si_abc".to_string(),
            "tools_def".to_string(),
            "gemini-flash".to_string(),
            Some(3600),
        );
        let result = cm.lookup_prefix(&hash);
        assert_eq!(result, Some(cache_name));
    }

    #[test]
    fn test_prefix_expiry() {
        let cm = CacheManager::new();
        let hash = "expiring_prefix_hash".to_string();

        cm.insert_prefix(
            hash.clone(),
            "cache_name".to_string(),
            "si".to_string(),
            "tools".to_string(),
            "model".to_string(),
            Some(0), // TTL=0 立即过期
        );

        thread::sleep(Duration::from_millis(10));
        let result = cm.lookup_prefix(&hash);
        assert!(result.is_none(), "Expired entry should not be returned");
    }

    #[test]
    fn test_record_implicit_hit() {
        let cm = CacheManager::new();
        let hash = "test_implicit_hash".to_string();

        cm.insert_prefix(
            hash.clone(),
            "cache".to_string(),
            "si".to_string(),
            "tools".to_string(),
            "model".to_string(),
            Some(3600),
        );

        cm.record_implicit_hit(&hash);
        cm.record_implicit_hit(&hash);

        let stats = cm.get_layer_stats();
        assert_eq!(stats.total_implicit_hits, 2);
    }

    #[test]
    fn test_record_explicit_hit() {
        let cm = CacheManager::new();
        let hash = "test_explicit_hash".to_string();

        cm.insert_prefix(
            hash.clone(),
            "cache".to_string(),
            "si".to_string(),
            "tools".to_string(),
            "model".to_string(),
            Some(3600),
        );

        cm.record_explicit_hit(&hash);

        let stats = cm.get_layer_stats();
        assert_eq!(stats.total_explicit_hits, 1);
    }

    // ===== Backward Compatibility Tests =====

    #[test]
    fn test_legacy_lookup() {
        let cm = CacheManager::new();
        let hash = "legacy_hash".to_string();
        cm.insert(hash.clone(), "legacy_cache".to_string(), Some(60));
        let result = cm.lookup(&hash);
        assert_eq!(result, Some("legacy_cache".to_string()));
    }

    #[test]
    fn test_legacy_insert_and_hit_tracking() {
        let cm = CacheManager::new();
        let hash = "legacy_with_hits".to_string();
        cm.insert(hash.clone(), "cache_name".to_string(), Some(3600));

        cm.lookup(&hash);
        cm.record_implicit_hit(&hash);

        let stats = cm.get_layer_stats();
        assert_eq!(stats.prefix_hits, 1);
        assert_eq!(stats.total_implicit_hits, 1);
    }

    // ===== Combined Scenario Tests =====

    #[test]
    fn test_full_three_layer_flow() {
        let cm = CacheManager::new();

        // Layer 1: SI
        let si_raw = "Current time: 2024-06-01\nYou are a coding assistant.";
        let si_key = CacheManager::compute_si_key(si_raw);
        cm.cache_si(si_key.clone(), "You are a coding assistant.".to_string());
        assert!(cm.lookup_si(&si_key).is_some());

        // Layer 2: Tools
        let tools_raw = r#"{"functionDeclarations":[{"name":"read_file"},{"name":"write_file"}]}"#;
        let tools_key = CacheManager::compute_tools_key(tools_raw);
        cm.cache_tools(tools_key.clone(), tools_raw.to_string());
        assert!(cm.lookup_tools(&tools_key).is_some());

        // Layer 3: Prefix
        let prefix_hash = CacheManager::compute_prefix_hash(si_raw, tools_raw);
        cm.insert_prefix(
            prefix_hash.clone(),
            format!("cachedContents/{}", &prefix_hash[..8]),
            si_key.clone(),
            tools_key.clone(),
            "gemini-3-flash".to_string(),
            Some(3600),
        );
        assert!(cm.lookup_prefix(&prefix_hash).is_some());

        // Stats should show all layers active
        let stats = cm.get_layer_stats();
        assert_eq!(stats.active_si_entries, 1);
        assert_eq!(stats.active_tools_entries, 1);
        assert_eq!(stats.active_prefix_entries, 1);
    }

    #[test]
    fn test_layer_independence() {
        let cm = CacheManager::new();

        // Cache only SI, not tools
        let si_key = CacheManager::compute_si_key("prompt");
        cm.cache_si(si_key.clone(), "sanitized prompt".to_string());

        // SI: hit, Tools: miss
        assert!(cm.lookup_si(&si_key).is_some());
        let tools_key = CacheManager::compute_tools_key("some tools");
        assert!(cm.lookup_tools(&tools_key).is_none());

        let stats = cm.get_layer_stats();
        assert_eq!(stats.si_hits, 1);
        assert_eq!(stats.tools_misses, 1);
    }

    #[test]
    fn test_evict_expired_all_layers() {
        let cm = CacheManager::new();

        // Insert into all layers with 0 TTL via insert_prefix
        cm.cache_si("si_key".to_string(), "text".to_string());
        cm.cache_tools("tools_key".to_string(), "{}".to_string());
        cm.insert_prefix(
            "prefix_key".to_string(),
            "cache".to_string(),
            "si".to_string(),
            "tools".to_string(),
            "model".to_string(),
            Some(0),
        );

        thread::sleep(Duration::from_millis(10));

        let evicted = cm.evict_expired();
        // prefix entry removed, SI and tools still valid (30min TTL)
        assert_eq!(evicted, 1, "Only prefix should be evicted");

        // should be expired and removed
        assert!(cm.lookup_prefix("prefix_key").is_none());

        let stats = cm.get_layer_stats();
        assert_eq!(stats.active_si_entries, 1);
        assert_eq!(stats.active_tools_entries, 1);
        assert_eq!(stats.active_prefix_entries, 0);
    }

    #[test]
    fn test_clear_all() {
        let cm = CacheManager::new();

        cm.cache_si("si".to_string(), "text".to_string());
        cm.cache_tools("tools".to_string(), "{}".to_string());
        cm.insert_prefix(
            "prefix".to_string(),
            "cache".to_string(),
            "s".to_string(),
            "t".to_string(),
            "m".to_string(),
            Some(60),
        );

        cm.clear();

        let stats = cm.get_layer_stats();
        assert_eq!(stats.active_si_entries, 0);
        assert_eq!(stats.active_tools_entries, 0);
        assert_eq!(stats.active_prefix_entries, 0);
        assert_eq!(stats.si_total, 0);
        assert_eq!(stats.prefix_total, 0);
    }

    #[test]
    fn test_global_singleton() {
        let cm = global_cache_manager();
        cm.clear();
        let stats = cm.get_layer_stats();
        assert_eq!(stats.si_total, 0);
        assert_eq!(stats.prefix_total, 0);
        assert_eq!(stats.active_si_entries, 0);
    }
}
