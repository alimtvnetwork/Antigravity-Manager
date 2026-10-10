//! Token 预过滤：能力过滤排序、固定账号（FIX #820）优先。

use super::types::OnDiskAccountState;
use super::ProxyToken;
use super::TokenManager;
use std::sync::Arc;

impl TokenManager {
    pub(crate) fn sort_candidates_by_capability(
        mut tokens_snapshot: Vec<ProxyToken>,
        target_model: &str,
    ) -> Result<Vec<ProxyToken>, String> {
        // [NEW] 1. 动态能力过滤 (Capability Filter)

        // 定义常量
        const RESET_TIME_THRESHOLD_SECS: i64 = 600; // 10 分钟阈值

        // 归一化目标模型名为标准 ID
        let normalized_target =
            crate::proxy::common::model_mapping::normalize_to_standard_id(target_model)
                .unwrap_or_else(|| target_model.to_string());

        // 仅保留明确拥有该模型配额的账号
        // 这一步确保了 "保证有模型才可以进入轮询"，特别是对 Opus 4.6 等高端模型
        let candidate_count_before = tokens_snapshot.len();

        // 此处假设所有受支持的模型都会出现在 model_quotas 中
        // 如果 API 返回的配额信息不完整，可能会导致误杀，但为了严格性，我们执行此过滤
        tokens_snapshot.retain(|t| t.model_quotas.contains_key(&normalized_target));

        if tokens_snapshot.is_empty() {
            if candidate_count_before > 0 {
                // 如果过滤前有账号，过滤后没了，说明所有账号都没有该模型的配额
                tracing::warn!(
                    "No accounts have satisfied quota for model: {}",
                    normalized_target
                );
                return Err(format!(
                    "No accounts available with quota for model: {}",
                    normalized_target
                ));
            }
            return Err("Token pool is empty".to_string());
        }

        tokens_snapshot.sort_by(|a, b| {
            let priority_cmp = a.priority.cmp(&b.priority);
            if priority_cmp != std::cmp::Ordering::Equal {
                return priority_cmp;
            }

            // Priority 0: 严格的订阅等级排序 (ULTRA > PRO > FREE)
            // 用户要求：轮询应当遵循 Ultra -> Pro -> Free
            // 既然已经过滤掉了不支持该模型的账号，剩下的都是支持的
            // 此时我们优先使用高级订阅
            // 统一走 models::quota::tier_priority，保证与 UI / 配额解析使用同一套关键词表。
            // 未知等级一律按 FREE 处理（不再返回 3），否则会出现「UI 显示 FREE、
            // 调度器却把它排在 FREE 之后」的隐形档位。
            let tier_priority =
                |tier: &Option<String>| crate::models::quota::tier_priority(tier.as_deref());

            let tier_cmp =
                tier_priority(&a.subscription_tier).cmp(&tier_priority(&b.subscription_tier));
            if tier_cmp != std::cmp::Ordering::Equal {
                return tier_cmp;
            }

            // Priority 1: 目标模型的 quota (higher is better) -> 保护低配额账号
            // 经过过滤，key 肯定存在
            let quota_a = a.model_quotas.get(&normalized_target).copied().unwrap_or(0);
            let quota_b = b.model_quotas.get(&normalized_target).copied().unwrap_or(0);

            let quota_cmp = quota_b.cmp(&quota_a);
            if quota_cmp != std::cmp::Ordering::Equal {
                return quota_cmp;
            }

            // Priority 2: Health score (higher is better)
            let health_cmp = b
                .health_score
                .partial_cmp(&a.health_score)
                .unwrap_or(std::cmp::Ordering::Equal);
            if health_cmp != std::cmp::Ordering::Equal {
                return health_cmp;
            }

            // Priority 3: Reset time (earlier is better, but only if diff > 10 min)
            let reset_a = a.reset_time.unwrap_or(i64::MAX);
            let reset_b = b.reset_time.unwrap_or(i64::MAX);
            if (reset_a - reset_b).abs() >= RESET_TIME_THRESHOLD_SECS {
                reset_a.cmp(&reset_b)
            } else {
                std::cmp::Ordering::Equal
            }
        });

        // 【调试日志】打印排序后的账号顺序（显示目标模型的 quota）
        tracing::debug!(
            "🔄 [Token Rotation] target={} Accounts: {:?}",
            normalized_target,
            tokens_snapshot
                .iter()
                .map(|t| format!(
                    "{}(quota={}%, reset={:?}, health={:.2})",
                    t.email,
                    t.model_quotas.get(&normalized_target).copied().unwrap_or(0),
                    t.reset_time.map(|ts| {
                        let now = chrono::Utc::now().timestamp();
                        let diff_secs = ts - now;
                        if diff_secs > 0 {
                            format!("{}m", diff_secs / 60)
                        } else {
                            "now".to_string()
                        }
                    }),
                    t.health_score
                ))
                .collect::<Vec<_>>()
        );
        Ok(tokens_snapshot)
    }

