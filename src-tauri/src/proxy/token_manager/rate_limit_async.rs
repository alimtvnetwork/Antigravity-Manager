//! 限流异步标记：fast/async 标记路径与持久化。

use super::helpers::classify_rate_limit_reason;
use super::helpers::truncate_reason;
use super::helpers::unix_timestamp_ceil;
use super::types::TrackerParserMode;
use super::TokenManager;

impl TokenManager {
    /// Register the in-memory image exclusion before releasing its account permit.
    /// Returns whether a slower quota refresh is still useful after the permit is released.
    pub async fn mark_rate_limited_fast(
        &self,
        email: &str,
        status: u16,
        retry_after_header: Option<&str>,
        error_body: &str,
        model: Option<&str>,
    ) -> bool {
        let normalized_model =
            model.and_then(crate::proxy::common::model_mapping::normalize_to_standard_id);
        let model_to_track = normalized_model.as_deref().or(model);
        let config = self.circuit_breaker_config.read().await.clone();
        if !config.enabled {
            return false;
        }

        let account_id = self
            .email_to_account_id(email)
            .unwrap_or_else(|| email.to_string());
        let has_explicit_retry_time = Self::has_explicit_retry_time(
            TrackerParserMode::Current,
            retry_after_header,
            error_body,
        );
        let reason = classify_rate_limit_reason(error_body);
        let recorded = self.record_rate_limit_atomic(
            &account_id,
            status,
            retry_after_header,
            error_body,
            model_to_track,
            &config.backoff_steps,
            TrackerParserMode::Current,
        );

        status == 429
            && recorded.is_some()
            && !has_explicit_retry_time
            && reason == crate::proxy::rate_limit::RateLimitReason::QuotaExhausted
    }

    pub async fn refresh_quota_lock_after_fast_mark(&self, email: &str, model: Option<&str>) {
        let normalized_model =
            model.and_then(crate::proxy::common::model_mapping::normalize_to_standard_id);
        let model_to_track = normalized_model.as_deref().or(model);
        let account_id = self
            .email_to_account_id(email)
            .unwrap_or_else(|| email.to_string());
        let reason = crate::proxy::rate_limit::RateLimitReason::QuotaExhausted;

        if self
            .fetch_and_lock_with_realtime_quota(email, reason, model_to_track.map(str::to_string))
            .await
        {
            tracing::info!("账号 {} 已使用实时配额精确锁定", email);
            return;
        }
        if self.set_precise_lockout(&account_id, reason, model_to_track.map(str::to_string)) {
            tracing::info!("账号 {} 已使用本地缓存配额锁定", account_id);
        }
    }

    pub async fn mark_rate_limited_async(
        &self,
        email: &str,
        status: u16,
        retry_after_header: Option<&str>,
        error_body: &str,
        model: Option<&str>,
    ) {
        self.mark_rate_limited_async_with_mode(
            email,
            status,
            retry_after_header,
            error_body,
            model,
            TrackerParserMode::Current,
        )
        .await;
    }

    pub async fn mark_rate_limited_async_baseline(
        &self,
        email: &str,
        status: u16,
        retry_after_header: Option<&str>,
        error_body: &str,
        model: Option<&str>,
    ) {
        self.mark_rate_limited_async_with_mode(
            email,
            status,
            retry_after_header,
            error_body,
            model,
            TrackerParserMode::Baseline,
        )
        .await;
    }

