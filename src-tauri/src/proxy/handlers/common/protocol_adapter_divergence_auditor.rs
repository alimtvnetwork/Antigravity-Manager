//! Protocol Adapter Divergence Auditor (`Imp-256`).
//!
//! Enforces AGENTS.md Architecture & Pipeline First invariant:
//! "Keep the pipeline strictly decoupled from specific protocols. The four protocols
//! function purely as Gemini adapters. Adapters are restricted to parameter normalization,
//! payload transformation, protocol divergence adaptation, and edge cases unsolvable
//! within the pipeline stage. The pipeline stage uniformly handles the converted Gemini
//! payloads, including thinking block backfilling, thinking budget filtering and backfilling,
//! unified context structural alignment, prefix stability, and the sanitization of risky
//! prompts and request headers."

use serde::{Deserialize, Serialize};
use std::fmt;

/// The four supported AI protocol adapters aggregating into Antigravity Gemini format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtocolAdapterKind {
    OpenAiResponses,
    OpenAiChatCompletions,
    AnthropicClaude,
    GoogleGemini,
}

/// Leaks of pipeline-stage responsibilities into protocol adapters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForbiddenPipelineLeak {
    ThinkingBudgetFilteringInAdapter { file: String, line: usize },
    PrefixStabilityEnforcementInAdapter { file: String, line: usize },
    ContextStructuralAlignmentInAdapter { file: String, line: usize },
    RequestHeaderSanitizationInAdapter { file: String, line: usize },
}

impl ForbiddenPipelineLeak {
    /// Returns the file path where the leak was detected.
    pub fn file(&self) -> &str {
        match self {
            Self::ThinkingBudgetFilteringInAdapter { file, .. } => file,
            Self::PrefixStabilityEnforcementInAdapter { file, .. } => file,
            Self::ContextStructuralAlignmentInAdapter { file, .. } => file,
            Self::RequestHeaderSanitizationInAdapter { file, .. } => file,
        }
    }

    /// Returns the 1-indexed line number where the leak was detected.
    pub fn line(&self) -> usize {
        match self {
            Self::ThinkingBudgetFilteringInAdapter { line, .. } => *line,
            Self::PrefixStabilityEnforcementInAdapter { line, .. } => *line,
            Self::ContextStructuralAlignmentInAdapter { line, .. } => *line,
            Self::RequestHeaderSanitizationInAdapter { line, .. } => *line,
        }
    }

    /// Returns the category identifier of the pipeline leak.
    pub fn leak_category(&self) -> &'static str {
        match self {
            Self::ThinkingBudgetFilteringInAdapter { .. } => "thinking_budget_filtering",
            Self::PrefixStabilityEnforcementInAdapter { .. } => "prefix_stability_enforcement",
            Self::ContextStructuralAlignmentInAdapter { .. } => "context_structural_alignment",
            Self::RequestHeaderSanitizationInAdapter { .. } => "request_header_sanitization",
        }
    }
}

impl fmt::Display for ForbiddenPipelineLeak {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ThinkingBudgetFilteringInAdapter { file, line } => {
                write!(
                    f,
                    "Thinking budget filtering leaked in adapter at {}:{}",
                    file, line
                )
            }
            Self::PrefixStabilityEnforcementInAdapter { file, line } => {
                write!(
                    f,
                    "Prefix stability enforcement leaked in adapter at {}:{}",
                    file, line
                )
            }
            Self::ContextStructuralAlignmentInAdapter { file, line } => {
                write!(
                    f,
                    "Context structural alignment leaked in adapter at {}:{}",
                    file, line
                )
            }
            Self::RequestHeaderSanitizationInAdapter { file, line } => {
                write!(
                    f,
                    "Request header sanitization leaked in adapter at {}:{}",
                    file, line
                )
            }
        }
    }
}

impl std::error::Error for ForbiddenPipelineLeak {}

/// Comprehensive report produced by auditing adapter source code for pipeline leaks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterAuditReport {
    /// Whether the adapter is strictly decoupled with zero pipeline leaks.
    pub is_decoupled: bool,
    /// Protocol adapter kind inspected.
    pub adapter_kind: ProtocolAdapterKind,
    /// List of forbidden pipeline leaks detected.
    pub leaks: Vec<ForbiddenPipelineLeak>,
    /// Total number of source lines scanned.
    pub lines_scanned: usize,
}

impl AdapterAuditReport {
    /// Returns true if any forbidden leaks were detected.
    pub fn has_leaks(&self) -> bool {
        !self.is_decoupled
    }

    /// Returns the total count of leaks detected.
    pub fn leak_count(&self) -> usize {
        self.leaks.len()
    }
}

