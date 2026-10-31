use super::*;

/// Anti-generation service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Whether to enable anti-generation service
    pub enabled: bool,

    /// Whether to allow LAN access
    /// - false: Only local access to 127.0.0.1 (default, privacy first)
    /// - true: Allow LAN access to 0.0.0.0
    #[serde(default)]
    pub allow_lan_access: bool,

    /// Authorization policy for the proxy.
    /// - off: no auth required
    /// - strict: auth required for all routes
    /// - all_except_health: auth required for all routes except `/healthz`
    /// - auto: recommended defaults (currently: allow_lan_access => all_except_health, else off)
    #[serde(default)]
    pub auth_mode: ProxyAuthMode,

    /// listening port
    pub port: u16,

    /// API key
    pub api_key: String,

    /// Web UI management background password (optional, use api_key if not set)
    pub admin_password: Option<String>,

    /// Whether to start automatically
    pub auto_start: bool,

    /// Customized accurate model mapping table (key: original model name, value: target model name)
    #[serde(default)]
    pub custom_mapping: std::collections::HashMap<String, String>,

    /// API request timeout (seconds)
    #[serde(default = "default_request_timeout")]
    pub request_timeout: u64,

    /// Whether to enable request logging (monitoring)
    #[serde(default)]
    pub enable_logging: bool,

    /// Whether to capture health check logs (default false: filter all GET /health /healthz requests and not store them in the database)
    #[serde(default)]
    pub capture_health_logs: bool,

    #[serde(default)]
    pub log_retention: LogRetentionConfig,

    /// Debug log configuration (save complete link)
    #[serde(default)]
    pub debug_logging: DebugLoggingConfig,

    /// Upstream proxy configuration
    #[serde(default)]
    pub upstream_proxy: UpstreamProxyConfig,

    /// Whether to expose real quota models only in /v1/models (hide built-in virtual aliases)
    #[serde(default)]
    pub only_raw_quota_models: bool,

    /// z.ai provider configuration (Anthropic-compatible).
    #[serde(default)]
    pub zai: ZaiConfig,

    /// Custom User-Agent request header (optional override)
    #[serde(default)]
    pub user_agent_override: Option<String>,

    /// Account scheduling configuration (sticky session/current limiting retry)
    #[serde(default)]
    pub scheduling: crate::proxy::sticky_config::StickySessionConfig,

    /// Experimental feature configuration
    #[serde(default)]
    pub experimental: ExperimentalConfig,

    /// Security monitoring configuration (IP black and white list)
    #[serde(default)]
    pub security_monitor: SecurityMonitorConfig,

    /// Account ID of fixed account mode (Fixed Account Mode)
    /// - None: use polling mode
    /// - Some(account_id): Fixed use of the specified account
    #[serde(default)]
    pub preferred_account_id: Option<String>,

    /// Saved User-Agent string (persisted even when override is disabled)
    #[serde(default)]
    pub saved_user_agent: Option<String>,

    /// Thinking Budget Configuration
    /// Control how to handle the token budget when AI deep thinking
    #[serde(default)]
    pub thinking_budget: ThinkingBudgetConfig,

    /// Global system prompt word configuration
    /// Automatically injected into systemInstruction of all API requests
    #[serde(default)]
    pub global_system_prompt: GlobalSystemPromptConfig,

    /// Image thinking mode configuration
    /// - enabled: retain thought chain (default)
    /// - disabled: remove thought chain (image quality priority)
    #[serde(default)]
    pub image_thinking_mode: Option<String>,

    /// The number of concurrent connections for a single account in the image upstream task (takes effect after restart)
    #[serde(default)]
    pub image_scheduler: ImageSchedulerConfig,

    /// Agent pool configuration
    #[serde(default)]
    pub proxy_pool: ProxyPoolConfig,

    #[serde(default = "default_true")]
    pub default_path_rewrite: bool,
    #[serde(default = "default_true")]
    pub enable_all_urls: bool,
    #[serde(default)]
    pub excluded_urls: Vec<String>,
}

/// Request log retention policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRetentionConfig {
    #[serde(default = "default_max_body_age_hours")]
    pub max_body_age_hours: u64,
    #[serde(default = "default_max_age_days")]
    pub max_age_days: u64,
    #[serde(default = "default_max_rows")]
    pub max_rows: u64,
    /// Application disk budget in MiB, including the database and WAL.
    #[serde(default = "default_max_disk_mb")]
    pub max_disk_mb: u64,
    /// Max log storage limit in GB (supports decimals, e.g. 0.5)
    #[serde(default = "default_max_storage_gb")]
    pub max_storage_gb: f64,
}

