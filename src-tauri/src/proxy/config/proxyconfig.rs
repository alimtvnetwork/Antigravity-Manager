use super::*;

/// 反代服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// 是否启用反代服务
    pub enabled: bool,

    /// 是否允许局域网访问
    /// - false: 仅本机访问 127.0.0.1（默认，隐私优先）
    /// - true: 允许局域网访问 0.0.0.0
    #[serde(default)]
    pub allow_lan_access: bool,

    /// Authorization policy for the proxy.
    /// - off: no auth required
    /// - strict: auth required for all routes
    /// - all_except_health: auth required for all routes except `/healthz`
    /// - auto: recommended defaults (currently: allow_lan_access => all_except_health, else off)
    #[serde(default)]
    pub auth_mode: ProxyAuthMode,

    /// 监听端口
    pub port: u16,

    /// API 密钥
    pub api_key: String,

    /// Web UI 管理后台密码 (可选，如未设置则使用 api_key)
    pub admin_password: Option<String>,

    /// 是否自动启动
    pub auto_start: bool,

    /// 自定义精确模型映射表 (key: 原始模型名, value: 目标模型名)
    #[serde(default)]
    pub custom_mapping: std::collections::HashMap<String, String>,

    /// API 请求超时时间(秒)
    #[serde(default = "default_request_timeout")]
    pub request_timeout: u64,

    /// 是否开启请求日志记录 (监控)
    #[serde(default)]
    pub enable_logging: bool,

    /// 是否捕获健康检查日志 (默认 false: 对 GET /health /healthz 请求全部过滤且不入库)
    #[serde(default)]
    pub capture_health_logs: bool,

    #[serde(default)]
    pub log_retention: LogRetentionConfig,

    /// 调试日志配置 (保存完整链路)
    #[serde(default)]
    pub debug_logging: DebugLoggingConfig,

    /// 上游代理配置
    #[serde(default)]
    pub upstream_proxy: UpstreamProxyConfig,

    /// 是否只在 /v1/models 中暴露真实配额模型（隐藏内置虚拟别名）
    #[serde(default)]
    pub only_raw_quota_models: bool,

    /// z.ai provider configuration (Anthropic-compatible).
    #[serde(default)]
    pub zai: ZaiConfig,

    /// 自定义 User-Agent 请求头 (可选覆盖)
    #[serde(default)]
    pub user_agent_override: Option<String>,

    /// 账号调度配置 (粘性会话/限流重试)
    #[serde(default)]
    pub scheduling: crate::proxy::sticky_config::StickySessionConfig,

    /// 实验性功能配置
    #[serde(default)]
    pub experimental: ExperimentalConfig,

    /// 安全监控配置 (IP 黑白名单)
    #[serde(default)]
    pub security_monitor: SecurityMonitorConfig,

    /// 固定账号模式的账号ID (Fixed Account Mode)
    /// - None: 使用轮询模式
    /// - Some(account_id): 固定使用指定账号
    #[serde(default)]
    pub preferred_account_id: Option<String>,

    /// Saved User-Agent string (persisted even when override is disabled)
    #[serde(default)]
    pub saved_user_agent: Option<String>,

    /// Thinking Budget 配置
    /// 控制如何处理 AI 深度思考时的 Token 预算
    #[serde(default)]
    pub thinking_budget: ThinkingBudgetConfig,

    /// 全局系统提示词配置
    /// 自动注入到所有 API 请求的 systemInstruction 中
    #[serde(default)]
    pub global_system_prompt: GlobalSystemPromptConfig,

    /// 图像思维模式配置
    /// - enabled: 保留思维链 (默认)
    /// - disabled: 移除思维链 (画质优先)
    #[serde(default)]
    pub image_thinking_mode: Option<String>,

    /// 图片上游任务的单账号并发数（重启后生效）
    #[serde(default)]
    pub image_scheduler: ImageSchedulerConfig,

    /// 代理池配置
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

