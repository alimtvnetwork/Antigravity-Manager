//! Shared admin audit-logging helper.

pub(crate) fn log_admin_audit(endpoint: &str, method: &str, status_code: i32) {
    let log = crate::modules::security_db::IpAccessLog {
        id: uuid::Uuid::new_v4().to_string(),
        client_ip: "127.0.0.1".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        method: Some(method.to_string()),
        path: Some(endpoint.to_string()),
        user_agent: Some("AGM-REST-Admin".to_string()),
        status: Some(status_code),
        duration: Some(0),
        api_key_hash: None,
        blocked: false,
        block_reason: None,
        username: Some("admin".to_string()),
    };
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        crate::modules::security_db::save_ip_access_log(&log),
        "save_ip_access_log",
    );
}

// ── Instance Handlers ──