pub(crate) fn default_max_body_age_hours() -> u64 {
    24
}

pub(crate) fn default_max_age_days() -> u64 {
    30
}

pub(crate) fn default_max_rows() -> u64 {
    100_000
}

pub(crate) fn default_max_disk_mb() -> u64 {
    1024
}

pub(crate) fn default_max_storage_gb() -> f64 {
    1.0
}

impl LogRetentionConfig {
    pub fn budget_bytes(&self) -> u64 {
        if self.max_storage_gb > 0.0 {
            (self.max_storage_gb * 1024.0 * 1024.0 * 1024.0) as u64
        } else if self.max_disk_mb > 0 {
            self.max_disk_mb.saturating_mul(1024 * 1024)
        } else {
            0
        }
    }
}

impl Default for LogRetentionConfig {
    fn default() -> Self {
        Self {
            max_body_age_hours: 24,
            max_age_days: 30,
            max_rows: 100_000,
            max_disk_mb: 1024,
            max_storage_gb: 1.0,
        }
    }
}

/// Upstream proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpstreamProxyConfig {
    /// Whether to enable
    pub enabled: bool,
    /// Proxy address (http://, https://, socks5://)
    pub url: String,
}

pub fn default_custom_mapping() -> std::collections::HashMap<String, String> {
    let mut m = std::collections::HashMap::new();
    m.insert(
        "gemini-3.6-flash".to_string(),
        "gemini-3.6-flash-tiered".to_string(),
    );
    m.insert(
        "gemini-3.7-flash".to_string(),
        "gemini-3.7-flash-tiered".to_string(),
    );
    m.insert(
        "gemini-3.8-flash".to_string(),
        "gemini-3.8-flash-tiered".to_string(),
    );
    m.insert(
        "gemini-3.x-flash".to_string(),
        "3.x-flash-tiered".to_string(),
    );
    m
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allow_lan_access: false, // By default, only local access is available, privacy is priority
            auth_mode: ProxyAuthMode::default(),
            port: 8045,
            api_key: format!("sk-{}", uuid::Uuid::new_v4().simple()),
            admin_password: None,
            auto_start: false,
            custom_mapping: default_custom_mapping(),
            request_timeout: default_request_timeout(),
            enable_logging: true, // Enabled by default, supports token statistics function
            capture_health_logs: false, // Closed by default, filtering GET /health to detect activity and not storing it in the database
            log_retention: LogRetentionConfig::default(),
            debug_logging: DebugLoggingConfig::default(),
            upstream_proxy: UpstreamProxyConfig::default(),
            only_raw_quota_models: false,
            zai: ZaiConfig::default(),
            scheduling: crate::proxy::sticky_config::StickySessionConfig::default(),
            experimental: ExperimentalConfig::default(),
            security_monitor: SecurityMonitorConfig::default(),
            preferred_account_id: None, // Polling mode is used by default
            user_agent_override: None,
            saved_user_agent: None,
            thinking_budget: ThinkingBudgetConfig::default(),
            global_system_prompt: GlobalSystemPromptConfig::default(),
            proxy_pool: ProxyPoolConfig::default(),
            image_thinking_mode: None,
            image_scheduler: ImageSchedulerConfig::default(),
            default_path_rewrite: true,
            enable_all_urls: true,
            excluded_urls: Vec::new(),
        }
    }
}

pub(crate) fn default_request_timeout() -> u64 {
    120 // Default is 120 seconds, originally 60 seconds was too short
}

pub(crate) fn default_zai_base_url() -> String {
    "https://api.z.ai/api/anthropic".to_string()
}

pub(crate) fn default_zai_opus_model() -> String {
    "glm-4.7".to_string()
}

pub(crate) fn default_zai_sonnet_model() -> String {
    "glm-4.7".to_string()
}

pub(crate) fn default_zai_haiku_model() -> String {
    "glm-4.5-air".to_string()
}