/// 上游代理配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpstreamProxyConfig {
    /// 是否启用
    pub enabled: bool,
    /// 代理地址 (http://, https://, socks5://)
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
            allow_lan_access: false, // 默认仅本机访问，隐私优先
            auth_mode: ProxyAuthMode::default(),
            port: 8045,
            api_key: format!("sk-{}", uuid::Uuid::new_v4().simple()),
            admin_password: None,
            auto_start: false,
            custom_mapping: default_custom_mapping(),
            request_timeout: default_request_timeout(),
            enable_logging: true,       // 默认开启，支持 token 统计功能
            capture_health_logs: false, // 默认关闭，过滤 GET /health 探活且不入库
            log_retention: LogRetentionConfig::default(),
            debug_logging: DebugLoggingConfig::default(),
            upstream_proxy: UpstreamProxyConfig::default(),
            only_raw_quota_models: false,
            zai: ZaiConfig::default(),
            scheduling: crate::proxy::sticky_config::StickySessionConfig::default(),
            experimental: ExperimentalConfig::default(),
            security_monitor: SecurityMonitorConfig::default(),
            preferred_account_id: None, // 默认使用轮询模式
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
    120 // 默认 120 秒,原来 60 秒太短
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
    /// 获取实际的监听地址
    /// - allow_lan_access = false: 返回 "127.0.0.1"（默认，隐私优先）
    /// - allow_lan_access = true: 返回 "0.0.0.0"（通配监听：底层自动启用 IPv6/IPv4 双栈监听，允许局域网与外部公网 IPv4/IPv6 访问）
    pub fn get_bind_address(&self) -> &str {
        if self.allow_lan_access {
            "0.0.0.0"
        } else {
            "127.0.0.1"
        }
    }
}

/// 代理认证信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyAuth {
    pub username: String,
    #[serde(
        serialize_with = "crate::utils::crypto::serialize_password",
        deserialize_with = "crate::utils::crypto::deserialize_password"
    )]
    pub password: String,
}

/// 单个代理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyEntry {
    pub id: String,                       // 唯一标识
    pub name: String,                     // 显示名称
    pub url: String,                      // 代理地址 (http://, https://, socks5://)
    pub auth: Option<ProxyAuth>,          // 认证信息 (可选)
    pub enabled: bool,                    // 是否启用
    pub priority: i32,                    // 优先级 (数字越小优先级越高)
    pub tags: Vec<String>,                // 标签 (如 "美国", "住宅IP")
    pub max_accounts: Option<usize>,      // 最大绑定账号数 (0 = 无限制)
    pub health_check_url: Option<String>, // 健康检查 URL
    pub last_check_time: Option<i64>,     // 上次检查时间
    pub is_healthy: bool,                 // 健康状态
    pub latency: Option<u64>,             // 延迟 (毫秒) [NEW]
}

/// 代理池配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyPoolConfig {
    pub enabled: bool, // 是否启用代理池
    // pub mode: ProxyPoolMode,        // [REMOVED] 代理池模式，统一为 Hybrid 逻辑
    pub proxies: Vec<ProxyEntry>,         // 代理列表
    pub health_check_interval: u64,       // 健康检查间隔 (秒)
    pub auto_failover: bool,              // 自动故障转移
    pub strategy: ProxySelectionStrategy, // 代理选择策略
    /// 账号到代理的绑定关系 (account_id -> proxy_id)，持久化存储
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

/// 代理选择策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProxySelectionStrategy {
    /// 轮询: 依次使用
    RoundRobin,
    /// 随机: 随机选择
    Random,
    /// 优先级: 按 priority 字段排序
    Priority,
    /// 最少连接: 选择当前使用最少的代理
    LeastConnections,
    /// 加权轮询: 根据健康状态和优先级
    WeightedRoundRobin,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_normalize_proxy_url() {
        // 测试已有协议
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

        // 测试缺少协议（默认补全 http://）
        assert_eq!(
            normalize_proxy_url("127.0.0.1:7890"),
            "http://127.0.0.1:7890"
        );
        assert_eq!(
            normalize_proxy_url("localhost:1082"),
            "http://localhost:1082"
        );

        // 测试边缘情况
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
