use super::*;

// ============================================================================
// 测试类别 6: IP 访问日志
// ============================================================================

#[test]
fn test_access_log_save_and_retrieve() {
    let _lock = setup_test();

    // 保存访问日志
    let log = IpAccessLog {
        id: uuid::Uuid::new_v4().to_string(),
        client_ip: "test.log.ip".to_string(),
        timestamp: now_timestamp(),
        method: Some("POST".to_string()),
        path: Some("/v1/messages".to_string()),
        user_agent: Some("TestClient/1.0".to_string()),
        status: Some(200),
        duration: Some(150),
        api_key_hash: Some("hash123".to_string()),
        blocked: false,
        block_reason: None,
        username: None,
    };

    let save_result = save_ip_access_log(&log);
    assert!(
        save_result.is_ok(),
        "Should save access log: {:?}",
        save_result.err()
    );

    // 检索日志
    let logs = get_ip_access_logs(10, 0, Some("test.log.ip"), false);
    assert!(logs.is_ok());

    let logs = logs.unwrap();
    assert!(!logs.is_empty(), "Should retrieve saved log");
    assert_eq!(logs[0].client_ip, "test.log.ip");

    cleanup_test_data();
}

#[test]
fn test_access_log_blocked_filter() {
    let _lock = setup_test();

    // 保存正常日志
    let normal_log = IpAccessLog {
        id: uuid::Uuid::new_v4().to_string(),
        client_ip: "normal.access.ip".to_string(),
        timestamp: now_timestamp(),
        method: Some("GET".to_string()),
        path: Some("/healthz".to_string()),
        user_agent: None,
        status: Some(200),
        duration: Some(10),
        api_key_hash: None,
        blocked: false,
        block_reason: None,
        username: None,
    };
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(save_ip_access_log(&normal_log), "save_ip_access_log");

    // 保存被阻止的日志
    let blocked_log = IpAccessLog {
        id: uuid::Uuid::new_v4().to_string(),
        client_ip: "blocked.access.ip".to_string(),
        timestamp: now_timestamp(),
        method: Some("POST".to_string()),
        path: Some("/v1/messages".to_string()),
        user_agent: None,
        status: Some(403),
        duration: Some(0),
        api_key_hash: None,
        blocked: true,
        block_reason: Some("IP in blacklist".to_string()),
        username: None,
    };
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(save_ip_access_log(&blocked_log), "save_ip_access_log");

    // 只检索被阻止的日志
    let blocked_only = get_ip_access_logs(10, 0, None, true).unwrap();
    assert_eq!(blocked_only.len(), 1);
    assert_eq!(blocked_only[0].client_ip, "blocked.access.ip");
    assert!(blocked_only[0].blocked);

    cleanup_test_data();
}

// ============================================================================
// 测试类别 7: 统计功能
// ============================================================================

#[test]
fn test_ip_stats() {
    let _lock = setup_test();

    // 添加一些测试数据
    for i in 0..5 {
        let log = IpAccessLog {
            id: uuid::Uuid::new_v4().to_string(),
            client_ip: format!("stats.test.{}", i % 3), // 3 个唯一 IP
            timestamp: now_timestamp(),
            method: Some("POST".to_string()),
            path: Some("/v1/messages".to_string()),
            user_agent: None,
            status: Some(200),
            duration: Some(100),
            api_key_hash: None,
            blocked: i == 4, // 最后一个被阻止
            block_reason: if i == 4 {
                Some("Test".to_string())
            } else {
                None
            },
            username: None,
        };
        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(save_ip_access_log(&log), "save_ip_access_log");
    }

    // 添加黑名单和白名单条目
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist("stats.black.1", None, None, "test"),
        "add_to_blacklist",
    );
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        add_to_blacklist("stats.black.2", None, None, "test"),
        "add_to_blacklist",
    );
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(add_to_whitelist("stats.white.1", None), "add_to_whitelist");

    // 获取统计
    let stats = get_ip_stats();
    assert!(stats.is_ok());

    let stats = stats.unwrap();
    assert!(stats.total_requests >= 5, "Should have at least 5 requests");
    assert!(stats.unique_ips >= 3, "Should have at least 3 unique IPs");
    assert!(
        stats.blocked_count >= 1,
        "Should have at least 1 blocked request"
    );
    assert_eq!(stats.blacklist_count, 2);
    assert_eq!(stats.whitelist_count, 1);

    cleanup_test_data();
}

// ============================================================================
// 测试类别 8: 清理功能
// ============================================================================

#[test]
fn test_cleanup_old_logs() {
    let _lock = setup_test();

    // 添加一条 "旧" 日志 (模拟 2 天前)
    let old_log = IpAccessLog {
        id: uuid::Uuid::new_v4().to_string(),
        client_ip: "old.log.ip".to_string(),
        timestamp: now_timestamp() - (2 * 24 * 3600), // 2 天前
        method: Some("GET".to_string()),
        path: Some("/old".to_string()),
        user_agent: None,
        status: Some(200),
        duration: Some(10),
        api_key_hash: None,
        blocked: false,
        block_reason: None,
        username: None,
    };
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(save_ip_access_log(&old_log), "save_ip_access_log");

    // 添加一条新日志
    let new_log = IpAccessLog {
        id: uuid::Uuid::new_v4().to_string(),
        client_ip: "new.log.ip".to_string(),
        timestamp: now_timestamp(),
        method: Some("GET".to_string()),
        path: Some("/new".to_string()),
        user_agent: None,
        status: Some(200),
        duration: Some(10),
        api_key_hash: None,
        blocked: false,
        block_reason: None,
        username: None,
    };
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(save_ip_access_log(&new_log), "save_ip_access_log");

    // 清理 1 天前的日志
    let deleted = cleanup_old_ip_logs(1);
    assert!(deleted.is_ok());
    assert!(deleted.unwrap() >= 1, "Should delete at least 1 old log");

    // 验证新日志仍然存在
    let logs = get_ip_access_logs(10, 0, Some("new.log.ip"), false).unwrap();
    assert!(!logs.is_empty(), "New log should still exist");

    // 验证旧日志已被清理
    let old_logs = get_ip_access_logs(10, 0, Some("old.log.ip"), false).unwrap();
    assert!(old_logs.is_empty(), "Old log should be cleaned up");

    cleanup_test_data();
}
