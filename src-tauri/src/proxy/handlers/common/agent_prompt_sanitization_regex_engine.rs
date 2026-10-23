//! Agent prompt sanitization regex engine.
//!
//! Provides regex-based pattern matching for sanitizing agent prompts against
//! prompt injection, system prompt override, and delimiter leaks while strictly
//! preserving pipeline system prompts and benign user queries.

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Compiled regex to extract and protect markdown code blocks (fenced and inline).
pub static RE_CODE_BLOCK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?ms)(```[\s\S]*?```|`[^`\r\n]+`)").expect("valid code block regex")
});

/// Compiled regex for detecting prompt override markers and pseudo-system tags.
pub static RE_OVERRIDE_MARKERS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        r"(?i)\[\s*(?:system(?:_prompt)?|developer)\s*(?:override|bypass|mode)\s*\]|",
        r"(?i)<\s*/?\s*(?:system_override|instruction_override|admin_override)\s*>"
    ))
    .expect("valid override markers regex")
});

/// Compiled regex for detecting system prompt escape attempts and delimiter leaks.
pub static RE_SYSTEM_ESCAPE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        r"(?im)^\s*(?:system|developer)\s*:\s*(?:you are now|override|bypass|ignore safety|forget all)\b|",
        r"(?i)<\|(?:im_start|im_end|endoftext|fim_prefix|fim_suffix|start_header_id|end_header_id)\|>"
    ))
    .expect("valid system escape regex")
});

/// Compiled regex for detecting unauthorized instruction resets and jailbreak attempts.
pub static RE_INSTRUCTION_RESETS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        r"(?i)\b(?:ignore|disregard|forget|bypass|override)\s+(?:all\s+)?(?:previous|prior|above|system)\s+(?:instructions|prompts|directions|rules|guidelines|context)\b|",
        r"(?i)\breset\s+(?:all\s+)?(?:instructions|prompts|rules|system\s+context)\b|",
        r"(?i)\b(?:dan\s+mode|jailbreak\s+mode)\s+(?:enabled|activated|on)\b"
    ))
    .expect("valid instruction resets regex")
});

/// Category of prompt sanitization violation adhering to PascalCase Type convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SanitizationCategoryType {
    PromptInjection,
    SystemPromptOverride,
    DelimiterLeak,
    InstructionReset,
}

/// Backwards compatibility alias for older references.
pub type PromptSanitizationCategory = SanitizationCategoryType;

impl SanitizationCategoryType {
    /// Returns the canonical string representation of the category.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PromptInjection => "PromptInjection",
            Self::SystemPromptOverride => "SystemPromptOverride",
            Self::DelimiterLeak => "DelimiterLeak",
            Self::InstructionReset => "InstructionReset",
        }
    }
}

impl fmt::Display for SanitizationCategoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A declarative rule for detecting and replacing unsafe prompt patterns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SanitizationRule {
    pub name: String,
    pub pattern: String,
    pub replacement: String,
    pub category: SanitizationCategoryType,
}

impl SanitizationRule {
    /// Creates a new sanitization rule.
    pub fn new(
        name: impl Into<String>,
        pattern: impl Into<String>,
        replacement: impl Into<String>,
        category: SanitizationCategoryType,
    ) -> Self {
        Self {
            name: name.into(),
            pattern: pattern.into(),
            replacement: replacement.into(),
            category,
        }
    }

    /// Creates a validated sanitization rule returning AppError on invalid regex.
    pub fn try_new(
        name: impl Into<String>,
        pattern: &str,
        replacement: impl Into<String>,
        category: SanitizationCategoryType,
    ) -> Result<Self, crate::error::AppError> {
        regex::Regex::new(pattern).map_err(|e| {
            crate::error::AppError::Config(format!(
                "invalid sanitization regex pattern '{pattern}': {e}"
            ))
        })?;
        Ok(Self::new(name, pattern, replacement, category))
    }
}

/// Comprehensive report of the prompt sanitization pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptSanitizationReport {
    pub total_inspections: usize,
    pub replacements_count: usize,
    pub matched_rules: Vec<String>,
    pub is_sanitized: bool,
    pub sanitized_content: String,
    pub is_clean: bool,
    pub sanitized_text: String,
    pub matched_categories: Vec<SanitizationCategoryType>,
}

impl PromptSanitizationReport {
    /// Returns true if the prompt required no sanitization replacements.
    pub fn is_clean(&self) -> bool {
        self.is_clean
    }

    /// Returns true if sanitization replacements occurred.
    pub fn is_sanitized(&self) -> bool {
        self.is_sanitized
    }

    /// Returns true if a specific category was matched.
    pub fn has_category(&self, category: SanitizationCategoryType) -> bool {
        self.matched_categories.contains(&category)
    }

    /// Returns true if a specific rule name was triggered.
    pub fn has_rule(&self, rule_name: &str) -> bool {
        self.matched_rules.iter().any(|r| r == rule_name)
    }
}

impl fmt::Display for PromptSanitizationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PromptSanitizationReport(inspections={}, sanitized={}, count={}, rules={:?})",
            self.total_inspections,
            self.is_sanitized,
            self.replacements_count,
            self.matched_rules
        )
    }
}

/// Builds the default collection of prompt sanitization rules.
pub fn default_sanitization_rules() -> Vec<SanitizationRule> {
    vec![
        SanitizationRule::new(
            "ignore_previous_instructions",
            r"(?i)\b(?:ignore|disregard|forget|bypass)\s+(?:all\s+)?(?:previous|prior|above)\s+(?:instructions|prompts|directions)\b",
            "[FILTERED_PROMPT_INJECTION]",
            SanitizationCategoryType::PromptInjection,
        ),
        SanitizationRule::new(
            "disregard_prior_rules",
            r"(?i)\bdisregard\s+(?:all\s+)?(?:prior|previous|existing)\s+(?:rules|guidelines|system\s+prompts)\b",
            "[FILTERED_PROMPT_INJECTION]",
            SanitizationCategoryType::PromptInjection,
        ),
        SanitizationRule::new(
            "unauthorized_instruction_reset",
            r"(?i)\breset\s+(?:all\s+)?(?:instructions|prompts|rules|system\s+context)\b",
            "[FILTERED_INSTRUCTION_RESET]",
            SanitizationCategoryType::InstructionReset,
        ),
        SanitizationRule::new(
            "jailbreak_activation",
            r"(?i)\b(?:dan\s+mode|jailbreak\s+mode)\s+(?:enabled|activated|on)\b",
            "[FILTERED_PROMPT_INJECTION]",
            SanitizationCategoryType::PromptInjection,
        ),
        SanitizationRule::new(
            "system_prompt_override_tag",
            r"(?i)\[\s*(?:system(?:_prompt)?|developer)\s*(?:override|bypass|mode)\s*\]",
            "[FILTERED_SYSTEM_OVERRIDE]",
            SanitizationCategoryType::SystemPromptOverride,
        ),
        SanitizationRule::new(
            "system_role_spoofing",
            r"(?im)^\s*(?:system|developer)\s*:\s*(?:you are now|override|bypass|ignore safety|forget all)\b",
            "\nsystem: [FILTERED_SYSTEM_OVERRIDE]",
            SanitizationCategoryType::SystemPromptOverride,
        ),
        SanitizationRule::new(
            "system_override_xml_tag",
            r"(?i)<\s*/?\s*(?:system_override|instruction_override|admin_override)\s*>",
            "[FILTERED_SYSTEM_OVERRIDE]",
            SanitizationCategoryType::SystemPromptOverride,
        ),
        SanitizationRule::new(
            "chatml_delimiter_leak",
            r"(?i)<\|(?:im_start|im_end|endoftext|fim_prefix|fim_suffix|start_header_id|end_header_id)\|>",
            "[FILTERED_DELIMITER_LEAK]",
            SanitizationCategoryType::DelimiterLeak,
        ),
        SanitizationRule::new(
            "raw_delimiter_leak",
            r"(?i)<<<\s*RAW_PROMPT_DELIMITER\s*>>>",
            "[FILTERED_DELIMITER_LEAK]",
            SanitizationCategoryType::DelimiterLeak,
        ),
    ]
}

/// Protects markdown code blocks by substituting indexed placeholders.
fn protect_code_blocks(input: &str) -> (String, Vec<String>) {
    let mut placeholders = Vec::new();
    let protected = RE_CODE_BLOCK.replace_all(input, |caps: &regex::Captures| {
        let idx = placeholders.len();
        placeholders.push(caps[0].to_string());
        format!("__PROMPT_SANITIZER_CODE_BLOCK_{idx}__")
    });
    (protected.into_owned(), placeholders)
}

/// Restores markdown code blocks from indexed placeholders.
fn restore_code_blocks(text: &str, placeholders: &[String]) -> String {
    let mut restored = text.to_string();
    for (idx, original) in placeholders.iter().enumerate() {
        let tag = format!("__PROMPT_SANITIZER_CODE_BLOCK_{idx}__");
        restored = restored.replace(&tag, original);
    }
    restored
}

/// Applies a single compiled sanitization rule to prompt text.
fn apply_rule_to_text(
    current_text: &str,
    rule: &SanitizationRule,
) -> (String, usize, bool) {
    let re = match regex::Regex::new(&rule.pattern) {
        Ok(compiled) => compiled,
        Err(_) => return (current_text.to_string(), 0, false),
    };

    let count = re.find_iter(current_text).count();
    if count == 0 {
        return (current_text.to_string(), 0, false);
    }

    let replaced = re.replace_all(current_text, rule.replacement.as_str()).into_owned();
    (replaced, count, true)
}

/// Appends a matched category to the list if not already present.
fn append_category_if_missing(
    categories: &mut Vec<SanitizationCategoryType>,
    category: SanitizationCategoryType,
) {
    if !categories.contains(&category) {
        categories.push(category);
    }
}

/// Pipeline processor applying rules sequentially across protected prompt text.
fn apply_rules_pipeline(
    initial_text: &str,
    rules: &[SanitizationRule],
    matched_rules: &mut Vec<String>,
    matched_categories: &mut Vec<SanitizationCategoryType>,
) -> (String, usize) {
    let mut current = initial_text.to_string();
    let mut total = 0;
    for rule in rules {
        let (next_text, count, matched) = apply_rule_to_text(&current, rule);
        if matched {
            current = next_text;
            total += count;
            matched_rules.push(rule.name.clone());
            append_category_if_missing(matched_categories, rule.category);
        }
    }
    (current, total)
}

/// Core regex engine for agent prompt sanitization.
#[derive(Debug, Clone)]
pub struct AgentPromptSanitizationRegexEngine {
    rules: Vec<SanitizationRule>,
}

impl Default for AgentPromptSanitizationRegexEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentPromptSanitizationRegexEngine {
    /// Creates a new engine configured with the default sanitization rules.
    pub fn new() -> Self {
        Self {
            rules: default_sanitization_rules(),
        }
    }

    /// Creates an engine configured with custom sanitization rules.
    pub fn with_rules(rules: Vec<SanitizationRule>) -> Self {
        Self { rules }
    }

    /// Returns the active rules configured in the engine.
    pub fn rules(&self) -> &[SanitizationRule] {
        &self.rules
    }

    /// Inspects whether text contains override markers via compiled static regex.
    pub fn inspect_override_markers(text: &str) -> bool {
        RE_OVERRIDE_MARKERS.is_match(text)
    }

    /// Inspects whether text contains system prompt escapes via compiled static regex.
    pub fn inspect_system_escape(text: &str) -> bool {
        RE_SYSTEM_ESCAPE.is_match(text)
    }

    /// Inspects whether text contains instruction resets via compiled static regex.
    pub fn inspect_instruction_resets(text: &str) -> bool {
        RE_INSTRUCTION_RESETS.is_match(text)
    }

    /// Sanitizes prompt text while protecting code blocks and tracking report stats.
    pub fn sanitize(&self, prompt: &str) -> PromptSanitizationReport {
        let (protected, blocks) = protect_code_blocks(prompt);
        let mut matched_rules = Vec::new();
        let mut matched_categories = Vec::new();
        let (sanitized_protected, replacements) = apply_rules_pipeline(
            &protected,
            &self.rules,
            &mut matched_rules,
            &mut matched_categories,
        );
        let restored = restore_code_blocks(&sanitized_protected, &blocks);
        let is_sanitized = replacements > 0;

        PromptSanitizationReport {
            total_inspections: self.rules.len(),
            replacements_count: replacements,
            matched_rules,
            is_sanitized,
            sanitized_content: restored.clone(),
            is_clean: !is_sanitized,
            sanitized_text: restored,
            matched_categories,
        }
    }

    /// Convenience alias for sanitizing prompt text.
    pub fn sanitize_prompt(prompt: &str) -> PromptSanitizationReport {
        Self::new().sanitize(prompt)
    }
}

/// Sanitizes incoming prompt text using the provided rules.
pub fn sanitize_prompt_text(
    input: &str,
    rules: &[SanitizationRule],
) -> PromptSanitizationReport {
    let engine = AgentPromptSanitizationRegexEngine::with_rules(rules.to_vec());
    engine.sanitize(input)
}

/// Convenience helper to sanitize prompt text with the default rule set.
pub fn sanitize_prompt_text_with_defaults(input: &str) -> PromptSanitizationReport {
    let engine = AgentPromptSanitizationRegexEngine::new();
    engine.sanitize(input)
}

/// Top-level helper function for agent prompt sanitization.
pub fn sanitize_agent_prompt(content: &str) -> PromptSanitizationReport {
    sanitize_prompt_text_with_defaults(content)
}
