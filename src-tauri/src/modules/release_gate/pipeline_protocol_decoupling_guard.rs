//! Pipeline Decoupling & Protocol Normalization Guard.
//!
//! Enforces architecture and pipeline decoupling invariants per AGENTS.md:
//! - **Architecture**: "This project is a gateway that aggregates four AI protocols —
//!   OpenAI Responses, OpenAI Chat Completions, Anthropic Claude, and Google Gemini —
//!   and outputs Antigravity-style Gemini protocol format."
//! - **Pipeline First**: "Keep the pipeline strictly decoupled from specific protocols.
//!   The four protocols function purely as Gemini adapters. Adapters are restricted to
//!   parameter normalization, payload transformation, protocol divergence adaptation,
//!   and edge cases unsolvable within the pipeline stage. The pipeline stage uniformly
//!   handles the converted Gemini payloads, including thinking block backfilling,
//!   thinking budget filtering and backfilling, unified context structural alignment,
//!   prefix stability, and the sanitization of risky prompts and request headers."
//! - **Backend Fix Strategy**: "Prioritize protocol-agnostic, generic fixes within the
//!   pipeline rather than localized adapter modifications. Treat adapter-level patches
//!   as a last resort only when a generic pipeline solution is infeasible or degrades
//!   compatibility."

use serde::{Deserialize, Serialize};
use std::fmt;

/// Canonical target protocol format produced by all adapters and handled by the pipeline.
pub const CANONICAL_OUTPUT_PROTOCOL: &str = "antigravity-gemini";

/// The four inbound AI protocols aggregated by Antigravity Manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiProtocol {
    /// OpenAI Responses API protocol (`/v1/responses`).
    OpenAiResponses,
    /// OpenAI Chat Completions protocol (`/v1/chat/completions`).
    OpenAiChatCompletions,
    /// Anthropic Claude Messages API protocol (`/v1/messages`).
    AnthropicClaude,
    /// Google Gemini Generative Language API protocol (`/v1beta/models/...:generateContent`).
    GoogleGemini,
}

impl AiProtocol {
    /// Returns the canonical protocol string identifier.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAiResponses => "openai_responses",
            Self::OpenAiChatCompletions => "openai_chat_completions",
            Self::AnthropicClaude => "anthropic_claude",
            Self::GoogleGemini => "google_gemini",
        }
    }

    /// Returns human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::OpenAiResponses => "OpenAI Responses",
            Self::OpenAiChatCompletions => "OpenAI Chat Completions",
            Self::AnthropicClaude => "Anthropic Claude",
            Self::GoogleGemini => "Google Gemini",
        }
    }

    /// Parses loose string representation into standard enum variant.
    pub fn from_str_loose(s: &str) -> Option<Self> {
        let normalized = s.trim().to_ascii_lowercase().replace(['-', ' '], "_");
        if normalized.contains("responses") {
            Some(Self::OpenAiResponses)
        } else if normalized.contains("openai") || normalized.contains("chat_completion") {
            Some(Self::OpenAiChatCompletions)
        } else if normalized.contains("anthropic") || normalized.contains("claude") {
            Some(Self::AnthropicClaude)
        } else if normalized.contains("gemini") || normalized.contains("google") {
            Some(Self::GoogleGemini)
        } else {
            None
        }
    }

    /// Whether the protocol is already Gemini native.
    pub fn is_gemini_native(&self) -> bool {
        matches!(self, Self::GoogleGemini)
    }

    /// Returns all four supported protocols.
    pub fn all() -> &'static [AiProtocol] {
        &[
            Self::OpenAiResponses,
            Self::OpenAiChatCompletions,
            Self::AnthropicClaude,
            Self::GoogleGemini,
        ]
    }
}

/// The architectural processing stage classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProcessingStage {
    /// Adapter stage: restricted to parameter normalization, payload transformation,
    /// protocol divergence adaptation, and unsolvable edge cases.
    AdapterNormalization,
    /// Pipeline stage: protocol-agnostic, uniformly handles thinking blocks, budgeting,
    /// structural context alignment, prefix stability, and prompt/header sanitization.
    PipelineUniformProcessing,
}

