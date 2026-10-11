use super::retryparsermode::FAILURE_COUNT_EXPIRY_SECONDS;
use super::*;

impl RateLimitTracker {
    pub fn set_lockout_until_iso_with_cap(
        &self,
        account_id: &str,
        reset_time_str: &str,
        reason: RateLimitReason,
        model: Option<String>,
        cap_to_max: bool,
    ) -> bool {
        // 尝试解析 ISO 8601 格式
        match chrono::DateTime::parse_from_rfc3339(reset_time_str) {
            Ok(dt) => {
                let reset_time =
                    SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(dt.timestamp() as u64);
                self.set_lockout_until_with_cap(account_id, reset_time, reason, model, cap_to_max);
                true
            }
            Err(e) => {
                tracing::warn!(
                    "无法解析配额刷新时间 '{}': {},将使用默认退避策略",
                    reset_time_str,
                    e
                );
                false
            }
        }
    }

    /// 使用 ISO 8601 时间字符串精确锁定账号
    ///
    /// 解析类似 "2026-01-08T17:00:00Z" 格式的时间字符串
    ///
    /// # 参数
    /// - `model`: 可选的模型名称,用于模型级别限流
    pub fn set_lockout_until_iso(
        &self,
        account_id: &str,
        reset_time_str: &str,
        reason: RateLimitReason,
        model: Option<String>,
    ) -> bool {
        self.set_lockout_until_iso_with_cap(account_id, reset_time_str, reason, model, true)
    }

    /// 从错误响应解析限流信息
    ///
    /// # Arguments
    /// * `account_id` - 账号 ID
    /// * `status` - HTTP 状态码
    /// * `retry_after_header` - Retry-After header 值
    /// * `body` - 错误响应 body
    pub fn parse_from_error(
        &self,
        account_id: &str,
        status: u16,
        retry_after_header: Option<&str>,
        body: &str,
        model: Option<String>,
        backoff_steps: &[u64], // [NEW] 传入退避配置
    ) -> Option<RateLimitInfo> {
        self.parse_from_error_with_mode(
            account_id,
            status,
            retry_after_header,
            body,
            model,
            backoff_steps,
            RetryParserMode::Current,
        )
    }

    pub fn parse_from_error_baseline(
        &self,
        account_id: &str,
        status: u16,
        retry_after_header: Option<&str>,
        body: &str,
        model: Option<String>,
        backoff_steps: &[u64],
    ) -> Option<RateLimitInfo> {
        self.parse_from_error_with_mode(
            account_id,
            status,
            retry_after_header,
            body,
            model,
            backoff_steps,
            RetryParserMode::Baseline,
        )
    }

