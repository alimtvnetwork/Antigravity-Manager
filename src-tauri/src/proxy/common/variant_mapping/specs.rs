// Verified real model specs (split from variant_mapping.rs)
use super::types::RealModelSpec;

// ── verified real model specs (from upstream spec) ──
// gemini-3.7-flash family (maxOutputTokens = 65536)
pub(crate) const SPEC_37_FLASH_LOW: RealModelSpec = RealModelSpec {
    id: "gemini-3.7-flash-low",
    thinking_budget: 1000,
    max_output_tokens: 65536,
    include_thoughts: true,
    preserve_client_budget: false,
};
pub(crate) const SPEC_37_FLASH_MEDIUM: RealModelSpec = RealModelSpec {
    id: "gemini-3.7-flash-medium",
    thinking_budget: 4000,
    max_output_tokens: 65536,
    include_thoughts: true,
    preserve_client_budget: false,
};
pub(crate) const SPEC_37_FLASH_HIGH: RealModelSpec = RealModelSpec {
    id: "gemini-3.7-flash-high",
    thinking_budget: 10000,
    max_output_tokens: 65536,
    include_thoughts: true,
    preserve_client_budget: false,
};

// gemini-3.5-flash family (maxOutputTokens = 65536)
pub(crate) const SPEC_35_FLASH_EXTRA_LOW: RealModelSpec = RealModelSpec {
    id: "gemini-3.5-flash-extra-low",
    thinking_budget: 1000,
    max_output_tokens: 65536,
    include_thoughts: true,
    preserve_client_budget: false,
};
pub(crate) const SPEC_35_FLASH_LOW: RealModelSpec = RealModelSpec {
    id: "gemini-3.5-flash-low",
    thinking_budget: 4000,
    max_output_tokens: 65536,
    include_thoughts: true,
    preserve_client_budget: false,
};
pub(crate) const SPEC_3_FLASH_AGENT: RealModelSpec = RealModelSpec {
    id: "gemini-3-flash-agent",
    thinking_budget: 10000,
    max_output_tokens: 65536,
    include_thoughts: true,
    preserve_client_budget: false,
};

// gemini-3.1-pro family (maxOutputTokens = 65535 — note the off-by-one vs Flash)
pub(crate) const SPEC_31_PRO_LOW: RealModelSpec = RealModelSpec {
    id: "gemini-3.1-pro-low",
    thinking_budget: 1001,
    max_output_tokens: 65535,
    include_thoughts: true,
    preserve_client_budget: false,
};
pub(crate) const SPEC_PRO_AGENT: RealModelSpec = RealModelSpec {
    id: "gemini-pro-agent",
    thinking_budget: 10001,
    max_output_tokens: 65535,
    include_thoughts: true,
    preserve_client_budget: false,
};

// Non-variant models
pub(crate) const SPEC_31_FLASH_LITE: RealModelSpec = RealModelSpec {
    id: "gemini-3.1-flash-lite",
    thinking_budget: 0,
    max_output_tokens: 16384,
    include_thoughts: false,
    preserve_client_budget: false,
};
pub(crate) const SPEC_CLAUDE_SONNET_46: RealModelSpec = RealModelSpec {
    id: "claude-sonnet-4-6",
    thinking_budget: 1024,
    max_output_tokens: 64000,
    include_thoughts: true,
    preserve_client_budget: true,
};
pub(crate) const SPEC_CLAUDE_OPUS_46: RealModelSpec = RealModelSpec {
    id: "claude-opus-4-6-thinking",
    thinking_budget: 1024,
    max_output_tokens: 64000,
    include_thoughts: true,
    preserve_client_budget: true,
};
pub(crate) const SPEC_GPT_OSS_120B: RealModelSpec = RealModelSpec {
    id: "gpt-oss-120b-medium",
    thinking_budget: 8192,
    max_output_tokens: 32768,
    include_thoughts: true,
    preserve_client_budget: false,
};

pub static GEMINI_FAMILIES: &[CanonicalFamily] = &[
    CanonicalFamily {
        canonical_id: "gemini-3.7-flash",
        display_name: "Gemini 3.7 Flash",
        context_limit: 1_000_000,
        output_limit: 65_536,
        input_modalities: &["text", "image", "audio", "video", "pdf"],
        output_modalities: &["text"],
        reasoning: true,
        tiers: &[
            (VariantTier::Low, SPEC_37_FLASH_LOW),
            (VariantTier::Medium, SPEC_37_FLASH_MEDIUM),
            (VariantTier::High, SPEC_37_FLASH_HIGH),
        ],
        aliases: &[
            ("gemini-3.7-flash-high", AliasPolicy::HonorTier),
            (
                "gemini-3.7-flash-medium",
                AliasPolicy::Fixed(VariantTier::Medium),
            ),
            ("gemini-3.7-flash-low", AliasPolicy::Fixed(VariantTier::Low)),
            ("gemini-3.7-flash-tiered", AliasPolicy::HonorTier),
            ("gemini-3.6-flash-high", AliasPolicy::HonorTier),
            (
                "gemini-3.6-flash-medium",
                AliasPolicy::Fixed(VariantTier::Medium),
            ),
            ("gemini-3.6-flash-low", AliasPolicy::Fixed(VariantTier::Low)),
            ("gemini-3.6-flash", AliasPolicy::HonorTier),
            ("gemini-3.6-flash-tiered", AliasPolicy::HonorTier),
        ],
    },
    CanonicalFamily {
        canonical_id: "gemini-3.5-flash",
        display_name: "Gemini 3.5 Flash",
        context_limit: 1_000_000,
        output_limit: 65_536,
        input_modalities: &["text", "image", "audio", "video", "pdf"],
        output_modalities: &["text"],
        reasoning: true,
        tiers: &[
            (VariantTier::Low, SPEC_35_FLASH_EXTRA_LOW),
            (VariantTier::Medium, SPEC_35_FLASH_LOW),
            (VariantTier::High, SPEC_3_FLASH_AGENT),
        ],
        aliases: &[
            (
                "gemini-3.5-flash-high",
                AliasPolicy::Fixed(VariantTier::High),
            ),
            (
                "gemini-3.5-flash-medium",
                AliasPolicy::Fixed(VariantTier::Medium),
            ),
            ("gemini-3.5-flash-low", AliasPolicy::Fixed(VariantTier::Low)),
            ("gemini-3-flash", AliasPolicy::HonorTier),
        ],
    },
    CanonicalFamily {
        canonical_id: "gemini-3.1-pro",
        display_name: "Gemini 3.1 Pro",
        context_limit: 1_048_576,
        output_limit: 65_535,
        input_modalities: &["text", "image", "audio", "video", "pdf"],
        output_modalities: &["text"],
        reasoning: true,
        tiers: &[
            (VariantTier::Low, SPEC_31_PRO_LOW),
            (VariantTier::Medium, SPEC_PRO_AGENT),
            (VariantTier::High, SPEC_PRO_AGENT),
        ],
        aliases: &[
            ("gemini-3.1-pro-high", AliasPolicy::Fixed(VariantTier::High)),
            ("gemini-pro", AliasPolicy::HonorTier),
            ("gemini-3.1-pro-low", AliasPolicy::Fixed(VariantTier::Low)),
        ],
    },
];
