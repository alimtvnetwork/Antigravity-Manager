//! Quota reading and statistics: check_and_protect_quota and quota JSON query.

use super::helpers::update_account_json;
use super::TokenManager;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

impl TokenManager {
    /// Check if the account should be quota protected
    /// If the quota is below the threshold, automatically disable the account and return true
    pub(crate) async fn check_and_protect_quota(
        &self,
        account_json: &mut serde_json::Value,
        account_path: &PathBuf,
    ) -> bool {
        // 1. Load quota protection configuration
        let config = match crate::modules::config::load_app_config() {
            Ok(cfg) => cfg.quota_protection,
            Err(_) => return false, // Configuration loading failed, protection skipped
        };

        if !config.enabled {
            // [FIX] When quota protection is turned off globally, clear the protected model list to avoid legacy lock display and scheduling filtering
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
            return false; // Quota protection is not enabled
        }

        // 2. Get quota information
        // Note: We need clone quota information to traverse to avoid borrowing conflicts, but the modification is for account_json
        let quota = match account_json.get("quota") {
            Some(q) => q.clone(),
            None => return false, // No quota information, skip
        };

        // 3. [Compatibility #621] Check whether it is disabled by the old version of account-level quota protection, try to restore and convert to model level
        let is_proxy_disabled = account_json
            .get("proxy_disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let reason = account_json
            .get("proxy_disabled_reason")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if is_proxy_disabled && reason == "quota_protection" {
            // If it was disabled by the old version of account-level protection, try to restore it and convert it to model level.
            return self
                .check_and_restore_quota(account_json, account_path, &quota, &config)
                .await;
        }

        // [Fix #1344] No longer handles other reasons for disabling, leaving the caller responsible for checking for manual disabling

        // 4. Get model list
        let models = match quota.get("models").and_then(|m| m.as_array()) {
            Some(m) => m,
            None => return false,
        };

        // 5. [Reconstruction] Aggregation decision logic: Group all models of the account according to Standard ID
        // Resolved an issue where Pro-Low (0%) and Pro-High (100%) caused status conflicts within the same account
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

        // 6. Traverse the monitored Standard IDs and perform locking or recovery based on the "best status" within the group
        let threshold = config.threshold_percentage as i32;
        let account_id = account_json
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let mut changed = false;

        for std_id in &config.monitored_models {
            // [FIX] The normalized monitoring model is a standard ID (for example, if the user selects gemini-3.7-flash in the UI, it is aligned to gemini-3-flash)
            let lookup_key = crate::proxy::common::model_mapping::normalize_to_standard_id(std_id)
                .unwrap_or_else(|| std_id.clone());

            // Get the highest percentage of the group. If the account does not have the model of the group, it will be regarded as 100%.
            let max_pct = group_max_percentage
                .get(&lookup_key)
                .cloned()
                .unwrap_or(100);

            if max_pct < threshold {
                // Only when all models in the group fail will the whole group protection be triggered.
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
                // Only if the whole group is fine (or if this model is not available), try to recover from the previous restricted state.
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

        let _ = changed; // Avoid unused warnings and continue to use them if subsequent logic requires it.

        // We no longer return true due to quota reasons (i.e. no longer skip accounts),
        // Instead load and filter on get_token.
        false
    }

    /// Calculate the maximum remaining quota percentage of the account (used for sorting)
    /// Return value: Option<i32> (max_percentage)
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

    /// Read model-specific quota percentage from disk [FIX] Sorting uses target model's quota instead of max
    ///
    /// # Parameters
    /// * `account_path` - Account JSON file path
    /// * `model_name` - target model name (normalized)
    #[allow(dead_code)] // Reserved for exact quota read logic
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

    /// Get quota refresh time from account file
    ///
    /// Returns the most recent quota refresh time string for this account (ISO 8601 format)
    ///
    /// # Parameters
    /// - `account_id`: account ID (used to find account files)
    pub fn get_quota_reset_time(&self, account_id: &str) -> Option<String> {
        // Directly use account_id to find the account file (the file name is {account_id} .json)
        let account_path = self
            .data_dir
            .join("accounts")
            .join(format!("{}.json", account_id));

        let content = std::fs::read_to_string(&account_path).ok()?;
        let account: serde_json::Value = serde_json::from_str(&content).ok()?;

        // Get the earliest reset_time in quota.models (the most conservative locking strategy)
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
