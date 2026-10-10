//! 配额读取与统计：check_and_protect_quota 及配额 JSON 查询。

use super::helpers::update_account_json;
use super::TokenManager;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

impl TokenManager {
    /// 检查账号是否应该被配额保护
    /// 如果配额低于阈值，自动禁用账号并返回 true
    pub(crate) async fn check_and_protect_quota(
        &self,
        account_json: &mut serde_json::Value,
        account_path: &PathBuf,
    ) -> bool {
        // 1. 加载配额保护配置
        let config = match crate::modules::config::load_app_config() {
            Ok(cfg) => cfg.quota_protection,
            Err(_) => return false, // 配置加载失败，跳过保护
        };

        if !config.enabled {
            // [FIX] 当配额保护在全局关闭时，清空受保护模型列表，避免遗留锁定显示与调度过滤
            if let Some(arr) = account_json
                .get_mut("protected_models")
                .and_then(|v| v.as_array_mut())
            {
                if !arr.is_empty() {
                    arr.clear();
                    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                    crate::error::record_ignored(
                        update_account_json(account_path, |latest| {
                            latest["protected_models"] = serde_json::Value::Array(Vec::new());
                        })
                        .await,
                        "update_account_json",
                    );
                }
            }
            return false; // 配额保护未启用
        }

        // 2. 获取配额信息
        // 注意：我们需要 clone 配额信息来遍历，避免借用冲突，但修改是针对 account_json 的
        let quota = match account_json.get("quota") {
            Some(q) => q.clone(),
            None => return false, // 无配额信息，跳过
        };

        // 3. [兼容性 #621] 检查是否被旧版账号级配额保护禁用,尝试恢复并转为模型级
        let is_proxy_disabled = account_json
            .get("proxy_disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let reason = account_json
            .get("proxy_disabled_reason")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if is_proxy_disabled && reason == "quota_protection" {
            // 如果是被旧版账号级保护禁用的,尝试恢复并转为模型级
            return self
                .check_and_restore_quota(account_json, account_path, &quota, &config)
                .await;
        }

        // [修复 #1344] 不再处理其他禁用原因,让调用方负责检查手动禁用

        // 4. 获取模型列表
        let models = match quota.get("models").and_then(|m| m.as_array()) {
            Some(m) => m,
            None => return false,
        };

        // 5. [重构] 聚合判定逻辑：按 Standard ID 对账号所有型号进行分组
        // 解决如 Pro-Low (0%) 和 Pro-High (100%) 在同一账号内导致状态冲突的问题
        let mut group_max_percentage: HashMap<String, i32> = HashMap::new();

        for model in models {
            let name = model.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let percentage = model
                .get("percentage")
                .and_then(|v| v.as_i64())
                .unwrap_or(100) as i32;

            if let Some(std_id) =
                crate::proxy::common::model_mapping::normalize_to_standard_id(name)
            {
                let entry = group_max_percentage.entry(std_id).or_insert(-1);
                if percentage > *entry {
                    *entry = percentage;
                }
            }
        }

        // 6. 遍历受监控的 Standard ID，根据组内“最好状态”执行锁定或恢复
        let threshold = config.threshold_percentage as i32;
        let account_id = account_json
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let mut changed = false;

        for std_id in &config.monitored_models {
            // [FIX] 归一化监控模型为标准 ID（例如用户在 UI 选了 gemini-3.7-flash，对齐到 gemini-3-flash）
            let lookup_key = crate::proxy::common::model_mapping::normalize_to_standard_id(std_id)
                .unwrap_or_else(|| std_id.clone());

            // 获取该组的最高百分比，如果账号没该组型号则视为 100%
            let max_pct = group_max_percentage
                .get(&lookup_key)
                .cloned()
                .unwrap_or(100);

            if max_pct < threshold {
                // 只有组内所有模型都不行，才触发全组保护
                if self
                    .trigger_quota_protection(
                        account_json,
                        &account_id,
                        account_path,
                        max_pct,
                        threshold,
                        &lookup_key,
                    )
                    .await
                    .unwrap_or(false)
                {
                    changed = true;
                }
            } else {
                // 只有全组都好（或者没这型号），才尝试从之前受限状态恢复
                let protected_models = account_json
                    .get("protected_models")
                    .and_then(|v| v.as_array());

                let is_protected = protected_models.map_or(false, |arr| {
                    arr.iter().any(|m| m.as_str() == Some(lookup_key.as_str()))
                });

                if is_protected {
                    if self
                        .restore_quota_protection(
                            account_json,
                            &account_id,
                            account_path,
                            &lookup_key,
                        )
                        .await
                        .unwrap_or(false)
                    {
                        changed = true;
                    }
                }
            }
        }

        let _ = changed; // 避免 unused 警告，如果后续逻辑需要可以继续使用

        // 我们不再因为配额原因返回 true（即不再跳过账号），
        // 而是加载并在 get_token 时进行过滤。
        false
    }

    /// 计算账号的最大剩余配额百分比（用于排序）
    /// 返回值: Option<i32> (max_percentage)
    pub(crate) fn calculate_quota_stats(&self, quota: &serde_json::Value) -> Option<i32> {
        let models = match quota.get("models").and_then(|m| m.as_array()) {
            Some(m) => m,
            None => return None,
        };

        let mut max_percentage = 0;
        let mut has_data = false;

        for model in models {
            if let Some(pct) = model.get("percentage").and_then(|v| v.as_i64()) {
                let pct_i32 = pct as i32;
                if pct_i32 > max_percentage {
                    max_percentage = pct_i32;
                }
                has_data = true;
            }
        }

        if has_data {
            Some(max_percentage)
        } else {
            None
        }
    }

    /// 从磁盘读取特定模型的 quota 百分比 [FIX] 排序使用目标模型的 quota 而非 max
    ///
    /// # 参数
    /// * `account_path` - 账号 JSON 文件路径
    /// * `model_name` - 目标模型名称（已标准化）
    #[allow(dead_code)] // 预留给精确配额读取逻辑
    pub(crate) fn get_model_quota_from_json(
        account_path: &PathBuf,
        model_name: &str,
    ) -> Option<i32> {
        let content = std::fs::read_to_string(account_path).ok()?;
        let account: serde_json::Value = serde_json::from_str(&content).ok()?;
        let models = account.get("quota")?.get("models")?.as_array()?;

        for model in models {
            if let Some(name) = model.get("name").and_then(|v| v.as_str()) {
                if crate::proxy::common::model_mapping::normalize_to_standard_id(name)
                    .unwrap_or_else(|| name.to_string())
                    == model_name
                {
                    return model
                        .get("percentage")
                        .and_then(|v| v.as_i64())
                        .map(|p| p as i32);
                }
            }
        }
        None
    }

    pub(crate) fn get_available_models_from_json(
        account_path: &PathBuf,
    ) -> Option<HashSet<String>> {
        let content = std::fs::read_to_string(account_path).ok()?;
        let account: serde_json::Value = serde_json::from_str(&content).ok()?;
        let models = account.get("quota")?.get("models")?.as_array()?;
        let mut result = HashSet::new();
        for model in models {
            if let Some(name) = model.get("name").and_then(|v| v.as_str()) {
                let normalized = name.trim().to_lowercase();
                if !normalized.is_empty() {
                    result.insert(normalized);
                }
            }
        }
        Some(result)
    }

    /// 从账号文件获取配额刷新时间
    ///
    /// 返回该账号最近的配额刷新时间字符串（ISO 8601 格式）
    ///
    /// # 参数
    /// - `account_id`: 账号 ID（用于查找账号文件）
    pub fn get_quota_reset_time(&self, account_id: &str) -> Option<String> {
        // 直接用 account_id 查找账号文件（文件名是 {account_id}.json）
        let account_path = self
            .data_dir
            .join("accounts")
            .join(format!("{}.json", account_id));

        let content = std::fs::read_to_string(&account_path).ok()?;
        let account: serde_json::Value = serde_json::from_str(&content).ok()?;

        // 获取 quota.models 中最早的 reset_time（最保守的锁定策略）
        account
            .get("quota")
            .and_then(|q| q.get("models"))
            .and_then(|m| m.as_array())
            .and_then(|models| {
                models
                    .iter()
                    .filter_map(|m| m.get("reset_time").and_then(|r| r.as_str()))
                    .filter(|s| !s.is_empty())
                    .min()
                    .map(|s| s.to_string())
            })
    }
}
