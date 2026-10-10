use crate::models::QuotaProtectionConfig;
use crate::proxy::common::model_mapping::normalize_to_standard_id;
use crate::proxy::token_manager::ProxyToken;
use std::path::PathBuf;

// ==================================================================================
// 辅助函数：创建模拟账号
// ==================================================================================

fn create_mock_token(
    account_id: &str,
    email: &str,
    protected_models: Vec<&str>,
    remaining_quota: Option<i32>,
) -> ProxyToken {
    ProxyToken {
        account_id: account_id.to_string(),
        priority: crate::models::account::default_priority(),
        access_token: format!("mock_access_token_{}", account_id),
        refresh_token: format!("mock_refresh_token_{}", account_id),
        expires_in: 3600,
        timestamp: chrono::Utc::now().timestamp() + 3600,
        email: email.to_string(),
        account_path: PathBuf::from(format!("/tmp/test_accounts/{}.json", account_id)),
        project_id: Some("test-project".to_string()),
        subscription_tier: Some("PRO".to_string()),
        remaining_quota,
        protected_models: protected_models.iter().map(|s| s.to_string()).collect(),
        health_score: 1.0,
        reset_time: None,
        validation_blocked: false,
        validation_blocked_until: 0,
        validation_url: None,
        model_quotas: std::collections::HashMap::new(),
        model_limits: std::collections::HashMap::new(),
    }
}

/// 辅助函数：创建带有自定义 account_path 的 mock token
fn create_mock_token_with_path(
    account_id: &str,
    email: &str,
    protected_models: Vec<&str>,
    remaining_quota: Option<i32>,
    account_path: PathBuf,
) -> ProxyToken {
    ProxyToken {
        account_id: account_id.to_string(),
        priority: crate::models::account::default_priority(),
        access_token: format!("mock_access_token_{}", account_id),
        refresh_token: format!("mock_refresh_token_{}", account_id),
        expires_in: 3600,
        timestamp: chrono::Utc::now().timestamp() + 3600,
        email: email.to_string(),
        account_path,
        project_id: Some("test-project".to_string()),
        subscription_tier: Some("PRO".to_string()),
        remaining_quota,
        protected_models: protected_models.iter().map(|s| s.to_string()).collect(),
        health_score: 1.0,
        reset_time: None,
        validation_blocked: false,
        validation_blocked_until: 0,
        validation_url: None,
        model_quotas: std::collections::HashMap::new(),
        model_limits: std::collections::HashMap::new(),
    }
}

mod full_flow;
mod normalization;
mod protection;
mod sticky_session;
mod sync;
