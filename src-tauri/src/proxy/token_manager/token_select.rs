//! Token 选择：P2C 算法、粘性会话调度、乐观重置。

use super::ProxyToken;
use super::TokenManager;
use crate::proxy::sticky_config::SchedulingMode;
use std::collections::HashSet;

impl TokenManager {
    /// Power of 2 Choices (P2C) 选择算法
    /// 从前 5 个候选中随机选 2 个，选择配额更高的 -> 避免热点
    /// 返回选中的索引
    ///
    /// # 参数
    /// * `candidates` - 已排序的候选 token 列表
    /// * `attempted` - 已尝试失败的账号 ID 集合
    /// * `normalized_target` - 归一化后的目标模型名
    /// * `quota_protection_enabled` - 是否启用配额保护
    pub(crate) fn select_with_p2c<'a>(
        &self,
        candidates: &'a [ProxyToken],
        attempted: &HashSet<String>,
        normalized_target: &str,
        quota_protection_enabled: bool,
    ) -> Option<&'a ProxyToken> {
        use rand::Rng;

        // 过滤可用 token
        let mut available: Vec<&ProxyToken> = candidates
            .iter()
            .filter(|t| !attempted.contains(&t.account_id))
            .filter(|t| {
                !quota_protection_enabled || !t.protected_models.contains(normalized_target)
            })
            .collect();

        // Keep lower-priority groups for retries; only this draw is restricted.
        let priority = available.iter().map(|t| t.priority).min()?;
        available.retain(|t| t.priority == priority);
        if available.len() == 1 {
            return Some(available[0]);
        }

        // P2C: 从前 min(P2C_POOL_SIZE, len) 个中随机选 2 个
        let pool_size = available.len().min(Self::P2C_POOL_SIZE);
        let mut rng = rand::thread_rng();

        let pick1 = rng.gen_range(0..pool_size);
        let pick2 = rng.gen_range(0..pool_size);
        // 确保选择不同的两个候选
        let pick2 = if pick2 == pick1 {
            (pick1 + 1) % pool_size
        } else {
            pick2
        };

        let c1 = available[pick1];
        let c2 = available[pick2];

        // 选择配额更高的
        let selected = if c1.remaining_quota.unwrap_or(0) >= c2.remaining_quota.unwrap_or(0) {
            c1
        } else {
            c2
        };

        tracing::debug!(
            "🎲 [P2C] Selected {} ({}%) from [{}({}%), {}({}%)]",
            selected.email,
            selected.remaining_quota.unwrap_or(0),
            c1.email,
            c1.remaining_quota.unwrap_or(0),
            c2.email,
            c2.remaining_quota.unwrap_or(0)
        );

        Some(selected)
    }

    /// 粘性会话与智能调度逻辑：从候选账号中选择目标 Token，并固化粘性会话绑定。
    pub(crate) async fn select_target_token(
        &self,
        tokens_snapshot: &[ProxyToken],
        attempted: &HashSet<String>,
        session_id: Option<&str>,
        scheduling_mode: SchedulingMode,
        quota_group: &str,
        quota_protection_enabled: bool,
        target_model: &str,
        rotate: bool,
        total: usize,
        last_used_account_id: &Option<(String, std::time::Instant)>,
        need_update_last_used: &mut Option<(String, std::time::Instant)>,
    ) -> Option<ProxyToken> {
        // ===== 【核心】粘性会话与智能调度逻辑 =====
        let mut target_token: Option<ProxyToken> = None;

        // 归一化目标模型名为标准 ID，用于配额保护检查
        let normalized_target =
            crate::proxy::common::model_mapping::normalize_to_standard_id(target_model)
                .unwrap_or_else(|| target_model.to_string());

        // 模式 A: 粘性会话处理 (CacheFirst 或 Balance 且有 session_id)
        if !rotate && session_id.is_some() && scheduling_mode != SchedulingMode::PerformanceFirst {
            let sid = session_id.unwrap();

            // 1. 检查会话是否已绑定账号
            if let Some(bound_id) = self.session_accounts.get(sid).map(|v| v.clone()) {
                // 【修复】先通过 account_id 找到对应的账号，获取其 email
                // 2. 转换 email -> account_id 检查绑定的账号是否限流
                if let Some(bound_token) = tokens_snapshot.iter().find(|t| t.account_id == bound_id)
                {
                    let key = self
                        .email_to_account_id(&bound_token.email)
                        .unwrap_or_else(|| bound_token.account_id.clone());
                    // [FIX] 传入目标模型标准化 ID，检查该模型是否已被熔断器精准锁定
                    let reset_sec = self
                        .rate_limit_tracker
                        .get_remaining_wait(&key, Some(&normalized_target));
                    if reset_sec > 0 {
                        // 【修复 Issue #284】立即解绑并切换账号，不再阻塞等待
                        // 原因：阻塞等待会导致并发请求时客户端 socket 超时 (UND_ERR_SOCKET)
                        tracing::debug!(
                                "Sticky Session: Bound account {} is rate-limited for {} ({}s), unbinding and switching.",
                                bound_token.email, normalized_target, reset_sec
                            );
                        self.session_accounts.remove(sid);
                    } else if !attempted.contains(&bound_id)
                        && !(quota_protection_enabled
                            && bound_token.protected_models.contains(&normalized_target))
                    {
                        // 3. 账号可用且未被标记为尝试失败，优先复用
                        tracing::info!(
                            "Sticky Session: Successfully reusing bound account {} for session {}",
                            bound_token.email,
                            sid
                        );
                        target_token = Some(bound_token.clone());
                        *need_update_last_used =
                            Some((bound_token.account_id.clone(), std::time::Instant::now()));
                    } else if quota_protection_enabled
                        && bound_token.protected_models.contains(&normalized_target)
                    {
                        tracing::debug!("Sticky Session: Bound account {} is quota-protected for model {} [{}], unbinding and switching.", bound_token.email, normalized_target, target_model);
                        self.session_accounts.remove(sid);
                    } else if attempted.contains(&bound_id) {
                        // [FIX] 绑定的账号在当前轮次请求中已尝试失败，立即解绑避免死锁
                        tracing::debug!("Sticky Session: Bound account {} already attempted in current request, unbinding", bound_token.email);
                        self.session_accounts.remove(sid);
                    }
                } else {
                    // 绑定的账号已不存在（可能被删除），解绑
                    tracing::debug!(
                        "Sticky Session: Bound account not found for session {}, unbinding",
                        sid
                    );
                    self.session_accounts.remove(sid);
                }
            }
        }

        // 模式 B: 原子化 60s 全局锁定 (针对无 session_id 情况的默认保护)
        // 【修复】性能优先模式应跳过 60s 锁定；
        if target_token.is_none()
            && !rotate
            && quota_group != "image_gen"
            && scheduling_mode != SchedulingMode::PerformanceFirst
        {
            // 仅针对无 session_id 的无状态请求，使用 60s 全局锁定保底避免轮换
            if session_id.is_none() {
                if let Some((account_id, last_time)) = &last_used_account_id {
                    // [FIX #3] 60s 锁定逻辑应检查 `attempted` 集合，避免重复尝试失败的账号
                    if last_time.elapsed().as_secs() < 60 && !attempted.contains(account_id) {
                        if let Some(found) =
                            tokens_snapshot.iter().find(|t| &t.account_id == account_id)
                        {
                            // 【修复】检查限流状态和配额保护，避免复用已被锁定的账号
                            if !self
                                .is_rate_limited(&found.account_id, Some(&normalized_target))
                                .await
                                && !(quota_protection_enabled
                                    && found.protected_models.contains(&normalized_target))
                            {
                                tracing::debug!(
                                    "60s Window: Force reusing last account: {}",
                                    found.email
                                );
                                target_token = Some(found.clone());
                                *need_update_last_used =
                                    Some((found.account_id.clone(), std::time::Instant::now()));
                            } else {
                                if self
                                    .is_rate_limited(&found.account_id, Some(&normalized_target))
                                    .await
                                {
                                    tracing::debug!(
                                        "60s Window: Last account {} is rate-limited, skipping",
                                        found.email
                                    );
                                } else {
                                    tracing::debug!("60s Window: Last account {} is quota-protected for model {} [{}], skipping", found.email, normalized_target, target_model);
                                }
                            }
                        }
                    }
                }
            }

            // 若无锁定或带有 session_id（会话首次分配），使用 P2C 均衡选择账号
            if target_token.is_none() {
                // 先过滤出未限流的账号
                let mut non_limited: Vec<ProxyToken> = Vec::new();
                for t in &tokens_snapshot {
                    if !self
                        .is_rate_limited(&t.account_id, Some(&normalized_target))
                        .await
                    {
                        non_limited.push(t.clone());
                    }
                }

                if let Some(selected) = self.select_with_p2c(
                    &non_limited,
                    &attempted,
                    &normalized_target,
                    quota_protection_enabled,
                ) {
                    target_token = Some(selected.clone());
                    *need_update_last_used =
                        Some((selected.account_id.clone(), std::time::Instant::now()));
                }
            }
        } else if target_token.is_none() {
            // 模式 C: P2C 选择 (替代纯轮询)
            tracing::debug!("🔄 [Mode C] P2C selection from {} candidates", total);

            // 先过滤出未限流的账号
            let mut non_limited: Vec<ProxyToken> = Vec::new();
            for t in &tokens_snapshot {
                if !self
                    .is_rate_limited(&t.account_id, Some(&normalized_target))
                    .await
                {
                    non_limited.push(t.clone());
                }
            }

            if let Some(selected) = self.select_with_p2c(
                &non_limited,
                &attempted,
                &normalized_target,
                quota_protection_enabled,
            ) {
                tracing::debug!("  {} - SELECTED via P2C", selected.email);
                target_token = Some(selected.clone());

                if rotate {
                    tracing::debug!("Force Rotation: Switched to account: {}", selected.email);
                }
            }
        }

        // 【核心固化】凡解析出可用账号且当前为粘性会话调度，确保立即固化绑定，防止轮换或会话漂移
        if let Some(ref selected) = target_token {
            if let Some(sid) = session_id {
                if scheduling_mode != SchedulingMode::PerformanceFirst && !rotate {
                    self.session_accounts
                        .insert(sid.to_string(), selected.account_id.clone());
                    tracing::info!(
                        "Sticky Session: Ensured binding account {} to session {}",
                        selected.email,
                        sid
                    );
                }
            }
        }
        target_token
    }

    /// 乐观重置策略（双层防护）：从候选 Token 解析出最终可用的 Token。
    pub(crate) async fn resolve_token_with_optimistic_reset(
        &self,
        target_token: Option<ProxyToken>,
        tokens_snapshot: &[ProxyToken],
        attempted: &HashSet<String>,
        normalized_target: &str,
        quota_protection_enabled: bool,
    ) -> Result<ProxyToken, String> {
        match target_token {
            Some(t) => Ok(t),
            None => {
                // 乐观重置策略: 双层防护机制
                // 计算最短等待时间
                let min_wait = tokens_snapshot
                    .iter()
                    .filter_map(|t| {
                        let wait = self
                            .rate_limit_tracker
                            .get_remaining_wait(&t.account_id, Some(&normalized_target));
                        if wait > 0 {
                            Some(wait)
                        } else {
                            None
                        }
                    })
                    .min();

                // Layer 1: 如果最短等待时间 <= 2秒,执行缓冲延迟
                if let Some(wait_sec) = min_wait {
                    if wait_sec <= 2 {
                        let wait_ms = (wait_sec as f64 * 1000.0) as u64;
                        tracing::warn!(
                                "All accounts rate-limited but shortest wait is {}s. Applying {}ms buffer for state sync...",
                                wait_sec, wait_ms
                            );

                        // 缓冲延迟
                        tokio::time::sleep(tokio::time::Duration::from_millis(wait_ms)).await;

                        // 重新尝试选择账号
                        let mut retry_token = None;
                        for token in &tokens_snapshot {
                            if attempted.contains(&token.account_id)
                                || self
                                    .is_rate_limited(&token.account_id, Some(&normalized_target))
                                    .await
                                || (quota_protection_enabled
                                    && token.protected_models.contains(&normalized_target))
                            {
                                continue;
                            }
                            retry_token = Some(token);
                            break;
                        }

                        if let Some(t) = retry_token {
                            tracing::info!(
                                "✅ Buffer delay successful! Found available account: {}",
                                t.email
                            );
                            t.clone()
                        } else {
                            // Layer 2: 缓冲后仍无可用账号,执行乐观重置
                            tracing::warn!(
                                    "Buffer delay failed. Executing optimistic reset for all {} accounts...",
                                    tokens_snapshot.len()
                                );

                            // 清除所有限流记录
                            self.rate_limit_tracker.clear_for_optimistic_reset();

                            // 再次尝试选择账号 (必须重新校验剩余限流状态，严禁放行周配额耗尽等长锁定账号)
                            let final_token = tokens_snapshot.iter().find(|t| {
                                !attempted.contains(&t.account_id)
                                    && !self
                                        .rate_limit_tracker
                                        .is_rate_limited(&t.account_id, Some(&normalized_target))
                                    && !(quota_protection_enabled
                                        && t.protected_models.contains(&normalized_target))
                                    && !self
                                        .rate_limit_tracker
                                        .is_rate_limited(&t.account_id, Some(&normalized_target))
                            });

                            if let Some(t) = final_token {
                                tracing::info!(
                                    "✅ Optimistic reset successful! Using account: {}",
                                    t.email
                                );
                                t.clone()
                            } else {
                                return Err(
                                    "All accounts failed after optimistic reset.".to_string()
                                );
                            }
                        }
                    } else {
                        return Err(format!("All accounts limited. Wait {}s.", wait_sec));
                    }
                } else {
                    return Err("All accounts failed or unhealthy.".to_string());
                }
            }
        }
    }
}
