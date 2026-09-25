use crate::proxy::token_manager::ProxyToken;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpec {
    pub max_output_tokens: Option<u64>,
    pub thinking_budget: Option<u64>,
    pub is_thinking: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SpecsConfig {
    models: HashMap<String, ModelSpec>,
    aliases: HashMap<String, String>,
}

static SPECS: Lazy<SpecsConfig> = Lazy::new(|| {
    let json_str = include_str!("../../resources/model_specs.json");
    serde_json::from_str(json_str).expect("Failed to parse model_specs.json")
});

/// Retrieves normalized model ID based on alias
pub fn resolve_alias(model_id: &str) -> String {
    SPECS
        .aliases
        .get(model_id)
        .cloned()
        .unwrap_or_else(|| model_id.to_string())
}

/// Retrieves model output token limit (dynamic data prioritized)
pub fn get_max_output_tokens(model_id: &str, token: Option<&ProxyToken>) -> u64 {
    let std_id = resolve_alias(model_id);

    // 1. Try reading from account dynamic data
    if let Some(t) = token {
        if let Some(&limit) = t.model_limits.get(&std_id) {
            return limit;
        }
        // If original ID not found, try lookup with normalized ID
        if let Some(&limit) = t.model_limits.get(model_id) {
            return limit;
        }
    }

    // 2. Fall back to static JSON
    if let Some(spec) = SPECS.models.get(&std_id) {
        if let Some(limit) = spec.max_output_tokens {
            return limit;
        }
    }

    // 3. Global fallback
    65535
}

/// Get reasoning budget (prioritizes dynamic tiers from model ID dictionary)
pub fn get_thinking_budget(model_id: &str, _token: Option<&ProxyToken>) -> u64 {
    let std_id = resolve_alias(model_id);
    let lower = std_id.to_lowercase();

    // 1. Explicit tier matching (Flash / Pro tiers: high, medium, low, extra-low, max)
    if lower.contains("high") || lower.contains("agent") || lower.contains("max") {
        if lower.contains("pro") {
            return 10001; // Google gemini-3.1-pro-high / gemini-pro-agent authoritative budget
        }
        return 10000; // gemini-3.x-flash-high maximum thinking budget
    }
    if lower.contains("medium") {
        return 4000; // gemini-3.x-flash-medium medium thinking budget
    }
    if lower.contains("extra-low") {
        return 1000; // gemini-3.5-flash-extra-low standard budget
    }
    if lower.contains("low") {
        if lower.contains("pro") {
            return 1001; // gemini-3.1-pro-low standard budget
        }
        return 1000; // gemini-3.x-flash-low standard budget
    }

    // 2. Static JSON configuration (model_specs.json)
    if let Some(spec) = SPECS.models.get(&std_id) {
        if let Some(budget) = spec.thinking_budget {
            return budget;
        }
    }

    // 3. All Gemini >= 3.0 Flash models without explicit suffixes:
    // Medium or bare models default to 4000
    if is_gemini_v3_or_above(&std_id) && lower.contains("flash") {
        return 4000;
    }

    // 4. Legacy model default budgets
    if lower.contains("claude") {
        16000
    } else if lower.contains("2.5-flash") || lower.contains("2.0-flash") {
        24576
    } else if lower.contains("pro") {
        49152
    } else {
        24576
    }
}

/// Check whether model matches explicit heuristic tier suffix (-high, -medium, -low, -extra-low, -max, -agent, -thinking, etc.)
pub fn is_explicit_heuristic_tier_model(model_id: &str) -> bool {
    let std_id = resolve_alias(model_id);
    let lower = std_id.to_lowercase();
    lower.ends_with("-high")
        || lower.ends_with("-medium")
        || lower.ends_with("-low")
        || lower.ends_with("-extra-low")
        || lower.ends_with("-max")
        || lower.ends_with("-thinking")
        || lower.ends_with("-agent")
        || lower.contains("-high-")
        || lower.contains("-medium-")
        || lower.contains("-low-")
        || lower.contains("-extra-low-")
        || lower.contains("-max-")
        || lower.contains("-agent")
        || lower.contains("-thinking")
}

/// Authoritatively resolve reasoning budget across protocols
pub fn resolve_authoritative_thinking_budget(
    model: &str,
    client_effort: Option<&str>,
    _client_budget: Option<u64>,
    token: Option<&ProxyToken>,
) -> u64 {
    // 1. If non-thinking Gemini < 3 model, return 0
    if is_gemini_under_v3(model) {
        return 0;
    }

    // 2. Explicit tier models: strict highest priority, ignore client effort
    if is_explicit_heuristic_tier_model(model) {
        return get_thinking_budget(model, token);
    }

    // 3. Non-Gemini 3 models (pure Claude or legacy models), use standard default
    if !is_gemini_v3_or_above(model) && !model.to_lowercase().contains("gemini") {
        return get_thinking_budget(model, token);
    }

    // 4. Bare models without explicit suffix (gemini-3-flash, gemini-3.1-pro, etc.)
    let std_id = resolve_alias(model);
    let lower = std_id.to_lowercase();
    let is_pro = lower.contains("pro");

    // Controlled by client effort / thinkingLevel:
    // high / max / xhigh → 10000 / 10001
    // medium / default / omitted -> 4000 / 10001
    // low / extra-low → 1000 / 1001
    // If client attempts to disable thinking (none / 0 / disabled) or omits parameter: enforce -medium default budget
    if let Some(effort) = client_effort {
        let eff_lower = effort.trim().to_lowercase();
        match eff_lower.as_str() {
            "high" | "max" | "xhigh" => {
                if is_pro {
                    10001
                } else {
                    10000
                }
            }
            "low" | "extra-low" => {
                if is_pro {
                    1001
                } else {
                    1000
                }
            }
            "medium" | "default" => {
                if is_pro {
                    10001
                } else {
                    4000
                }
            }
            "none" | "0" | "disabled" => {
                // Attempted disable: enforce -medium default budget
                if is_pro {
                    10001
                } else {
                    4000
                }
            }
            _ => {
                // Unknown effort, default to -medium budget
                if is_pro {
                    10001
                } else {
                    4000
                }
            }
        }
    } else {
        // Missing effort: default to -medium budget
        if is_pro {
            10001
        } else {
            4000
        }
    }
}

/// Check if model is a thinking model
#[allow(dead_code)]
pub fn is_thinking_model(model_id: &str) -> bool {
    let std_id = resolve_alias(model_id);
    if let Some(spec) = SPECS.models.get(&std_id) {
        return spec.is_thinking.unwrap_or(false);
    }
    model_id.contains("-thinking") || model_id.contains("thinking")
}

/// Check whether model is Gemini with major version < 3.0 (e.g. gemini-2.5-flash, gemini-2.0-flash, gemini-1.5-pro).
/// Such models do not support thinkingConfig on Google official API; thinking parameters must not be injected.
pub fn is_gemini_under_v3(model: &str) -> bool {
    let lower = model.to_lowercase();
    if !lower.contains("gemini") {
        return false;
    }
    // Special alias/agent models: gemini-pro-agent / gemini-flash-agent belong to gemini-3; -exp / thinking-exp are thinking experiments
    if lower.contains("agent")
        || lower.contains("-exp")
        || lower.contains("thinking-exp")
        || lower.contains("-thinking")
    {
        return false;
    }
    // Check explicit gemini-X pattern
    if let Some(idx) = lower.find("gemini-") {
        let rest = &lower[idx + "gemini-".len()..];
        let version_part: String = rest
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if let Ok(ver) = version_part.parse::<f32>() {
            return ver < 3.0;
        }
    }
    // Fallback compatibility for gemini-1 / gemini-2
    lower.contains("gemini-1") || lower.contains("gemini-2")
}

/// Check whether model is Gemini 3.0 or above (e.g. gemini-3, gemini-3.1, gemini-3.7, gemini-3.8).
/// Such models support thinking mode.
pub fn is_gemini_v3_or_above(model: &str) -> bool {
    let lower = model.to_lowercase();
    if !lower.contains("gemini") {
        return false;
    }
    if lower.contains("agent") || lower == "gemini-pro" || lower == "gemini-flash" {
        return true;
    }
    if let Some(idx) = lower.find("gemini-") {
        let rest = &lower[idx + "gemini-".len()..];
        let version_part: String = rest
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if let Ok(ver) = version_part.parse::<f32>() {
            return ver >= 3.0;
        }
    }
    lower.contains("gemini-3") || lower.contains("gemini-4")
}

/// Checks whether model is a Tiered Flash model (e.g., gemini-3.8-flash-tiered)
pub fn is_tiered_flash_model(model: &str) -> bool {
    let model_id = model
        .rsplit('/')
        .next()
        .unwrap_or(model)
        .to_ascii_lowercase();
    model_id
        .strip_prefix("gemini-")
        .and_then(|rest| rest.strip_suffix("-flash-tiered"))
        .is_some_and(|version| !version.is_empty())
}

/// Checks whether model is a bare Flash derivative model >= 3.6 without suffix (e.g. gemini-3.6-flash, gemini-3.7-flash, etc.)
pub fn is_bare_gemini_v36_or_above_flash(model: &str) -> bool {
    let lower = model.to_lowercase();
    if !lower.contains("gemini") || !lower.contains("flash") {
        return false;
    }
    // Exclude models that already have suffixes or variant markers
    if lower.ends_with("-high")
        || lower.ends_with("-medium")
        || lower.ends_with("-low")
        || lower.ends_with("-extra-low")
        || lower.ends_with("-tiered")
        || lower.ends_with("-preview")
        || lower.ends_with("-agent")
        || lower.ends_with("-thinking")
        || lower.ends_with("-image")
        || lower.contains("-high-")
        || lower.contains("-medium-")
        || lower.contains("-low-")
        || lower.contains("-tiered-")
    {
        return false;
    }
    // Check if version is >= 3.6
    if let Some(idx) = lower.find("gemini-") {
        let rest = &lower[idx + "gemini-".len()..];
        let version_part: String = rest
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if let Ok(ver) = version_part.parse::<f32>() {
            return ver >= 3.6;
        }
    }
    false
}

/// Checks if model matches `gemini-3.x-flash` wildcard where x > 8 (e.g. gemini-3.9-flash, gemini-3.10-flash).
/// If matched and x > 8, routes uniformly to 3.x-flash-tiered (e.g. "gemini-3.9-flash-tiered").
/// Strict requirement: x must be > 8; 3.6 / 3.7 / 3.8 are handled by dedicated presets, 3.5 and below are excluded.
pub fn resolve_gemini_3x_flash_tiered(model: &str) -> Option<String> {
    let lower = model.to_lowercase();
    let prefix = "gemini-3.";
    let suffix = "-flash";
    if lower.starts_with(prefix) && lower.ends_with(suffix) {
        let middle = &lower[prefix.len()..lower.len() - suffix.len()];
        if let Ok(x) = middle.parse::<f32>() {
            if x > 8.0 {
                return Some(format!("gemini-3.{}-flash-tiered", middle));
            }
        }
    }
    None
}

/// Normalizes client-provided thinking effort level string (max, xhigh, high, medium, low, min, extra-low, etc.)
pub fn normalize_client_thinking_level(effort: &str) -> Option<&'static str> {
    match effort.trim().to_lowercase().as_str() {
        "low" | "extra-low" | "min" => Some("LOW"),
        "medium" => Some("MEDIUM"),
        "high" | "xhigh" | "max" | "extreme" => Some("HIGH"),
        _ => None,
    }
}

