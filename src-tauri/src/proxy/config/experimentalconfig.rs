use super::*;

/// Experimental feature configuration (Feature Flags)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentalConfig {
    /// Enable double-layer signature cache (Signature Cache)
    #[serde(default = "default_true")]
    pub enable_signature_cache: bool,

    /// Enable automatic tool loop recovery (Tool Loop Recovery)
    #[serde(default = "default_true")]
    pub enable_tool_loop_recovery: bool,

    /// Enable cross-model compatibility checks (Cross-Model Checks)
    #[serde(default = "default_true")]
    pub enable_cross_model_checks: bool,

    /// Enable Context Usage Scaling
    /// Aggressive mode: Scale usage and activate automatic compression to exceed the 200k limit
    /// Turned off by default to maintain transparency and allow clients to trigger native compression commands
    #[serde(default = "default_false")]
    pub enable_usage_scaling: bool,

    /// Compression Level
    /// disabled, low, medium, high
    #[serde(default = "default_compression_level")]
    pub compression_level: String,

    /// Context compression threshold L1 (Tool Trimming)
    #[serde(default = "default_threshold_l1")]
    pub context_compression_threshold_l1: f32,

    /// Context compression threshold L2 (Thinking Compression)
    #[serde(default = "default_threshold_l2")]
    pub context_compression_threshold_l2: f32,

    /// Context compression threshold L3 (Fork + Summary)
    #[serde(default = "default_threshold_l3")]
    pub context_compression_threshold_l3: f32,

    /// Payload storage mode: `simple` (default concise storage) or `full` (raw payload)
    #[serde(default = "default_payload_storage_mode")]
    pub payload_storage_mode: String,

    /// Request log retention days
    #[serde(default = "default_log_retention_days")]
    pub log_retention_days: u32,

    /// Server-side thinking block backfill (default enabled)
    #[serde(default = "default_thinking_store_enabled")]
    pub thinking_store_enabled: bool,

    /// Thinking block SQLite retention days
    #[serde(default = "default_thinking_retention_days")]
    pub thinking_retention_days: u32,

    /// Maximum thinking turns in memory per session (default 600)
    #[serde(default = "default_thinking_max_memory_turns")]
    pub thinking_max_memory_turns: u32,
}

impl Default for ExperimentalConfig {
    fn default() -> Self {
        Self {
            enable_signature_cache: true,
            enable_tool_loop_recovery: true,
            enable_cross_model_checks: true,
            enable_usage_scaling: false,
            compression_level: "disabled".to_string(),
            context_compression_threshold_l1: 0.4,
            context_compression_threshold_l2: 0.55,
            context_compression_threshold_l3: 0.7,
            payload_storage_mode: default_payload_storage_mode(),
            log_retention_days: default_log_retention_days(),
            thinking_store_enabled: default_thinking_store_enabled(),
            thinking_retention_days: default_thinking_retention_days(),
            thinking_max_memory_turns: default_thinking_max_memory_turns(),
        }
    }
}

pub(crate) fn default_thinking_max_memory_turns() -> u32 {
    600
}

pub(crate) fn default_threshold_l1() -> f32 {
    0.4
}

pub(crate) fn default_threshold_l2() -> f32 {
    0.55
}

pub(crate) fn default_threshold_l3() -> f32 {
    0.7
}

pub(crate) fn default_compression_level() -> String {
    "disabled".to_string()
}

pub(crate) fn default_payload_storage_mode() -> String {
    "simple".to_string()
}

pub(crate) fn default_log_retention_days() -> u32 {
    30
}

pub(crate) fn default_thinking_store_enabled() -> bool {
    true
}

pub(crate) fn default_thinking_retention_days() -> u32 {
    15
}

/// Thinking budget authority control source
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ThinkingControlSource {
    /// Gateway authority control (preferred / recommended): gateway manages parsing and injection
    Gateway,
    /// Client direct control: passthrough client-supplied thinking budget directly to upstream
    Client,
}

impl Default for ThinkingControlSource {
    fn default() -> Self {
        Self::Gateway
    }
}

/// Thinking Budget Mode
/// Controls how thinking budget is handled during model invocations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ThinkingBudgetMode {
    /// Default mode (official adaptive): passthrough official model ID without injecting thinkingBudget
    #[serde(rename = "default")]
    Default,
    /// Custom mode: inject configured budget per tier
    #[serde(rename = "custom")]
    Custom,
    /// Legacy compatibility mode: auto clamp
    #[serde(rename = "auto")]
    Auto,
    /// Legacy compatibility mode: passthrough
    #[serde(rename = "passthrough")]
    Passthrough,
    /// Legacy compatibility mode: adaptive
    #[serde(rename = "adaptive")]
    Adaptive,
}

impl Default for ThinkingBudgetMode {
    fn default() -> Self {
        Self::Custom
    }
}

