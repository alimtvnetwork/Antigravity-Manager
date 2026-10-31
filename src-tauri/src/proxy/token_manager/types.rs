//! Core type: ProxyToken / TokenManager structure and internal enumeration.

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
    pub account_path: PathBuf, // Account file path, used for updates
    pub project_id: Option<String>,
    pub subscription_tier: Option<String>, // "FREE" | "PRO" | "ULTRA"
    pub remaining_quota: Option<i32>,      // [FIX #563] Remaining quota for priority sorting
    pub protected_models: HashSet<String>, // [NEW #621]
    pub health_score: f32,                 // [NEW] Health Score (0.0 - 1.0)
    pub reset_time: Option<i64>, // [NEW] Quota refresh timestamp (for sorting optimization)
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
    pub(crate) rate_limit_tracker: Arc<RateLimitTracker>, // New: Throttle Tracker
    pub(crate) sticky_config: Arc<tokio::sync::RwLock<StickySessionConfig>>, // New: Scheduling configuration
    pub(crate) session_accounts: Arc<DashMap<String, String>>, // New: Session and account mapping (SessionID -> AccountID)
    pub(crate) preferred_account_id: Arc<tokio::sync::RwLock<Option<String>>>, // [FIX #820] Priority account ID (fixed account mode)
    pub(crate) health_scores: Arc<DashMap<String, f32>>, // account_id -> health_score
    pub(crate) circuit_breaker_config:
        Arc<tokio::sync::RwLock<crate::models::CircuitBreakerConfig>>, // [NEW] Circuit breaker configuration cache

    // [NEW] Synchronous refresh lock assigned by account.
    // Used to implement Double-Checked Locking to prevent concurrent requests from causing a single account to call OAuth Refresh multiple times in a short period of time.
    pub(crate) refresh_locks: Arc<DashMap<String, Arc<tokio::sync::Mutex<()>>>>,

    // [NEW] Asynchronous SingleFlight merge table for loadCodeAssist (fetch_project_id)
    // Key is account_id, Value is the result observer, ensuring that concurrent requests share the same upstream detection result
    pub(crate) load_code_assist_inflight:
        Arc<DashMap<String, tokio::sync::watch::Receiver<Option<Result<String, String>>>>>,

    // [NEW] Record the number of consecutive invalid_grant failures for an account to prevent accidental account deactivation due to a single incident of network jitter.
    pub(crate) invalid_grant_failures: Arc<DashMap<String, u32>>,

    /// Supports active abort background tasks during graceful shutdown
    pub(crate) auto_cleanup_handle: Arc<tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>>,
    pub(crate) cancel_token: CancellationToken,
    pub(crate) image_scheduler: std::sync::RwLock<Option<Weak<ImageScheduler>>>,
}