/// Returns the canonical kebab/snake name for the specified protocol adapter.
pub fn adapter_name(adapter: ProtocolAdapterKind) -> &'static str {
    match adapter {
        ProtocolAdapterKind::OpenAiResponses => "openai_responses",
        ProtocolAdapterKind::OpenAiChatCompletions => "openai_chat_completions",
        ProtocolAdapterKind::AnthropicClaude => "anthropic_claude",
        ProtocolAdapterKind::GoogleGemini => "google_gemini",
    }
}

/// Detects if a source line contains forbidden thinking budget filtering or clamping logic.
fn detect_thinking_budget_leak(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let has_thinking_keyword = lower.contains("thinking_budget")
        || lower.contains("thinking budget")
        || lower.contains("thinkingbudget");
    let has_action_keyword = lower.contains("filter")
        || lower.contains("clamp")
        || lower.contains("strip")
        || lower.contains("backfill")
        || lower.contains("headroom");

    (has_thinking_keyword && has_action_keyword)
        || lower.contains("thinking_budget_filter")
        || lower.contains("clamp_thinking_budget")
        || lower.contains("filter_thinking_budget")
        || lower.contains("strip_thinking_budget")
        || lower.contains("backfill_thinking_budget")
        || lower.contains("thinking_budget_clamping")
        || lower.contains("budget_filtering")
}

/// Detects if a source line contains forbidden prefix stability checks or turn hashing logic.
fn detect_prefix_stability_leak(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("prefix_stability")
        || lower.contains("prefix stability")
        || lower.contains("turn_hashing")
        || lower.contains("turn hashing")
        || lower.contains("dialog_turn_hash")
        || lower.contains("turn_order_altered")
        || lower.contains("system_prompt_mutated")
        || lower.contains("enforce_prefix_stability")
        || lower.contains("validate_prefix_stability")
        || lower.contains("prefix_stability_pipeline_guard")
}

/// Detects if a source line contains forbidden context structural alignment logic.
fn detect_context_structural_alignment_leak(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("context_structural_alignment")
        || lower.contains("context structural alignment")
        || lower.contains("structural_alignment")
        || lower.contains("structural alignment")
        || lower.contains("align_context_structure")
        || lower.contains("unified_context_structural_aligner")
        || lower.contains("reorder_context_turns")
        || lower.contains("align_context_turns")
}

/// Detects if a source line contains forbidden hop-by-hop or request header sanitization logic.
fn detect_request_header_sanitization_leak(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("hop_by_hop")
        || lower.contains("hop-by-hop")
        || lower.contains("sanitize_request_headers")
        || lower.contains("request_header_sanitiz")
        || lower.contains("strip_hop_by_hop")
        || lower.contains("header_sanitization")
        || lower.contains("header sanitization")
        || lower.contains("pipeline_request_header_sanitizer")
        || lower.contains("header_allowlist_filter")
}

/// Scans a single source line and registers all detected forbidden pipeline leaks.
fn scan_line_for_leaks(
    line: &str,
    line_num: usize,
    file_path: &str,
    leaks: &mut Vec<ForbiddenPipelineLeak>,
) {
    if detect_thinking_budget_leak(line) {
        leaks.push(ForbiddenPipelineLeak::ThinkingBudgetFilteringInAdapter {
            file: file_path.to_string(),
            line: line_num,
        });
    }
    if detect_prefix_stability_leak(line) {
        leaks.push(ForbiddenPipelineLeak::PrefixStabilityEnforcementInAdapter {
            file: file_path.to_string(),
            line: line_num,
        });
    }
    if detect_context_structural_alignment_leak(line) {
        leaks.push(ForbiddenPipelineLeak::ContextStructuralAlignmentInAdapter {
            file: file_path.to_string(),
            line: line_num,
        });
    }
    if detect_request_header_sanitization_leak(line) {
        leaks.push(ForbiddenPipelineLeak::RequestHeaderSanitizationInAdapter {
            file: file_path.to_string(),
            line: line_num,
        });
    }
}

/// Audits adapter source code line by line to verify strict decoupling from pipeline responsibilities.
pub fn audit_adapter_source_code(
    source: &str,
    adapter: ProtocolAdapterKind,
    file_path: &str,
) -> AdapterAuditReport {
    let mut leaks = Vec::new();
    let mut lines_scanned = 0;

    for (idx, line) in source.lines().enumerate() {
        lines_scanned += 1;
        scan_line_for_leaks(line, idx + 1, file_path, &mut leaks);
    }

    let is_decoupled = leaks.is_empty();
    AdapterAuditReport {
        is_decoupled,
        adapter_kind: adapter,
        leaks,
        lines_scanned,
    }
}
