use super::*;

// ============================================================================
// 测试类别 1: 数据库初始化
// ============================================================================

#[test]
fn test_db_initialization() {
    let _lock = crate::modules::security_db::TEST_SECURITY_DB_MUTEX
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    // 验证数据库初始化不会 panic
    let result = init_db();
    assert!(
        result.is_ok(),
        "Database initialization should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_db_multiple_initializations() {
    let _lock = crate::modules::security_db::TEST_SECURITY_DB_MUTEX
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    // 验证多次初始化不会出错 (幂等性)
    for _ in 0..3 {
        let result = init_db();
        assert!(
            result.is_ok(),
            "Multiple DB initializations should be idempotent"
        );
    }
}
