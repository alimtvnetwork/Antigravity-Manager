//! Target Model Quota Filter
//! Filters candidate accounts based on quota headroom for the specific target model
//! (e.g. Gemini 2.0 Flash, Gemini 1.5 Pro, Claude 3.7 Sonnet) before auto-switch execution.

use std::collections::HashMap;

pub struct ModelQuotaFilter;

impl ModelQuotaFilter {
    /// Determines if an account's quota map has adequate headroom for the requested model.
    pub fn has_model_headroom(
        model_quotas: &HashMap<String, f64>,
        target_model: &str,
        min_quota_threshold: f64,
    ) -> bool {
        let target_lower = target_model.to_ascii_lowercase();

        // 1. Direct model key match
        if let Some(&quota) = model_quotas.get(&target_lower) {
            return quota >= min_quota_threshold;
        }

        // 2. Family wildcard match (e.g. "gemini-2.0-flash" matches "gemini-flash")
        for (key, &quota) in model_quotas {
            let key_lower = key.to_ascii_lowercase();
            if (target_lower.contains("flash") && key_lower.contains("flash"))
                || (target_lower.contains("pro") && key_lower.contains("pro"))
                || (target_lower.contains("claude") && key_lower.contains("claude"))
            {
                return quota >= min_quota_threshold;
            }
        }

        // 3. Fallback to default/general quota bucket if present
        if let Some(&quota) = model_quotas.get("default") {
            return quota >= min_quota_threshold;
        }

        // If no model-specific quota is present, allow by default if threshold is zero
        min_quota_threshold <= 0.0
    }
}