    /// [FIX #820] 固定账号模式：优先使用指定账号。
    /// 返回 `Some` 表示调用方可直接返回该结果；返回 `None` 表示回退到常规轮询。
    pub(crate) async fn try_preferred_account(
        &self,
        tokens_snapshot: &mut Vec<ProxyToken>,
        total: &mut usize,
        pref_id: &str,
        target_model: &str,
        quota_protection_enabled: bool,
    ) -> Option<Result<(String, String, String, String, u64), String>> {
        // 查找优先账号
        if let Some(preferred_token) = tokens_snapshot
            .iter()
            .find(|t| &t.account_id == pref_id)
            .cloned()
        {
            // 检查账号是否可用（未限流、未被配额保护）
            match Self::get_account_state_on_disk(&preferred_token.account_path).await {
                OnDiskAccountState::Disabled => {
                    tracing::warn!(
                            "🔒 [FIX #820] Preferred account {} is disabled on disk, purging and falling back",
                            preferred_token.email
                        );
                    self.remove_account(&preferred_token.account_id);
                    tokens_snapshot.retain(|t| t.account_id != preferred_token.account_id);
                    *total = tokens_snapshot.len();

                    {
                        let mut preferred = self.preferred_account_id.write().await;
                        if preferred.as_deref() == Some(pref_id.as_str()) {
                            *preferred = None;
                        }
                    }

                    if *total == 0 {
                        return Some(Err("Token pool is empty".to_string()));
                    }
                }
                OnDiskAccountState::Unknown => {
                    tracing::warn!(
                            "🔒 [FIX #820] Preferred account {} state on disk is unavailable, falling back",
                            preferred_token.email
                        );
                    // Don't purge on transient read/parse failures; just skip this token for this request.
                    tokens_snapshot.retain(|t| t.account_id != preferred_token.account_id);
                    *total = tokens_snapshot.len();
                    if *total == 0 {
                        return Some(Err("Token pool is empty".to_string()));
                    }
                }
                OnDiskAccountState::Enabled => {
                    let normalized_target =
                        crate::proxy::common::model_mapping::normalize_to_standard_id(target_model)
                            .unwrap_or_else(|| target_model.to_string());

                    let is_rate_limited = self
                        .is_rate_limited(&preferred_token.account_id, Some(&normalized_target))
                        .await;
                    let is_quota_protected = quota_protection_enabled
                        && preferred_token
                            .protected_models
                            .contains(&normalized_target);

                    if !is_rate_limited && !is_quota_protected {
                        tracing::info!(
                            "🔒 [FIX #820] Using preferred account: {} (fixed mode)",
                            preferred_token.email
                        );

                        // 直接使用优先账号，跳过轮询逻辑
                        let mut token = preferred_token.clone();

                        // [NEW] 检查 token 是否过期（调整刷新时机对齐官方：90s 宽限期）
                        let now = chrono::Utc::now().timestamp();
                        if now >= token.timestamp - 90 {
                            // [NEW] 双重检查锁定逻辑 (Double-Checked Locking)
                            // 1. 获取（或创建）该账号专属的刷新锁
                            let refresh_mu = self
                                .refresh_locks
                                .entry(token.account_id.clone())
                                .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
                                .clone();

                            // 2. 尝试获取锁
                            let _guard = refresh_mu.lock().await;

                            // 3. 再次检查本账号最新状态（可能已被其他并发请求刷新完毕）
                            let latest_token_opt =
                                self.tokens.get(&token.account_id).map(|r| r.clone());
                            if let Some(latest) = latest_token_opt {
                                if now < latest.timestamp - 90 {
                                    // 已经被别人刷过了，同步最新数据并跳过刷新动作
                                    token = latest.clone();
                                    tracing::debug!(
                                        "账号 {} 已由并发线程刷新，跳过重复刷新",
                                        token.email
                                    );
                                } else {
                                    // 确实需要刷新
                                    tracing::debug!(
                                        "账号 {} 的 token 即将过期 ({}s)，正在刷新...",
                                        token.email,
                                        token.timestamp - now
                                    );
                                    match crate::modules::oauth::refresh_access_token(
                                        &token.refresh_token,
                                        Some(&token.account_id),
                                    )
                                    .await
                                    {
                                        Ok(token_response) => {
                                            token.access_token =
                                                token_response.access_token.clone();
                                            token.expires_in = token_response.expires_in;
                                            token.timestamp = now + token_response.expires_in;

                                            if let Some(mut entry) =
                                                self.tokens.get_mut(&token.account_id)
                                            {
                                                entry.access_token = token.access_token.clone();
                                                entry.expires_in = token.expires_in;
                                                entry.timestamp = token.timestamp;
                                            }
                                            // [FIX] 写盘操作后台化：避免阻塞 get_token 的 5s 超时窗口
                                            // 内存已更新完毕，将磁盘持久化 spawn 到 blocking 线程池
                                            {
                                                let write_path = token.account_path.clone();
                                                let access_token =
                                                    token_response.access_token.clone();
                                                let expires_in = token_response.expires_in;
                                                let id_token = token_response.id_token.clone();
                                                let new_rt = token_response.refresh_token.clone();
                                                let write_ts = now + token_response.expires_in;
                                                tokio::task::spawn_blocking(move || {
                                                    let Ok(_lk) = crate::modules::account::lock_account_file_updates() else { return; };
                                                    let Ok(raw) =
                                                        std::fs::read_to_string(&write_path)
                                                    else {
                                                        return;
                                                    };
                                                    let Ok(mut val) =
                                                        serde_json::from_str::<serde_json::Value>(
                                                            &raw,
                                                        )
                                                    else {
                                                        return;
                                                    };
                                                    val["token"]["access_token"] =
                                                        access_token.into();
                                                    val["token"]["expires_in"] = expires_in.into();
                                                    val["token"]["expiry_timestamp"] =
                                                        write_ts.into();
                                                    if let Some(it) = id_token {
                                                        val["token"]["id_token"] = it.into();
                                                    }
                                                    if let Some(rt) = new_rt {
                                                        val["token"]["refresh_token"] = rt.into();
                                                    }
                                                    if let Ok(s) =
                                                        serde_json::to_string_pretty(&val)
                                                    {
                                                        // Justification: best-effort file write; failure is logged and surfaces on the next read
                                                        crate::error::record_ignored(
                                                            std::fs::write(&write_path, s),
                                                            "fs::write",
                                                        );
                                                    }
                                                });
                                            }
                                        }
                                        Err(e) => {
                                            tracing::warn!(
                                                "Preferred account token refresh failed: {}",
                                                e
                                            );
                                            // 继续使用旧 token，让后续逻辑处理失败
                                        }
                                    }
                                }
                            }
                        }

                        // 确保有 project_id (filter empty strings to trigger re-fetch)
                        let project_id = if let Some(pid) = &token.project_id {
                            if pid.is_empty() {
                                None
                            } else {
                                Some(pid.clone())
                            }
                        } else {
                            None
                        };
                        let project_id = if let Some(pid) = project_id {
                            pid
                        } else {
                            match crate::proxy::project_resolver::fetch_project_id(
                                &token.access_token,
                            )
                            .await
                            {
                                Ok(pid) => {
                                    if let Some(mut entry) = self.tokens.get_mut(&token.account_id)
                                    {
                                        entry.project_id = Some(pid.clone());
                                    }
                                    // [FIX] 写盘后台化：project_id 已写入内存，磁盘持久化不阻塞热路径
                                    {
                                        let write_path = token.account_path.clone();
                                        let pid_clone = pid.clone();
                                        tokio::task::spawn_blocking(move || {
                                            let Ok(_lk) =
                                                crate::modules::account::lock_account_file_updates(
                                                )
                                            else {
                                                return;
                                            };
                                            let Ok(raw) = std::fs::read_to_string(&write_path)
                                            else {
                                                return;
                                            };
                                            let Ok(mut val) =
                                                serde_json::from_str::<serde_json::Value>(&raw)
                                            else {
                                                return;
                                            };
                                            val["token"]["project_id"] = pid_clone.into();
                                            if let Ok(s) = serde_json::to_string_pretty(&val) {
                                                // Justification: best-effort file write; failure is logged and surfaces on the next read
                                                crate::error::record_ignored(
                                                    std::fs::write(&write_path, s),
                                                    "fs::write",
                                                );
                                            }
                                        });
                                    }
                                    pid
                                }
                                Err(_) => "bamboo-precept-lgxtn".to_string(), // fallback
                            }
                        };

                        return Some(Ok((
                            token.access_token,
                            project_id,
                            token.email,
                            token.account_id,
                            0,
                        )));
                    } else {
                        if is_rate_limited {
                            tracing::warn!("🔒 [FIX #820] Preferred account {} is rate-limited, falling back to round-robin", preferred_token.email);
                        } else {
                            tracing::warn!("🔒 [FIX #820] Preferred account {} is quota-protected for {}, falling back to round-robin", preferred_token.email, target_model);
                        }
                    }
                }
            }
        } else {
            tracing::warn!(
                "🔒 [FIX #820] Preferred account {} not found in pool, falling back to round-robin",
                pref_id
            );
        }
        None
    }
}
