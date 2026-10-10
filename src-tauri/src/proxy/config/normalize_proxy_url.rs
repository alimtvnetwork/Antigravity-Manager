use super::*;

// ============================================================================
// 辅助工具函数
// ============================================================================

/// 标准化代理 URL，如果缺失协议则默认补全 http://
pub fn normalize_proxy_url(url: &str) -> String {
    let url = url.trim();
    if url.is_empty() {
        return String::new();
    }
    if !url.contains("://") {
        format!("http://{}", url)
    } else {
        url.to_string()
    }
}

// ============================================================================
// 全局 Thinking Budget 配置存储
// 用于在 request transform 函数中访问配置（无需修改函数签名）
// ============================================================================
#[cfg(test)]
pub static TEST_CONFIG_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Returns true when running in CI/CD (e.g. GitHub Actions) unless AGM_RUN_HEAVY_LOCAL_TESTS=1 is explicitly set.
pub fn is_ci_environment() -> bool {
    if std::env::var("AGM_RUN_HEAVY_LOCAL_TESTS").as_deref() == Ok("1") {
        return false;
    }
    std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok()
}

static GLOBAL_THINKING_BUDGET_CONFIG: OnceLock<RwLock<ThinkingBudgetConfig>> = OnceLock::new();

#[cfg(test)]
pub static TEST_THINKING_BUDGET_MUTEX: &std::sync::Mutex<()> = &TEST_CONFIG_LOCK;

/// 获取当前 Thinking Budget 配置
pub fn get_thinking_budget_config() -> ThinkingBudgetConfig {
    GLOBAL_THINKING_BUDGET_CONFIG
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|cfg| cfg.clone())
        .unwrap_or_default()
}

/// 更新全局 Thinking Budget 配置
pub fn update_thinking_budget_config(config: ThinkingBudgetConfig) {
    if let Some(lock) = GLOBAL_THINKING_BUDGET_CONFIG.get() {
        if let Ok(mut cfg) = lock.write() {
            *cfg = config.clone();
            tracing::info!(
                "[Thinking-Budget] Global config updated: source={:?}, flash_mode={:?} (L:{}, M:{}, H:{}, T:{}), pro_mode={:?} (L:{}, H:{}), claude_mode={:?} (L:{}, M:{}, H:{})",
                config.control_source,
                config.flash_mode,
                config.flash_low,
                config.flash_medium,
                config.flash_high,
                config.flash_tiered,
                config.pro_mode,
                config.pro_low,
                config.pro_high,
                config.claude_mode,
                config.claude_low,
                config.claude_medium,
                config.claude_high
            );
        }
    } else {
        // Initial setup
        // Justification: OnceLock::set fails only if already initialized; double-init is a benign no-op by design.
        let _ = GLOBAL_THINKING_BUDGET_CONFIG.set(RwLock::new(config.clone()));
        tracing::info!(
            "[Thinking-Budget] Global config initialized: source={:?}, flash_mode={:?} (L:{}, M:{}, H:{}, T:{}), pro_mode={:?} (L:{}, H:{}), claude_mode={:?} (L:{}, M:{}, H:{})",
            config.control_source,
            config.flash_mode,
            config.flash_low,
            config.flash_medium,
            config.flash_high,
            config.flash_tiered,
            config.pro_mode,
            config.pro_low,
            config.pro_high,
            config.claude_mode,
            config.claude_low,
            config.claude_medium,
            config.claude_high
        );
    }
}

// ============================================================================
// 全局系统提示词配置存储
// 用户可在设置中配置一段全局提示词，自动注入到所有请求的 systemInstruction 中
// ============================================================================
static GLOBAL_SYSTEM_PROMPT_CONFIG: OnceLock<RwLock<GlobalSystemPromptConfig>> = OnceLock::new();

/// 获取当前全局系统提示词配置
pub fn get_global_system_prompt() -> GlobalSystemPromptConfig {
    GLOBAL_SYSTEM_PROMPT_CONFIG
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|cfg| cfg.clone())
        .unwrap_or_default()
}

