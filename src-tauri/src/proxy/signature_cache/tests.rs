use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_tool_signature_cache() {
        let cache = SignatureCache::new();
        let sig = "x".repeat(60); // Valid length

        cache.cache_tool_signature("tool_1", sig.clone());
        assert_eq!(cache.get_tool_signature("tool_1"), Some(sig));
        assert_eq!(cache.get_tool_signature("tool_2"), None);
    }

    #[test]
    fn test_min_length() {
        let cache = SignatureCache::new();
        cache.cache_tool_signature("tool_short", "short".to_string());
        assert_eq!(cache.get_tool_signature("tool_short"), None);
    }

    #[test]
    fn test_thinking_family() {
        let cache = SignatureCache::new();
        let sig = "y".repeat(60);

        cache.cache_thinking_family(sig.clone(), "claude".to_string());
        assert_eq!(cache.get_signature_family(&sig), Some("claude".to_string()));
    }

    #[test]
    fn test_session_signature() {
        let cache = SignatureCache::new();
        let sig1 = "a".repeat(60);
        let sig2 = "b".repeat(80); // Longer, should replace
        let sig3 = "c".repeat(20); // Too short (< 32), should be ignored

        // Initially empty
        assert!(cache.get_session_signature("sid-test123").is_none());

        // Store first signature
        cache.cache_session_signature("sid-test123", sig1.clone(), 5);
        assert_eq!(
            cache.get_session_signature("sid-test123"),
            Some(sig1.clone())
        );

        // Longer signature should replace (same msg count)
        cache.cache_session_signature("sid-test123", sig2.clone(), 5);
        assert_eq!(
            cache.get_session_signature("sid-test123"),
            Some(sig2.clone())
        );

        // Shorter valid signature should NOT replace (same msg count)
        cache.cache_session_signature("sid-test123", sig1.clone(), 5);
        assert_eq!(
            cache.get_session_signature("sid-test123"),
            Some(sig2.clone())
        );

        // Rewind: Shorter signature MUST replace if message count is lower
        cache.cache_session_signature("sid-test123", sig1.clone(), 3);
        assert_eq!(
            cache.get_session_signature("sid-test123"),
            Some(sig1.clone())
        );

        // Too short signature should be ignored entirely (even if rewind)
        cache.cache_session_signature("sid-test123", sig3, 1);
        assert_eq!(cache.get_session_signature("sid-test123"), Some(sig1));

        // Different session should be isolated
        assert!(cache.get_session_signature("sid-other").is_none());
    }

    #[test]
    fn test_clear_all_caches() {
        let cache = SignatureCache::new();
        let sig = "x".repeat(60);

        cache.cache_tool_signature("tool_1", sig.clone());
        cache.cache_thinking_family(sig.clone(), "model".to_string());
        cache.cache_session_signature("sid-1", sig.clone(), 1);

        assert!(cache.get_tool_signature("tool_1").is_some());
        assert!(cache.get_signature_family(&sig).is_some());
        assert!(cache.get_session_signature("sid-1").is_some());

        cache.clear();

        assert!(cache.get_tool_signature("tool_1").is_none());
        assert!(cache.get_signature_family(&sig).is_none());
        assert!(cache.get_session_signature("sid-1").is_none());
    }

    #[test]
    fn test_tool_signature_sqlite_recovery() {
        let tool_id = "call_sig_sqlite_recovery_unique";
        let sig = "s".repeat(60);

        // 1. Direct save to SQLite
        let db_res = crate::modules::proxy_db::save_tool_signature(tool_id, &sig);
        if let Err(e) = db_res {
            eprintln!("Skipping DB test if DB not initialized: {}", e);
            return;
        }

        // 2. New in-memory cache (simulating proxy restart)
        let cache = SignatureCache::new();

        // 3. get_tool_signature should fall back to SQLite and populate L1 cache
        let recovered = cache.get_tool_signature(tool_id);
        assert_eq!(recovered, Some(sig.clone()));

        // 4. Second lookup should hit L1
        assert_eq!(cache.get_tool_signature(tool_id), Some(sig));
    }

    #[test]
    fn test_tool_signature_normalization_match() {
        let cache = SignatureCache::new();
        let sig = "s".repeat(60);

        // Case A: Upstream generated call_573077, client requests with call573077
        cache.cache_tool_signature("call_573077", sig.clone());
        assert_eq!(cache.get_tool_signature("call573077"), Some(sig.clone()));
        assert_eq!(cache.get_tool_signature("call_573077"), Some(sig.clone()));

        // Case B: Client stored with call888888, lookup with call_888888
        let sig2 = "t".repeat(60);
        cache.cache_tool_signature("call888888", sig2.clone());
        assert_eq!(cache.get_tool_signature("call_888888"), Some(sig2.clone()));
        assert_eq!(cache.get_tool_signature("call888888"), Some(sig2));
    }
}
