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

/// 获取归一化后的模型 ID (基于别名)
pub fn resolve_alias(model_id: &str) -> String {
    SPECS
        .aliases
        .get(model_id)
        .cloned()
        .unwrap_or_else(|| model_id.to_string())
}

/// 获取模型输出 Token 限额 (动态优先)
pub fn get_max_output_tokens(model_id: &str, token: Option<&ProxyToken>) -> u64 {
    let std_id = resolve_alias(model_id);

    // 1. 尝试从账号动态数据中读取
    if let Some(t) = token {
        if let Some(&limit) = t.model_limits.get(&std_id) {
            return limit;
        }
        // 如果原始 ID 没找到，尝试用归一化后的 ID 找
        if let Some(&limit) = t.model_limits.get(model_id) {
            return limit;
        }
    }

    // 2. 回退到静态 JSON
    if let Some(spec) = SPECS.models.get(&std_id) {
        if let Some(limit) = spec.max_output_tokens {
            return limit;
        }
    }

    // 3. 全局兜底
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
}
