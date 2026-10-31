use super::*;

// ============================================================================
// Test Category 1: Database Initialization
// ============================================================================

#[test]
fn test_db_initialization() {
    let _lock = crate::modules::security_db::TEST_SECURITY_DB_MUTEX
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    // Verify database initialization does not panic
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
    // Verify that multiple initializations will not go wrong (idempotence)
    for _ in 0..3 {
        let result = init_db();
        assert!(
            result.is_ok(),
            "Multiple DB initializations should be idempotent"
        );
    }
}