/// 更新全局系统提示词配置
pub fn update_global_system_prompt_config(config: GlobalSystemPromptConfig) {
    if let Some(lock) = GLOBAL_SYSTEM_PROMPT_CONFIG.get() {
        if let Ok(mut cfg) = lock.write() {
            *cfg = config.clone();
            tracing::info!(
                "[Global-System-Prompt] Config updated: enabled={}, content_len={}",
                config.enabled,
                config.content.len()
            );
        }
    } else {
        // 首次初始化
        // Justification: OnceLock::set fails only if already initialized; double-init is a benign no-op by design.
        let _ = GLOBAL_SYSTEM_PROMPT_CONFIG.set(RwLock::new(config.clone()));
        tracing::info!(
            "[Global-System-Prompt] Config initialized: enabled={}, content_len={}",
            config.enabled,
            config.content.len()
        );
    }
}

// ============================================================================
// 全局图像思维模式配置存储
// ============================================================================
static GLOBAL_IMAGE_THINKING_MODE: OnceLock<RwLock<String>> = OnceLock::new();

pub fn get_image_thinking_mode() -> String {
    GLOBAL_IMAGE_THINKING_MODE
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|s| s.clone())
        .unwrap_or_else(|| "enabled".to_string())
}

pub fn update_image_thinking_mode(mode: Option<String>) {
    let val = mode.unwrap_or_else(|| "enabled".to_string());
    if let Some(lock) = GLOBAL_IMAGE_THINKING_MODE.get() {
        if let Ok(mut cfg) = lock.write() {
            if *cfg != val {
                *cfg = val.clone();
                tracing::info!("[Image-Thinking] Global config updated: {}", val);
            }
        }
    } else {
        // Justification: OnceLock::set fails only if already initialized; double-init is a benign no-op by design.
        let _ = GLOBAL_IMAGE_THINKING_MODE.set(RwLock::new(val.clone()));
    }
}

// ============================================================================
// 全局压缩等级配置存储
// ============================================================================
static GLOBAL_COMPRESSION_LEVEL: OnceLock<RwLock<String>> = OnceLock::new();

static GLOBAL_USAGE_SCALING: OnceLock<RwLock<bool>> = OnceLock::new();

static GLOBAL_THRESHOLD_L1: OnceLock<RwLock<f32>> = OnceLock::new();

static GLOBAL_THRESHOLD_L2: OnceLock<RwLock<f32>> = OnceLock::new();

static GLOBAL_THRESHOLD_L3: OnceLock<RwLock<f32>> = OnceLock::new();

static GLOBAL_PAYLOAD_STORAGE_MODE: OnceLock<RwLock<String>> = OnceLock::new();

static GLOBAL_LOG_RETENTION_DAYS: OnceLock<RwLock<u32>> = OnceLock::new();

static GLOBAL_THINKING_STORE_ENABLED: OnceLock<RwLock<bool>> = OnceLock::new();

static GLOBAL_THINKING_RETENTION_DAYS: OnceLock<RwLock<u32>> = OnceLock::new();

static GLOBAL_THINKING_MAX_MEMORY_TURNS: OnceLock<RwLock<u32>> = OnceLock::new();

pub(crate) fn write_or_init<T: Clone>(slot: &OnceLock<RwLock<T>>, value: T) {
    if let Some(lock) = slot.get() {
        if let Ok(mut cfg) = lock.write() {
            *cfg = value;
        }
    } else {
        // Justification: OnceLock::set fails only if already initialized; double-init is a benign no-op by design.
        let _ = slot.set(RwLock::new(value));
    }
}

pub fn get_payload_storage_mode() -> String {
    GLOBAL_PAYLOAD_STORAGE_MODE
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|v| v.clone())
        .unwrap_or_else(|| "simple".to_string())
}

