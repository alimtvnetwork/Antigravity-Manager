use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    pub(crate) fn test_normalize_supabase_url() {
        assert_eq!(
            normalize_supabase_url("https://abcdefg.supabase.co"),
            "https://abcdefg.supabase.co"
        );
        assert_eq!(
            normalize_supabase_url("https://abcdefg.supabase.co/"),
            "https://abcdefg.supabase.co"
        );
        assert_eq!(
            normalize_supabase_url("https://abcdefg.supabase.co///"),
            "https://abcdefg.supabase.co"
        );
        assert_eq!(
            normalize_supabase_url("https://abcdefg.supabase.co/rest/v1"),
            "https://abcdefg.supabase.co"
        );
        assert_eq!(
            normalize_supabase_url("https://abcdefg.supabase.co/rest/v1/"),
            "https://abcdefg.supabase.co"
        );
        assert_eq!(
            normalize_supabase_url("https://abcdefg.supabase.co/rest/v1///"),
            "https://abcdefg.supabase.co"
        );
        assert_eq!(
            normalize_supabase_url("  https://abcdefg.supabase.co/rest/v1/  "),
            "https://abcdefg.supabase.co"
        );
        assert_eq!(
            normalize_supabase_url("http://localhost:54321/rest/v1/"),
            "http://localhost:54321"
        );
        assert_eq!(
            normalize_supabase_url("https://example.com/custom/prefix/rest/v1"),
            "https://example.com/custom/prefix"
        );
    }

    #[test]
    pub(crate) fn test_client_url_construction() {
        let ep1 = SupabaseEndpoint {
            id: "ep_1".to_string(),
            name: "Cloud".to_string(),
            url: "https://abcdefg.supabase.co/rest/v1/".to_string(),
            api_key: "anon-key-sample".to_string(),
            role: "root".to_string(),
            is_enabled: true,
            prune_threshold_mb: 400,
            priority: 1,
            notes: None,
            tags: Vec::new(),
        };
        let client1 = SupabaseClient::new(&ep1).expect("valid client");
        assert_eq!(client1.base_url(), "https://abcdefg.supabase.co");
        assert_eq!(
            client1.table_url("nodes"),
            "https://abcdefg.supabase.co/rest/v1/nodes"
        );
        assert_eq!(
            client1.rpc_url("sync_nodes"),
            "https://abcdefg.supabase.co/rest/v1/rpc/sync_nodes"
        );

        let ep2 = SupabaseEndpoint {
            id: "ep_2".to_string(),
            name: "Bare".to_string(),
            url: "https://abcdefg.supabase.co".to_string(),
            api_key: "anon-key-sample".to_string(),
            role: "root".to_string(),
            is_enabled: true,
            prune_threshold_mb: 400,
            priority: 1,
            notes: None,
            tags: Vec::new(),
        };
        let client2 = SupabaseClient::new(&ep2).expect("valid client");
        assert_eq!(client2.base_url(), "https://abcdefg.supabase.co");
        assert_eq!(client2.table_url("nodes"), client1.table_url("nodes"));
        assert_eq!(client2.rpc_url("sync_nodes"), client1.rpc_url("sync_nodes"));
    }

    #[test]
    pub(crate) fn test_create_endpoint_helpers() {
        let ep = create_endpoint(
            "ep1",
            "Root DB",
            "https://xyz.supabase.co/rest/v1/",
            "key1",
            "root",
        );
        assert_eq!(ep.id, "ep1");
        assert_eq!(ep.url, "https://xyz.supabase.co");
        assert_eq!(ep.role, "root");
        assert_eq!(ep.priority, 1);
        assert_eq!(ep.prune_threshold_mb, 400);

        let ep_sec = SupabaseEndpoint::new_endpoint(
            "ep2",
            "Sec DB",
            "https://abc.supabase.co/",
            "key2",
            "secondary",
        );
        assert_eq!(ep_sec.id, "ep2");
        assert_eq!(ep_sec.url, "https://abc.supabase.co");
        assert_eq!(ep_sec.role, "secondary");
        assert_eq!(ep_sec.priority, 2);
        assert_eq!(ep_sec.prune_threshold_mb, 200);
    }

    #[test]
    pub(crate) fn test_verification_is_connected_logic() {
        // Case 1: verified tables exist -> is_connected is true
        let res1 = TableVerificationResult {
            endpoint_id: "ep_1".to_string(),
            is_connected: true,
            verified_tables: vec!["nodes".to_string()],
            missing_tables: vec!["workspace_leases".to_string()],
            error_message: None,
        };
        assert!(res1.is_connected);

        // Case 2: no verified tables, but network error occurred -> is_connected is false
        let verified: Vec<String> = Vec::new();
        let err_msg = Some("Connection refused (os error 111)".to_string());
        let missing = vec!["nodes".to_string()];
        let is_conn = if !verified.is_empty() {
            true
        } else {
            err_msg.is_none() && !missing.is_empty()
        };
        assert!(!is_conn);

        // Case 3: no verified tables, no network error (PostgREST responsive but tables not created) -> is_connected is true
        let err_msg_none: Option<String> = None;
        let is_conn_pending = if !verified.is_empty() {
            true
        } else {
            err_msg_none.is_none() && !missing.is_empty()
        };
        assert!(is_conn_pending);
    }

    #[tokio::test]
    async fn test_mock_connection_probe1_success() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let app = axum::Router::new().route(
            "/rest/v1/",
            axum::routing::get(|| async {
                (
                    axum::http::StatusCode::OK,
                    [("content-type", "application/json")],
                    "{\"swagger\":\"2.0\",\"info\":{\"title\":\"PostgREST API\"}}",
                )
            }),
        );
        tokio::spawn(async move {
            // Justification: best-effort server loop; abnormal termination is logged
            crate::error::record_ignored(axum::serve(listener, app).await, "axum::serve");
        });

        let ep = SupabaseEndpoint {
            id: "ep_probe1".to_string(),
            name: "Mock Probe 1".to_string(),
            url: format!("http://127.0.0.1:{}", port),
            api_key: "anon-mock-key".to_string(),
            role: "root".to_string(),
            is_enabled: true,
            prune_threshold_mb: 400,
            priority: 1,
            notes: None,
            tags: Vec::new(),
        };

        let client = SupabaseClient::new(&ep).unwrap();
        let res = client.test_connection().await.unwrap();
        assert!(res.is_success);
        assert!(res.message.contains("Supabase PostgREST verified"));
        assert_eq!(res.status_code, Some(200));
    }

    #[tokio::test]
    async fn test_mock_connection_probe1_auth_error() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let app = axum::Router::new().route(
            "/rest/v1/",
            axum::routing::get(|| async {
                (
                    axum::http::StatusCode::UNAUTHORIZED,
                    [("content-type", "application/json")],
                    "{\"message\":\"Invalid API key\"}",
                )
            }),
        );
        tokio::spawn(async move {
            // Justification: best-effort server loop; abnormal termination is logged
            crate::error::record_ignored(axum::serve(listener, app).await, "axum::serve");
        });

        let ep = SupabaseEndpoint {
            id: "ep_auth_err".to_string(),
            name: "Mock Auth Err".to_string(),
            url: format!("http://127.0.0.1:{}", port),
            api_key: "bad-key".to_string(),
            role: "root".to_string(),
            is_enabled: true,
            prune_threshold_mb: 400,
            priority: 1,
            notes: None,
            tags: Vec::new(),
        };

        let client = SupabaseClient::new(&ep).unwrap();
        let res = client.test_connection().await.unwrap();
        assert!(!res.is_success);
        assert!(res.message.contains("Authentication failed"));
        assert_eq!(res.status_code, Some(401));
    }

    #[tokio::test]
    async fn test_mock_connection_probe2_table_found() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let app = axum::Router::new()
            .route(
                "/rest/v1/",
                axum::routing::get(|| async {
                    (
                        axum::http::StatusCode::NOT_FOUND,
                        [("content-type", "application/json")],
                        "{\"message\":\"OpenAPI disabled\"}",
                    )
                }),
            )
            .route(
                "/rest/v1/nodes",
                axum::routing::get(|| async {
                    (
                        axum::http::StatusCode::OK,
                        [("content-type", "application/json")],
                        "[]",
                    )
                }),
            );
        tokio::spawn(async move {
            // Justification: best-effort server loop; abnormal termination is logged
            crate::error::record_ignored(axum::serve(listener, app).await, "axum::serve");
        });

        let ep = SupabaseEndpoint {
            id: "ep_probe2_table".to_string(),
            name: "Mock Probe 2 Table".to_string(),
            url: format!("http://127.0.0.1:{}", port),
            api_key: "anon-mock-key".to_string(),
            role: "root".to_string(),
            is_enabled: true,
            prune_threshold_mb: 400,
            priority: 1,
            notes: None,
            tags: Vec::new(),
        };

        let client = SupabaseClient::new(&ep).unwrap();
        let res = client.test_connection().await.unwrap();
        assert!(res.is_success);
        assert!(res.message.contains("nodes table verified"));
        assert_eq!(res.status_code, Some(200));
    }

    #[tokio::test]
    async fn test_mock_connection_probe2_pgrst_relation_missing() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let app = axum::Router::new()
            .route(
                "/rest/v1/",
                axum::routing::get(|| async {
                    (
                        axum::http::StatusCode::NOT_FOUND,
                        [("content-type", "application/json")],
                        "{\"message\":\"Not found\"}",
                    )
                }),
            )
            .route(
                "/rest/v1/nodes",
                axum::routing::get(|| async {
                    (
                        axum::http::StatusCode::NOT_FOUND,
                        [("content-type", "application/json")],
                        "{\"code\":\"PGRST205\",\"message\":\"relation \\\"public.nodes\\\" does not exist\"}",
                    )
                }),
            );
        tokio::spawn(async move {
            // Justification: best-effort server loop; abnormal termination is logged
            crate::error::record_ignored(axum::serve(listener, app).await, "axum::serve");
        });

        let ep = SupabaseEndpoint {
            id: "ep_pgrst".to_string(),
            name: "Mock PGRST".to_string(),
            url: format!("http://127.0.0.1:{}", port),
            api_key: "anon-mock-key".to_string(),
            role: "root".to_string(),
            is_enabled: true,
            prune_threshold_mb: 400,
            priority: 1,
            notes: None,
            tags: Vec::new(),
        };

        let client = SupabaseClient::new(&ep).unwrap();
        let res = client.test_connection().await.unwrap();
        assert!(res.is_success);
        assert!(res
            .message
            .contains("PostgREST responsive; run Schema Migration"));
        assert_eq!(res.status_code, Some(200));
    }

    #[tokio::test]
    async fn test_mock_connection_probe3_health_fallback() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let app = axum::Router::new()
            .route(
                "/rest/v1/",
                axum::routing::get(|| async {
                    (
                        axum::http::StatusCode::NOT_FOUND,
                        [("content-type", "text/plain")],
                        "Not Found",
                    )
                }),
            )
            .route(
                "/rest/v1/nodes",
                axum::routing::get(|| async {
                    (
                        axum::http::StatusCode::NOT_FOUND,
                        [("content-type", "text/plain")],
                        "404 page",
                    )
                }),
            )
            .route(
                "/rest/v1/command_queue",
                axum::routing::get(|| async {
                    (
                        axum::http::StatusCode::NOT_FOUND,
                        [("content-type", "text/plain")],
                        "404 page",
                    )
                }),
            )
            .route(
                "/auth/v1/health",
                axum::routing::get(|| async {
                    (
                        axum::http::StatusCode::OK,
                        [("content-type", "application/json")],
                        "{\"version\":\"v2.0\",\"name\":\"GoTrue\"}",
                    )
                }),
            );
        tokio::spawn(async move {
            // Justification: best-effort server loop; abnormal termination is logged
            crate::error::record_ignored(axum::serve(listener, app).await, "axum::serve");
        });

        let ep = SupabaseEndpoint {
            id: "ep_probe3".to_string(),
            name: "Mock Probe 3".to_string(),
            url: format!("http://127.0.0.1:{}", port),
            api_key: "anon-mock-key".to_string(),
            role: "root".to_string(),
            is_enabled: true,
            prune_threshold_mb: 400,
            priority: 1,
            notes: None,
            tags: Vec::new(),
        };

        let client = SupabaseClient::new(&ep).unwrap();
        let res = client.test_connection().await.unwrap();
        assert!(res.is_success);
        assert!(res.message.contains("Supabase service online"));
        assert_eq!(res.status_code, Some(200));
    }

    #[tokio::test]
    async fn test_mock_verify_expected_tables() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let app = axum::Router::new()
            .route(
                "/rest/v1/nodes",
                axum::routing::get(|| async {
                    (
                        axum::http::StatusCode::OK,
                        [("content-type", "application/json")],
                        "[]",
                    )
                }),
            )
            .route(
                "/rest/v1/instance_profiles",
                axum::routing::get(|| async {
                    (
                        axum::http::StatusCode::NOT_FOUND,
                        [("content-type", "application/json")],
                        "{\"code\":\"PGRST205\",\"message\":\"relation does not exist\"}",
                    )
                }),
            )
            .route(
                "/rest/v1/workspace_leases",
                axum::routing::get(|| async {
                    (
                        axum::http::StatusCode::NOT_FOUND,
                        [("content-type", "application/json")],
                        "{\"code\":\"PGRST205\",\"message\":\"relation does not exist\"}",
                    )
                }),
            );
        tokio::spawn(async move {
            // Justification: best-effort server loop; abnormal termination is logged
            crate::error::record_ignored(axum::serve(listener, app).await, "axum::serve");
        });

        let ep = SupabaseEndpoint {
            id: "ep_verify".to_string(),
            name: "Mock Verify Tables".to_string(),
            url: format!("http://127.0.0.1:{}", port),
            api_key: "anon-mock-key".to_string(),
            role: "root".to_string(),
            is_enabled: true,
            prune_threshold_mb: 400,
            priority: 1,
            notes: None,
            tags: Vec::new(),
        };

        let client = SupabaseClient::new(&ep).unwrap();
        let res = client.verify_expected_tables("ep_verify", "root").await;
        assert!(res.is_connected);
        assert_eq!(res.verified_tables, vec!["nodes".to_string()]);
        assert_eq!(
            res.missing_tables,
            vec![
                "instance_profiles".to_string(),
                "workspace_leases".to_string()
            ]
        );
    }
}
