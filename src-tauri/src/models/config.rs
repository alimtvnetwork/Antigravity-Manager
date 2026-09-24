use crate::modules::cloudflared::CloudflaredConfig;
use crate::proxy::ProxyConfig;
use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub language: String,
    pub theme: String,
    pub auto_refresh: bool,
    pub refresh_interval: i32, // minutes
    #[serde(default = "default_auto_sync")]
    pub auto_sync: bool,
    #[serde(default = "default_true")]
    pub auto_sync_migrated: bool,
    #[serde(default = "default_sync_interval")]
    pub sync_interval: i32, // minutes
    pub default_export_path: Option<String>,
    #[serde(default)]
    pub proxy: ProxyConfig,
    pub antigravity_executable: Option<String>, // [NEW] Manually specified Antigravity executable path
    pub antigravity_ide_executable: Option<String>, // [NEW] Manually specified Antigravity IDE executable path
    pub antigravity_cli_executable: Option<String>, // [NEW] Manually specified Antigravity CLI (agy) path
    pub antigravity_args: Option<Vec<String>>,      // [NEW] Antigravity startup arguments
    #[serde(default)]
    pub auto_launch: bool,     // Launch on startup
    #[serde(default)]
    pub scheduled_warmup: ScheduledWarmupConfig, // [NEW] Scheduled warmup configuration
    #[serde(default)]
    pub quota_protection: QuotaProtectionConfig, // [NEW] Quota protection configuration
    #[serde(default)]
    pub pinned_quota_models: PinnedQuotaModelsConfig, // [NEW] Pinned quota models list
    #[serde(default)]
    pub circuit_breaker: CircuitBreakerConfig, // [NEW] Circuit breaker configuration
    #[serde(default)]
    pub hidden_menu_items: Vec<String>, // Hidden menu item path list
    #[serde(default)]
    pub cloudflared: CloudflaredConfig, // [NEW] Cloudflared configuration
    pub auto_profile_switcher: AutoProfileSwitcherConfig, // [NEW] Auto profile switcher configuration
    #[serde(default = "default_instance_clone_mode")]
    pub instance_clone_mode: String,
    #[serde(default)]
    pub conversation_cleanup: ConversationCleanupConfig,
    #[serde(default)]
    pub lightweight_mode: bool, // [NEW] Lightweight mode: destroy webview on minimize/close to tray
    #[serde(default)]
    pub training_api_enabled: bool, // [NEW] Enable /api/v1/training REST API endpoints
}

fn default_auto_sync() -> bool {
    true
}

fn default_sync_interval() -> i32 {
    5
}

fn default_instance_clone_mode() -> String {
    "full".to_string()
}

/// Scheduled warmup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledWarmupConfig {
    /// Whether smart warmup is enabled
    pub enabled: bool,

    /// List of models to warmup
    #[serde(default = "default_warmup_models")]
    pub monitored_models: Vec<String>,
}

fn default_warmup_models() -> Vec<String> {
    vec![
        "gemini-3-flash".to_string(),
        "claude".to_string(),
        "gemini-3-pro-high".to_string(),
        "gemini-3.1-flash-image".to_string(),
    ]
}

impl ScheduledWarmupConfig {
    pub fn new() -> Self {
        Self {
            enabled: false,
            monitored_models: default_warmup_models(),
        }
    }
}

impl Default for ScheduledWarmupConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Quota protection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaProtectionConfig {
    /// Whether quota protection is enabled
    pub enabled: bool,

    /// Reserved quota percentage (1-99)
    pub threshold_percentage: u32,

    /// List of monitored models (e.g. gemini-3-flash, gemini-3-pro-high, gemini-3.1-pro-high, claude-sonnet-4-6)
    #[serde(default = "default_monitored_models")]
    pub monitored_models: Vec<String>,
}

fn default_monitored_models() -> Vec<String> {
    vec![
        "claude".to_string(),
        "gemini-3-pro-high".to_string(),
        "gemini-3-flash".to_string(),
        "gemini-3.1-flash-image".to_string(),
    ]
}

impl QuotaProtectionConfig {
    pub fn new() -> Self {
        Self {
            enabled: false,
            threshold_percentage: 10, // Default 10% reserve
            monitored_models: default_monitored_models(),
        }
    }
}

impl Default for QuotaProtectionConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Pinned quota models configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedQuotaModelsConfig {
    /// List of pinned models (displayed outside the account list)
    #[serde(default = "default_pinned_models")]
    pub models: Vec<String>,
}

fn default_pinned_models() -> Vec<String> {
    vec![
        "gemini-3-pro-high".to_string(),
        "gemini-3-flash".to_string(),
        "gemini-3.1-flash-image".to_string(),
        "claude-sonnet-4-6-thinking".to_string(),
    ]
}

impl PinnedQuotaModelsConfig {
    pub fn new() -> Self {
        Self {
            models: default_pinned_models(),
        }
    }
}

impl Default for PinnedQuotaModelsConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Whether circuit breaker is enabled
    pub enabled: bool,

    /// Unified backoff steps (seconds)
    /// Default: [60, 300, 1800, 7200]
    #[serde(default = "default_backoff_steps")]
    pub backoff_steps: Vec<u64>,

    /// Optional 5h zero-quota lock; exhausted weekly quota always blocks scheduling.
    #[serde(default = "default_lock_on_zero_quota")]
    pub lock_on_zero_quota: bool,
}

