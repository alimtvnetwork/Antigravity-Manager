//! 健康度：成功/失败记录、熔断器同步。

use super::TokenManager;

impl TokenManager {
    /// 标记账号请求成功，重置连续失败计数
    ///
    /// 在请求成功完成后调用，将该账号的失败计数归零，
    /// 下次失败时从最短的锁定时间开始（智能限流）。
    pub fn mark_account_success(&self, account_id: &str) {
        self.rate_limit_tracker.mark_success(account_id);
    }

    /// 记录请求成功，增加健康分
    pub fn record_success(&self, account_id: &str) {
        self.health_scores
            .entry(account_id.to_string())
            .and_modify(|s| *s = (*s + 0.05).min(1.0))
            .or_insert(1.0);
        tracing::debug!("📈 Health score increased for account {}", account_id);
    }

    /// 记录请求失败，降低健康分
    pub fn record_failure(&self, account_id: &str) {
        self.health_scores
            .entry(account_id.to_string())
            .and_modify(|s| *s = (*s - 0.2).max(0.0))
            .or_insert(0.8);
        tracing::warn!("📉 Health score decreased for account {}", account_id);
    }

    /// [NEW] 从账号配额信息中提取最近的刷新时间戳
    ///
    /// Claude 模型（sonnet/opus）共用同一个刷新时间，只需取 claude 系列的 reset_time
    /// 返回 Unix 时间戳（秒），用于排序时比较
    pub(crate) fn extract_earliest_reset_time(&self, account: &serde_json::Value) -> Option<i64> {
        let models = account
            .get("quota")
            .and_then(|q| q.get("models"))
            .and_then(|m| m.as_array())?;

        let mut earliest_ts: Option<i64> = None;

        for model in models {
            // 优先取 claude 系列的 reset_time（sonnet/opus 共用）
            let model_name = model.get("name").and_then(|n| n.as_str()).unwrap_or("");
            if !model_name.contains("claude") {
                continue;
            }

            if let Some(reset_time_str) = model.get("reset_time").and_then(|r| r.as_str()) {
                if reset_time_str.is_empty() {
                    continue;
                }
                // 解析 ISO 8601 时间字符串为时间戳
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(reset_time_str) {
                    let ts = dt.timestamp();
                    if earliest_ts.is_none() || ts < earliest_ts.unwrap() {
                        earliest_ts = Some(ts);
                    }
                }
            }
        }

        // 如果没有 claude 模型的时间，尝试取任意模型的最近时间
        if earliest_ts.is_none() {
            for model in models {
                if let Some(reset_time_str) = model.get("reset_time").and_then(|r| r.as_str()) {
                    if reset_time_str.is_empty() {
                        continue;
                    }
                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(reset_time_str) {
                        let ts = dt.timestamp();
                        if earliest_ts.is_none() || ts < earliest_ts.unwrap() {
                            earliest_ts = Some(ts);
                        }
                    }
                }
            }
        }

        earliest_ts
    }

    /// Restore official quota windows without replacing independent upstream limits.
    pub(crate) fn sync_zero_quota_circuit_breaker(
        &self,
        account_id: &str,
        account: &serde_json::Value,
    ) {
        let lock_on_zero = if let Ok(cfg) = self.circuit_breaker_config.try_read() {
            cfg.enabled && cfg.lock_on_zero_quota
        } else {
            false
        };

        let quota = match account.get("quota") {
            Some(q) => q,
            None => return,
        };

        let observed_at = quota
            .get("last_updated")
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
            .saturating_mul(1000);
        if let Some(groups) = quota.get("quota_groups").and_then(|g| g.as_array()) {
            for group in groups {
                let group_name = group
                    .get("display_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let is_claude_group = group_name.to_lowercase().contains("claude")
                    || group_name.to_lowercase().contains("gpt");
                let is_gemini_group = group_name.to_lowercase().contains("gemini");

                if let Some(buckets) = group.get("buckets").and_then(|b| b.as_array()) {
                    for bucket in buckets {
                        let bucket_id = bucket
                            .get("bucket_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let window = bucket.get("window").and_then(|v| v.as_str()).unwrap_or("");
                        let window_key = format!("{} {}", bucket_id, window).to_lowercase();
                        let weekly = window_key.contains("week") || window_key.contains("7d");
                        if !weekly
                            && (!lock_on_zero
                                || !(window_key.contains("5h") || window_key.contains("hour")))
                        {
                            continue;
                        }
                        let Some(fraction) =
                            bucket.get("remaining_fraction").and_then(|v| v.as_f64())
                        else {
                            continue;
                        };
                        let exhausted_until = if fraction <= 0.001 {
                            let Some(reset) = bucket
                                .get("reset_time")
                                .and_then(|v| v.as_str())
                                .and_then(|v| chrono::DateTime::parse_from_rfc3339(v).ok())
                            else {
                                continue;
                            };
                            Some(std::time::SystemTime::from(reset))
                        } else {
                            None
                        };
                        let third_party = is_claude_group || bucket_id.contains("3p");
                        let gemini = is_gemini_group || bucket_id.contains("gemini");
                        let mut models: Vec<&str> = if third_party {
                            vec!["claude", "claude-sonnet-4-6", "gpt-oss-120b-medium"]
                        } else if gemini {
                            vec![
                                "gemini-3-flash",
                                "gemini-3.1-pro-high",
                                "gemini-3.1-flash-image",
                                "gemini-3-pro-image",
                            ]
                        } else {
                            Vec::new()
                        };
                        if let Some(available) = quota.get("models").and_then(|v| v.as_array()) {
                            models.extend(
                                available
                                    .iter()
                                    .filter_map(|m| m.get("name")?.as_str())
                                    .filter(|name| {
                                        if third_party {
                                            name.starts_with("claude") || name.starts_with("gpt")
                                        } else {
                                            gemini && name.starts_with("gemini")
                                        }
                                    }),
                            );
                        }
                        for model in models {
                            let normalized =
                                crate::proxy::common::model_mapping::normalize_to_standard_id(
                                    model,
                                )
                                .unwrap_or_else(|| model.to_string());
                            self.rate_limit_tracker.sync_quota_bucket(
                                account_id,
                                &normalized,
                                bucket_id,
                                bucket
                                    .get("observed_at")
                                    .and_then(|v| v.as_i64())
                                    .unwrap_or(observed_at),
                                exhausted_until,
                                weekly,
                            );
                        }
                    }
                }
            }
            if !groups.is_empty() {
                return;
            }
        }

        // 2. 回退到 models 配额检查
        if !lock_on_zero {
            return;
        }
        if let Some(models) = quota.get("models").and_then(|m| m.as_array()) {
            // 只要受监控核心模型或全部模型为 0%，且有有效 reset_time
            let all_zero = models
                .iter()
                .all(|m| m.get("percentage").and_then(|p| p.as_i64()).unwrap_or(100) == 0);

            if all_zero && !models.is_empty() {
                if let Some(reset_time_str) = self.get_quota_reset_time(account_id) {
                    if !chrono::DateTime::parse_from_rfc3339(&reset_time_str)
                        .is_ok_and(|reset| reset > chrono::Utc::now())
                    {
                        return;
                    }
                    tracing::warn!(
                        "[CircuitBreaker] 账号 {} 的模型配额已全部为 0%, 持续锁定至 {}",
                        account_id,
                        reset_time_str
                    );
                    self.rate_limit_tracker.set_lockout_until_iso_with_cap(
                        account_id,
                        &reset_time_str,
                        crate::proxy::rate_limit::RateLimitReason::QuotaExhausted,
                        None,
                        false,
                    );
                }
            }
        }
    }
}
