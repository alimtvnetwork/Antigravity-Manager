//! 配额保护：触发 / 检查恢复 / 恢复配额保护。

use super::helpers::update_account_json;
use super::TokenManager;
use std::collections::HashMap;
use std::path::PathBuf;

impl TokenManager {
    /// 触发配额保护，限制特定模型 (Issue #621)
    /// 返回 true 如果发生了改变
    pub(crate) async fn trigger_quota_protection(
        &self,
        account_json: &mut serde_json::Value,
        account_id: &str,
        account_path: &PathBuf,
        current_val: i32,
        threshold: i32,
        model_name: &str,
    ) -> Result<bool, String> {
        // 1. 初始化 protected_models 数组（如果不存在）
        if account_json.get("protected_models").is_none() {
            account_json["protected_models"] = serde_json::Value::Array(Vec::new());
        }

        let protected_models = account_json["protected_models"].as_array_mut().unwrap();

        // 2. 检查是否已存在
        if !protected_models
            .iter()
            .any(|m| m.as_str() == Some(model_name))
        {
            protected_models.push(serde_json::Value::String(model_name.to_string()));

            tracing::info!(
                "账号 {} 的模型 {} 因配额受限（{}% < {}%）已被加入保护列表",
                account_id,
                model_name,
                current_val,
                threshold
            );

            // 3. 写入磁盘
            let model_name_owned = model_name.to_string();
            update_account_json(account_path, move |latest| {
                if latest
                    .get("protected_models")
                    .and_then(|value| value.as_array())
                    .is_none()
                {
                    latest["protected_models"] = serde_json::Value::Array(Vec::new());
                }
                let protected_models = latest["protected_models"].as_array_mut().unwrap();
                if !protected_models
                    .iter()
                    .any(|model| model.as_str() == Some(&model_name_owned))
                {
                    protected_models.push(serde_json::Value::String(model_name_owned));
                }
            })
            .await?;

            // [FIX] 触发 TokenManager 的账号重新加载信号，确保内存中的 protected_models 同步
            crate::proxy::server::trigger_account_reload(account_id);

            return Ok(true);
        }

        Ok(false)
    }

    /// 检查并从账号级保护恢复（迁移至模型级，Issue #621）
    pub(crate) async fn check_and_restore_quota(
        &self,
        account_json: &mut serde_json::Value,
        account_path: &PathBuf,
        quota: &serde_json::Value,
        config: &crate::models::QuotaProtectionConfig,
    ) -> bool {
        // [兼容性] 如果该账号当前处于 proxy_disabled=true 且原因是 quota_protection，
        // 我们将其 proxy_disabled 设为 false，但同时更新其 protected_models 列表。
        tracing::info!(
            "正在迁移账号 {} 从全局配额保护模式至模型级保护模式",
            account_json
                .get("email")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
        );

        account_json["proxy_disabled"] = serde_json::Value::Bool(false);
        account_json["proxy_disabled_reason"] = serde_json::Value::Null;
        account_json["proxy_disabled_at"] = serde_json::Value::Null;

        let threshold = config.threshold_percentage as i32;
        let mut protected_list: Vec<serde_json::Value> = Vec::new();

        if let Some(models) = quota.get("models").and_then(|m| m.as_array()) {
            let mut group_max_percentage: HashMap<String, i32> = HashMap::new();

            for model in models {
                let name = model.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let percentage = model
                    .get("percentage")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;

                if let Some(std_id) =
                    crate::proxy::common::model_mapping::normalize_to_standard_id(name)
                {
                    let entry = group_max_percentage.entry(std_id).or_insert(-1);
                    if percentage > *entry {
                        *entry = percentage;
                    }
                }
            }

            for std_id in &config.monitored_models {
                let lookup_key =
                    crate::proxy::common::model_mapping::normalize_to_standard_id(std_id)
                        .unwrap_or_else(|| std_id.clone());
                let max_pct = group_max_percentage
                    .get(&lookup_key)
                    .cloned()
                    .unwrap_or(100);
                if max_pct < threshold
                    && !protected_list
                        .iter()
                        .any(|v| v.as_str() == Some(lookup_key.as_str()))
                {
                    protected_list.push(serde_json::Value::String(lookup_key));
                }
            }
        }

        account_json["protected_models"] = serde_json::Value::Array(protected_list.clone());

        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
        crate::error::record_ignored(
            update_account_json(account_path, |latest| {
                latest["proxy_disabled"] = serde_json::Value::Bool(false);
                latest["proxy_disabled_reason"] = serde_json::Value::Null;
                latest["proxy_disabled_at"] = serde_json::Value::Null;
                latest["protected_models"] = serde_json::Value::Array(protected_list);
            })
            .await,
            "update_account_json",
        );

        false // 返回 false 表示现在已可以尝试加载该账号（模型级过滤会在 get_token 时发生）
    }

    /// 恢复特定模型的配额保护 (Issue #621)
    /// 返回 true 如果发生了改变
    pub(crate) async fn restore_quota_protection(
        &self,
        account_json: &mut serde_json::Value,
        account_id: &str,
        account_path: &PathBuf,
        model_name: &str,
    ) -> Result<bool, String> {
        if let Some(arr) = account_json
            .get_mut("protected_models")
            .and_then(|v| v.as_array_mut())
        {
            let original_len = arr.len();
            arr.retain(|m| m.as_str() != Some(model_name));

            if arr.len() < original_len {
                tracing::info!(
                    "账号 {} 的模型 {} 配额已恢复，移出保护列表",
                    account_id,
                    model_name
                );
                let model_name_owned = model_name.to_string();
                update_account_json(account_path, move |latest| {
                    if let Some(protected_models) = latest
                        .get_mut("protected_models")
                        .and_then(|value| value.as_array_mut())
                    {
                        protected_models.retain(|model| model.as_str() != Some(&model_name_owned));
                    }
                })
                .await?;
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// P2C 算法的候选池大小 - 从前 N 个最优候选中随机选择
    pub(crate) const P2C_POOL_SIZE: usize = 5;
}
