// Variant -> real model resolution (split from variant_mapping.rs)
use super::specs::{
    GEMINI_FAMILIES, SPEC_31_FLASH_LITE, SPEC_CLAUDE_OPUS_46, SPEC_CLAUDE_SONNET_46,
    SPEC_GPT_OSS_120B,
};
use super::types::{AliasPolicy, RealModelSpec, VariantTier};

/// Infer the variant tier from the client-sent `thinking.budget_tokens`.
///
/// OpenCode derives budgetTokens from its model capabilities (max = budget-1,
/// high = floor(budget/2)), so the exact value is not the real budget —
/// we only use its magnitude to guess which tier the user selected. When no budget

pub fn infer_tier(budget_tokens: Option<u32>) -> VariantTier {
    match budget_tokens {
        // 1024 is the Anthropic Claude SDK default minimum; do not misinterpret as Low to avoid truncation
        Some(1024) => VariantTier::High,
        Some(b) if b < 2000 => VariantTier::Low,
        Some(b) if b < 7000 => VariantTier::Medium,
        _ => VariantTier::High,
    }
}

/// Parse a supported Anthropic SDK output effort into a Gemini variant tier.
pub fn tier_from_effort(effort: Option<&str>) -> Option<VariantTier> {
    match effort.map(|s| s.trim().to_lowercase()).as_deref() {
        Some("low") | Some("extra-low") => Some(VariantTier::Low),
        Some("medium") | Some("default") => Some(VariantTier::Medium),
        Some("high") | Some("max") | Some("xhigh") => Some(VariantTier::High),
        _ => None,
    }
}

/// Resolve a canonical model + tier to the real model + params.
///
/// Returns None for models that have no variant split (use
/// [`resolve_non_variant_model`] for those).
pub fn resolve_real_model(canonical: &str, tier: VariantTier) -> Option<RealModelSpec> {
    // Normalize: lowercase, trim a trailing variant-like suffix the client may have appended.
    let key = canonical.to_lowercase();
    // Defensive guard: models ending with -high / -medium / -low have strict priority and never downgrade
    let forced_tier = if key.ends_with("-high") {
        Some(VariantTier::High)
    } else if key.ends_with("-medium") {
        Some(VariantTier::Medium)
    } else if key.ends_with("-low") || key.ends_with("-extra-low") {
        Some(VariantTier::Low)
    } else {
        None
    };

    for family in GEMINI_FAMILIES {
        let is_canonical = family.canonical_id == key.as_str();
        let alias_match = family
            .aliases
            .iter()
            .find(|(alias, _)| *alias == key.as_str());

        if !is_canonical && alias_match.is_none() {
            continue;
        }

        let resolved_tier = if let Some(ft) = forced_tier {
            ft
        } else if is_canonical {
            tier
        } else if let Some((_, policy)) = alias_match {
            match *policy {
                AliasPolicy::HonorTier => tier,
                AliasPolicy::Fixed(fixed_tier) => fixed_tier,
            }
        } else {
            continue;
        };

        let mut spec = family
            .tiers
            .iter()
            .find(|(candidate_tier, _)| *candidate_tier == resolved_tier)
            .map(|(_, spec)| *spec)?;

        if is_canonical {
            if family.canonical_id == "gemini-3.7-flash" {
                spec.id = "gemini-3.7-flash-tiered";
            }
        }

        return Some(spec);
    }

    None
}

/// Resolve a canonical model that has NO variant split — use real ID + params directly.
///
/// Covers models the user may configure without variants (flash-lite, claude, etc.).
/// Also accepts the real IDs themselves (idempotent passthrough).
pub fn resolve_non_variant_model(model: &str) -> Option<RealModelSpec> {
    let key = model.to_lowercase();
    // gemini-3.1-flash-lite: checkpoint-only model, no thinking.
    if matches!(
        key.as_str(),
        "gemini-3.1-flash-lite"
            | "gemini-2.5-flash-lite"
            | "gemini-2.5-flash"
            | "gemini-2.5-flash-thinking"
    ) {
        return Some(SPEC_31_FLASH_LITE);
    }
    if key == "claude-sonnet-4-6" {
        return Some(SPEC_CLAUDE_SONNET_46);
    }
    if matches!(key.as_str(), "claude-opus-4-6-thinking" | "claude-opus-4-6") {
        return Some(SPEC_CLAUDE_OPUS_46);
    }
    if key == "gpt-oss-120b-medium" {
        return Some(SPEC_GPT_OSS_120B);
    }
    None
}