/// Resolves authoritative thought token budget based on system config, model, and request parameters:
/// - Some(budget): inject specific numeric thinkingBudget upstream
/// - None: do not inject thinkingBudget, only enable includeThoughts (adaptive mode, or configured to -1)
pub fn resolve_custom_budget(
    model: &str,
    client_effort: Option<&str>,
    client_budget: Option<u64>,
    tb_config: &crate::proxy::config::ThinkingBudgetConfig,
    token: Option<&ProxyToken>,
) -> Option<i64> {
    use crate::proxy::config::{ThinkingBudgetMode, ThinkingControlSource};

    // 1. If Gemini < 3 non-thinking model, return None (thinking not supported)
    if is_gemini_under_v3(model) {
        return None;
    }

    // 2. Control source: Client direct control mode
    if tb_config.control_source == ThinkingControlSource::Client {
        if let Some(b) = client_budget {
            if b > 0 {
                return Some(b as i64);
            }
        }
        return None;
    }

    // 3. Control source: Gateway authoritative control
    let std_id = resolve_alias(model);
    let lower = std_id.to_lowercase();
    let is_claude = lower.contains("claude");
    let is_legacy_thinking = lower.contains("gemini") && lower.contains("thinking");

    // 3.0 Legacy experimental thinking models (e.g. gemini-2.0-flash-thinking, gemini-2.0-flash-thinking-exp)
    if is_legacy_thinking {
        let max_cap = get_thinking_budget(&std_id, token) as i64;
        if tb_config.mode == ThinkingBudgetMode::Custom
            && tb_config.custom_value != 24576
            && tb_config.custom_value > 0
        {
            let custom = tb_config.custom_value as i64;
            return Some(if custom > max_cap { max_cap } else { custom });
        }
        if max_cap > 0 {
            return Some(max_cap);
        } else {
            return None;
        }
    }

    let is_pro = lower.contains("pro");
    let is_flash =
        is_tiered_flash_model(&std_id) || lower.contains("flash") || is_gemini_v3_or_above(&std_id);

    // Compatibility for single-value custom tests: applies only to bare non-tiered models without explicit tier suffix
    if tb_config.mode == ThinkingBudgetMode::Custom
        && tb_config.custom_value != 24576
        && tb_config.custom_value > 0
        && !is_tiered_flash_model(&std_id)
        && !lower.contains("-high")
        && !lower.contains("-low")
        && !lower.contains("-medium")
    {
        return Some(tb_config.custom_value as i64);
    }

    // 3.1 Claude family
    if is_claude {
        if tb_config.claude_mode == ThinkingBudgetMode::Default {
            return None;
        }
        let eff = client_effort.map(|s| s.trim().to_lowercase());
        let is_low = matches!(eff.as_deref(), Some("low") | Some("extra-low"))
            || lower.contains("-low")
            || lower.contains("haiku");
        let is_med = matches!(eff.as_deref(), Some("medium") | Some("default"))
            || lower.contains("-med")
            || lower.contains("-medium");
        let is_high = matches!(eff.as_deref(), Some("high") | Some("max") | Some("xhigh"))
            || lower.contains("-high")
            || lower.contains("-max");

        if is_low {
            if tb_config.claude_low > 0 {
                Some(tb_config.claude_low as i64)
            } else {
                None
            }
        } else if is_med {
            if tb_config.claude_medium > 0 {
                Some(tb_config.claude_medium as i64)
            } else {
                None
            }
        } else if is_high {
            if tb_config.claude_high > 0 {
                Some(tb_config.claude_high as i64)
            } else {
                None
            }
        } else {
            // For Claude thinking models without tier suffix (e.g. claude-3-7-sonnet-thinking, claude-opus-4-6-thinking):
            // Apply claude_budget (or claude_high)
            let main_budget = if tb_config.claude_budget != 0 {
                tb_config.claude_budget
            } else if tb_config.claude_high != 0 {
                tb_config.claude_high
            } else {
                16000
            };
            if main_budget > 0 {
                Some(main_budget as i64)
            } else {
                None
            }
        }
    } else if is_pro {
        // 3.2 Gemini Pro family (Google official supports Low and High tiers)
        if tb_config.pro_mode == ThinkingBudgetMode::Default {
            return None;
        }
        let eff = client_effort.map(|s| s.trim().to_lowercase());
        let is_low = lower.contains("-low")
            || lower.ends_with("-low")
            || matches!(eff.as_deref(), Some("low") | Some("extra-low"));

        if is_low {
            if tb_config.pro_low > 0 {
                Some(tb_config.pro_low as i64)
            } else {
                None
            }
        } else {
            // High tier (unspecified effort and medium map to High to ensure Pro deep reasoning)
            if tb_config.pro_high > 0 {
                Some(tb_config.pro_high as i64)
            } else {
                None
            }
        }
    } else if is_flash {
        // 3.3 Gemini Flash family
        if is_tiered_flash_model(&std_id) || lower.contains("tiered") {
            // [TIERED-THINKING] Tiered adaptive models:
            // Supports client effort parameters (low / medium / high) to switch tiers,
            // mapping to flash_low, flash_medium, flash_high in gateway configuration.
            let client_level = client_effort.and_then(normalize_client_thinking_level);
            if let Some(level) = client_level {
                match level {
                    "LOW" => {
                        if tb_config.flash_low > 0 {
                            Some(tb_config.flash_low as i64)
                        } else {
                            None
                        }
                    }
                    "HIGH" => {
                        if tb_config.flash_high > 0 {
                            Some(tb_config.flash_high as i64)
                        } else {
                            None
                        }
                    }
                    _ => {
                        // MEDIUM
                        if tb_config.flash_medium > 0 {
                            Some(tb_config.flash_medium as i64)
                        } else {
                            None
                        }
                    }
                }
            } else {
                // If client did not specify effort:
                // If Default mode or -1, use None (adaptive without budget token);
                // Otherwise if > 0, return configured tiered budget.
                if tb_config.flash_mode == ThinkingBudgetMode::Default {
                    None
                } else if tb_config.flash_tiered > 0 {
                    Some(tb_config.flash_tiered as i64)
                } else {
                    None
                }
            }
        } else {
            // [NON-TIERED FLASH] Named non-tiered models (e.g. gemini-3.7-flash-high, gemini-3.5-flash-low):
            // In gateway mode, tiers are locked by model suffix and not altered by client effort.
            if tb_config.flash_mode == ThinkingBudgetMode::Default {
                return None;
            }
            let is_high = lower.contains("-high")
                || lower.ends_with("-high")
                || lower.contains("-max")
                || lower.contains("agent");
            let is_low =
                lower.contains("-low") || lower.ends_with("-low") || lower.contains("-extra-low");

            if is_high {
                if tb_config.flash_high > 0 {
                    Some(tb_config.flash_high as i64)
                } else {
                    None
                }
            } else if is_low {
                if tb_config.flash_low > 0 {
                    Some(tb_config.flash_low as i64)
                } else {
                    None
                }
            } else {
                // Bare Flash model: adopts client effort/level if provided, otherwise defaults to balanced medium tier
                let client_level = client_effort.and_then(normalize_client_thinking_level);
                match client_level {
                    Some("HIGH") => {
                        if tb_config.flash_high > 0 {
                            Some(tb_config.flash_high as i64)
                        } else {
                            None
                        }
                    }
                    Some("LOW") => {
                        if tb_config.flash_low > 0 {
                            Some(tb_config.flash_low as i64)
                        } else {
                            None
                        }
                    }
                    _ => {
                        // Medium / default balanced tier (NONE or omitted)
                        if tb_config.flash_medium > 0 {
                            Some(tb_config.flash_medium as i64)
                        } else {
                            None
                        }
                    }
                }
            }
        }
    } else {
        // 3.4 Legacy thinking models (e.g. gemini-2.0-flash-thinking-exp)
        let b = get_thinking_budget(&std_id, token);
        if b > 0 {
            Some(b as i64)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_version_checks() {
        assert!(is_gemini_under_v3("gemini-2.5-flash"));
        assert!(is_gemini_under_v3("gemini-2.5-flash-lite"));
        assert!(is_gemini_under_v3("gemini-2.5-pro"));
        assert!(is_gemini_under_v3("gemini-2.0-flash"));
        assert!(is_gemini_under_v3("gemini-1.5-pro"));
        assert!(!is_gemini_under_v3("gemini-3.7-flash-high"));
        assert!(!is_gemini_under_v3("gemini-3-flash"));
        assert!(!is_gemini_under_v3("gemini-3-pro"));
        assert!(!is_gemini_under_v3("gemini-3.1-pro"));
        assert!(!is_gemini_under_v3("gemini-3.8-flash"));
        assert!(!is_gemini_under_v3("gemini-pro-agent"));
        assert!(!is_gemini_under_v3("claude-3-7-sonnet"));

        assert!(!is_gemini_v3_or_above("gemini-2.5-flash"));
        assert!(!is_gemini_v3_or_above("gemini-2.0-flash"));
        assert!(is_gemini_v3_or_above("gemini-3.7-flash-high"));
        assert!(is_gemini_v3_or_above("gemini-3.7-flash"));
        assert!(is_gemini_v3_or_above("gemini-3-flash"));
        assert!(is_gemini_v3_or_above("gemini-3-pro"));
        assert!(is_gemini_v3_or_above("gemini-3.1-pro"));
        assert!(is_gemini_v3_or_above("gemini-3.8-flash"));
        assert!(is_gemini_v3_or_above("gemini-pro-agent"));
        assert!(!is_gemini_v3_or_above("claude-3-7-sonnet"));
    }

    #[test]
    fn test_gemini_thinking_budget() {
        // Explicit -high suffix yields 10000 budget
        assert_eq!(get_thinking_budget("gemini-3.7-flash-high", None), 10000);
        assert_eq!(get_thinking_budget("gemini-3.8-flash-high", None), 10000);
        assert_eq!(get_thinking_budget("gemini-3.9-flash-high", None), 10000);

        // Explicit -medium suffix yields 4000 budget
        assert_eq!(get_thinking_budget("gemini-3.7-flash-medium", None), 4000);
        assert_eq!(get_thinking_budget("gemini-3.8-flash-medium", None), 4000);

        // Explicit -low suffix yields 1000 budget
        assert_eq!(get_thinking_budget("gemini-3.7-flash-low", None), 1000);
        assert_eq!(get_thinking_budget("gemini-3.8-flash-low", None), 1000);

        // Bare Gemini >= 3.0 models default to medium budget (4000)
        assert_eq!(get_thinking_budget("gemini-3.7-flash", None), 4000);
        assert_eq!(get_thinking_budget("gemini-3.8-flash", None), 4000);
        assert_eq!(get_thinking_budget("gemini-3.9-flash", None), 4000);

        // Pro family
        assert_eq!(get_thinking_budget("gemini-3.1-pro-high", None), 10001);
        assert_eq!(get_thinking_budget("gemini-pro-agent", None), 10001);
        assert_eq!(get_thinking_budget("gemini-3.1-pro-low", None), 1001);
    }

    #[test]
    fn test_resolve_authoritative_thinking_budget() {
        // 1. Explicit models: enforce dictionary budget, ignore client effort
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3.7-flash-high", Some("low"), None, None),
            10000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget(
                "gemini-3.7-flash-high",
                Some("none"),
                None,
                None
            ),
            10000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3.1-pro-low", Some("high"), None, None),
            1001
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-pro-agent", Some("low"), None, None),
            10001
        );

        // 2. Bare Flash models: adopt client effort
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("high"), None, None),
            10000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("max"), None, None),
            10000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("xhigh"), None, None),
            10000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("low"), None, None),
            1000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("extra-low"), None, None),
            1000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("medium"), None, None),
            4000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("default"), None, None),
            4000
        );

        // Bare Flash models: client omitted/disabled enforces -medium fallback (4000)
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", None, None, None),
            4000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("none"), None, None),
            4000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("disabled"), None, None),
            4000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("0"), None, None),
            4000
        );

        // 3. Bare Pro models: adopt client effort
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3.1-pro", Some("high"), None, None),
            10001
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3.1-pro", Some("low"), None, None),
            1001
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3.1-pro", Some("medium"), None, None),
            10001
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3.1-pro", None, None, None),
            10001
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3.1-pro", Some("none"), None, None),
            10001
        );

        // 4. Gemini < 3 non-thinking models: return 0
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-2.5-flash", Some("high"), None, None),
            0
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-1.5-pro", None, None, None),
            0
        );

        // 5. Bare models ignore client_budget, authoritatively governed by effort tier or server fallback
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", Some("high"), Some(1024), None),
            10000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", None, Some(1024), None),
            4000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3-flash", None, Some(32000), None),
            4000
        );
        assert_eq!(
            resolve_authoritative_thinking_budget("gemini-3.1-pro", None, Some(1000), None),
            10001
        );
    }

    #[test]
    fn test_bare_gemini_v36_or_above_flash() {
        assert!(is_bare_gemini_v36_or_above_flash("gemini-3.6-flash"));
        assert!(is_bare_gemini_v36_or_above_flash("gemini-3.7-flash"));
        assert!(is_bare_gemini_v36_or_above_flash("gemini-3.8-flash"));
        assert!(is_bare_gemini_v36_or_above_flash("gemini-3.9-flash"));
        assert!(is_bare_gemini_v36_or_above_flash("gemini-4.0-flash"));
        assert!(is_bare_gemini_v36_or_above_flash("GEMINI-3.7-FLASH"));

        // 3.5 and below does not match
        assert!(!is_bare_gemini_v36_or_above_flash("gemini-3.5-flash"));
        assert!(!is_bare_gemini_v36_or_above_flash("gemini-3-flash"));
        assert!(!is_bare_gemini_v36_or_above_flash("gemini-2.5-flash"));

        // Already having explicit suffix or variant marker does not match
        assert!(!is_bare_gemini_v36_or_above_flash("gemini-3.7-flash-high"));
        assert!(!is_bare_gemini_v36_or_above_flash(
            "gemini-3.7-flash-medium"
        ));
        assert!(!is_bare_gemini_v36_or_above_flash("gemini-3.7-flash-low"));
        assert!(!is_bare_gemini_v36_or_above_flash(
            "gemini-3.7-flash-tiered"
        ));
        assert!(!is_bare_gemini_v36_or_above_flash(
            "gemini-3.8-flash-tiered"
        ));
        assert!(!is_bare_gemini_v36_or_above_flash("gemini-3.7-pro"));
    }

    #[test]
    fn test_tiered_flash_budget_and_client_effort() {
        let mut tb = crate::proxy::config::ThinkingBudgetConfig::default();
        tb.flash_mode = crate::proxy::config::ThinkingBudgetMode::Custom;
        tb.flash_low = 1024;
        tb.flash_medium = 4096;
        tb.flash_high = 16384;
        tb.flash_tiered = -1;

        // 1. No effort parameter, and flash_tiered is -1: returns None (adaptive)
        assert_eq!(
            resolve_custom_budget("gemini-3.7-flash-tiered", None, None, &tb, None),
            None
        );

        // 2. Ignore client numeric budget; no effort still returns None
        assert_eq!(
            resolve_custom_budget("gemini-3.7-flash-tiered", None, Some(9999), &tb, None),
            None
        );

        // 3. Client effort low / medium / high mapped to flash_low, flash_medium, flash_high
        assert_eq!(
            resolve_custom_budget(
                "gemini-3.7-flash-tiered",
                Some("low"),
                Some(5000),
                &tb,
                None
            ),
            Some(1024)
        );
        assert_eq!(
            resolve_custom_budget(
                "gemini-3.7-flash-tiered",
                Some("extra-low"),
                None,
                &tb,
                None
            ),
            Some(1024)
        );
        assert_eq!(
            resolve_custom_budget("gemini-3.7-flash-tiered", Some("medium"), None, &tb, None),
            Some(4096)
        );
        assert_eq!(
            resolve_custom_budget("gemini-3.7-flash-tiered", Some("high"), None, &tb, None),
            Some(16384)
        );
        assert_eq!(
            resolve_custom_budget(
                "gemini-3.7-flash-tiered",
                Some("max"),
                Some(2048),
                &tb,
                None
            ),
            Some(16384)
        );

        // 4. Custom positive flash_tiered without client effort returns configured positive value
        tb.flash_tiered = 8192;
        assert_eq!(
            resolve_custom_budget("gemini-3.7-flash-tiered", None, None, &tb, None),
            Some(8192)
        );

        // 5. Default mode without client effort returns None (adaptive)
        tb.flash_mode = crate::proxy::config::ThinkingBudgetMode::Default;
        assert_eq!(
            resolve_custom_budget("gemini-3.7-flash-tiered", None, None, &tb, None),
            None
        );

        // 6. Named non-tiered model: locked to tier indicated by suffix, unaffected by client effort
        tb.flash_mode = crate::proxy::config::ThinkingBudgetMode::Custom;
        assert_eq!(
            resolve_custom_budget("gemini-3.7-flash-high", Some("low"), None, &tb, None),
            Some(16384) // Preserves high tier setting (16384), never downgraded by client low effort
        );
    }

    #[test]
    fn test_resolve_gemini_3x_flash_tiered_wildcard() {
        // x > 8 matches and routes to 3.x-flash-tiered
        assert_eq!(
            resolve_gemini_3x_flash_tiered("gemini-3.9-flash"),
            Some("gemini-3.9-flash-tiered".to_string())
        );
        assert_eq!(
            resolve_gemini_3x_flash_tiered("gemini-3.10-flash"),
            Some("gemini-3.10-flash-tiered".to_string())
        );
        assert_eq!(
            resolve_gemini_3x_flash_tiered("gemini-3.9.1-flash"),
            Some("gemini-3.9.1-flash-tiered".to_string())
        );

        // x <= 8 must return None, governed by dedicated mappings
        assert_eq!(resolve_gemini_3x_flash_tiered("gemini-3.8-flash"), None);
        assert_eq!(resolve_gemini_3x_flash_tiered("gemini-3.7-flash"), None);
        assert_eq!(resolve_gemini_3x_flash_tiered("gemini-3.6-flash"), None);
        assert_eq!(resolve_gemini_3x_flash_tiered("gemini-3.5-flash"), None);

        // Non-flash models or those with suffixes do not match
        assert_eq!(
            resolve_gemini_3x_flash_tiered("gemini-3.9-flash-high"),
            None
        );
        assert_eq!(resolve_gemini_3x_flash_tiered("gemini-3.9-pro"), None);
    }
}
