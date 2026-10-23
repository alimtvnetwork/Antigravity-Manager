//! Wildcard Model Routing & Regex Sanitization Guard.
//!
//! Enforces code quality and future-proofing invariants per AGENTS.md:
//! - "Prioritize root-cause, future-proof fixes rather than hardcoded logic, dead code, or speculative changes."
//! - "Model Routing Example: Prioritize wildcard patterns (e.g., `gemini-*-flash-*`, `claude-3-5-*`, `gpt-4o-*`)
//!   to anticipate future model releases rather than exact string matches."
//! - "Prompt Sanitization Example: For agent-client prompt sanitization, prioritize regex-based pattern matching
//!   over static keyword replacement, ensuring full coverage without stripping pipeline system prompts or user queries."

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Type of routing pattern used to match model names to protocol adapters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingPatternType {
    /// Recommended: Wildcard glob pattern (e.g. `gemini-*-flash-*`, `claude-3-5-*`).
    Wildcard(String),
    /// Regular expression pattern.
    Regex(String),
    /// Forbidden in routing specifications: Brittle exact string comparison.
    BrittleExact(String),
}

/// Recognized AI model families aggregated by Antigravity-Manager.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelFamily {
    /// Google Gemini protocol family (e.g., `gemini-1.5-pro`, `gemini-2.0-flash-exp`).
    GoogleGemini,
    /// Anthropic Claude protocol family (e.g., `claude-3-5-sonnet`, `claude-3-opus`).
    AnthropicClaude,
    /// OpenAI Chat Completions protocol family (e.g., `gpt-4o`, `o1-preview`, `o3-mini`).
    OpenAiChat,
    /// OpenAI Responses protocol family (e.g., Codex CLI responses endpoint).
    OpenAiResponses,
    /// DeepSeek open/hosted models (e.g., `deepseek-chat`, `deepseek-reasoner`).
    DeepSeek,
    /// Custom third-party model family.
    Custom(String),
    /// Unknown or unclassified model.
    Unknown,
}

impl ModelFamily {
    /// Returns the canonical slug for the model family.
    pub fn as_str(&self) -> &str {
        match self {
            Self::GoogleGemini => "google_gemini",
            Self::AnthropicClaude => "anthropic_claude",
            Self::OpenAiChat => "openai_chat",
            Self::OpenAiResponses => "openai_responses",
            Self::DeepSeek => "deepseek",
            Self::Custom(name) => name.as_str(),
            Self::Unknown => "unknown",
        }
    }

    /// Returns the default protocol adapter identifier for this family.
    pub fn default_protocol_adapter(&self) -> &'static str {
        match self {
            Self::GoogleGemini => "google_gemini_adapter",
            Self::AnthropicClaude => "anthropic_claude_adapter",
            Self::OpenAiChat => "openai_chat_adapter",
            Self::OpenAiResponses => "openai_responses_adapter",
            Self::DeepSeek => "openai_chat_adapter",
            Self::Custom(_) => "custom_adapter",
            Self::Unknown => "default_adapter",
        }
    }

    /// Returns true if this family is supported by default adapters.
    pub fn is_supported(&self) -> bool {
        !matches!(self, Self::Unknown)
    }
}

impl fmt::Display for ModelFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Detects the model family from a candidate model name using future-proof pattern recognition.
pub fn detect_model_family(model_name: &str) -> ModelFamily {
    let lower = model_name.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return ModelFamily::Unknown;
    }

    // Google Gemini family patterns
    if match_wildcard("gemini-*", &lower) || match_wildcard("*gemini*", &lower) {
        return ModelFamily::GoogleGemini;
    }

    // Anthropic Claude family patterns
    if match_wildcard("claude-*", &lower) || match_wildcard("*claude*", &lower) {
        return ModelFamily::AnthropicClaude;
    }

    // OpenAI Responses / Codex specific patterns
    if match_wildcard("codex-*", &lower)
        || match_wildcard("*-responses", &lower)
        || match_wildcard("responses-*", &lower)
    {
        return ModelFamily::OpenAiResponses;
    }

    // OpenAI Chat Completions family patterns (gpt-*, o1-*, o3-*, chatgpt-*, text-embedding-*, dall-e-*)
    if match_wildcard("gpt-*", &lower)
        || match_wildcard("o1-*", &lower)
        || match_wildcard("o3-*", &lower)
        || match_wildcard("o4-*", &lower)
        || match_wildcard("chatgpt-*", &lower)
        || match_wildcard("text-embedding-*", &lower)
        || match_wildcard("dall-e-*", &lower)
        || match_wildcard("*gpt*", &lower)
    {
        return ModelFamily::OpenAiChat;
    }

    // DeepSeek family patterns
    if match_wildcard("deepseek-*", &lower) || match_wildcard("*deepseek*", &lower) {
        return ModelFamily::DeepSeek;
    }

    ModelFamily::Unknown
}