impl ProcessingStage {
    /// Returns string identifier for the stage.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AdapterNormalization => "adapter_normalization",
            Self::PipelineUniformProcessing => "pipeline_uniform_processing",
        }
    }

    /// True if this is the adapter stage.
    pub fn is_adapter(&self) -> bool {
        matches!(self, Self::AdapterNormalization)
    }

    /// True if this is the uniform pipeline stage.
    pub fn is_pipeline(&self) -> bool {
        matches!(self, Self::PipelineUniformProcessing)
    }
}

/// Permitted operations strictly confined to the AdapterNormalization stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdapterOperationType {
    /// Normalizing input parameters (temperature, top_p, max_tokens -> maxOutputTokens).
    ParameterNormalization,
    /// Translating vendor message envelopes into canonical Gemini Content/Part payloads.
    PayloadTransformation,
    /// Adapting protocol divergences (e.g. image base64 formats, stop sequence arrays).
    ProtocolDivergenceAdaptation,
    /// Isolated edge cases mathematically or structurally unsolvable within the pipeline stage.
    UnsolvableEdgeCase,
}

impl AdapterOperationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ParameterNormalization => "parameter_normalization",
            Self::PayloadTransformation => "payload_transformation",
            Self::ProtocolDivergenceAdaptation => "protocol_divergence_adaptation",
            Self::UnsolvableEdgeCase => "unsolvable_edge_case",
        }
    }

    /// Permitted in adapter normalization stage.
    pub fn is_permitted_in_adapter(&self) -> bool {
        true
    }

    /// Forbidden in uniform pipeline stage.
    pub fn is_permitted_in_pipeline(&self) -> bool {
        false
    }
}

/// Permitted operations strictly mandated for the PipelineUniformProcessing stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PipelineOperationType {
    /// Uniform backfilling of reasoning thoughts and thinking blocks in streaming/responses.
    ThinkingBlockBackfilling,
    /// Evaluating and filtering thinking budget based on target model capabilities.
    ThinkingBudgetFilteringAndBackfilling,
    /// Alternation normalization (user/model turns), adjacent role compaction, context hierarchy.
    UnifiedContextStructuralAlignment,
    /// Ensuring stable system prompt prefixes and prompt caching alignment.
    PrefixStability,
    /// Protocol-agnostic regex sanitization of risky prompts and system queries.
    RiskyPromptSanitization,
    /// Scrubbing sensitive or internal transport headers across all requests uniformly.
    RequestHeaderSanitization,
}

impl PipelineOperationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ThinkingBlockBackfilling => "thinking_block_backfilling",
            Self::ThinkingBudgetFilteringAndBackfilling => {
                "thinking_budget_filtering_and_backfilling"
            }
            Self::UnifiedContextStructuralAlignment => "unified_context_structural_alignment",
            Self::PrefixStability => "prefix_stability",
            Self::RiskyPromptSanitization => "risky_prompt_sanitization",
            Self::RequestHeaderSanitization => "request_header_sanitization",
        }
    }

    /// Permitted in pipeline uniform processing stage.
    pub fn is_permitted_in_pipeline(&self) -> bool {
        true
    }

    /// Forbidden in adapter normalization stage (violates single responsibility / DRY).
    pub fn is_permitted_in_adapter(&self) -> bool {
        false
    }
}

/// Documented justification for a protocol divergence patch in an adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolDivergenceJustification {
    pub protocol: AiProtocol,
    pub description: String,
    pub is_unsolvable_in_pipeline: bool,
    pub degrades_compatibility_if_in_pipeline: bool,
}

impl ProtocolDivergenceJustification {
    pub fn new(
        protocol: AiProtocol,
        description: impl Into<String>,
        is_unsolvable_in_pipeline: bool,
        degrades_compatibility_if_in_pipeline: bool,
    ) -> Self {
        Self {
            protocol,
            description: description.into(),
            is_unsolvable_in_pipeline,
            degrades_compatibility_if_in_pipeline,
        }
    }

