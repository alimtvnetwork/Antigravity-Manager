//! 核心类型：ProxyToken / TokenManager 结构体及内部枚举。

use crate::proxy::rate_limit::RateLimitTracker;
use crate::proxy::server::ImageScheduler;
use crate::proxy::sticky_config::StickySessionConfig;
use dashmap::DashMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::sync::Weak;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OnDiskAccountState {
    Enabled,
    Disabled,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TrackerParserMode {
    Current,
    Baseline,
}

#[derive(Debug, Clone)]
pub struct ProxyToken {
    pub account_id: String,
    pub priority: u8,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub timestamp: i64,
    pub email: String,
    pub account_path: PathBuf, // 账号文件路径，用于更新
    pub project_id: Option<String>,
    pub subscription_tier: Option<String>, // "FREE" | "PRO" | "ULTRA"
    pub remaining_quota: Option<i32>,      // [FIX #563] Remaining quota for priority sorting
    pub protected_models: HashSet<String>, // [NEW #621]
    pub health_score: f32,                 // [NEW] 健康分数 (0.0 - 1.0)
    pub reset_time: Option<i64>,           // [NEW] 配额刷新时间戳（用于排序优化）
    pub validation_blocked: bool, // [NEW] Check for validation block (VALIDATION_REQUIRED temporary block)
    pub validation_blocked_until: i64, // [NEW] Timestamp until which the account is blocked
    pub validation_url: Option<String>, // [NEW] Validation URL (#1522)
    pub model_quotas: HashMap<String, i32>, // [OPTIMIZATION] In-memory cache for model-specific quotas
    pub model_limits: HashMap<String, u64>, // [NEW] max_output_tokens per model from quota data
}

pub struct TokenManager {
    pub(crate) tokens: Arc<DashMap<String, ProxyToken>>, // account_id -> ProxyToken
    pub(crate) current_index: Arc<AtomicUsize>,
    pub(crate) last_used_account: Arc<tokio::sync::Mutex<Option<(String, std::time::Instant)>>>,
    pub(crate) data_dir: PathBuf,
    pub(crate) rate_limit_tracker: Arc<RateLimitTracker>, // 新增: 限流跟踪器
    pub(crate) sticky_config: Arc<tokio::sync::RwLock<StickySessionConfig>>, // 新增：调度配置
    pub(crate) session_accounts: Arc<DashMap<String, String>>, // 新增：会话与账号映射 (SessionID -> AccountID)
    pub(crate) preferred_account_id: Arc<tokio::sync::RwLock<Option<String>>>, // [FIX #820] 优先使用的账号ID（固定账号模式）
    pub(crate) health_scores: Arc<DashMap<String, f32>>, // account_id -> health_score
    pub(crate) circuit_breaker_config:
        Arc<tokio::sync::RwLock<crate::models::CircuitBreakerConfig>>, // [NEW] 熔断配置缓存

    // [NEW] 按账号分配的同步刷新锁。
    // 用于实现 Double-Checked Locking，防止并发请求导致单个账号短时间内多次调用 OAuth Refresh。
    pub(crate) refresh_locks: Arc<DashMap<String, Arc<tokio::sync::Mutex<()>>>>,

    // [NEW] loadCodeAssist (fetch_project_id) 的异步 SingleFlight 合并表
    // Key 为 account_id，Value 为结果观察者，确保并发请求共享同一个上游探测结果
    pub(crate) load_code_assist_inflight:
        Arc<DashMap<String, tokio::sync::watch::Receiver<Option<Result<String, String>>>>>,

    // [NEW] 记录账号连续 invalid_grant 失败次数，防止单次偶发网络抖动误停用账号
    pub(crate) invalid_grant_failures: Arc<DashMap<String, u32>>,

    /// 支持优雅关闭时主动 abort 后台任务
    pub(crate) auto_cleanup_handle: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
    pub(crate) cancel_token: CancellationToken,
    pub(crate) image_scheduler: std::sync::RwLock<Option<Weak<ImageScheduler>>>,
}
