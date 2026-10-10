//! 生命周期管理：构造、后台清理任务、优雅关闭、调度器注册。

use super::TokenManager;
use crate::proxy::rate_limit::RateLimitTracker;
use crate::proxy::server::ImageScheduler;
use crate::proxy::sticky_config::StickySessionConfig;
use dashmap::DashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::sync::Weak;
use tokio_util::sync::CancellationToken;

impl TokenManager {
    pub(crate) fn resolved_data_dir(&self) -> PathBuf {
        self.data_dir.clone()
    }

    /// 创建新的 TokenManager
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            tokens: Arc::new(DashMap::new()),
            current_index: Arc::new(AtomicUsize::new(0)),
            last_used_account: Arc::new(tokio::sync::Mutex::new(None)),
            data_dir,
            rate_limit_tracker: Arc::new(RateLimitTracker::new()),
            sticky_config: Arc::new(tokio::sync::RwLock::new(StickySessionConfig::default())),
            session_accounts: Arc::new(DashMap::new()),
            preferred_account_id: Arc::new(tokio::sync::RwLock::new(None)), // [FIX #820]
            health_scores: Arc::new(DashMap::new()),
            circuit_breaker_config: Arc::new(tokio::sync::RwLock::new(
                crate::models::CircuitBreakerConfig::default(),
            )),
            refresh_locks: Arc::new(DashMap::new()),
            load_code_assist_inflight: Arc::new(DashMap::new()), // 初始化 inflight 表
            invalid_grant_failures: Arc::new(DashMap::new()),
            auto_cleanup_handle: Arc::new(tokio::sync::Mutex::new(None)),
            cancel_token: CancellationToken::new(),
            image_scheduler: std::sync::RwLock::new(None),
        }
    }

    pub(crate) fn register_image_scheduler(&self, scheduler: &Arc<ImageScheduler>) {
        if let Ok(mut slot) = self.image_scheduler.write() {
            *slot = Some(Arc::downgrade(scheduler));
        }
        scheduler.sync_accounts(self.enabled_account_ids());
    }

    pub(crate) fn enabled_account_ids(&self) -> Vec<String> {
        self.tokens
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    pub(crate) fn sync_image_scheduler_accounts(&self) {
        let scheduler = self
            .image_scheduler
            .read()
            .ok()
            .and_then(|slot| slot.as_ref().and_then(Weak::upgrade));
        if let Some(scheduler) = scheduler {
            scheduler.sync_accounts(self.enabled_account_ids());
        }
    }

    /// 启动限流记录自动清理后台任务（每15秒检查并清除过期记录）
    pub async fn start_auto_cleanup(&self) {
        let tracker = self.rate_limit_tracker.clone();
        let cancel = self.cancel_token.child_token();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(15));
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => {
                        tracing::info!("Auto-cleanup task received cancel signal");
                        break;
                    }
                    _ = interval.tick() => {
                        let cleaned = tracker.cleanup_expired();
                        if cleaned > 0 {
                            tracing::info!(
                                "Auto-cleanup: Removed {} expired rate limit record(s)",
                                cleaned
                            );
                        }
                    }
                }
            }
        });

        // 先 abort 旧任务（防止任务泄漏），再存储新 handle
        let mut guard = self.auto_cleanup_handle.lock().await;
        if let Some(old) = guard.take() {
            old.abort();
            tracing::warn!("Aborted previous auto-cleanup task");
        }
        *guard = Some(handle);

        tracing::info!("Rate limit auto-cleanup task started (interval: 15s)");
    }

    /// 先发送取消信号，再带超时等待任务完成
    ///
    /// # 参数
    /// * `timeout` - 等待任务完成的超时时间
    pub async fn graceful_shutdown(&self, timeout: std::time::Duration) {
        tracing::info!("Initiating graceful shutdown of background tasks...");

        // 发送取消信号给所有后台任务
        self.cancel_token.cancel();

        // 带超时等待任务完成
        match tokio::time::timeout(timeout, self.abort_background_tasks()).await {
            Ok(_) => tracing::info!("All background tasks cleaned up gracefully"),
            Err(_) => tracing::warn!(
                "Graceful cleanup timed out after {:?}, tasks were force-aborted",
                timeout
            ),
        }
    }

    /// 中止并等待所有后台任务完成
    /// abort() 仅设置取消标志，必须 await 确认清理完成
    pub async fn abort_background_tasks(&self) {
        Self::abort_task(&self.auto_cleanup_handle, "Auto-cleanup task").await;
    }

    /// 中止单个后台任务并记录结果
    ///
    /// # 参数
    /// * `handle` - 任务句柄的 Mutex 引用
    /// * `task_name` - 任务名称（用于日志）
    async fn abort_task(
        handle: &tokio::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
        task_name: &str,
    ) {
        let Some(handle) = handle.lock().await.take() else {
            return;
        };

        handle.abort();
        match handle.await {
            Ok(()) => tracing::debug!("{} completed", task_name),
            Err(e) if e.is_cancelled() => tracing::info!("{} aborted", task_name),
            Err(e) => tracing::warn!("{} error: {}", task_name, e),
        }
    }

    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// 获取当前 Token 池内有效账号数量
    pub fn tokens_count(&self) -> usize {
        self.tokens.len()
    }
}
