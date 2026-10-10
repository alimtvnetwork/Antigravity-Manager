use crate::proxy::common::variant_mapping::GEMINI_FAMILIES;

/// Variant type for model variants
#[derive(Debug, Clone, Copy)]
pub(crate) enum VariantType {
    /// Claude-style thinking with budget_tokens
    ClaudeThinking,
    /// Gemini 3 Pro style with thinking budgets
    Gemini3Pro,
    /// Gemini 3 Flash style with thinking budgets
    Gemini3Flash,
    /// Gemini 2.5 thinking style
    Gemini25Thinking,
}

/// Model definition with metadata and variants
#[derive(Debug, Clone)]
pub(crate) struct ModelDef {
    pub(crate) id: &'static str,
    pub(crate) name: &'static str,
    pub(crate) context_limit: u32,
    pub(crate) output_limit: u32,
    pub(crate) input_modalities: &'static [&'static str],
    pub(crate) output_modalities: &'static [&'static str],
    pub(crate) reasoning: bool,
    pub(crate) variant_type: Option<VariantType>,
}

/// Build the complete model catalog for antigravity-manager provider
pub(crate) fn build_model_catalog() -> Vec<ModelDef> {
    let mut catalog = vec![
        // Claude models
        ModelDef {
            id: "claude-sonnet-4-6",
            name: "Claude Sonnet 4.6",
            context_limit: 200_000,
            output_limit: 64_000,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: Some(VariantType::ClaudeThinking),
        },
        ModelDef {
            id: "claude-sonnet-4-6-thinking",
            name: "Claude Sonnet 4.6 Thinking",
            context_limit: 200_000,
            output_limit: 64_000,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: Some(VariantType::ClaudeThinking),
        },
        ModelDef {
            id: "claude-sonnet-4-5",
            name: "Claude Sonnet 4.5",
            context_limit: 200_000,
            output_limit: 64_000,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: Some(VariantType::ClaudeThinking),
        },
        ModelDef {
            id: "claude-sonnet-4-5-thinking",
            name: "Claude Sonnet 4.5 Thinking",
            context_limit: 200_000,
            output_limit: 64_000,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: Some(VariantType::ClaudeThinking),
        },
        ModelDef {
            id: "claude-opus-4-5",
            name: "Claude Opus 4.5",
            context_limit: 200_000,
            output_limit: 64_000,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: Some(VariantType::ClaudeThinking),
        },
        ModelDef {
            id: "claude-opus-4-5-thinking",
            name: "Claude Opus 4.5 Thinking",
            context_limit: 200_000,
            output_limit: 64_000,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: Some(VariantType::ClaudeThinking),
        },
        ModelDef {
            id: "claude-opus-4-6",
            name: "Claude Opus 4.6",
            context_limit: 200_000,
            output_limit: 64_000,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: Some(VariantType::ClaudeThinking),
        },
        ModelDef {
            id: "claude-opus-4-6-thinking",
            name: "Claude Opus 4.6 Thinking",
            context_limit: 200_000,
            output_limit: 64_000,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: Some(VariantType::ClaudeThinking),
        },
    ];

    catalog.extend(GEMINI_FAMILIES.iter().map(|family| ModelDef {
        id: family.canonical_id,
        name: family.display_name,
        context_limit: family.context_limit,
        output_limit: family.output_limit,
        input_modalities: family.input_modalities,
        output_modalities: family.output_modalities,
        reasoning: family.reasoning,
        variant_type: match family.canonical_id {
            "gemini-3.1-pro" => Some(VariantType::Gemini3Pro),
            "gemini-3.7-flash" | "gemini-3.5-flash" => Some(VariantType::Gemini3Flash),
            _ => None,
        },
    }));

    catalog.extend([
        ModelDef {
            id: "gemini-3.1-flash-lite",
            name: "Gemini 3.1 Flash Lite",
            context_limit: 1_048_576,
            output_limit: 65_536,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: None,
        },
        ModelDef {
            id: "gemini-3-pro-image",
            name: "Gemini 3 Pro Image",
            context_limit: 1_048_576,
            output_limit: 65_535,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text", "image"],
            reasoning: false,
            variant_type: None,
        },
        // Gemini 2.5 models
        ModelDef {
            id: "gemini-2.5-flash",
            name: "Gemini 2.5 Flash",
            context_limit: 1_048_576,
            output_limit: 65_536,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: false,
            variant_type: None,
        },
        ModelDef {
            id: "gemini-2.5-flash-lite",
            name: "Gemini 2.5 Flash Lite",
            context_limit: 1_048_576,
            output_limit: 65_536,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: false,
            variant_type: None,
        },
        ModelDef {
            id: "gemini-2.5-flash-thinking",
            name: "Gemini 2.5 Flash Thinking",
            context_limit: 1_048_576,
            output_limit: 65_536,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: Some(VariantType::Gemini25Thinking),
        },
        ModelDef {
            id: "gemini-2.5-pro",
            name: "Gemini 2.5 Pro",
            context_limit: 1_048_576,
            output_limit: 65_536,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: None,
        },
    ]);

    catalog
}

/// Normalize OpenCode base URL to ensure it ends with `/v1` (Anthropic protocol requirement)
/// - Trims trailing `/`
/// - If already ends with `/v1`, keeps it as-is
/// - Otherwise appends `/v1`
pub(crate) fn normalize_opencode_base_url(input: &str) -> String {
    let trimmed = input.trim().trim_end_matches('/');
    if trimmed.ends_with("/v1") {
        trimmed.to_string()
    } else {
        format!("{}/v1", trimmed)
    }
}
