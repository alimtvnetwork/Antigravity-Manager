#[cfg(test)]
mod tests {
    use super::super::dynamic_dns_resolver::DynamicDnsResolver;
    use std::time::Duration;

    #[tokio::test]
    async fn test_resolver_default_and_new() {
        let resolver_def = DynamicDnsResolver::default();
        let resolver_custom = DynamicDnsResolver::new(Duration::from_millis(100));

        let addr1 = resolver_def.resolve("127.0.0.1", 8080).await;
        assert!(addr1.is_ok());
        let list1 = addr1.unwrap();
        assert!(!list1.is_empty());
        assert_eq!(list1[0].port(), 8080);

        let addr2 = resolver_custom.resolve("127.0.0.1", 9090).await;
        assert!(addr2.is_ok());
        let list2 = addr2.unwrap();
        assert!(!list2.is_empty());
        assert_eq!(list2[0].port(), 9090);
    }

    #[tokio::test]
    async fn test_resolver_caching_and_case_insensitivity() {
        let resolver = DynamicDnsResolver::new(Duration::from_secs(60));

        let res1 = resolver.resolve("127.0.0.1", 443).await.unwrap();
        let res2 = resolver.resolve("127.0.0.1", 443).await.unwrap();
        assert_eq!(res1, res2);

        // Case-insensitivity check (e.g. "localhost" vs "LOCALHOST")
        let res_lower = resolver.resolve("localhost", 80).await;
        let res_upper = resolver.resolve("LOCALHOST", 80).await;
        if let (Ok(l), Ok(u)) = (res_lower, res_upper) {
            assert_eq!(l, u);
        }
    }

    #[tokio::test]
    async fn test_resolver_sweep_expired() {
        let resolver = DynamicDnsResolver::new(Duration::from_millis(10));
        let _ = resolver.resolve("127.0.0.1", 8081).await;

        tokio::time::sleep(Duration::from_millis(25)).await;
        resolver.sweep_expired().await;

        // Resolving again will repopulate fresh entry
        let res = resolver.resolve("127.0.0.1", 8081).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_resolver_invalid_host() {
        let resolver = DynamicDnsResolver::default();
        let invalid = resolver.resolve("invalid.unresolvable.local.invalid", 80).await;
        assert!(invalid.is_err());
    }
}
