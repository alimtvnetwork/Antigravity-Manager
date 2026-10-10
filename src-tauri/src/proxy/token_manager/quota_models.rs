//! 动态模型候选集构建与账号模型解析。

use super::TokenManager;
use std::collections::HashSet;
use std::path::PathBuf;

impl TokenManager {
    pub(crate) fn build_dynamic_model_candidates(model_name: &str) -> Option<Vec<String>> {
        let model = model_name.trim().to_lowercase();
        if model.is_empty() {
            return None;
        }

        // Image models: drift ONLY across versions within the SAME tier
        // (pro-image ↔ pro-image, flash-image ↔ flash-image). Never silently downgrade
        // pro→flash. If the account has no model in the requested tier, the name is left
        // unchanged and upstream returns 404 — which is honest (the account lacks that model).
        // To alias e.g. gemini-3-pro-image to a flash model, use the app's Model Routing Center.
        let pro_image = ["gemini-3-pro-image", "gemini-3.1-pro-image"];
        let flash_image = ["gemini-3-flash-image", "gemini-3.1-flash-image"];
        let is_pro_image = pro_image.contains(&model.as_str());
        let is_flash_image = flash_image.contains(&model.as_str());
        if is_pro_image || is_flash_image {
            let mut out = Vec::new();
            let mut seen = HashSet::new();
            let mut push = |candidate: &str| {
                let c = candidate.to_string();
                if seen.insert(c.clone()) {
                    out.push(c);
                }
            };
            push(&model); // requested first
            if is_pro_image {
                push("gemini-3.1-pro-image");
                push("gemini-3-pro-image");
            } else {
                push("gemini-3.1-flash-image");
                push("gemini-3-flash-image");
            }
            return Some(out);
        }

        let pro_family = [
            "gemini-3-pro",
            "gemini-3-pro-preview",
            "gemini-3-pro-high",
            "gemini-3-pro-low",
            "gemini-3.1-pro",
            "gemini-3.1-pro-preview",
            "gemini-3.1-pro-high",
            "gemini-3.1-pro-low",
            "gemini-pro-agent",
        ];

        if !pro_family.contains(&model.as_str()) {
            return None;
        }

        let mut out = Vec::new();
        let mut seen = HashSet::new();
        let mut push = |candidate: &str| {
            let c = candidate.to_string();
            if seen.insert(c.clone()) {
                out.push(c);
            }
        };

        // Keep requested model as top priority, then fallback across the same family.
        push(&model);
        push("gemini-pro-agent");
        push("gemini-3.1-pro-preview");
        push("gemini-3-pro-preview");
        push("gemini-3.1-pro-high");
        push("gemini-3-pro-high");
        push("gemini-3.1-pro-low");
        push("gemini-3-pro-low");

        Some(out)
    }

    pub async fn resolve_dynamic_model_for_account(
        &self,
        account_id: &str,
        mapped_model: &str,
    ) -> String {
        let candidates = match Self::build_dynamic_model_candidates(mapped_model) {
            Some(c) => c,
            None => return mapped_model.to_string(),
        };

        let account_path = match self.tokens.get(account_id) {
            Some(token) => token.account_path.clone(),
            None => return mapped_model.to_string(),
        };

        let available_models = match Self::get_available_models_from_json(&account_path) {
            Some(models) if !models.is_empty() => models,
            _ => return mapped_model.to_string(),
        };

        for candidate in candidates {
            if available_models.contains(&candidate) {
                if candidate != mapped_model.to_lowercase() {
                    tracing::info!(
                        "[Dynamic-Model-Rewrite] account={} {} -> {}",
                        account_id,
                        mapped_model,
                        candidate
                    );
                }
                return candidate;
            }
        }

        mapped_model.to_string()
    }

    /// 测试辅助函数：公开访问 get_model_quota_from_json
    #[cfg(test)]
    pub fn get_model_quota_from_json_for_test(
        account_path: &PathBuf,
        model_name: &str,
    ) -> Option<i32> {
        Self::get_model_quota_from_json(account_path, model_name)
    }
}
