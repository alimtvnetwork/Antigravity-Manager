use super::*;

pub(crate) const MAX_LOCKOUT_SECONDS: u64 = 300;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum RetryParserMode {
    Current,
    Baseline,
}

/// 限流原因类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RateLimitReason {
    /// 配额耗尽 (QUOTA_EXHAUSTED)
    QuotaExhausted,
    /// 速率限制 (RATE_LIMIT_EXCEEDED)
    RateLimitExceeded,
    /// 模型容量耗尽 (MODEL_CAPACITY_EXHAUSTED)
    ModelCapacityExhausted,
    /// 服务器错误 (5xx)
    ServerError,
    /// 未知原因
    Unknown,
}

pub(crate) fn normalize_image_model_id(model: &str) -> Option<String> {
    let normalized = crate::proxy::common::model_mapping::normalize_to_standard_id(model)?;
    matches!(
        normalized.as_str(),
        "gemini-3.1-flash-image" | "gemini-3-pro-image"
    )
    .then_some(normalized)
}

pub(crate) fn has_explicit_quota_exhausted(body: &str) -> bool {
    body.to_ascii_uppercase().contains("QUOTA_EXHAUSTED")
}

pub(crate) fn is_active_persisted_long_limit(
    _model_key: &str,
    status: &crate::models::account::LiveLimitStatus,
    now: i64,
) -> bool {
    status.status == 429
        && status.reason == "QuotaExhausted"
        && status.until > now
        && status.until.saturating_sub(status.detected_at) > MAX_LOCKOUT_SECONDS as i64
        && status.message.as_deref().is_some_and(|message| {
            has_explicit_quota_exhausted(message)
                && crate::proxy::upstream::retry::parse_retry_delay(message, None).is_some()
        })
}

pub(crate) fn is_active_persisted_long_image_limit(
    model_key: &str,
    status: &crate::models::account::LiveLimitStatus,
    now: i64,
) -> bool {
    normalize_image_model_id(model_key).is_some()
        && is_active_persisted_long_limit(model_key, status, now)
}

/// 限流信息
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// 限流重置时间
    pub reset_time: SystemTime,
    /// 重试间隔(秒)
    #[allow(dead_code)]
    pub retry_after_sec: u64,
    /// 检测时间
    #[allow(dead_code)]
    pub detected_at: SystemTime,
    /// 限流原因
    #[allow(dead_code)] // Used for logging and diagnostics
    pub reason: RateLimitReason,
    /// 关联的模型 (用于模型级别限流)
    /// None 表示账号级别限流,Some(model) 表示特定模型限流
    #[allow(dead_code)] // Used for model-level rate limiting
    pub model: Option<String>,
}

/// 失败计数过期时间：1小时（超过此时间未失败则重置计数）
const FAILURE_COUNT_EXPIRY_SECONDS: u64 = 3600;

/// 限流跟踪器
pub(crate) struct QuotaBucketLimit {
    pub(crate) observed_at: i64,
    pub(crate) reset_time: Option<SystemTime>,
    pub(crate) weekly: bool,
}

pub struct RateLimitTracker {
    pub(crate) limits: DashMap<String, RateLimitInfo>,
    // Independent official quota windows must survive transient-limit resets.
    pub(crate) quota_limits: DashMap<(String, String), QuotaBucketLimit>,
    /// 连续失败计数（用于智能指数退避），带时间戳用于自动过期
    pub(crate) failure_counts: DashMap<String, (u32, SystemTime)>,
}

impl RateLimitTracker {
    pub fn new() -> Self {
        Self {
            limits: DashMap::new(),
            quota_limits: DashMap::new(),
            failure_counts: DashMap::new(),
        }
    }

    /// 生成限流 Key
    /// - 账号级: "account_id"
    /// - 模型级: "account_id:model_id"
    pub(crate) fn get_limit_key(&self, account_id: &str, model: Option<&str>) -> String {
        match model {
            Some(m) if !m.is_empty() => format!("{}:{}", account_id, m),
            _ => account_id.to_string(),
        }
    }

