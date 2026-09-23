// Model name mapping
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::collections::HashMap;

// Dynamic deprecated model redirection table (old_model_id -> new_model_id)
pub static DYNAMIC_MODEL_FORWARDING_RULES: Lazy<DashMap<String, String>> =
    Lazy::new(|| DashMap::new());

pub fn update_dynamic_forwarding_rules(old_model: String, new_model: String) {
    if !DYNAMIC_MODEL_FORWARDING_RULES.contains_key(&old_model) {
        crate::modules::logger::log_info(&format!(
            "[Mapping] Registered automatic forwarding rule: {} -> {}",
            old_model, new_model
        ));
    }
    DYNAMIC_MODEL_FORWARDING_RULES.insert(old_model, new_model);
}

static CLAUDE_TO_GEMINI: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();

    // Directly supported models
    m.insert("claude-sonnet-4-6", "claude-sonnet-4-6");
    m.insert("claude-sonnet-4-6-thinking", "claude-sonnet-4-6-thinking");

    // [Redirect] Sonnet 4.5 -> Sonnet 4.6
    m.insert("claude-sonnet-4-5", "claude-sonnet-4-6");
    m.insert("claude-sonnet-4-5-thinking", "claude-sonnet-4-6-thinking");

    // Alias mapping
    m.insert("claude-sonnet-4-5-20250929", "claude-sonnet-4-6-thinking");
    m.insert("claude-3-5-sonnet-20241022", "claude-sonnet-4-6");
    m.insert("claude-3-5-sonnet-20240620", "claude-sonnet-4-6");
    // [Redirect] Opus 4.5 -> Opus 4.6 (Issue #1743)
    m.insert("claude-opus-4", "claude-opus-4-6-thinking");
    m.insert("claude-opus-4-5-thinking", "claude-opus-4-6-thinking");
    m.insert("claude-opus-4-5-20251101", "claude-opus-4-6-thinking");

    // Claude Opus 4.6
    m.insert("claude-opus-4-6-thinking", "claude-opus-4-6-thinking");
    m.insert("claude-opus-4-6", "claude-opus-4-6-thinking");
    m.insert("claude-opus-4.6-thinking", "claude-opus-4-6-thinking");
    m.insert("claude-opus-4.6", "claude-opus-4-6-thinking");
    m.insert("claude-opus-4-6-20260201", "claude-opus-4-6-thinking");

    m.insert("claude-haiku-4", "claude-sonnet-4-6");
    m.insert("claude-3-haiku-20240307", "claude-sonnet-4-6");
    m.insert("claude-haiku-4-5-20251001", "claude-sonnet-4-6");
    // OpenAI protocol mapping table
    m.insert("gpt-4", "gemini-2.5-flash");
    m.insert("gpt-4-turbo", "gemini-2.5-flash");
    m.insert("gpt-4-turbo-preview", "gemini-2.5-flash");
    m.insert("gpt-4-0125-preview", "gemini-2.5-flash");
    m.insert("gpt-4-1106-preview", "gemini-2.5-flash");
    m.insert("gpt-4-0613", "gemini-2.5-flash");

    m.insert("gpt-4o", "gemini-2.5-flash");
    m.insert("gpt-4o-2024-05-13", "gemini-2.5-flash");
    m.insert("gpt-4o-2024-08-06", "gemini-2.5-flash");

    m.insert("gpt-4o-mini", "gemini-2.5-flash");
    m.insert("gpt-4o-mini-2024-07-18", "gemini-2.5-flash");

    m.insert("gpt-3.5-turbo", "gemini-2.5-flash");
    m.insert("gpt-3.5-turbo-16k", "gemini-2.5-flash");
    m.insert("gpt-3.5-turbo-0125", "gemini-2.5-flash");
    m.insert("gpt-3.5-turbo-1106", "gemini-2.5-flash");
    m.insert("gpt-3.5-turbo-0613", "gemini-2.5-flash");

    // Gemini protocol mapping table
    m.insert("gemini-2.5-flash-lite", "gemini-2.5-flash");
    m.insert("gemini-2.5-flash-thinking", "gemini-2.5-flash-thinking");
    // Gemini Pro family:
    // - Concrete model IDs should pass through unchanged.
    // - Generic aliases (without tier) still route to preview as fallback entrypoint.
    m.insert("gemini-3.1-pro-low", "gemini-3.1-pro-low");
    m.insert("gemini-3.1-pro-high", "gemini-pro-agent");
    m.insert("gemini-3.1-pro-preview", "gemini-3.1-pro-preview");
    m.insert("gemini-3.1-pro", "gemini-3.1-pro-preview");
    m.insert("gemini-3-pro-low", "gemini-3-pro-low");
    m.insert("gemini-3-pro-high", "gemini-pro-agent");
    m.insert("gemini-3-pro-preview", "gemini-3-pro-preview");
    m.insert("gemini-3-pro", "gemini-3-pro-preview");
    m.insert("gemini-2.5-flash", "gemini-2.5-flash");
    m.insert("gemini-3-flash", "gemini-3-flash");
    m.insert("gemini-3.5-flash", "gemini-3.5-flash");
    m.insert("gemini-3.6-flash", "gemini-3.6-flash-tiered");
    m.insert("gemini-3.7-flash", "gemini-3.7-flash-tiered");
    m.insert("gemini-3.8-flash", "gemini-3.8-flash-tiered");
    m.insert("gemini-3.7-flash-tiered", "gemini-3.7-flash-tiered");
    m.insert("gemini-3.7-flash-low", "gemini-3.7-flash-low");
    m.insert("gemini-3.7-flash-medium", "gemini-3.7-flash-medium");
    m.insert("gemini-3.7-flash-high", "gemini-3.7-flash-high");
    m.insert("gemini-3-pro-image", "gemini-3-pro-image");

    // [New] Unified Virtual ID for Background Tasks (Title, Summary, etc.)
    // Allows users to override all background tasks via custom_mapping
    m.insert("internal-background-task", "gemini-2.5-flash");

    m
});