fn default_backoff_steps() -> Vec<u64> {
    vec![60, 300, 1800, 7200]
}

fn default_lock_on_zero_quota() -> bool {
    false
}

impl CircuitBreakerConfig {
    pub fn new() -> Self {
        Self {
            enabled: true,
            backoff_steps: default_backoff_steps(),
            lock_on_zero_quota: false,
        }
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            language: crate::modules::i18n::default_language(),
            theme: "system".to_string(),
            auto_refresh: true,
            refresh_interval: 15,
            auto_sync: true,
            auto_sync_migrated: true,
            sync_interval: 5,
            default_export_path: None,
            proxy: ProxyConfig::default(),
            antigravity_executable: None,
            antigravity_ide_executable: None,
            antigravity_cli_executable: None,
            antigravity_args: None,
            auto_launch: false,
            scheduled_warmup: ScheduledWarmupConfig::default(),
            quota_protection: QuotaProtectionConfig::default(),
            pinned_quota_models: PinnedQuotaModelsConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            hidden_menu_items: Vec::new(),
            cloudflared: CloudflaredConfig::default(),
            auto_profile_switcher: AutoProfileSwitcherConfig::default(),
            instance_clone_mode: default_instance_clone_mode(),
            conversation_cleanup: ConversationCleanupConfig::default(),
            lightweight_mode: false,
            training_api_enabled: false,
        }
    }
}

fn default_caution_interval() -> u32 {
    60
}

fn default_critical_interval() -> u32 {
    40
}

fn default_critical_threshold() -> f64 {
    12.0
}

fn default_true() -> bool {
    true
}

fn default_watchdog_interval() -> u32 {
    120
}

fn default_recency_threshold() -> u32 {
    3600
}

/// Auto profile switcher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoProfileSwitcherConfig {
    pub is_enabled: bool,
    pub check_interval_seconds: u32,
    pub low_quota_threshold_percent: f64,
    pub target_model: String,
    pub has_auto_resume: bool,
    pub cooldown_seconds: u32,
    #[serde(default = "default_caution_interval")]
    pub caution_interval_seconds: u32,
    #[serde(default = "default_critical_interval")]
    pub critical_interval_seconds: u32,
    #[serde(default = "default_critical_threshold")]
    pub critical_threshold_percent: f64,
    #[serde(default = "default_true")]
    pub auto_fast_forward_on_critical: bool,
    #[serde(default = "default_true")]
    pub auto_resume_recent_prompts: bool,
    #[serde(default = "default_false")]
    pub auto_focus_window: bool,
    #[serde(default = "default_watchdog_interval")]
    pub watchdog_interval_seconds: u32,
    #[serde(default = "default_recency_threshold")]
    pub prompt_recency_threshold_seconds: u32,
    #[serde(default = "default_fast_forward_shortcut")]
    pub fast_forward_shortcut: String,
}

impl Default for AutoProfileSwitcherConfig {
    fn default() -> Self {
        Self {
            is_enabled: true,
            check_interval_seconds: 300,
            low_quota_threshold_percent: 15.0,
            target_model: "gemini-3.8-flash-high".to_string(),
            has_auto_resume: true,
            cooldown_seconds: 180,
            caution_interval_seconds: 60,
            critical_interval_seconds: 40,
            critical_threshold_percent: 12.0,
            auto_fast_forward_on_critical: true,
            auto_resume_recent_prompts: true,
            auto_focus_window: false,
            watchdog_interval_seconds: 120,
            prompt_recency_threshold_seconds: 3600,
            fast_forward_shortcut: "Ctrl+Shift+F".to_string(),
        }
    }
}

fn default_fast_forward_shortcut() -> String {
    "Ctrl+Shift+F".to_string()
}

fn default_false() -> bool {
    false
}

fn default_cleanup_interval_hours() -> u32 {
    1
}

fn default_cleanup_keep_count() -> usize {
    40
}

/// Conversation cleanup and retention configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationCleanupConfig {
    #[serde(default = "default_false", alias = "enabled")]
    pub is_enabled: bool,
    #[serde(default = "default_cleanup_interval_hours")]
    pub interval_hours: u32,
    #[serde(default = "default_cleanup_keep_count")]
    pub keep_count: usize,
}

impl Default for ConversationCleanupConfig {
    fn default() -> Self {
        Self {
            is_enabled: false,
            interval_hours: 1,
            keep_count: 40,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::AppConfig;

    #[test]
    fn saved_language_is_preserved_when_loading_config() {
        let mut config = AppConfig::new();
        for language in ["en", "zh", "zh-TW", "ru"] {
            config.language = language.to_string();
            let saved = serde_json::to_string(&config).unwrap();
            let restored: AppConfig = serde_json::from_str(&saved).unwrap();
            assert_eq!(restored.language, language);
        }
    }

    #[test]
    fn test_auto_sync_default_is_true() {
        let config = AppConfig::new();
        assert!(config.auto_sync);
        assert!(config.auto_sync_migrated);
        assert_eq!(config.sync_interval, 5);

        // Deserializing JSON without auto_sync should default to true
        let json_str =
            r#"{"language":"en","theme":"system","auto_refresh":true,"refresh_interval":15}"#;
        let restored: AppConfig = serde_json::from_str(json_str).unwrap();
        assert!(restored.auto_sync);
        assert!(restored.auto_sync_migrated);
        assert_eq!(restored.sync_interval, 5);
    }
}
