use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpec {
    pub max_output_tokens: Option<u64>,
    pub thinking_budget: Option<u64>,
    pub is_thinking: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SpecsConfig {
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