/// A model routing rule mapping a model pattern to a protocol adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRouteRule {
    pub target_pattern: String,
    pub protocol_adapter: String,
    pub pattern_type: RoutingPatternType,
}

impl ModelRouteRule {
    /// Creates a new wildcard model routing rule (future-proof, recommended per AGENTS.md).
    pub fn wildcard(pattern: impl Into<String>, adapter: impl Into<String>) -> Self {
        let pat = pattern.into();
        Self {
            target_pattern: pat.clone(),
            protocol_adapter: adapter.into(),
            pattern_type: RoutingPatternType::Wildcard(pat),
        }
    }

    /// Creates a new regex model routing rule (future-proof, recommended per AGENTS.md).
    pub fn regex(pattern: impl Into<String>, adapter: impl Into<String>) -> Self {
        let pat = pattern.into();
        Self {
            target_pattern: pat.clone(),
            protocol_adapter: adapter.into(),
            pattern_type: RoutingPatternType::Regex(pat),
        }
    }

    /// Creates a brittle exact model routing rule (flagged by release gate).
    pub fn exact(pattern: impl Into<String>, adapter: impl Into<String>) -> Self {
        let pat = pattern.into();
        Self {
            target_pattern: pat.clone(),
            protocol_adapter: adapter.into(),
            pattern_type: RoutingPatternType::BrittleExact(pat),
        }
    }

    /// Returns true if this rule adheres to the future-proof pattern invariant.
    pub fn is_future_proof(&self) -> bool {
        matches!(
            self.pattern_type,
            RoutingPatternType::Wildcard(_) | RoutingPatternType::Regex(_)
        )
    }
}

/// Successfully resolved route outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedRoute {
    pub model_name: String,
    pub matched_adapter: String,
    pub family: ModelFamily,
    pub pattern_type: RoutingPatternType,
    pub matched_pattern: String,
    pub is_future_proof: bool,
}

/// Route resolver configuring rule lists and automatic model family fallback.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RouteResolver {
    pub rules: Vec<ModelRouteRule>,
    pub fallback_to_family: bool,
}

impl RouteResolver {
    /// Creates a new empty route resolver with family fallback enabled by default.
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            fallback_to_family: true,
        }
    }

    /// Creates a new route resolver with the specified rules.
    pub fn with_rules(rules: Vec<ModelRouteRule>) -> Self {
        Self {
            rules,
            fallback_to_family: true,
        }
    }

    /// Enables or disables automatic model family fallback when no specific rule matches.
    pub fn with_fallback_to_family(mut self, fallback: bool) -> Self {
        self.fallback_to_family = fallback;
        self
    }

    /// Appends a routing rule.
    pub fn add_rule(&mut self, rule: ModelRouteRule) {
        self.rules.push(rule);
    }

    /// Resolves a candidate model name to an adapter route.
    pub fn resolve(&self, model_name: &str) -> Result<ResolvedRoute, ModelRoutingViolation> {
        let clean_model = model_name.trim();
        if clean_model.is_empty() {
            return Err(ModelRoutingViolation::EmptyModelPattern);
        }

        // Validate that no rules violate the brittle exact match policy
        for rule in &self.rules {
            if let RoutingPatternType::BrittleExact(pat) = &rule.pattern_type {
                return Err(ModelRoutingViolation::BrittleExactMatchForbidden {
                    pattern: pat.clone(),
                });
            }
        }

        let family = detect_model_family(clean_model);

        // Evaluate configured rules in order
        for rule in &self.rules {
            match &rule.pattern_type {
                RoutingPatternType::Wildcard(pat) => {
                    if match_wildcard(pat, clean_model) {
                        return Ok(ResolvedRoute {
                            model_name: clean_model.to_string(),
                            matched_adapter: rule.protocol_adapter.clone(),
                            family,
                            pattern_type: rule.pattern_type.clone(),
                            matched_pattern: pat.clone(),
                            is_future_proof: true,
                        });
                    }
                }
                RoutingPatternType::Regex(pat) => {
                    let re = Regex::new(pat).map_err(|e| {
                        ModelRoutingViolation::InvalidRegexPattern {
                            pattern: pat.clone(),
                            reason: e.to_string(),
                        }
                    })?;
                    if re.is_match(clean_model) {
                        return Ok(ResolvedRoute {
                            model_name: clean_model.to_string(),
                            matched_adapter: rule.protocol_adapter.clone(),
                            family,
                            pattern_type: rule.pattern_type.clone(),
                            matched_pattern: pat.clone(),
                            is_future_proof: true,
                        });
                    }
                }
                RoutingPatternType::BrittleExact(_) => unreachable!(),
            }
        }

        // Fallback to detected model family default adapter if enabled
        if self.fallback_to_family && family.is_supported() {
            let default_adapter = family.default_protocol_adapter().to_string();
            let synthesized_pattern = format!("{}-*", family.as_str());
            return Ok(ResolvedRoute {
                model_name: clean_model.to_string(),
                matched_adapter: default_adapter,
                family,
                pattern_type: RoutingPatternType::Wildcard(synthesized_pattern.clone()),
                matched_pattern: synthesized_pattern,
                is_future_proof: true,
            });
        }

        Err(ModelRoutingViolation::UnmatchedModelNoFallback {
            model: clean_model.to_string(),
        })
    }
}