    /// Whether this justification meets the strict AGENTS.md criteria for an adapter patch.
    pub fn is_valid_workaround(&self) -> bool {
        self.is_unsolvable_in_pipeline || self.degrades_compatibility_if_in_pipeline
    }
}

/// Violations identified when verifying protocol decoupling and pipeline invariants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecouplingViolation {
    /// A protocol-specific data structure, header, or dialect leaked into the uniform pipeline stage.
    ProtocolLeakInPipeline {
        protocol: AiProtocol,
        leaked_symbol: String,
        context: String,
    },
    /// An adapter performed a pipeline-stage responsibility prematurely.
    PrematurePipelineOperationInAdapter {
        protocol: AiProtocol,
        operation: PipelineOperationType,
        context: String,
    },
    /// An adapter applied a localized workaround without proving it is unsolvable in the generic pipeline.
    UnjustifiedAdapterModification {
        protocol: AiProtocol,
        description: String,
    },
    /// An adapter payload failed to normalize into canonical Gemini format.
    PayloadNotNormalizedToGemini {
        protocol: AiProtocol,
        missing_canonical_field: String,
        details: String,
    },
    /// Backend Fix Strategy violation: localized patch attempted where a generic pipeline fix is mandated.
    BackendFixStrategyViolation {
        protocol: AiProtocol,
        localized_patch: String,
        generic_pipeline_recommendation: String,
    },
    /// Non-Gemini output protocol format attempted.
    NonGeminiOutputProtocolForbidden {
        attempted_format: String,
    },
    /// Incompatible operation declared for the specified processing stage.
    InvalidStageOperationCombination {
        stage: ProcessingStage,
        operation_name: String,
        reason: String,
    },
}

impl fmt::Display for DecouplingViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProtocolLeakInPipeline {
                protocol,
                leaked_symbol,
                context,
            } => {
                write!(
                    f,
                    "Protocol leak in pipeline: symbol '{leaked_symbol}' from {} detected in uniform pipeline ({context}). Pipeline must remain strictly decoupled.",
                    protocol.display_name()
                )
            }
            Self::PrematurePipelineOperationInAdapter {
                protocol,
                operation,
                context,
            } => {
                write!(
                    f,
                    "Premature pipeline operation '{}' in adapter for {} ({context}). Adapters are restricted to parameter normalization and payload transformation; pipeline stage must handle this uniformly.",
                    operation.as_str(),
                    protocol.display_name()
                )
            }
            Self::UnjustifiedAdapterModification {
                protocol,
                description,
            } => {
                write!(
                    f,
                    "Unjustified adapter modification for {}: '{description}'. Adapter patches are a last resort only when unsolvable in pipeline or degrading compatibility.",
                    protocol.display_name()
                )
            }
            Self::PayloadNotNormalizedToGemini {
                protocol,
                missing_canonical_field,
                details,
            } => {
                write!(
                    f,
                    "Payload from {} not normalized to canonical Gemini format: missing '{missing_canonical_field}' ({details}). All adapters must output Antigravity-style Gemini format.",
                    protocol.display_name()
                )
            }
            Self::BackendFixStrategyViolation {
                protocol,
                localized_patch,
                generic_pipeline_recommendation,
            } => {
                write!(
                    f,
                    "Backend Fix Strategy violation in {} adapter ('{localized_patch}'). Prioritize generic fix: '{generic_pipeline_recommendation}'.",
                    protocol.display_name()
                )
            }
            Self::NonGeminiOutputProtocolForbidden { attempted_format } => {
                write!(
                    f,
                    "Output format '{attempted_format}' is forbidden. Antigravity Manager strictly outputs '{CANONICAL_OUTPUT_PROTOCOL}'.",
                )
            }
            Self::InvalidStageOperationCombination {
                stage,
                operation_name,
                reason,
            } => {
                write!(
                    f,
                    "Invalid stage operation combination for {}: operation '{operation_name}' is forbidden ({reason}).",
                    stage.as_str()
                )
            }
        }
    }
}

