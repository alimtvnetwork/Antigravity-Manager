use super::*;

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
            return Some(b as i64);
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