/// Resolves a model name against a slice of rules without fallback.
pub fn resolve_route(
    model_name: &str,
    rules: &[ModelRouteRule],
) -> Result<ResolvedRoute, ModelRoutingViolation> {
    let resolver = RouteResolver {
        rules: rules.to_vec(),
        fallback_to_family: false,
    };
    resolver.resolve(model_name)
}

/// Violations identified when inspecting model routing or prompt sanitization logic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelRoutingViolation {
    /// Brittle exact match is forbidden; wildcard or regex must be used.
    BrittleExactMatchForbidden { pattern: String },
    /// Invalid regular expression syntax.
    InvalidRegexPattern { pattern: String, reason: String },
    /// Empty model pattern provided.
    EmptyModelPattern,
    /// Static keyword replacement detected where regex pattern matching is mandated.
    StaticKeywordSanitizationForbidden { keyword: String, context: String },
    /// Candidate model did not match any rule and no fallback was available.
    UnmatchedModelNoFallback { model: String },
    /// Static model equality statement detected during code inspection.
    StaticModelEqualityDetected {
        line: usize,
        snippet: String,
        model_literal: String,
    },
}

impl fmt::Display for ModelRoutingViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BrittleExactMatchForbidden { pattern } => {
                write!(
                    f,
                    "Brittle exact match '{pattern}' is forbidden in model routing. Use wildcard pattern (e.g. 'gemini-*-flash-*') per AGENTS.md"
                )
            }
            Self::InvalidRegexPattern { pattern, reason } => {
                write!(f, "Invalid regex pattern '{pattern}': {reason}")
            }
            Self::EmptyModelPattern => {
                write!(f, "Model routing pattern cannot be empty")
            }
            Self::StaticKeywordSanitizationForbidden { keyword, context } => {
                write!(
                    f,
                    "Static keyword replacement for '{keyword}' forbidden in prompt sanitization ({context}). Use regex-based pattern matching per AGENTS.md"
                )
            }
            Self::UnmatchedModelNoFallback { model } => {
                write!(
                    f,
                    "Model '{model}' could not be routed to any adapter and no family fallback was available"
                )
            }
            Self::StaticModelEqualityDetected {
                line,
                snippet,
                model_literal,
            } => {
                write!(
                    f,
                    "Line {line}: Static model equality comparison against '{model_literal}' in '{snippet}'. Prioritize wildcard patterns per AGENTS.md"
                )
            }
        }
    }
}

impl std::error::Error for ModelRoutingViolation {}

/// Report summarizing legacy model match evaluation for backwards compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelMatchReport {
    pub matched_adapter: Option<String>,
    pub is_wildcard_matched: bool,
    pub rule_pattern: String,
    pub model_name: String,
    pub model_family: Option<ModelFamily>,
}

/// Checks whether a pattern contains wildcard glob characters (`*` or `?`).
pub fn is_wildcard_pattern(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?')
}

/// Returns true if a pattern conforms to future-proof guidelines (wildcard or regex).
pub fn has_future_proof_pattern(pattern: &str) -> bool {
    is_wildcard_pattern(pattern) || pattern.starts_with('^') || pattern.ends_with('$')
}

/// Matches a candidate string against a wildcard glob pattern supporting `*` and `?`.
pub fn match_wildcard(pattern: &str, candidate: &str) -> bool {
    let p_chars: Vec<char> = pattern.chars().collect();
    let c_chars: Vec<char> = candidate.chars().collect();

    let mut p_idx = 0;
    let mut c_idx = 0;
    let mut star_idx = None;
    let mut match_idx = 0;

    while c_idx < c_chars.len() {
        if p_idx < p_chars.len() && (p_chars[p_idx] == '?' || p_chars[p_idx] == c_chars[c_idx]) {
            p_idx += 1;
            c_idx += 1;
        } else if p_idx < p_chars.len() && p_chars[p_idx] == '*' {
            star_idx = Some(p_idx);
            match_idx = c_idx;
            p_idx += 1;
        } else if let Some(s_idx) = star_idx {
            p_idx = s_idx + 1;
            match_idx += 1;
            c_idx = match_idx;
        } else {
            return false;
        }
    }

    while p_idx < p_chars.len() && p_chars[p_idx] == '*' {
        p_idx += 1;
    }

    p_idx == p_chars.len()
}