impl std::error::Error for DecouplingViolation {}

/// Compliance report summarizing decoupling verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineDecouplingReport {
    pub is_compliant: bool,
    pub stage: ProcessingStage,
    pub protocol: Option<AiProtocol>,
    pub violations: Vec<DecouplingViolation>,
    pub warnings: Vec<String>,
}

/// Known protocol-specific tokens and fields that must NEVER leak into the uniform pipeline stage.
pub const LEAKY_OPENAI_SYMBOLS: &[&str] = &[
    "messages",
    "function_call",
    "functions",
    "tool_calls[].function",
    "chat/completions",
    "openai-organization",
    "openai-project",
    "response_format.json_schema",
];

pub const LEAKY_CLAUDE_SYMBOLS: &[&str] = &[
    "anthropic-version",
    "anthropic-beta",
    "x-api-key",
    "claude-3",
    "thinking.type",
    "thinking.budget_tokens",
    "tool_use",
    "tool_result",
];

/// Validates that an output protocol identifier adheres strictly to Antigravity-style Gemini.
pub fn validate_output_protocol(output_protocol: &str) -> Result<(), DecouplingViolation> {
    let clean = output_protocol.trim().to_ascii_lowercase();
    if clean == CANONICAL_OUTPUT_PROTOCOL || clean == "gemini" || clean == "google-gemini" {
        Ok(())
    } else {
        Err(DecouplingViolation::NonGeminiOutputProtocolForbidden {
            attempted_format: output_protocol.to_string(),
        })
    }
}

