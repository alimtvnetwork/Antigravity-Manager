use super::claude_gemini::map_claude_model_to_gemini;
use super::forwarding::DYNAMIC_MODEL_FORWARDING_RULES;

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

/// Core model routing analysis engine
/// Priority: Exact match > Wildcard match > System default mapping
///
/// # Parameters
/// - `original_model`: original model name
/// - `custom_mapping`: user-defined mapping table
///
/// # Return
/// The mapped target model name
pub fn resolve_model_route(
    original_model: &str,
    custom_mapping: &std::collections::HashMap<String, String>,
) -> String {
    // 0. API hot update obsolete model forwarding (highest physical priority, forced correction)
    // If the user insists on using a model that has been removed and an official fallback path is issued, we will intercept and correct it here.
    if let Some(forwarded) = DYNAMIC_MODEL_FORWARDING_RULES.get(original_model) {
        crate::modules::logger::log_info(&format!(
            "[Router] Official obsolete redirection: {} -> {}",
            original_model,
            forwarded.value()
        ));
        return forwarded.value().clone();
    }

    // 1. Exact match (second highest priority)
    if let Some(target) = custom_mapping.get(original_model) {
        crate::modules::logger::log_info(&format!(
            "[Router] Exact mapping: {} -> {}",
            original_model, target
        ));
        return target.clone();
    }

    // 1.5 [NEW] Check if wildcard rule `gemini-3.x-flash` in custom mapping is hit (requires x > 8)
    // Unified conversion to 3.x-flash-tiered model
    if custom_mapping.contains_key("gemini-3.x-flash") {
        if let Some(target) =
            crate::proxy::model_specs::resolve_gemini_3x_flash_tiered(original_model)
        {
            crate::modules::logger::log_info(&format!(
                "[Router] Hit built-in wildcard rule gemini-3.x-flash (x > 8): {} -> {}",
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
    // [NEW] Check if it is an unsuffixed Flash derivative model >= 3.6 (such as gemini-3.6-flash, gemini-3.7-flash, gemini-3.8-flash, etc.)
    // Unify and automatically preset routing to the corresponding tiered adaptive thinking model
    if crate::proxy::model_specs::is_bare_gemini_v36_or_above_flash(original_model) {
        let routed = format!("{}-tiered", original_model);
        crate::modules::logger::log_info(&format!(
            "[Router] No suffix Gemini >= 3.6 Flash model unified default routing to Tiered: {} -> {}",
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
/// Determine whether it is a high-end Flash model for Gemini 3.5 Flash and above (shares advanced quota with 3.1 Pro)
/// Strict semantic wildcard: gemini-
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

    // 2. High-end Flash models of 3.5 Flash and above (such as 3.5-flash, 3.7-flash, 3.8-flash, etc.) share advanced quotas with 3.1 Pro
    if is_high_tier_flash(&lower) {
        return Some("gemini-3-pro-high".to_string());
    }

    // 3. gemini-3-flash (including ordinary 1.5-flash, 2.0-flash, 2.5-flash, 3.0-flash and other basic flash variants)
    if lower.contains("flash") {
        return Some("gemini-3-flash".to_string());
    }

    // 4. gemini-3-pro-high (includes pro variant)
    if lower.contains("pro") && !lower.contains("image") {
        return Some("gemini-3-pro-high".to_string());
    }

    // 4. Claude series (merging Opus, Sonnet, and Haiku into a unified protection group 'claude')
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
    use super::super::map_claude_model_to_gemini;
    use super::*;
    use crate::proxy::common::model_mapping::get_all_dynamic_models;
    use std::collections::HashMap;

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
            "gemini-3.1-pro-preview"
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

        // 1. 3.6 / 3.7 / 3.8 exact match default preset
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

        // 2. x > 8 hits the wildcard rule gemini-3.x-flash and is uniformly converted to 3.x-flash-tiered
        assert_eq!(
            resolve_model_route("gemini-3.9-flash", &custom),
            "gemini-3.9-flash-tiered"
        );
        assert_eq!(
            resolve_model_route("gemini-3.10-flash", &custom),
            "gemini-3.10-flash-tiered"
        );

        // 3. If the user’s customization accurately covers gemini-3.9-flash, the user’s customization will be given priority.
        custom.insert(
            "gemini-3.9-flash".to_string(),
            "gemini-3.9-flash-high".to_string(),
        );
        assert_eq!(
            resolve_model_route("gemini-3.9-flash", &custom),
            "gemini-3.9-flash-high"
        );

        // 4. Future models greater than 3.8 will be uniformly tiered (including 4.x) even if they are not in the exact table.
        assert_eq!(
            resolve_model_route("gemini-4.0-flash", &custom),
            "gemini-4.0-flash-tiered"
        );
    }
}
