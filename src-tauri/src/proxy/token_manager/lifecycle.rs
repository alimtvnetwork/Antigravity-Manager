//! Life cycle management: construction, background cleanup tasks, graceful shutdown, scheduler registration.

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

    /// Create a new TokenManager
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
            load_code_assist_inflight: Arc::new(DashMap::new()), // Initialize inflight table
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

    /// Start current-limiting record automatic cleaning background task (check and clear expired records every 15 seconds)
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

        // Abort the old task first (to prevent task leakage), and then store the new handle
        let mut guard = self.auto_cleanup_handle.lock().await;
        if let Some(old) = guard.take() {
            old.abort();
            tracing::warn!("Aborted previous auto-cleanup task");
        }
        *guard = Some(handle);

        tracing::info!("Rate limit auto-cleanup task started (interval: 15s)");
    }

    /// Send a cancellation signal first, then wait for the task to complete with a timeout
    ///
    /// # Parameters
    /// * `timeout` - timeout to wait for task completion
    pub async fn graceful_shutdown(&self, timeout: std::time::Duration) {
        tracing::info!("Initiating graceful shutdown of background tasks...");

        // Send cancellation signal to all background tasks
        self.cancel_token.cancel();

        // Wait for task completion with timeout
        match tokio::time::timeout(timeout, self.abort_background_tasks()).await {
            Ok(_) => tracing::info!("All background tasks cleaned up gracefully"),
            Err(_) => tracing::warn!(
                "Graceful cleanup timed out after {:?}, tasks were force-aborted",
                timeout
            ),
        }
    }

    /// Abort and wait for all background tasks to complete
    /// abort() only sets the cancellation flag and must await confirmation that cleanup is complete
    pub async fn abort_background_tasks(&self) {
        Self::abort_task(&self.auto_cleanup_handle, "Auto-cleanup task").await;
    }

    /// Abort a single background task and log the results
    ///
    /// # Parameters
    /// * `handle` - Mutex reference to the task handle
    /// * `task_name` - task name (for logging)
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

    /// Get the number of valid accounts in the current Token pool
    pub fn tokens_count(&self) -> usize {
        self.tokens.len()
    }
}