    fn parse_from_error_with_mode(
        &self,
        account_id: &str,
        status: u16,
        retry_after_header: Option<&str>,
        body: &str,
        model: Option<String>,
        backoff_steps: &[u64],
        parser_mode: RetryParserMode,
    ) -> Option<RateLimitInfo> {
        // 关键防御：网关内部自产生的排队/无可用账号错误，绝对禁止解析为上游限流，杜绝自噬死循环
        let lower_body = body.to_lowercase();
        if lower_body.contains("all accounts limited")
            || lower_body.contains("no accounts available")
            || lower_body.contains("all accounts failed")
            || lower_body.contains("token pool is empty")
            || lower_body.contains("all accounts exhausted")
            || lower_body.contains("all accounts unhealthy")
        {
            return None;
        }

        // 仅对真正的上游限流 429 和过载 529 进行账号冷却跟踪；500/503 属于服务瞬时不可用，绝不打入冷却池！
        if status != 429 && status != 529 {
            return None;
        }

        // 1. 解析限流原因类型
        let reason = if status == 429 {
            tracing::warn!("Google 429 Error Body: {}", body);
            self.parse_rate_limit_reason(body)
        } else {
            RateLimitReason::ServerError
        };

        let retry_after_sec = match parser_mode {
            RetryParserMode::Current => {
                crate::proxy::upstream::retry::parse_retry_delay(body, retry_after_header)
                    .map(|delay_ms| delay_ms.saturating_add(999) / 1000)
            }
            RetryParserMode::Baseline => retry_after_header
                .and_then(|value| value.parse::<u64>().ok())
                .or_else(|| self.parse_retry_time_from_body_baseline(body)),
        };
        let has_explicit_retry_time = retry_after_sec.is_some();
        let preserve_long_image_quota = parser_mode == RetryParserMode::Current
            && status == 429
            && reason == RateLimitReason::QuotaExhausted
            && has_explicit_quota_exhausted(body)
            && has_explicit_retry_time
            && model
                .as_deref()
                .and_then(normalize_image_model_id)
                .is_some();

        // 4. 处理默认值与软避让逻辑（根据限流类型设置不同默认值）
        let retry_sec = match retry_after_sec {
            Some(s) => {
                // 设置安全缓冲区：最小 2 秒，防止极高频无效重试
                if s < 2 {
                    2
                } else {
                    s
                }
            }
            None => {
                // 获取连续失败次数，用于指数退避（带自动过期逻辑）
                // [FIX] ServerError (5xx) 不累加 failure_count，避免污染 429 的退避阶梯
                let failure_count = if reason != RateLimitReason::ServerError {
                    // 只有非 ServerError 才累加失败计数（用于指数退避）
                    let now = SystemTime::now();
                    // 这里我们使用 account_id 作为 key，不区分模型，
                    // 因为这里是为了计算连续"账号级"问题的退避。
                    // 如果需要针对模型的连续失败计数，可能需要改变 failure_counts 的 key。
                    // 暂时保持 account_id，这样如果一个模型一直挂，也会增加计数，符合逻辑。
                    let mut entry = self
                        .failure_counts
                        .entry(account_id.to_string())
                        .or_insert((0, now));

                    let elapsed = now
                        .duration_since(entry.1)
                        .unwrap_or(Duration::from_secs(0))
                        .as_secs();
                    if elapsed > FAILURE_COUNT_EXPIRY_SECONDS {
                        tracing::debug!(
                            "账号 {} 失败计数已过期（{}秒），重置为 0",
                            account_id,
                            elapsed
                        );
                        *entry = (0, now);
                    }
                    entry.0 += 1;
                    entry.1 = now;
                    entry.0
                } else {
                    // ServerError (5xx) 使用固定值 1，不累加，避免污染 429 的退避阶梯
                    1
                };

                match reason {
                    RateLimitReason::QuotaExhausted => {
                        // [智能限流] 根据 failure_count 和配置的 backoff_steps 计算
                        let index = (failure_count as usize).saturating_sub(1);
                        let lockout = if index < backoff_steps.len() {
                            backoff_steps[index]
                        } else {
                            *backoff_steps.last().unwrap_or(&7200)
                        };

                        tracing::warn!(
                            "检测到配额耗尽 (QUOTA_EXHAUSTED)，第{}次连续失败，根据配置锁定 {} 秒",
                            failure_count,
                            lockout
                        );
                        lockout
                    }
                    RateLimitReason::RateLimitExceeded => {
                        // 速率限制 (TPM/RPM)
                        let body_lower = body.to_lowercase();
                        let lockout = if body_lower.contains("resource has been exhausted")
                            || body_lower.contains("resource_exhausted")
                        {
                            30
                        } else {
                            5
                        };
                        tracing::debug!(
                            "检测到速率限制 (RATE_LIMIT_EXCEEDED)，使用默认值 {}秒",
                            lockout
                        );
                        lockout
                    }
                    RateLimitReason::ModelCapacityExhausted => {
                        // 模型容量耗尽
                        let lockout = match failure_count {
                            1 => 5,
                            2 => 10,
                            _ => 15,
                        };
                        tracing::warn!(
                            "检测到模型容量不足 (MODEL_CAPACITY_EXHAUSTED)，第{}次失败，{}秒后重试",
                            failure_count,
                            lockout
                        );
                        lockout
                    }
                    RateLimitReason::ServerError => {
                        let lockout = 8;
                        tracing::warn!("检测到 {} 错误, 执行 {}s 软避让...", status, lockout);
                        lockout
                    }
                    RateLimitReason::Unknown => {
                        // 未知原因
                        tracing::debug!("无法解析 429 限流原因, 使用默认值 60秒");
                        60
                    }
                }
            }
        };

        let mut retry_sec = retry_sec;
        let max_allowed_lockout = backoff_steps
            .iter()
            .copied()
            .max()
            .unwrap_or(MAX_LOCKOUT_SECONDS)
            .max(MAX_LOCKOUT_SECONDS);
        if retry_sec > max_allowed_lockout && !preserve_long_image_quota {
            tracing::info!(
                "Capping retry lockout time for {} from {}s to {}s (max backoff limit)",
                account_id,
                retry_sec,
                max_allowed_lockout
            );
            retry_sec = max_allowed_lockout;
        }

        let info = RateLimitInfo {
            reset_time: SystemTime::now() + Duration::from_secs(retry_sec),
            retry_after_sec: retry_sec,
            detected_at: SystemTime::now(),
            reason,
            model: model.clone(),
        };

        // [FIX] 细粒度模型隔离：只要调用方传入了具体的 model，限流必须针对该 model 进行隔离！
        // 杜绝因某单个模型（或不存在的模型/特定模型限流）而将全账号的所有模型连坐封锁，导致正常账号宕机。
        let key = if let Some(m) = model.as_deref().filter(|s| !s.is_empty()) {
            self.get_limit_key(account_id, Some(m))
        } else {
            account_id.to_string()
        };

        self.limits.insert(key, info.clone());

        tracing::warn!(
            "账号 {} [{}] 限流类型: {:?}, 重置延时: {}秒",
            account_id,
            status,
            reason,
            retry_sec
        );

        Some(info)
    }