/// Resolve a model with an optional explicit variant tier.
///
/// An explicit tier takes precedence over the client thinking budget.
pub fn resolve_with_tier(
    canonical: &str,
    explicit_tier: Option<VariantTier>,
    budget_tokens: Option<u32>,
) -> Option<RealModelSpec> {
    // 1. Match standalone models without variants first
    if let Some(spec) = resolve_non_variant_model(canonical) {
        return Some(spec);
    }

    let lower = canonical.to_lowercase();
    let name_tier = if lower.ends_with("-high") {
        Some(VariantTier::High)
    } else if lower.ends_with("-medium") {
        Some(VariantTier::Medium)
    } else if lower.ends_with("-low") || lower.ends_with("-extra-low") {
        Some(VariantTier::Low)
    } else {
        None
    };

    let is_v3 = crate::proxy::model_specs::is_gemini_v3_or_above(canonical);

    // Explicit model tier suffix (-high, -medium, -low) has strict highest priority;
    // followed by explicitly passed effort tier;
    // [USER RULE] For bare Gemini >= 3 models, client effort takes precedence; budget_tokens is ignored
    // When effort is omitted, default to Medium (Flash 4000, Pro 10001) rather than inferring from budget
    // Legacy models without explicit tier infer from budget if present, else default to Medium
    let tier = if let Some(nt) = name_tier {
        nt
    } else if let Some(et) = explicit_tier {
        et
    } else if is_v3 {
        VariantTier::Medium
    } else if let Some(bt) = budget_tokens {
        infer_tier(Some(bt))
    } else {
        VariantTier::Medium
    };

    // 2. Match from known family registry
    if let Some(spec) = resolve_real_model(canonical, tier) {
        return Some(spec);
    }

    // 3. Universal dynamic resolution:
    // For any Gemini >= 3.0 derivative (gemini-3.8-flash, gemini-3.9-flash, etc.),
    // dynamically generate RealModelSpec based on tier,
    // ensuring thinking mode is enabled with calibrated budget and never downgrades
    let is_gemini_3_family =
        is_v3 && (lower.contains("flash") || lower.contains("pro") || lower.contains("agent"));
    if is_gemini_3_family {
        // [NEW] 如果是 >= 3.6 的无后缀 Flash 衍生模型，统一预设路由为 tiered 真实模型 ID，彻底杜绝上游 429
        let resolved_id = if crate::proxy::model_specs::is_bare_gemini_v36_or_above_flash(canonical)
        {
            format!("{}-tiered", canonical)
        } else {
            canonical.to_string()
        };

        let dynamic_tier = if let Some(nt) = name_tier {
            nt
        } else if let Some(et) = explicit_tier {
            et
        } else if lower.contains("high") || lower.contains("agent") || lower.contains("pro") {
            VariantTier::High
        } else if lower.contains("low") {
            VariantTier::Low
        } else {
            // medium or bare suffix defaults to 4000 (Medium specification)
            VariantTier::Medium
        };
        let budget = match dynamic_tier {
            VariantTier::High => {
                if lower.contains("pro") {
                    10001
                } else {
                    10000
                }
            }
            VariantTier::Medium => {
                if lower.contains("pro") {
                    10001
                } else {
                    4000
                }
            }
            VariantTier::Low => {
                if lower.contains("pro") {
                    1001
                } else {
                    1000
                }
            }
        };
        let max_output_tokens = if lower.contains("pro") { 65535 } else { 65536 };
        let id: &'static str = Box::leak(resolved_id.into_boxed_str());
        return Some(RealModelSpec {
            id,
            thinking_budget: budget,
            max_output_tokens,
            include_thoughts: true,
            preserve_client_budget: false,
        });
    }

    None
}

/// Top-level compatibility resolver using the client thinking budget.
pub fn resolve(canonical: &str, budget_tokens: Option<u32>) -> Option<RealModelSpec> {
    resolve_with_tier(canonical, None, budget_tokens)
}