/// Map Claude model names to Gemini model names
///
/// # Mapping Strategy
/// 1. **Exact Match**: Check CLAUDE_TO_GEMINI mapping table
/// 2. **Known Prefix Passthrough**: gemini-* and *-thinking models pass through directly
/// 3. **[NEW] Direct Passthrough**: Unknown model IDs are passed directly to Google API
///
/// # Parameters
/// - `input`: Original model name
///
/// # Returns
/// Mapped target model name
///
/// # Examples
/// ```ignore
/// use antigravity_tools_lib::proxy::common::model_mapping::map_claude_model_to_gemini;
/// // Exact match
/// assert_eq!(map_claude_model_to_gemini("claude-opus-4"), "claude-opus-4-5-thinking");
///
/// // Gemini model passthrough
/// assert_eq!(map_claude_model_to_gemini("gemini-2.5-flash"), "gemini-2.5-flash");
///
/// // Direct passthrough for unknown models
/// assert_eq!(map_claude_model_to_gemini("claude-opus-4-6"), "claude-opus-4-6");
/// assert_eq!(map_claude_model_to_gemini("claude-sonnet-5"), "claude-sonnet-5");
/// ```
pub fn map_claude_model_to_gemini(input: &str) -> String {
    // 1. Check exact match in map
    if let Some(mapped) = CLAUDE_TO_GEMINI.get(input) {
        return mapped.to_string();
    }

    // 2. Pass-through known prefixes (gemini-, -thinking) to support dynamic suffixes
    if input.starts_with("gemini-") || input.contains("thinking") {
        return input.to_string();
    }

    // 3. [ENHANCED] Directly pass through unknown model IDs instead of forced fallback
    // This allows users to experience unreleased models via custom mappings
    // Google API will handle invalid models and return errors as needed
    input.to_string()
}

/// Get all built-in supported model list keywords
pub fn get_supported_models() -> Vec<String> {
    CLAUDE_TO_GEMINI.keys().map(|s| s.to_string()).collect()
}