impl ProxyConfig {
    /// Get the actual listening address
    /// - allow_lan_access = false: return "127.0.0.1" (default, privacy first)
    /// - allow_lan_access = true: returns "0.0.0.0" (wildcard listening: the bottom layer automatically enables IPv6/IPv4 dual-stack monitoring, allowing IPv4/IPv6 access to the LAN and external public networks)
    pub fn get_bind_address(&self) -> &str {
        if self.allow_lan_access {
            "0.0.0.0"
        } else {
            "127.0.0.1"
        }
    }
}

/// Agent authentication information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyAuth {
    pub username: String,
    #[serde(
        serialize_with = "crate::utils::crypto::serialize_password",
        deserialize_with = "crate::utils::crypto::deserialize_password"
    )]
    pub password: String,
}

/// Single agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyEntry {
    pub id: String,                       // unique identifier
    pub name: String,                     // display name
    pub url: String,                      // Proxy address (http://, https://, socks5://)
    pub auth: Option<ProxyAuth>,          // Authentication information (optional)
    pub enabled: bool,                    // Whether to enable
    pub priority: i32, // Priority (the lower the number, the higher the priority)
    pub tags: Vec<String>, // Tag (e.g. "United States", "Residential IP")
    pub max_accounts: Option<usize>, // Maximum number of bound accounts (0 = no limit)
    pub health_check_url: Option<String>, // Health check URL
    pub last_check_time: Option<i64>, // Last check time
    pub is_healthy: bool, // health status
    pub latency: Option<u64>, // Delay (milliseconds) [NEW]
}

/// Agent pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyPoolConfig {
    pub enabled: bool, // Whether to enable proxy pool
    // pub mode: ProxyPoolMode, // [REMOVED] Proxy pool mode, unified into Hybrid logic
    pub proxies: Vec<ProxyEntry>,         // proxy list
    pub health_check_interval: u64,       // Health check interval (seconds)
    pub auto_failover: bool,              // Automatic failover
    pub strategy: ProxySelectionStrategy, // Agent selection strategy
    /// Binding relationship between account and proxy (account_id -> proxy_id), persistent storage
    #[serde(default)]
    pub account_bindings: HashMap<String, String>,
}

impl Default for ProxyPoolConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            // mode: ProxyPoolMode::Global,
            proxies: Vec::new(),
            health_check_interval: 300,
            auto_failover: true,
            strategy: ProxySelectionStrategy::Priority,
            account_bindings: HashMap::new(),
        }
    }
}

/// Agent selection strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProxySelectionStrategy {
    /// Polling: Used sequentially
    RoundRobin,
    /// Random: randomly selected
    Random,
    /// Priority: Sort by priority field
    Priority,
    /// Least Connected: Select the proxy currently used the least
    LeastConnections,
    /// Weighted polling: based on health status and priority
    WeightedRoundRobin,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_normalize_proxy_url() {
        // Test existing protocols
        assert_eq!(
            normalize_proxy_url("http://127.0.0.1:7890"),
            "http://127.0.0.1:7890"
        );
        assert_eq!(
            normalize_proxy_url("https://proxy.com"),
            "https://proxy.com"
        );
        assert_eq!(
            normalize_proxy_url("socks5://127.0.0.1:1080"),
            "socks5://127.0.0.1:1080"
        );
        assert_eq!(
            normalize_proxy_url("socks5h://127.0.0.1:1080"),
            "socks5h://127.0.0.1:1080"
        );

        // Test missing protocol (default completion http://)
        assert_eq!(
            normalize_proxy_url("127.0.0.1:7890"),
            "http://127.0.0.1:7890"
        );
        assert_eq!(
            normalize_proxy_url("localhost:1082"),
            "http://localhost:1082"
        );

        // Test edge cases
        assert_eq!(normalize_proxy_url(""), "");
        assert_eq!(normalize_proxy_url("   "), "");
    }

    #[test]
    fn test_proxy_config_url_routing_defaults() {
        let default_cfg = ProxyConfig::default();
        assert!(default_cfg.default_path_rewrite);
        assert!(default_cfg.enable_all_urls);
        assert!(default_cfg.excluded_urls.is_empty());

        let json_str = r#"{
            "enabled": true,
            "port": 8045,
            "api_key": "test-key",
            "auto_start": false
        }"#;
        let parsed: ProxyConfig = serde_json::from_str(json_str).unwrap();
        assert!(parsed.default_path_rewrite);
        assert!(parsed.enable_all_urls);
        assert!(parsed.excluded_urls.is_empty());
    }
}