/// Thinking Budget Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBudgetConfig {
    /// Authority control: Gateway control vs Client control
    #[serde(default = "default_control_source")]
    pub control_source: ThinkingControlSource,

    // --- Gemini Flash Tier Configurations ---
    #[serde(default = "default_thinking_budget_mode")]
    pub flash_mode: ThinkingBudgetMode,
    #[serde(default = "default_flash_low")]
    pub flash_low: i32, // Default 1000
    #[serde(default = "default_flash_medium")]
    pub flash_medium: i32, // Default 4000
    #[serde(default = "default_flash_high")]
    pub flash_high: i32, // Default 10000
    #[serde(default = "default_flash_tiered")]
    pub flash_tiered: i32, // Default -1

    // --- Gemini Pro Tier Configurations ---
    #[serde(default = "default_thinking_budget_mode")]
    pub pro_mode: ThinkingBudgetMode,
    #[serde(default = "default_pro_low")]
    pub pro_low: i32, // Default 1001
    #[serde(default = "default_pro_high")]
    pub pro_high: i32, // Default 10001

    // --- Claude Series Configurations ---
    #[serde(default = "default_thinking_budget_mode")]
    pub claude_mode: ThinkingBudgetMode,
    #[serde(default = "default_claude_budget")]
    pub claude_budget: i32, // Unified thinking budget (default 16000, -1 for adaptive)
    #[serde(default = "default_claude_low")]
    pub claude_low: i32, // Default 1024
    #[serde(default = "default_claude_medium")]
    pub claude_medium: i32, // Default 4096
    #[serde(default = "default_claude_high")]
    pub claude_high: i32, // Default 16000

    // --- Backward Compatibility Fields ---
    #[serde(default = "default_thinking_budget_mode")]
    pub mode: ThinkingBudgetMode,
    #[serde(default = "default_thinking_budget_custom_value")]
    pub custom_value: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(default = "default_flash_low")]
    pub custom_low: i32,
    #[serde(default = "default_flash_medium")]
    pub custom_medium: i32,
    #[serde(default = "default_flash_high")]
    pub custom_high: i32,
    #[serde(default = "default_flash_tiered")]
    pub custom_tiered: i32,
}

pub(crate) fn default_control_source() -> ThinkingControlSource {
    ThinkingControlSource::Gateway
}

pub(crate) fn default_thinking_budget_mode() -> ThinkingBudgetMode {
    ThinkingBudgetMode::Custom
}

pub(crate) fn default_flash_low() -> i32 {
    1024
}

pub(crate) fn default_flash_medium() -> i32 {
    4096
}

pub(crate) fn default_flash_high() -> i32 {
    16384
}

pub(crate) fn default_flash_tiered() -> i32 {
    -1
}

pub(crate) fn default_pro_low() -> i32 {
    1001
}

pub(crate) fn default_pro_high() -> i32 {
    10001
}

pub(crate) fn default_claude_budget() -> i32 {
    16384
}

pub(crate) fn default_claude_low() -> i32 {
    1024
}

pub(crate) fn default_claude_medium() -> i32 {
    4096
}

pub(crate) fn default_claude_high() -> i32 {
    16384
}

impl Default for ThinkingBudgetConfig {
    fn default() -> Self {
        Self {
            control_source: default_control_source(),
            flash_mode: default_thinking_budget_mode(),
            flash_low: default_flash_low(),
            flash_medium: default_flash_medium(),
            flash_high: default_flash_high(),
            flash_tiered: default_flash_tiered(),

            pro_mode: default_thinking_budget_mode(),
            pro_low: default_pro_low(),
            pro_high: default_pro_high(),

            claude_mode: default_thinking_budget_mode(),
            claude_budget: default_claude_budget(),
            claude_low: default_claude_low(),
            claude_medium: default_claude_medium(),
            claude_high: default_claude_high(),

            mode: default_thinking_budget_mode(),
            custom_value: default_thinking_budget_custom_value(),
            effort: None,
            custom_low: default_flash_low(),
            custom_medium: default_flash_medium(),
            custom_high: default_flash_high(),
            custom_tiered: default_flash_tiered(),
        }
    }
}

pub(crate) fn default_thinking_budget_custom_value() -> u32 {
    24576
}

pub(crate) fn default_true() -> bool {
    true
}

pub(crate) fn default_false() -> bool {
    false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugLoggingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub output_dir: Option<String>,
}

impl Default for DebugLoggingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            output_dir: None,
        }
    }
}

/// IP blacklist configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpBlacklistConfig {
    /// Whether to enable blacklist
    #[serde(default)]
    pub enabled: bool,

    /// Custom ban message
    #[serde(default = "default_block_message")]
    pub block_message: String,
}

impl Default for IpBlacklistConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            block_message: default_block_message(),
        }
    }
}

pub(crate) fn default_block_message() -> String {
    "Access denied".to_string()
}

/// IP whitelist configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpWhitelistConfig {
    /// Whether to enable whitelist mode (only whitelist IP access is allowed after enabling)
    #[serde(default)]
    pub enabled: bool,

    /// Whitelist priority mode (whitelist IP skips blacklist check)
    #[serde(default = "default_true")]
    pub whitelist_priority: bool,
}

impl Default for IpWhitelistConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            whitelist_priority: true,
        }
    }
}

/// Security monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMonitorConfig {
    /// IP blacklist configuration
    #[serde(default)]
    pub blacklist: IpBlacklistConfig,

    /// IP whitelist configuration
    #[serde(default)]
    pub whitelist: IpWhitelistConfig,
}

impl Default for SecurityMonitorConfig {
    fn default() -> Self {
        Self {
            blacklist: IpBlacklistConfig::default(),
            whitelist: IpWhitelistConfig::default(),
        }
    }
}

/// Image task scheduling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSchedulerConfig {
    #[serde(default = "default_image_per_account_concurrency")]
    pub per_account_concurrency: usize,
}

impl Default for ImageSchedulerConfig {
    fn default() -> Self {
        Self {
            per_account_concurrency: default_image_per_account_concurrency(),
        }
    }
}

pub(crate) fn default_image_per_account_concurrency() -> usize {
    4
}