    /// 解析限流原因类型
    pub(crate) fn parse_rate_limit_reason(&self, body: &str) -> RateLimitReason {
        // 尝试从 JSON 中提取 reason 字段
        let trimmed = body.trim();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
                if let Some(reason_str) = json
                    .get("error")
                    .and_then(|e| e.get("details"))
                    .and_then(|d| d.as_array())
                    .and_then(|a| a.get(0))
                    .and_then(|o| o.get("reason"))
                    .and_then(|v| v.as_str())
                {
                    return match reason_str {
                        "QUOTA_EXHAUSTED" => RateLimitReason::QuotaExhausted,
                        "RATE_LIMIT_EXCEEDED" => RateLimitReason::RateLimitExceeded,
                        "MODEL_CAPACITY_EXHAUSTED" => RateLimitReason::ModelCapacityExhausted,
                        _ => RateLimitReason::Unknown,
                    };
                }
                // [NEW] 尝试从 message 字段进行文本匹配（防止 missed reason）
                if let Some(msg) = json
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|v| v.as_str())
                {
                    let msg_lower = msg.to_lowercase();
                    if msg_lower.contains("per minute") || msg_lower.contains("rate limit") {
                        return RateLimitReason::RateLimitExceeded;
                    }
                }
            }
        }

        // 如果无法从 JSON 解析，尝试从消息文本判断
        let body_lower = body.to_lowercase();
        // [FIX] 优先判断分钟级限制，避免将 TPM 误判为 Quota
        let generic_resource_exhausted = body_lower.contains("resource has been exhausted")
            || body_lower.contains("resource_exhausted");
        let explicit_quota_exhausted = body_lower.contains("quotaresetdelay")
            || body_lower.contains("quotareset")
            || body_lower.contains("quota reset")
            || body_lower.contains("quota limit")
            || body_lower.contains("per day")
            || body_lower.contains("daily quota")
            || (body_lower.contains("quota_exhausted")
                && crate::proxy::upstream::retry::parse_retry_delay(body, None).is_some());

        if body_lower.contains("per minute")
            || body_lower.contains("rate limit")
            || body_lower.contains("too many requests")
            || (generic_resource_exhausted && !explicit_quota_exhausted)
        {
            RateLimitReason::RateLimitExceeded
        } else if explicit_quota_exhausted {
            RateLimitReason::QuotaExhausted
        } else if body_lower.contains("exhausted") || body_lower.contains("quota") {
            RateLimitReason::RateLimitExceeded
        } else {
            RateLimitReason::Unknown
        }
    }

    /// 从错误消息 body 中解析重置时间
    pub(crate) fn parse_retry_time_from_body(&self, body: &str) -> Option<u64> {
        crate::proxy::upstream::retry::parse_retry_delay(body, None)
            .map(|delay_ms| delay_ms.saturating_add(999) / 1000)
    }

    fn parse_duration_string_baseline(&self, value: &str) -> Option<u64> {
        let re = Regex::new(r"(?:(\d+)h)?(?:(\d+)m)?(?:(\d+(?:\.\d+)?)s)?(?:(\d+(?:\.\d+)?)ms)?")
            .ok()?;
        let captures = re.captures(value)?;
        let hours = captures
            .get(1)
            .and_then(|value| value.as_str().parse::<u64>().ok())
            .unwrap_or(0);
        let minutes = captures
            .get(2)
            .and_then(|value| value.as_str().parse::<u64>().ok())
            .unwrap_or(0);
        let seconds = captures
            .get(3)
            .and_then(|value| value.as_str().parse::<f64>().ok())
            .unwrap_or(0.0);
        let milliseconds = captures
            .get(4)
            .and_then(|value| value.as_str().parse::<f64>().ok())
            .unwrap_or(0.0);
        let total_seconds = hours * 3600
            + minutes * 60
            + seconds.ceil() as u64
            + (milliseconds / 1000.0).ceil() as u64;
        (total_seconds > 0).then_some(total_seconds)
    }

    fn parse_retry_time_from_body_baseline(&self, body: &str) -> Option<u64> {
        let trimmed = body.trim();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
                if let Some(delay) = json
                    .get("error")
                    .and_then(|error| error.get("details"))
                    .and_then(|details| details.as_array())
                    .and_then(|details| details.first())
                    .and_then(|detail| detail.get("metadata"))
                    .and_then(|metadata| metadata.get("quotaResetDelay"))
                    .and_then(|value| value.as_str())
                    .and_then(|value| self.parse_duration_string_baseline(value))
                {
                    return Some(delay);
                }
                if let Some(retry) = json
                    .get("error")
                    .and_then(|error| error.get("retry_after"))
                    .and_then(|value| value.as_u64())
                {
                    return Some(retry);
                }
            }
        }

        for pattern in [
            r"(?i)try again in (\d+)m\s*(\d+)s",
            r"(?i)(?:try again in|backoff for|wait)\s*(\d+)s",
            r"(?i)quota will reset in (\d+) second",
            r"(?i)retry after (\d+) second",
            r"\(wait (\d+)s\)",
        ] {
            let captures = Regex::new(pattern).ok()?.captures(body);
            let Some(captures) = captures else {
                continue;
            };
            if captures.len() == 3 {
                let minutes = captures.get(1)?.as_str().parse::<u64>().ok()?;
                let seconds = captures.get(2)?.as_str().parse::<u64>().ok()?;
                return Some(minutes * 60 + seconds);
            }
            if let Some(seconds) = captures
                .get(1)
                .and_then(|value| value.as_str().parse::<u64>().ok())
            {
                return Some(seconds);
            }
        }
        None
    }

    /// 获取账号的限流信息
    pub fn get(&self, account_id: &str) -> Option<RateLimitInfo> {
        self.limits.get(account_id).map(|r| r.clone())
    }
}