    async fn mark_rate_limited_async_with_mode(
        &self,
        email: &str,
        status: u16,
        retry_after_header: Option<&str>,
        error_body: &str,
        model: Option<&str>,
        parser_mode: TrackerParserMode,
    ) {
        // 关键门禁 1：仅对真正的上游 429 (配额耗尽/速率限制) 和 529 (Overloaded) 记录限流；500/503/404 等绝对不打入冷却池！
        if status != 429 && status != 529 {
            return;
        }

        // 关键门禁 2：内部错误文字（All accounts limited / No accounts available / Token pool is empty 等）严禁递归自锁！
        let lower_err = error_body.to_lowercase();
        if lower_err.contains("all accounts limited")
            || lower_err.contains("no accounts available")
            || lower_err.contains("all accounts failed")
            || lower_err.contains("token pool is empty")
            || lower_err.contains("all accounts exhausted")
            || lower_err.contains("all accounts unhealthy")
        {
            return;
        }

        let normalized_model =
            model.and_then(crate::proxy::common::model_mapping::normalize_to_standard_id);
        let model_to_track = normalized_model.as_deref().or(model);
        let config = self.circuit_breaker_config.read().await.clone();
        if !config.enabled {
            return;
        }

        let account_id = self
            .email_to_account_id(email)
            .unwrap_or_else(|| email.to_string());
        if Self::has_explicit_retry_time(parser_mode, retry_after_header, error_body) {
            self.record_rate_limit_atomic(
                &account_id,
                status,
                retry_after_header,
                error_body,
                model_to_track,
                &config.backoff_steps,
                parser_mode,
            );
            return;
        }

        let reason = classify_rate_limit_reason(error_body);
        if reason != crate::proxy::rate_limit::RateLimitReason::QuotaExhausted {
            self.record_rate_limit_atomic(
                &account_id,
                status,
                retry_after_header,
                error_body,
                model_to_track,
                &config.backoff_steps,
                parser_mode,
            );
            return;
        }

        if self
            .fetch_and_lock_with_realtime_quota(email, reason, model_to_track.map(str::to_string))
            .await
        {
            tracing::info!("账号 {} 已使用实时配额精确锁定", email);
            return;
        }
        if self.set_precise_lockout(&account_id, reason, model_to_track.map(str::to_string)) {
            tracing::info!("账号 {} 已使用本地缓存配额锁定", account_id);
            return;
        }

        tracing::warn!("账号 {} 无法获取配额刷新时间,使用指数退避策略", account_id);
        self.record_rate_limit_atomic(
            &account_id,
            status,
            retry_after_header,
            error_body,
            model_to_track,
            &config.backoff_steps,
            parser_mode,
        );
    }

    pub(crate) fn persist_live_limit_locked(
        &self,
        account_id: &str,
        model: Option<&str>,
        status: u16,
        retry_after_header: Option<&str>,
        error_body: &str,
        info: &crate::proxy::rate_limit::RateLimitInfo,
    ) {
        let Some(model_key) = model.and_then(crate::proxy::rate_limit::normalize_image_model_id)
        else {
            return;
        };
        let Some(explicit_delay_ms) =
            crate::proxy::upstream::retry::parse_retry_delay(error_body, retry_after_header)
        else {
            return;
        };
        if status != 429
            || info.reason != crate::proxy::rate_limit::RateLimitReason::QuotaExhausted
            || info.retry_after_sec <= 300
            || !crate::proxy::rate_limit::has_explicit_quota_exhausted(error_body)
        {
            return;
        }
        let (Some(until), Some(detected_at)) = (
            unix_timestamp_ceil(info.reset_time),
            unix_timestamp_ceil(info.detected_at),
        ) else {
            return;
        };

        let path = if let Some(entry) = self.tokens.get(account_id) {
            entry.account_path.clone()
        } else {
            self.resolved_data_dir()
                .join("accounts")
                .join(format!("{}.json", account_id))
        };

        let Ok(raw) = std::fs::read_to_string(&path) else {
            return;
        };
        let Ok(mut content) = serde_json::from_str::<serde_json::Value>(&raw) else {
            return;
        };

        if !content
            .get("live_limited_models")
            .and_then(|v| v.as_object())
            .is_some()
        {
            content["live_limited_models"] = serde_json::Value::Object(serde_json::Map::new());
        }

        content["live_limited_models"][&model_key] = serde_json::json!({
            "model": model_key,
            "status": status,
            "reason": format!("{:?}", info.reason),
            "until": until,
            "detected_at": detected_at,
            "message": format!(
                "QUOTA_EXHAUSTED; retry after {}ms; {}",
                explicit_delay_ms,
                truncate_reason(error_body, 400)
            ),
        });

        let Ok(serialized) = serde_json::to_string_pretty(&content) else {
            return;
        };
        if let Err(e) = std::fs::write(&path, serialized) {
            tracing::debug!("Failed to persist live limit for {}: {}", account_id, e);
        }
    }
}