/// Dynamically get all available models (built-in, custom, and upstream quota models)
pub async fn get_all_dynamic_models(
    custom_mapping: &tokio::sync::RwLock<std::collections::HashMap<String, String>>,
    token_manager: Option<&crate::proxy::token_manager::TokenManager>,
    only_raw_quota_models: bool,
) -> Vec<String> {
    use std::collections::HashSet;
    let mut model_ids = HashSet::new();

    // 1. Get dynamic models aggregated from upstream (Quota Models)
    if let Some(tm) = token_manager {
        for dynamic_model in tm.get_all_collected_models() {
            model_ids.insert(dynamic_model);
        }
    }

    // If not only_raw_quota_models, append custom_mapping, built-in aliases, and image models
    if !only_raw_quota_models {
        // 2. Get all custom mapping models (Custom)
        {
            let mapping = custom_mapping.read().await;
            for key in mapping.keys() {
                model_ids.insert(key.clone());
            }
        }

        // 3. Get all built-in mapping models
        for m in get_supported_models() {
            model_ids.insert(m);
        }

        // 4. Ensure commonly used Gemini/image generation model IDs are included
        model_ids.insert("gemini-3.1-pro-low".to_string());

        // Issue #247: Dynamically generate all Image Gen Combinations
        let base = "gemini-3-pro-image";
        let resolutions = vec!["", "-2k", "-4k"];
        let ratios = vec!["", "-1x1", "-4x3", "-3x4", "-16x9", "-9x16", "-21x9"];

        for res in resolutions {
            for ratio in ratios.iter() {
                let mut id = base.to_string();
                id.push_str(res);
                id.push_str(ratio);
                model_ids.insert(id);
            }
        }

        model_ids.insert("gemini-2.0-flash-exp".to_string());
        model_ids.insert("gemini-2.5-flash".to_string());
        model_ids.insert("gemini-3-flash".to_string());
        model_ids.insert("gemini-3.1-pro-high".to_string());
        model_ids.insert("gemini-3.1-pro-low".to_string());
    }

    let mut sorted_ids: Vec<_> = model_ids.into_iter().collect();
    sorted_ids.sort();
    sorted_ids
}

/// Wildcard matching - supports multiple wildcards
///
/// **Note**: Matching is **case-sensitive**. Pattern `GPT-4*` will NOT match `gpt-4-turbo`.
///
/// Examples:
/// - `gpt-4*` matches `gpt-4`, `gpt-4-turbo` ✓
/// - `claude-*-sonnet-*` matches `claude-3-5-sonnet-20241022` ✓
/// - `*-thinking` matches `claude-opus-4-5-thinking` ✓
/// - `a*b*c` matches `a123b456c` ✓
fn wildcard_match(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();

    // No wildcard - exact match
    if parts.len() == 1 {
        return pattern == text;
    }

    let mut text_pos = 0;

    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue; // Skip empty segments from consecutive wildcards
        }

        if i == 0 {
            // First segment must match start
            if !text[text_pos..].starts_with(part) {
                return false;
            }
            text_pos += part.len();
        } else if i == parts.len() - 1 {
            // Last segment must match end
            return text[text_pos..].ends_with(part);
        } else {
            // Middle segments - find next occurrence
            if let Some(pos) = text[text_pos..].find(part) {
                text_pos += pos + part.len();
            } else {
                return false;
            }
        }
    }

    true
}

/// Core model routing resolution engine
/// Priority: Exact match > Wildcard match > System default mapping
///
/// # Parameters
/// - `original_model`: Original model name
/// - `custom_mapping`: User custom mapping table
///
/// # Returns
/// Mapped target model name
pub fn resolve_model_route(
    original_model: &str,
    custom_mapping: &std::collections::HashMap<String, String>,
) -> String {
    // 0. Deprecated model redirection (highest priority, forced correction)
    // Intercept removed models and redirect along upstream fallback path
    if let Some(forwarded) = DYNAMIC_MODEL_FORWARDING_RULES.get(original_model) {
        crate::modules::logger::log_info(&format!(
            "[Router] Deprecated model redirection: {} -> {}",
            original_model,
            forwarded.value()
        ));
        return forwarded.value().clone();
    }

    // 1. Exact match (secondary priority)
    if let Some(target) = custom_mapping.get(original_model) {
        crate::modules::logger::log_info(&format!(
            "[Router] Exact mapping: {} -> {}",
            original_model, target
        ));
        return target.clone();
    }

    // 1.5 [NEW] 检查是否命中自定义映射中的通配符规则 `gemini-3.x-flash`（要求 x > 8）
    // 统一转为 3.x-flash-tiered 模型
    if custom_mapping.contains_key("gemini-3.x-flash") {
        if let Some(target) =
            crate::proxy::model_specs::resolve_gemini_3x_flash_tiered(original_model)
        {
            crate::modules::logger::log_info(&format!(
                "[Router] 命中内置通配符规则 gemini-3.x-flash (x > 8): {} -> {}",
                original_model, target
            ));
            return target;
        }
    }

    // 2. Wildcard match - most specific (highest non-wildcard chars) wins
    // Note: When multiple patterns have the SAME specificity, HashMap iteration order
    // determines the result (non-deterministic). Users can avoid this by making patterns
    // more specific. Future improvement: use IndexMap + frontend sorting for full control.
    let mut best_match: Option<(&str, &str, usize)> = None;

    for (pattern, target) in custom_mapping.iter() {
        if pattern.contains('*') && wildcard_match(pattern, original_model) {
            let specificity = pattern.chars().count() - pattern.matches('*').count();
            if best_match.is_none() || specificity > best_match.unwrap().2 {
                best_match = Some((pattern.as_str(), target.as_str(), specificity));
            }
        }
    }

    if let Some((pattern, target, _)) = best_match {
        crate::modules::logger::log_info(&format!(
            "[Router] Wildcard match: {} -> {} (rule: {})",
            original_model, target, pattern
        ));
        return target.to_string();
    }

    // 3. System default mapping
    // Check if bare Flash derivative >= 3.6 (e.g. gemini-3.6-flash, gemini-3.7-flash, gemini-3.8-flash)
    // Preset route automatically to the corresponding tiered adaptive thinking model
    if crate::proxy::model_specs::is_bare_gemini_v36_or_above_flash(original_model) {
        let routed = format!("{}-tiered", original_model);
        crate::modules::logger::log_info(&format!(
            "[Router] Suffix-less Gemini >= 3.6 Flash model routed to Tiered: {} -> {}",
            original_model, routed
        ));
        return routed;
    }
    let result = map_claude_model_to_gemini(original_model);
    if result != original_model {
        crate::modules::logger::log_info(&format!(
            "[Router] System default mapping: {} -> {}",
            original_model, result
        ));
    }
    result
}