    /// 获取账号剩余的等待时间(秒)
    /// 支持检查账号级和模型级锁
    pub fn get_remaining_wait(&self, account_id: &str, model: Option<&str>) -> u64 {
        let now = SystemTime::now();
        let model_key = self.get_limit_key(account_id, model);
        let direct_wait = [account_id, model_key.as_str()]
            .into_iter()
            .filter_map(|key| self.limits.get(key))
            .filter_map(|info| info.reset_time.duration_since(now).ok())
            .map(|duration| duration.as_secs().max(1))
            .max()
            .unwrap_or(0)
            .max(self.get_quota_wait(account_id, model, false));

        if direct_wait > 0 {
            return direct_wait;
        }

        // [双重守卫] 若直接匹配未命中，通过归一化标准 ID 兜底检查（确保 gemini-*-pro 等所有变体对齐标准组锁定）
        if let Some(m) = model {
            if let Some(std_id) = crate::proxy::common::model_mapping::normalize_to_standard_id(m) {
                if std_id != m {
                    let std_key = self.get_limit_key(account_id, Some(&std_id));
                    let std_wait = self
                        .limits
                        .get(&std_key)
                        .and_then(|info| info.reset_time.duration_since(now).ok())
                        .map(|duration| duration.as_secs().max(1))
                        .unwrap_or(0)
                        .max(self.get_quota_wait(account_id, Some(&std_id), false));
                    if std_wait > 0 {
                        return std_wait;
                    }
                }
            }
        }

        0
    }

    pub fn get_quota_wait(&self, account_id: &str, model: Option<&str>, weekly_only: bool) -> u64 {
        let key = self.get_limit_key(account_id, model);
        let now = SystemTime::now();
        self.quota_limits
            .iter()
            .filter(|entry| {
                (entry.key().0 == key || entry.key().0 == account_id)
                    && (!weekly_only || entry.weekly)
            })
            .filter_map(|entry| entry.reset_time?.duration_since(now).ok())
            .map(|duration| duration.as_secs().max(1))
            .max()
            .unwrap_or(0)
    }

    pub fn sync_quota_bucket(
        &self,
        account_id: &str,
        model: &str,
        bucket_id: &str,
        observed_at: i64,
        exhausted_until: Option<SystemTime>,
        weekly: bool,
    ) {
        let key = (
            self.get_limit_key(account_id, Some(model)),
            bucket_id.to_string(),
        );
        // Keep recovered observations too: reloading an older snapshot must not relock/unlock.
        self.quota_limits
            .entry(key)
            .and_modify(|current| {
                if observed_at > current.observed_at {
                    *current = QuotaBucketLimit {
                        observed_at,
                        reset_time: exhausted_until,
                        weekly,
                    };
                }
            })
            .or_insert(QuotaBucketLimit {
                observed_at,
                reset_time: exhausted_until,
                weekly,
            });
    }

    /// 标记账号请求成功，重置连续失败计数
    ///
    /// 当账号成功完成请求后调用此方法，将其失败计数归零，
    /// 这样下次失败时会从最短的锁定时间（60秒）开始。
    pub fn mark_success(&self, account_id: &str) {
        if self.failure_counts.remove(account_id).is_some() {
            tracing::debug!("账号 {} 请求成功，已重置失败计数", account_id);
        }
        // 清除账号级限流
        self.limits.remove(account_id);
        // 注意：我们暂时无法清除该账号下的所有模型级锁，因为我们不知道哪些模型被锁了
        // 除非遍历 limits。考虑到模型级锁通常是 QuotaExhausted，让其自然过期也是可以接受的。
        // 或者我们可以引入索引，但为了简单，暂时只清除 Account 级锁。
    }