pub fn get_log_retention_days() -> u32 {
    GLOBAL_LOG_RETENTION_DAYS
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|v| *v)
        .unwrap_or(30)
        .clamp(1, 3650)
}

pub fn is_thinking_store_enabled() -> bool {
    GLOBAL_THINKING_STORE_ENABLED
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|v| *v)
        .unwrap_or(true)
}

pub fn get_thinking_retention_days() -> u32 {
    GLOBAL_THINKING_RETENTION_DAYS
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|v| *v)
        .unwrap_or(15)
        .clamp(1, 3650)
}

pub fn get_thinking_max_memory_turns() -> usize {
    GLOBAL_THINKING_MAX_MEMORY_TURNS
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|v| *v as usize)
        .unwrap_or(600)
        .clamp(10, 10_000)
}

pub fn update_global_audit_config(
    payload_storage_mode: String,
    log_retention_days: u32,
    thinking_store_enabled: bool,
    thinking_retention_days: u32,
    thinking_max_memory_turns: Option<u32>,
) {
    let mode = if payload_storage_mode == "full" {
        "full"
    } else {
        "simple"
    };
    write_or_init(&GLOBAL_PAYLOAD_STORAGE_MODE, mode.to_string());
    write_or_init(
        &GLOBAL_LOG_RETENTION_DAYS,
        log_retention_days.clamp(1, 3650),
    );
    write_or_init(&GLOBAL_THINKING_STORE_ENABLED, thinking_store_enabled);
    write_or_init(
        &GLOBAL_THINKING_RETENTION_DAYS,
        thinking_retention_days.clamp(1, 3650),
    );
    let max_turns = thinking_max_memory_turns.unwrap_or(600).clamp(10, 10_000);
    write_or_init(&GLOBAL_THINKING_MAX_MEMORY_TURNS, max_turns);
    tracing::info!(
        "[Audit] storage_mode={}, log_retention_days={}, thinking_store={}, thinking_retention_days={}, thinking_max_memory_turns={}",
        mode,
        log_retention_days.clamp(1, 3650),
        thinking_store_enabled,
        thinking_retention_days.clamp(1, 3650),
        max_turns
    );
}

pub fn get_global_threshold_l1() -> f32 {
    GLOBAL_THRESHOLD_L1
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|v| *v)
        .unwrap_or(0.6)
}

pub fn get_global_threshold_l2() -> f32 {
    GLOBAL_THRESHOLD_L2
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|v| *v)
        .unwrap_or(0.75)
}

pub fn get_global_threshold_l3() -> f32 {
    GLOBAL_THRESHOLD_L3
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|v| *v)
        .unwrap_or(0.9)
}

pub fn get_global_compression_level() -> String {
    let level = GLOBAL_COMPRESSION_LEVEL
        .get()
        .and_then(|lock| lock.read().ok())
        .map(|cfg| cfg.clone())
        .unwrap_or_else(|| "disabled".to_string());

    if level == "disabled" {
        let scaling = GLOBAL_USAGE_SCALING
            .get()
            .and_then(|lock| lock.read().ok())
            .map(|s| *s)
            .unwrap_or(false);
        if scaling {
            "high".to_string()
        } else {
            "disabled".to_string()
        }
    } else {
        level
    }
}

pub fn update_global_compression_level(level: String, scaling: bool) {
    if let Some(lock) = GLOBAL_COMPRESSION_LEVEL.get() {
        if let Ok(mut cfg) = lock.write() {
            *cfg = level;
        }
    } else {
        // Justification: OnceLock::set fails only if already initialized; double-init is a benign no-op by design.
        let _ = GLOBAL_COMPRESSION_LEVEL.set(RwLock::new(level));
    }

    if let Some(lock) = GLOBAL_USAGE_SCALING.get() {
        if let Ok(mut cfg) = lock.write() {
            *cfg = scaling;
        }
    } else {
        // Justification: OnceLock::set fails only if already initialized; double-init is a benign no-op by design.
        let _ = GLOBAL_USAGE_SCALING.set(RwLock::new(scaling));
    }
}