/// Inspects transformed JSON payload to ensure complete normalization into Gemini target schema.
pub fn validate_normalized_gemini_json(
    json_str: &str,
    source_protocol: AiProtocol,
) -> Result<(), Vec<DecouplingViolation>> {
    let mut violations = Vec::new();

    let val: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            violations.push(DecouplingViolation::PayloadNotNormalizedToGemini {
                protocol: source_protocol,
                missing_canonical_field: "valid_json".to_string(),
                details: format!("Failed to parse JSON: {e}"),
            });
            return Err(violations);
        }
    };

    let obj = match val.as_object() {
        Some(o) => o,
        None => {
            violations.push(DecouplingViolation::PayloadNotNormalizedToGemini {
                protocol: source_protocol,
                missing_canonical_field: "root_object".to_string(),
                details: "Payload root must be a JSON object".to_string(),
            });
            return Err(violations);
        }
    };

    // Check for mandatory Gemini canonical structure
    if !obj.contains_key("contents") {
        violations.push(DecouplingViolation::PayloadNotNormalizedToGemini {
            protocol: source_protocol,
            missing_canonical_field: "contents".to_string(),
            details: "Missing canonical 'contents' array required by Gemini protocol".to_string(),
        });
    }

    // Check for leaked raw OpenAI structures
    if obj.contains_key("messages") {
        violations.push(DecouplingViolation::ProtocolLeakInPipeline {
            protocol: AiProtocol::OpenAiChatCompletions,
            leaked_symbol: "messages".to_string(),
            context: "Found raw OpenAI 'messages' array at root instead of Gemini 'contents'"
                .to_string(),
        });
    }
    if obj.contains_key("functions") || obj.contains_key("function_call") {
        violations.push(DecouplingViolation::ProtocolLeakInPipeline {
            protocol: AiProtocol::OpenAiChatCompletions,
            leaked_symbol: "functions".to_string(),
            context: "Found raw legacy OpenAI function calling fields instead of Gemini 'tools'"
                .to_string(),
        });
    }

    // Check for leaked raw Claude structures
    if obj.contains_key("anthropic_version") || obj.contains_key("anthropic-version") {
        violations.push(DecouplingViolation::ProtocolLeakInPipeline {
            protocol: AiProtocol::AnthropicClaude,
            leaked_symbol: "anthropic-version".to_string(),
            context: "Found Claude metadata header/key inside transformed payload body".to_string(),
        });
    }

    // Verify 'contents' elements have 'role' and 'parts'
    if let Some(contents) = obj.get("contents").and_then(|c| c.as_array()) {
        for (idx, item) in contents.iter().enumerate() {
            if let Some(item_obj) = item.as_object() {
                if !item_obj.contains_key("role") {
                    violations.push(DecouplingViolation::PayloadNotNormalizedToGemini {
                        protocol: source_protocol,
                        missing_canonical_field: format!("contents[{idx}].role"),
                        details: "Gemini contents turn must have a 'role' ('user' or 'model')"
                            .to_string(),
                    });
                }
                if !item_obj.contains_key("parts") {
                    violations.push(DecouplingViolation::PayloadNotNormalizedToGemini {
                        protocol: source_protocol,
                        missing_canonical_field: format!("contents[{idx}].parts"),
                        details: "Gemini contents turn must have a 'parts' array".to_string(),
                    });
                }
            } else {
                violations.push(DecouplingViolation::PayloadNotNormalizedToGemini {
                    protocol: source_protocol,
                    missing_canonical_field: format!("contents[{idx}]"),
                    details: "Each contents item must be a JSON object".to_string(),
                });
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Evaluates declared architectural operations and justifications across stages.
pub fn validate_architectural_barrier(
    stage: ProcessingStage,
    protocol: Option<AiProtocol>,
    declared_operations: &[&str],
    justifications: &[ProtocolDivergenceJustification],
) -> PipelineDecouplingReport {
    let mut violations = Vec::new();
    let mut warnings = Vec::new();

    match stage {
        ProcessingStage::AdapterNormalization => {
            let proto = protocol.unwrap_or(AiProtocol::GoogleGemini);
            for op in declared_operations {
                let op_clean = op.trim().to_ascii_lowercase();

                // Check for premature pipeline operations in adapter
                if op_clean.contains("thinking_block_backfill")
                    || op_clean.contains("backfill_thinking")
                {
                    violations.push(DecouplingViolation::PrematurePipelineOperationInAdapter {
                        protocol: proto,
                        operation: PipelineOperationType::ThinkingBlockBackfilling,
                        context: format!("Operation '{op}' must be executed uniformly in pipeline"),
                    });
                }
                if op_clean.contains("thinking_budget") || op_clean.contains("filter_budget") {
                    violations.push(DecouplingViolation::PrematurePipelineOperationInAdapter {
                        protocol: proto,
                        operation: PipelineOperationType::ThinkingBudgetFilteringAndBackfilling,
                        context: format!("Operation '{op}' must be executed uniformly in pipeline"),
                    });
                }
                if op_clean.contains("context_alignment") || op_clean.contains("role_alternation") {
                    violations.push(DecouplingViolation::PrematurePipelineOperationInAdapter {
                        protocol: proto,
                        operation: PipelineOperationType::UnifiedContextStructuralAlignment,
                        context: format!("Operation '{op}' must be executed uniformly in pipeline"),
                    });
                }
                if op_clean.contains("prefix_stability") {
                    violations.push(DecouplingViolation::PrematurePipelineOperationInAdapter {
                        protocol: proto,
                        operation: PipelineOperationType::PrefixStability,
                        context: format!("Operation '{op}' must be executed uniformly in pipeline"),
                    });
                }
                if op_clean.contains("prompt_sanitiz") || op_clean.contains("sanitize_prompt") {
                    violations.push(DecouplingViolation::PrematurePipelineOperationInAdapter {
                        protocol: proto,
                        operation: PipelineOperationType::RiskyPromptSanitization,
                        context: format!("Operation '{op}' must be executed uniformly in pipeline"),
                    });
                }

                // Check for adapter modifications without valid justification
                if op_clean.contains("adapter_patch") || op_clean.contains("localized_fix") {
                    let has_valid_justification = justifications
                        .iter()
                        .any(|j| j.protocol == proto && j.is_valid_workaround());
                    if !has_valid_justification {
                        violations.push(DecouplingViolation::UnjustifiedAdapterModification {
                            protocol: proto,
                            description: format!(
                                "Localized patch '{op}' lacks valid justification proving unsolvability in generic pipeline"
                            ),
                        });
                    }
                }
            }
        }
        ProcessingStage::PipelineUniformProcessing => {
            for op in declared_operations {
                let op_clean = op.trim().to_ascii_lowercase();

                // Pipeline stage must never execute protocol-specific operations
                for proto in AiProtocol::all() {
                    let tag = proto.as_str();
                    if op_clean.contains(tag) && !op_clean.contains("gemini") {
                        violations.push(DecouplingViolation::ProtocolLeakInPipeline {
                            protocol: *proto,
                            leaked_symbol: op.to_string(),
                            context: format!(
                                "Pipeline stage contains protocol-specific logic for {}",
                                proto.display_name()
                            ),
                        });
                    }
                }

                // Warn if pipeline lacks prefix stability or context alignment
                if op_clean.contains("unaligned_context") {
                    warnings.push("Context structural alignment appears bypassed".to_string());
                }
            }
        }
    }

    let is_compliant = violations.is_empty();
    PipelineDecouplingReport {
        is_compliant,
        stage,
        protocol,
        violations,
        warnings,
    }
}

/// Static code linter that inspects pipeline source files for protocol leakage.
pub fn lint_pipeline_source_code(source: &str, file_name: &str) -> Vec<DecouplingViolation> {
    let mut violations = Vec::new();

    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        let line_num = line_idx + 1;

        // Skip comments
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }

        // Check for OpenAI protocol leaks in pipeline core
        if trimmed.contains("x-openai-")
            || trimmed.contains("openai_organization")
            || trimmed.contains("role == \"function\"")
            || trimmed.contains("function_call")
        {
            violations.push(DecouplingViolation::ProtocolLeakInPipeline {
                protocol: AiProtocol::OpenAiChatCompletions,
                leaked_symbol: trimmed.to_string(),
                context: format!("{file_name}:{line_num}"),
            });
        }

        // Check for Claude protocol leaks in pipeline core
        if trimmed.contains("anthropic-version")
            || trimmed.contains("anthropic_beta")
            || trimmed.contains("type == \"tool_use\"")
            || trimmed.contains("type == \"tool_result\"")
        {
            violations.push(DecouplingViolation::ProtocolLeakInPipeline {
                protocol: AiProtocol::AnthropicClaude,
                leaked_symbol: trimmed.to_string(),
                context: format!("{file_name}:{line_num}"),
            });
        }
    }

    violations
}

/// Static code linter that inspects adapter source files for premature pipeline handling.
pub fn lint_adapter_source_code(
    source: &str,
    file_name: &str,
    protocol: AiProtocol,
) -> Vec<DecouplingViolation> {
    let mut violations = Vec::new();

    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        let line_num = line_idx + 1;

        // Skip comments
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }

        // Adapters must not backfill thinking blocks
        if trimmed.contains("backfill_thinking_block") || trimmed.contains("populate_thought_parts")
        {
            violations.push(DecouplingViolation::PrematurePipelineOperationInAdapter {
                protocol,
                operation: PipelineOperationType::ThinkingBlockBackfilling,
                context: format!("{file_name}:{line_num} ('{trimmed}')"),
            });
        }

        // Adapters must not calculate or filter thinking budgets
        if trimmed.contains("filter_thinking_budget")
            || trimmed.contains("calculate_thinking_budget")
        {
            violations.push(DecouplingViolation::PrematurePipelineOperationInAdapter {
                protocol,
                operation: PipelineOperationType::ThinkingBudgetFilteringAndBackfilling,
                context: format!("{file_name}:{line_num} ('{trimmed}')"),
            });
        }

        // Adapters must not do localized prompt sanitization
        if trimmed.contains("sanitize_risky_prompts")
            || trimmed.contains("strip_pipeline_prompt_headers")
        {
            violations.push(DecouplingViolation::PrematurePipelineOperationInAdapter {
                protocol,
                operation: PipelineOperationType::RiskyPromptSanitization,
                context: format!("{file_name}:{line_num} ('{trimmed}')"),
            });
        }
    }

    violations
}
