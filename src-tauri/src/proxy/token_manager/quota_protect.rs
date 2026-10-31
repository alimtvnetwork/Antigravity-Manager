//! Quota Protection: Trigger/Check Recovery/Restore Quota Protection.

use super::helpers::update_account_json;
use super::TokenManager;
use std::collections::HashMap;
use std::path::PathBuf;

impl TokenManager {
    /// Trigger quota protection to restrict specific models (Issue #621)
    /// Returns true if a change has occurred
    pub(crate) async fn trigger_quota_protection(
        &self,
        account_json: &mut serde_json::Value,
        account_id: &str,
        account_path: &PathBuf,
        current_val: i32,
        threshold: i32,
        model_name: &str,
    ) -> Result<bool, String> {
        // 1. Initialize the protected_models array (if it does not exist)
        if account_json.get("protected_models").is_none() {
            account_json["protected_models"] = serde_json::Value::Array(Vec::new());
        }

        let protected_models = account_json["protected_models"].as_array_mut().unwrap();

        // 2. Check if it already exists
        if !protected_models
            .iter()
            .any(|m| m.as_str() == Some(model_name))
        {
            protected_models.push(serde_json::Value::String(model_name.to_string()));

            tracing::info!(
                "Model {} of account {} has been added to the protection list due to quota restrictions ( {} % < {} %)",
                account_id,
                model_name,
                current_val,
                threshold
            );

            // 3. Write to disk
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

            // [FIX] Trigger TokenManager's account reload signal to ensure synchronization of protected_models in memory
            crate::proxy::server::trigger_account_reload(account_id);

            return Ok(true);
        }

        Ok(false)
    }

    /// Check and restore from account level protection (migrated to model level, Issue #621)
    pub(crate) async fn check_and_restore_quota(
        &self,
        account_json: &mut serde_json::Value,
        account_path: &PathBuf,
        quota: &serde_json::Value,
        config: &crate::models::QuotaProtectionConfig,
    ) -> bool {
        // [Compatibility] If the account is currently proxy_disabled=true and the reason is quota_protection,
        // We set its proxy_disabled to false, but also update its protected_models list.
        tracing::info!(
            "Migrating account {} from global quota protection mode to model-level protection mode",
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

        false // Returning false indicates that you can now try to load the account (model-level filtering will occur during get_token)
    }

    /// Restore quota protection for specific models (Issue #621)
    /// Returns true if a change has occurred
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
                    "The quota of model {} of account {} has been restored and removed from the protection list",
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

    /// Candidate pool size for P2C algorithm - randomly selected from the top N best candidates
    pub(crate) const P2C_POOL_SIZE: usize = 5;
}