pub fn update_global_thresholds(l1: f32, l2: f32, l3: f32) {
    if let Some(lock) = GLOBAL_THRESHOLD_L1.get() {
        if let Ok(mut cfg) = lock.write() {
            *cfg = l1;
        }
    } else {
        // Justification: OnceLock::set fails only if already initialized; double-init is a benign no-op by design.
        let _ = GLOBAL_THRESHOLD_L1.set(RwLock::new(l1));
    }

    if let Some(lock) = GLOBAL_THRESHOLD_L2.get() {
        if let Ok(mut cfg) = lock.write() {
            *cfg = l2;
        }
    } else {
        // Justification: OnceLock::set fails only if already initialized; double-init is a benign no-op by design.
        let _ = GLOBAL_THRESHOLD_L2.set(RwLock::new(l2));
    }

    if let Some(lock) = GLOBAL_THRESHOLD_L3.get() {
        if let Ok(mut cfg) = lock.write() {
            *cfg = l3;
        }
    } else {
        // Justification: OnceLock::set fails only if already initialized; double-init is a benign no-op by design.
        let _ = GLOBAL_THRESHOLD_L3.set(RwLock::new(l3));
    }
}

/// 全局系统提示词配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSystemPromptConfig {
    /// 是否启用全局系统提示词
    #[serde(default)]
    pub enabled: bool,
    /// 系统提示词内容
    #[serde(default)]
    pub content: String,
}

impl Default for GlobalSystemPromptConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            content: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyAuthMode {
    Off,
    Strict,
    AllExceptHealth,
    Auto,
}

impl Default for ProxyAuthMode {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ZaiDispatchMode {
    /// Never use z.ai.
    Off,
    /// Use z.ai for all Anthropic protocol requests.
    Exclusive,
    /// Treat z.ai as one additional slot in the shared pool.
    Pooled,
    /// Use z.ai only when the Google pool is unavailable.
    Fallback,
}

impl Default for ZaiDispatchMode {
    fn default() -> Self {
        Self::Off
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZaiModelDefaults {
    /// Default model for "opus" family (when the incoming model is a Claude id).
    #[serde(default = "default_zai_opus_model")]
    pub opus: String,
    /// Default model for "sonnet" family (when the incoming model is a Claude id).
    #[serde(default = "default_zai_sonnet_model")]
    pub sonnet: String,
    /// Default model for "haiku" family (when the incoming model is a Claude id).
    #[serde(default = "default_zai_haiku_model")]
    pub haiku: String,
}

impl Default for ZaiModelDefaults {
    fn default() -> Self {
        Self {
            opus: default_zai_opus_model(),
            sonnet: default_zai_sonnet_model(),
            haiku: default_zai_haiku_model(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZaiMcpConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub web_search_enabled: bool,
    #[serde(default)]
    pub web_reader_enabled: bool,
    #[serde(default)]
    pub vision_enabled: bool,
}

impl Default for ZaiMcpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            web_search_enabled: false,
            web_reader_enabled: false,
            vision_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZaiConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_zai_base_url")]
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub dispatch_mode: ZaiDispatchMode,
    /// Optional per-model mapping overrides for Anthropic/Claude model ids.
    /// Key: incoming `model` string, Value: upstream z.ai model id (e.g. `glm-4.7`).
    #[serde(default)]
    pub model_mapping: HashMap<String, String>,
    #[serde(default)]
    pub models: ZaiModelDefaults,
    #[serde(default)]
    pub mcp: ZaiMcpConfig,
}

impl Default for ZaiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: default_zai_base_url(),
            api_key: String::new(),
            dispatch_mode: ZaiDispatchMode::Off,
            model_mapping: HashMap::new(),
            models: ZaiModelDefaults::default(),
            mcp: ZaiMcpConfig::default(),
        }
    }
}