    /// 精确锁定账号到指定时间点 (支持是否遵循 MAX_LOCKOUT_SECONDS 上限)
    ///
    /// # 参数
    /// - `model`: 可选的模型名称,用于模型级别限流。None 表示账号级别限流
    /// - `cap_to_max`: 是否将锁定时长限制在 MAX_LOCKOUT_SECONDS (300s) 以内。
    ///   如果为 false，则直接锁定到真实的 reset_time（用于零配额持续熔断）。
    pub fn set_lockout_until_with_cap(
        &self,
        account_id: &str,
        reset_time: SystemTime,
        reason: RateLimitReason,
        model: Option<String>,
        cap_to_max: bool,
    ) {
        let now = SystemTime::now();
        let (mut retry_sec, mut effective_reset_time) = reset_time
            .duration_since(now)
            .map(|duration| (duration.as_secs(), reset_time))
            .unwrap_or((60, now + Duration::from_secs(60)));

        if cap_to_max && retry_sec > MAX_LOCKOUT_SECONDS {
            tracing::info!(
                "Capping lockout time for {} from {}s to 300s (5 minutes)",
                account_id,
                retry_sec
            );
            retry_sec = MAX_LOCKOUT_SECONDS;
            effective_reset_time = now + Duration::from_secs(retry_sec);
        }

        let info = RateLimitInfo {
            reset_time: effective_reset_time,
            retry_after_sec: retry_sec,
            detected_at: now,
            reason,
            model: model.clone(), // 🆕 支持模型级别限流
        };

        let key = self.get_limit_key(account_id, model.as_deref());

        // [防倒退保护] 若已有更长、未过期的锁定时间，防止被后续较短的重置时间覆盖（如周配额 5 天不被 5H 窗口覆盖）
        if let Some(existing) = self.limits.get(&key) {
            if existing.reset_time > now && existing.reset_time > effective_reset_time {
                tracing::info!(
                    "Retaining existing longer lockout for {} (existing: {}s > new: {}s)",
                    key,
                    existing.retry_after_sec,
                    retry_sec
                );
                return;
            }
        }

        self.limits.insert(key, info);

        if let Some(m) = &model {
            tracing::info!(
                "账号 {} 的模型 {} 已精确锁定到配额刷新时间,剩余 {} 秒 (cap_to_max: {})",
                account_id,
                m,
                retry_sec,
                cap_to_max
            );
        } else {
            tracing::info!(
                "账号 {} 已精确锁定到配额刷新时间,剩余 {} 秒 (cap_to_max: {})",
                account_id,
                retry_sec,
                cap_to_max
            );
        }
    }

    /// 精确锁定账号到指定时间点 (默认限制在 300s 内)
    pub fn set_lockout_until(
        &self,
        account_id: &str,
        reset_time: SystemTime,
        reason: RateLimitReason,
        model: Option<String>,
    ) {
        self.set_lockout_until_with_cap(account_id, reset_time, reason, model, true);
    }

    pub fn restore_persisted_long_limit(
        &self,
        account_id: &str,
        reset_time: SystemTime,
        detected_at: SystemTime,
        model: &str,
    ) -> bool {
        let normalized_model = crate::proxy::common::model_mapping::normalize_to_standard_id(model)
            .unwrap_or_else(|| model.to_string());
        let now = SystemTime::now();
        let Ok(original_duration) = reset_time.duration_since(detected_at) else {
            return false;
        };
        let Ok(remaining) = reset_time.duration_since(now) else {
            return false;
        };
        if original_duration <= Duration::from_secs(MAX_LOCKOUT_SECONDS) {
            return false;
        }

        let info = RateLimitInfo {
            reset_time,
            retry_after_sec: remaining.as_secs(),
            detected_at,
            reason: RateLimitReason::QuotaExhausted,
            model: Some(normalized_model.clone()),
        };
        let key = self.get_limit_key(account_id, Some(&normalized_model));
        self.limits.insert(key, info);
        true
    }

    pub fn restore_persisted_long_image_limit(
        &self,
        account_id: &str,
        reset_time: SystemTime,
        detected_at: SystemTime,
        model: &str,
    ) -> bool {
        if normalize_image_model_id(model).is_none() {
            return false;
        }
        self.restore_persisted_long_limit(account_id, reset_time, detected_at, model)
    }
}
