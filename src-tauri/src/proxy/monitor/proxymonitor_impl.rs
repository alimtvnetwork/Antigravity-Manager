use super::*;

impl ProxyMonitor {
    pub fn new(max_logs: usize, app_handle: Option<tauri::AppHandle>) -> Self {
        // Initialize DB
        if let Err(e) = crate::modules::proxy_db::init_db() {
            tracing::error!("Failed to initialize proxy DB: {}", e);
        }

        let thinking_days = crate::proxy::config::get_thinking_retention_days() as i64;
        let retention = crate::modules::config::load_app_config()
            .map(|config| config.proxy.log_retention)
            .unwrap_or_default();
        tokio::task::spawn_blocking(move || {
            match crate::modules::proxy_db::apply_retention(&retention) {
                Ok((cleared, deleted)) => {
                    if cleared > 0 || deleted > 0 {
                        tracing::info!(
                            "Proxy log retention: cleared {} bodies, deleted {} rows",
                            cleared,
                            deleted
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to cleanup old logs: {}", e);
                }
            }
            match crate::modules::proxy_db::cleanup_old_thinking_records(thinking_days) {
                Ok(deleted) => {
                    if deleted > 0 {
                        tracing::info!(
                            "Auto cleanup: removed {} old thinking/signature records (>{} days)",
                            deleted,
                            thinking_days
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to cleanup thinking records: {}", e);
                }
            }
        });

        tokio::spawn(async {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
            interval.tick().await;
            loop {
                interval.tick().await;
                let thinking_days = crate::proxy::config::get_thinking_retention_days() as i64;
                let retention = crate::modules::config::load_app_config()
                    .map(|config| config.proxy.log_retention)
                    .unwrap_or_default();
                let result = tokio::task::spawn_blocking(move || {
                    let retention_res = crate::modules::proxy_db::apply_retention(&retention);
                    let thinking_res =
                        crate::modules::proxy_db::cleanup_old_thinking_records(thinking_days);
                    (retention_res, thinking_res)
                })
                .await;
                match result {
                    Ok((retention_res, thinking_res)) => {
                        match retention_res {
                            Ok((cleared, deleted)) => {
                                if cleared > 0 || deleted > 0 {
                                    tracing::info!(
                                        "Proxy log retention: cleared {} bodies, deleted {} rows",
                                        cleared,
                                        deleted
                                    );
                                }
                            }
                            Err(error) => {
                                tracing::error!("Failed to apply proxy log retention: {}", error)
                            }
                        }
                        if let Ok(deleted) = thinking_res {
                            if deleted > 0 {
                                tracing::info!(
                                    "Auto cleanup: removed {} old thinking/signature records",
                                    deleted
                                );
                            }
                        } else if let Err(e) = thinking_res {
                            tracing::error!("Failed to cleanup thinking records: {}", e);
                        }
                    }
                    Err(error) => tracing::error!("Proxy log retention task failed: {}", error),
                }
            }
        });

        Self {
            logs: RwLock::new(VecDeque::with_capacity(max_logs)),
            stats: RwLock::new(ProxyStats::default()),
            max_logs,
            enabled: Arc::new(AtomicBool::new(false)), // Default to disabled
            capture_health_logs: Arc::new(AtomicBool::new(false)), // Default to false
            app_handle,
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    pub fn set_capture_health_logs(&self, enabled: bool) {
        self.capture_health_logs.store(enabled, Ordering::Relaxed);
    }

    pub fn is_capture_health_logs(&self) -> bool {
        self.capture_health_logs.load(Ordering::Relaxed)
    }

    pub async fn log_request(&self, log: ProxyRequestLog) {
        if let (Some(account), Some(input), Some(output)) =
            (&log.account_email, log.input_tokens, log.output_tokens)
        {
            let model = log.model.clone().unwrap_or_else(|| "unknown".to_string());
            let account = account.clone();
            let cached = log.cached_tokens.unwrap_or(0);
            tokio::task::spawn_blocking(move || {
                if let Err(e) = crate::modules::token_stats::record_usage(
                    &account, &model, input, output, cached,
                ) {
                    tracing::debug!("Failed to record token stats: {}", e);
                }
            });
        }

        if !self.is_enabled() {
            return;
        }
        tracing::info!("[Monitor] Logging request: {} {}", log.method, log.url);
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
            if log.status >= 200 && log.status < 400 {
                stats.success_count += 1;
            } else {
                stats.error_count += 1;
            }
        }

        let summary = log.summary();
        // Add only the same summary used by the frontend to memory.
        {
            let mut logs = self.logs.write().await;
            if logs.len() >= self.max_logs {
                logs.pop_back();
            }
            logs.push_front(summary.clone());
        }

        if let Some(app) = &self.app_handle {
            // Justification: best-effort frontend event; a dropped event only skips a UI refresh
            crate::error::record_ignored(
                app.emit("proxy://request", &summary),
                "emit proxy://request",
            );
        }

        let Ok(permit) = LOG_WRITERS.try_acquire() else {
            tracing::debug!("Skipping proxy log persistence: writers busy");
            return;
        };
        // Save to DB (respect server-side simple/full storage mode)
        let mut log_to_save = log;
        let storage_mode = crate::proxy::config::get_payload_storage_mode();
        log_to_save.request_body = crate::proxy::payload_audit::apply_storage_mode_to_body(
            log_to_save.request_body,
            &storage_mode,
        );
        log_to_save.upstream_request_body = crate::proxy::payload_audit::apply_storage_mode_to_body(
            log_to_save.upstream_request_body,
            &storage_mode,
        );
        log_to_save.response_body = crate::proxy::payload_audit::apply_storage_mode_to_body(
            log_to_save.response_body,
            &storage_mode,
        );

        let enabled = Arc::clone(&self.enabled);
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            if !enabled.load(Ordering::Relaxed) {
                return;
            }
            if let Err(e) = crate::modules::proxy_db::save_log(log_to_save) {
                tracing::error!("Failed to save proxy log to DB: {}", e);
            }

            // Sync to Security DB (IpAccessLogs) so it appears in Security Monitor
            if let Some(ip) = &summary.client_ip {
                let security_log = crate::modules::security_db::IpAccessLog {
                    id: uuid::Uuid::new_v4().to_string(),
                    client_ip: ip.clone(),
                    timestamp: summary.timestamp / 1000, // ms to s
                    method: Some(summary.method.clone()),
                    path: Some(summary.url.clone()),
                    user_agent: None, // We don't have UA in ProxyRequestLog easily accessible here without plumbing
                    status: Some(summary.status as i32),
                    duration: Some(summary.duration as i64),
                    api_key_hash: None,
                    blocked: false, // This comes from monitor, so it wasn't blocked by IP filter
                    block_reason: None,
                    username: summary.username.clone(),
                };

                if let Err(e) = crate::modules::security_db::save_ip_access_log(&security_log) {
                    tracing::error!("Failed to save security log: {}", e);
                }
            }
        });
    }

    pub async fn get_logs(&self, limit: usize) -> Vec<ProxyRequestLog> {
        // Try to get from DB first for true history
        let db_result =
            tokio::task::spawn_blocking(move || crate::modules::proxy_db::get_logs(limit)).await;

        match db_result {
            Ok(Ok(logs)) => logs,
            Ok(Err(e)) => {
                tracing::error!("Failed to get logs from DB: {}", e);
                // Fallback to memory
                let logs = self.logs.read().await;
                logs.iter().take(limit).cloned().collect()
            }
            Err(e) => {
                tracing::error!("Spawn blocking failed for get_logs: {}", e);
                let logs = self.logs.read().await;
                logs.iter().take(limit).cloned().collect()
            }
        }
    }

    pub async fn get_stats(&self) -> ProxyStats {
        let db_result = tokio::task::spawn_blocking(|| crate::modules::proxy_db::get_stats()).await;

        match db_result {
            Ok(Ok(stats)) => stats,
            Ok(Err(e)) => {
                tracing::error!("Failed to get stats from DB: {}", e);
                self.stats.read().await.clone()
            }
            Err(e) => {
                tracing::error!("Spawn blocking failed for get_stats: {}", e);
                self.stats.read().await.clone()
            }
        }
    }

    pub async fn get_logs_filtered(
        &self,
        page: usize,
        page_size: usize,
        search_text: Option<String>,
        level: Option<String>,
    ) -> Result<Vec<ProxyRequestLog>, String> {
        let offset = (page.max(1) - 1) * page_size;
        let errors_only = level.as_deref() == Some("error");
        let search = search_text.unwrap_or_default();

        let res = tokio::task::spawn_blocking(move || {
            crate::modules::proxy_db::get_logs_filtered(&search, errors_only, page_size, offset)
        })
        .await;

        match res {
            Ok(r) => r,
            Err(e) => Err(format!("Spawn blocking failed: {}", e)),
        }
    }

    pub async fn clear(&self) {
        let mut logs = self.logs.write().await;
        logs.clear();
        let mut stats = self.stats.write().await;
        *stats = ProxyStats::default();

        // Justification: best-effort call; failure logged without changing control flow
        crate::error::record_ignored(
            tokio::task::spawn_blocking(|| {
                if let Err(e) = crate::modules::proxy_db::clear_logs() {
                    tracing::error!("Failed to clear logs in DB: {}", e);
                }
            })
            .await,
            "spawn_blocking",
        );
    }
}