/// Evaluates a model name against a set of routing rules, enforcing wildcard precedence.
pub fn evaluate_model_match(
    model_name: &str,
    rules: &[ModelRouteRule],
) -> Result<ModelMatchReport, ModelRoutingViolation> {
    let clean_model = model_name.trim();
    if clean_model.is_empty() {
        return Err(ModelRoutingViolation::EmptyModelPattern);
    }

    // Validate all rules against the brittle exact match policy
    for rule in rules {
        if let RoutingPatternType::BrittleExact(pat) = &rule.pattern_type {
            return Err(ModelRoutingViolation::BrittleExactMatchForbidden {
                pattern: pat.clone(),
            });
        }
    }

    let family = detect_model_family(clean_model);

    // Evaluate rules in order
    for rule in rules {
        match &rule.pattern_type {
            RoutingPatternType::Wildcard(pat) => {
                if match_wildcard(pat, clean_model) {
                    return Ok(ModelMatchReport {
                        matched_adapter: Some(rule.protocol_adapter.clone()),
                        is_wildcard_matched: true,
                        rule_pattern: pat.clone(),
                        model_name: clean_model.to_string(),
                        model_family: Some(family),
                    });
                }
            }
            RoutingPatternType::Regex(pat) => {
                let re = Regex::new(pat).map_err(|e| {
                    ModelRoutingViolation::InvalidRegexPattern {
                        pattern: pat.clone(),
                        reason: e.to_string(),
                    }
                })?;
                if re.is_match(clean_model) {
                    return Ok(ModelMatchReport {
                        matched_adapter: Some(rule.protocol_adapter.clone()),
                        is_wildcard_matched: true,
                        rule_pattern: pat.clone(),
                        model_name: clean_model.to_string(),
                        model_family: Some(family),
                    });
                }
            }
            RoutingPatternType::BrittleExact(_) => unreachable!(),
        }
    }

    Ok(ModelMatchReport {
        matched_adapter: None,
        is_wildcard_matched: false,
        rule_pattern: "none".to_string(),
        model_name: clean_model.to_string(),
        model_family: Some(family),
    })
}

/// Static code linter that checks routing and prompt sanitization code for AGENTS.md anti-patterns.
pub fn lint_routing_code(source: &str) -> Vec<ModelRoutingViolation> {
    let mut violations = Vec::new();

    for (idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Check for hardcoded model exact string equality (e.g. `model == "gemini-1.5-flash"`)
        if (trimmed.contains("model ==")
            || trimmed.contains("model.as_str() ==")
            || trimmed.contains("== model")
            || trimmed.contains(".equals(")
            || trimmed.starts_with("case \"gemini-")
            || trimmed.starts_with("case \"claude-")
            || trimmed.starts_with("case \"gpt-"))
            && (trimmed.contains("\"gemini-")
                || trimmed.contains("\"claude-")
                || trimmed.contains("\"gpt-")
                || trimmed.contains("\"o1-")
                || trimmed.contains("\"o3-"))
        {
            violations.push(ModelRoutingViolation::StaticModelEqualityDetected {
                line: idx + 1,
                snippet: trimmed.to_string(),
                model_literal: extract_model_literal(trimmed),
            });
        }

        // Check for static string replace in prompt sanitization
        if trimmed.contains(".replace(")
            && (trimmed.contains("prompt")
                || trimmed.contains("sanitize")
                || trimmed.contains("system_prompt"))
        {
            violations.push(ModelRoutingViolation::StaticKeywordSanitizationForbidden {
                keyword: trimmed.to_string(),
                context: "Use compiled regex instead of chained static string replacements"
                    .to_string(),
            });
        }
    }

    violations
}

/// Specialized detector for static model match comparisons in source code.
pub fn detect_static_model_matches(source: &str) -> Vec<ModelRoutingViolation> {
    lint_routing_code(source)
}

fn extract_model_literal(snippet: &str) -> String {
    if let Some(start) = snippet.find('"') {
        if let Some(end) = snippet[start + 1..].find('"') {
            return snippet[start + 1..start + 1 + end].to_string();
        }
    }
    snippet.to_string()
}