/// Normalize any physical model name to one of the 3 standard protection IDs.
/// This ensures quota protection works consistently regardless of API versioning or request variations.
///
/// Standard IDs:
/// - `gemini-3-flash`: All Flash variants (1.5-flash, 2.5-flash, 3-flash, etc.)
/// - `gemini-3.1-flash-image`: Flash image generation/edit quota.
/// - `gemini-3-pro-high`: All Pro variants (1.5-pro, 2.5-pro, etc.)
/// - `gemini-3-pro-image`: Pro image generation quota.
/// - `claude-sonnet-4-5`: All Claude Sonnet variants (3-5-sonnet, sonnet-4-5, etc.)
///
/// Returns `None` if the model doesn't match any of the 3 protected categories.
/// Check if model is Gemini 3.5 Flash or higher (shares high quota with 3.1 Pro)
/// Semantic pattern match: gemini-{ver}-flash* where version ver >= 3.5 (supports future 3.10, 4.x, etc.)
fn is_high_tier_flash(lower: &str) -> bool {
    if !lower.contains("flash") {
        return false;
    }

    if let Some(pos) = lower.find("gemini-") {
        let rest = &lower[pos + 7..];
        if let Some(flash_pos) = rest.find("-flash") {
            let ver = &rest[..flash_pos];
            let mut parts = ver.split('.');
            if let Some(major_s) = parts.next() {
                if let Ok(major) = major_s.parse::<u32>() {
                    if major > 3 {
                        return true;
                    }
                    if major == 3 {
                        if let Some(minor_s) = parts.next() {
                            if let Ok(minor) = minor_s.parse::<u32>() {
                                return minor >= 5;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

pub fn normalize_to_standard_id(model_name: &str) -> Option<String> {
    let lower = model_name.to_lowercase();

    // 1. Image resources must keep Flash image and Pro image in separate quota buckets.
    // The quota API exposes `gemini-3.1-flash-image` separately, so grouping it under
    // `gemini-3-pro-image` makes available Flash image quota look exhausted.
    if lower.contains("image") {
        if lower.contains("flash") {
            return Some("gemini-3.1-flash-image".to_string());
        }
        return Some("gemini-3-pro-image".to_string());
    }

    // 2. High-tier Flash models (3.5-flash, 3.7-flash, etc.) share high quota with 3.1 Pro
    if is_high_tier_flash(&lower) {
        return Some("gemini-3-pro-high".to_string());
    }

    // 3. Standard Flash variants (1.5-flash, 2.0-flash, 2.5-flash, 3.0-flash, etc.)
    if lower.contains("flash") {
        return Some("gemini-3-flash".to_string());
    }

    // 4. gemini-3-pro-high (including pro variants)
    if lower.contains("pro") && !lower.contains("image") {
        return Some("gemini-3-pro-high".to_string());
    }

    // 4. Claude series (merge Opus, Sonnet, Haiku into unified protection group 'claude')
    if lower.contains("claude")
        || lower.contains("opus")
        || lower.contains("sonnet")
        || lower.contains("haiku")
    {
        return Some("claude".to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_mapping() {
        assert_eq!(
            map_claude_model_to_gemini("claude-3-5-sonnet-20241022"),
            "claude-sonnet-4-6"
        );
        // [Redirect] Sonnet 4.5 -> Sonnet 4.6
        assert_eq!(
            map_claude_model_to_gemini("claude-sonnet-4-5"),
            "claude-sonnet-4-6"
        );
        assert_eq!(
            map_claude_model_to_gemini("claude-sonnet-4-5-thinking"),
            "claude-sonnet-4-6-thinking"
        );
        assert_eq!(
            map_claude_model_to_gemini("claude-opus-4"),
            "claude-opus-4-6-thinking"
        );
        // Test gemini pass-through (should not be caught by "mini" rule)
        assert_eq!(
            map_claude_model_to_gemini("gemini-2.5-flash-mini-test"),
            "gemini-2.5-flash-mini-test"
        );
        assert_eq!(map_claude_model_to_gemini("unknown-model"), "unknown-model");
        // Gemini Pro concrete IDs should pass through unchanged.
        assert_eq!(
            map_claude_model_to_gemini("gemini-3-pro-high"),
            "gemini-pro-agent"
        );
        assert_eq!(
            map_claude_model_to_gemini("gemini-3-pro-low"),
            "gemini-3-pro-low"
        );
    }

    #[tokio::test]
    async fn test_get_all_dynamic_models_only_raw_quota_models() {
        let custom_mapping = tokio::sync::RwLock::new(
            [("gpt-4o".to_string(), "gemini-3.1-pro-high".to_string())]
                .into_iter()
                .collect(),
        );

        // When only_raw_quota_models is TRUE, custom_mapping & built-in aliases (like gpt-4o) should be filtered out
        let models_raw = get_all_dynamic_models(&custom_mapping, None, true).await;
        assert!(!models_raw.contains(&"gpt-4o".to_string()));

        // When only_raw_quota_models is FALSE, custom_mapping should be included
        let models_all = get_all_dynamic_models(&custom_mapping, None, false).await;
        assert!(models_all.contains(&"gpt-4o".to_string()));
    }

    #[test]
    fn test_mappings_continued() {
        assert_eq!(
            map_claude_model_to_gemini("gemini-3.1-pro-high"),
            "gemini-pro-agent"
        );
        assert_eq!(
            map_claude_model_to_gemini("gemini-3.1-pro-low"),
            "gemini-3.1-pro-low"
        );
        // Generic aliases still map to preview entrypoint.
        assert_eq!(
            map_claude_model_to_gemini("gemini-3-pro"),
            "gemini-3-pro-preview"
        );
        assert_eq!(
            map_claude_model_to_gemini("gemini-3.1-pro"),
            "gemini-3.1-pro-preview"
        );

        // Test Normalization (Opus 4.6 now merged into "claude" group)
        assert_eq!(
            normalize_to_standard_id("claude-opus-4-6-thinking"),
            Some("claude".to_string())
        );
        assert_eq!(
            normalize_to_standard_id("claude-sonnet-4-5"),
            Some("claude".to_string())
        );

        // [Regression] gemini-3-pro-image must NOT be grouped with gemini-3-pro-high
        assert_eq!(
            normalize_to_standard_id("gemini-3-pro-image"),
            Some("gemini-3-pro-image".to_string())
        );
        assert_eq!(
            normalize_to_standard_id("gemini-3-pro-high"),
            Some("gemini-3-pro-high".to_string())
        );

        // [FIX #1955] Test normalization with image suffixes
        assert_eq!(
            normalize_to_standard_id("gemini-3-pro-image-4k"),
            Some("gemini-3-pro-image".to_string())
        );
        assert_eq!(
            normalize_to_standard_id("gemini-3-pro-image-16x9"),
            Some("gemini-3-pro-image".to_string())
        );
        assert_eq!(
            normalize_to_standard_id("gemini-3-pro-image-4k-16x9"),
            Some("gemini-3-pro-image".to_string())
        );
        assert_eq!(
            normalize_to_standard_id("gemini-3.1-flash-image"),
            Some("gemini-3.1-flash-image".to_string())
        );
        assert_eq!(
            normalize_to_standard_id("gemini-3.1-flash-image-4k"),
            Some("gemini-3.1-flash-image".to_string())
        );
    }

    #[test]
    fn test_wildcard_priority() {
        let mut custom = HashMap::new();
        custom.insert("gpt*".to_string(), "fallback".to_string());
        custom.insert("gpt-4*".to_string(), "specific".to_string());
        custom.insert("claude-opus-*".to_string(), "opus-default".to_string());
        custom.insert(
            "claude-opus*thinking".to_string(),
            "opus-thinking".to_string(),
        );

        // More specific pattern wins
        assert_eq!(resolve_model_route("gpt-4-turbo", &custom), "specific");
        assert_eq!(resolve_model_route("gpt-3.5", &custom), "fallback");
        // Suffix constraint is more specific than prefix-only
        assert_eq!(
            resolve_model_route("claude-opus-4-5-thinking", &custom),
            "opus-thinking"
        );
        assert_eq!(
            resolve_model_route("claude-opus-4", &custom),
            "opus-default"
        );
    }

    #[test]
    fn test_multi_wildcard_support() {
        let mut custom = HashMap::new();
        custom.insert(
            "claude-*-sonnet-*".to_string(),
            "sonnet-versioned".to_string(),
        );
        custom.insert("gpt-*-*".to_string(), "gpt-multi".to_string());
        custom.insert("*thinking*".to_string(), "has-thinking".to_string());

        // Multi-wildcard patterns should work
        assert_eq!(
            resolve_model_route("claude-3-5-sonnet-20241022", &custom),
            "sonnet-versioned"
        );
        assert_eq!(
            resolve_model_route("gpt-4-turbo-preview", &custom),
            "gpt-multi"
        );
        assert_eq!(
            resolve_model_route("claude-thinking-extended", &custom),
            "has-thinking"
        );

        // Negative case: *thinking* should NOT match models without "thinking"
        assert_eq!(
            resolve_model_route("random-model-name", &custom),
            "random-model-name" // Falls back to system default (pass-through)
        );
    }

    #[test]
    fn test_wildcard_edge_cases() {
        let mut custom = HashMap::new();
        custom.insert("prefix*".to_string(), "prefix-match".to_string());
        custom.insert("*".to_string(), "catch-all".to_string());
        custom.insert("a*b*c".to_string(), "multi-wild".to_string());

        // Specificity: "prefix*" (6) > "*" (0)
        assert_eq!(
            resolve_model_route("prefix-anything", &custom),
            "prefix-match"
        );
        // Catch-all has lowest specificity
        assert_eq!(resolve_model_route("random-model", &custom), "catch-all");
        // Multi-wildcard: "a*b*c" (3)
        assert_eq!(resolve_model_route("a-test-b-foo-c", &custom), "multi-wild");
    }

    #[test]
    fn test_gemini_3x_flash_wildcard_route() {
        let mut custom = crate::proxy::config::default_custom_mapping();
        assert!(custom.contains_key("gemini-3.6-flash"));
        assert!(custom.contains_key("gemini-3.7-flash"));
        assert!(custom.contains_key("gemini-3.8-flash"));
        assert!(custom.contains_key("gemini-3.x-flash"));

        // 1. 3.6 / 3.7 / 3.8 精确匹配默认预设
        assert_eq!(
            resolve_model_route("gemini-3.6-flash", &custom),
            "gemini-3.6-flash-tiered"
        );
        assert_eq!(
            resolve_model_route("gemini-3.7-flash", &custom),
            "gemini-3.7-flash-tiered"
        );
        assert_eq!(
            resolve_model_route("gemini-3.8-flash", &custom),
            "gemini-3.8-flash-tiered"
        );

        // 2. x > 8 命中通配符规则 gemini-3.x-flash，统一转为 3.x-flash-tiered
        assert_eq!(
            resolve_model_route("gemini-3.9-flash", &custom),
            "gemini-3.9-flash-tiered"
        );
        assert_eq!(
            resolve_model_route("gemini-3.10-flash", &custom),
            "gemini-3.10-flash-tiered"
        );

        // 3. 用户如果自定义精确覆盖 gemini-3.9-flash，用户自定义优先
        custom.insert(
            "gemini-3.9-flash".to_string(),
            "gemini-3.9-flash-high".to_string(),
        );
        assert_eq!(
            resolve_model_route("gemini-3.9-flash", &custom),
            "gemini-3.9-flash-high"
        );

        // 4. 大于 3.8 的未来模型即使不在精确表中也统一走 tiered（含 4.x）
        assert_eq!(
            resolve_model_route("gemini-4.0-flash", &custom),
            "gemini-4.0-flash-tiered"
        );
    }
}
