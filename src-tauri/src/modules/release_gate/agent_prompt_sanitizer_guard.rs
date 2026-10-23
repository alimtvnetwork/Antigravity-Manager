//! Agent-Client Prompt Sanitization Regex Guard.
//!
//! Enforces prompt sanitization discipline per AGENTS.md:
//! - "Prompt Sanitization Example: For agent-client prompt sanitization, prioritize
//!   regex-based pattern matching over static keyword replacement, ensuring full
//!   coverage without stripping pipeline system prompts or user queries."
//!
//! Features:
//! 1. Regex-based pattern matching for sensitive authentication tokens and secrets,
//!    replacing them with standardized placeholders `REDACTED_SECRET` and `REDACTED_AUTH`.
//! 2. Prompt integrity evaluation ensuring pipeline system prompts and user queries
//!    are never wiped, blanked, or excessively stripped.
//! 3. Detection of destructive naive keyword replacements (e.g. mangled English words,
//!    total prompt wipeout, or syntax corruption).
//! 4. Static rule auditing preventing naive static keywords from entering sanitization rules.

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Standard placeholder for redacted secrets, private keys, API keys, and passwords.
pub const REDACTED_SECRET: &str = "REDACTED_SECRET";

/// Standard placeholder for redacted authentication headers and bearer/basic tokens.
pub const REDACTED_AUTH: &str = "REDACTED_AUTH";

// Regex Definitions using Lazy initialization for zero runtime compilation overhead.

static PEM_PRIVATE_KEY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"-----BEGIN [A-Z\s]+PRIVATE KEY-----[\s\S]+?-----END [A-Z\s]+PRIVATE KEY-----")
        .expect("valid pem private key regex")
});

static DB_URI_AUTH_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b([a-z0-9+]+://[^:\s]+:)[^@\s]+(@[^/\s]+)")
        .expect("valid db uri regex")
});

static GENERIC_SECRET_ASSIGN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)\b((?:api[_-]?key|secret[_-]?key|client[_-]?secret|access[_-]?token|auth[_-]?token|private[_-]?key)\s*[:=]\s*["']?)[A-Za-z0-9_\-\.!@#$%^&*]{16,}(["']?)"#)
        .expect("valid generic secret assign regex")
});

static PASSWORD_ASSIGN_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)\b((?:password|passwd|pwd)\s*[:=]\s*["']?)[^\s"',;]{8,}(["']?)"#)
        .expect("valid password assignment regex")
});

static AUTH_HEADER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b((?:authorization|x-api-key|proxy-authorization):\s*(?:bearer\s+)?)[A-Za-z0-9_\-\.]{16,}")
        .expect("valid auth header regex")
});

static BEARER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(bearer\s+)[A-Za-z0-9_\-\.]{16,}")
        .expect("valid bearer regex")
});

static BASIC_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(basic\s+)[A-Za-z0-9+/=]{16,}")
        .expect("valid basic auth regex")
});

static OPENAI_KEY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b(?:sk|org)-[A-Za-z0-9]{20,}\b")
        .expect("valid openai key regex")
});

static GEMINI_KEY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\bAIza[0-9A-Za-z\-_]{35}\b")
        .expect("valid gemini key regex")
});

static ANTHROPIC_KEY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\bsk-ant-[A-Za-z0-9_\-]{20,}\b")
        .expect("valid anthropic key regex")
});

static MANGLED_WORD_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\b[A-Za-z0-9_]*(?:REDACTED_SECRET|REDACTED_AUTH|\[REDACTED\])[A-Za-z0-9_]*\b")
        .expect("valid mangled word regex")
});

/// Classification of the redacted credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RedactionKind {
    Secret,
    Auth,
}

/// Metadata describing a single redaction match within a prompt text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionMatch {
    pub rule_name: String,
    pub kind: RedactionKind,
    pub snippet_preview: String,
    pub start: usize,
    pub end: usize,
}

/// Sanitization result for raw text.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SanitizationReport {
    pub has_redactions: bool,
    pub redacted_count: usize,
    pub redactions: Vec<RedactionMatch>,
    pub sanitized_text: String,
}

/// Structured prompt payload containing system prompt, user query, and context messages.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PromptPayload {
    pub system_prompt: Option<String>,
    pub user_query: String,
    pub context_messages: Vec<String>,
}

/// Report evaluating the preservation and integrity of prompt components post-sanitization.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PromptIntegrityReport {
    pub is_intact: bool,
    pub system_prompt_preserved: bool,
    pub user_query_preserved: bool,
    pub context_messages_preserved: bool,
    pub original_system_len: usize,
    pub sanitized_system_len: usize,
    pub original_user_len: usize,
    pub sanitized_user_len: usize,
    pub violations: Vec<String>,
}

/// Types of destructive keyword violations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DestructiveViolationType {
    /// A benign word was mutilated by naive substring replacement (e.g. "keyboard" -> "REDACTED_SECRETboard").
    MangledBenignWord,
    /// An entire prompt or query was wiped out (reduced to empty or placeholder-only).
    PromptWipeout,
    /// Code or structured syntax was corrupted.
    SyntaxCorruption,
    /// Static bare keyword rule detected in configuration.
    StaticBareKeywordRule,
}

/// Violation details for destructive keyword replacement detection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DestructiveKeywordViolation {
    pub violation_type: DestructiveViolationType,
    pub keyword: String,
    pub detail: String,
}

/// Sanitizes raw prompt text using regex pattern matching.
///
/// Preserves system prompt instructions, user query text, and code snippets
/// while strictly masking secrets with `REDACTED_SECRET` and auth tokens with `REDACTED_AUTH`.
pub fn sanitize_prompt_text(input: &str) -> SanitizationReport {
    let mut current = input.to_string();
    let mut redactions = Vec::new();

    // 1. PEM Private Keys
    for m in PEM_PRIVATE_KEY_REGEX.find_iter(&current) {
        redactions.push(RedactionMatch {
            rule_name: "PemPrivateKey".to_string(),
            kind: RedactionKind::Secret,
            snippet_preview: "-----BEGIN PRIVATE KEY...".to_string(),
            start: m.start(),
            end: m.end(),
        });
    }
    current = PEM_PRIVATE_KEY_REGEX
        .replace_all(&current, format!("-----BEGIN PRIVATE KEY-----\n{REDACTED_SECRET}\n-----END PRIVATE KEY-----"))
        .to_string();

    // 2. Database URI embedded credentials
    for m in DB_URI_AUTH_REGEX.find_iter(&current) {
        redactions.push(RedactionMatch {
            rule_name: "DatabaseUriAuth".to_string(),
            kind: RedactionKind::Secret,
            snippet_preview: mask_snippet(m.as_str()),
            start: m.start(),
            end: m.end(),
        });
    }
    current = DB_URI_AUTH_REGEX
        .replace_all(&current, format!("${{1}}{REDACTED_SECRET}${{2}}"))
        .to_string();

    // 3. Generic Secret Assignments (api_key = "...", etc.)
    for m in GENERIC_SECRET_ASSIGN_REGEX.find_iter(&current) {
        redactions.push(RedactionMatch {
            rule_name: "GenericSecretAssign".to_string(),
            kind: RedactionKind::Secret,
            snippet_preview: mask_snippet(m.as_str()),
            start: m.start(),
            end: m.end(),
        });
    }
    current = GENERIC_SECRET_ASSIGN_REGEX
        .replace_all(&current, format!("${{1}}{REDACTED_SECRET}${{2}}"))
        .to_string();

    // 4. Password Assignments (password = "...", etc.)
    for m in PASSWORD_ASSIGN_REGEX.find_iter(&current) {
        redactions.push(RedactionMatch {
            rule_name: "PasswordAssign".to_string(),
            kind: RedactionKind::Secret,
            snippet_preview: mask_snippet(m.as_str()),
            start: m.start(),
            end: m.end(),
        });
    }
    current = PASSWORD_ASSIGN_REGEX
        .replace_all(&current, format!("${{1}}{REDACTED_SECRET}${{2}}"))
        .to_string();

    // 5. Auth Headers (Authorization: Bearer ..., x-api-key: ...)
    for m in AUTH_HEADER_REGEX.find_iter(&current) {
        redactions.push(RedactionMatch {
            rule_name: "AuthHeader".to_string(),
            kind: RedactionKind::Auth,
            snippet_preview: mask_snippet(m.as_str()),
            start: m.start(),
            end: m.end(),
        });
    }
    current = AUTH_HEADER_REGEX
        .replace_all(&current, format!("${{1}}{REDACTED_AUTH}"))
        .to_string();

    // 6. Bearer Tokens
    for m in BEARER_REGEX.find_iter(&current) {
        redactions.push(RedactionMatch {
            rule_name: "BearerToken".to_string(),
            kind: RedactionKind::Auth,
            snippet_preview: mask_snippet(m.as_str()),
            start: m.start(),
            end: m.end(),
        });
    }
    current = BEARER_REGEX
        .replace_all(&current, format!("${{1}}{REDACTED_AUTH}"))
        .to_string();

    // 7. Basic Auth Tokens
    for m in BASIC_REGEX.find_iter(&current) {
        redactions.push(RedactionMatch {
            rule_name: "BasicAuth".to_string(),
            kind: RedactionKind::Auth,
            snippet_preview: mask_snippet(m.as_str()),
            start: m.start(),
            end: m.end(),
        });
    }
    current = BASIC_REGEX
        .replace_all(&current, format!("${{1}}{REDACTED_AUTH}"))
        .to_string();

    // 8. OpenAI API Keys (sk-..., org-...)
    for m in OPENAI_KEY_REGEX.find_iter(&current) {
        redactions.push(RedactionMatch {
            rule_name: "OpenAiApiKey".to_string(),
            kind: RedactionKind::Secret,
            snippet_preview: mask_snippet(m.as_str()),
            start: m.start(),
            end: m.end(),
        });
    }
    current = OPENAI_KEY_REGEX
        .replace_all(&current, REDACTED_SECRET)
        .to_string();

    // 9. Gemini API Keys (AIza...)
    for m in GEMINI_KEY_REGEX.find_iter(&current) {
        redactions.push(RedactionMatch {
            rule_name: "GeminiApiKey".to_string(),
            kind: RedactionKind::Secret,
            snippet_preview: mask_snippet(m.as_str()),
            start: m.start(),
            end: m.end(),
        });
    }
    current = GEMINI_KEY_REGEX
        .replace_all(&current, REDACTED_SECRET)
        .to_string();

    // 10. Anthropic API Keys (sk-ant-...)
    for m in ANTHROPIC_KEY_REGEX.find_iter(&current) {
        redactions.push(RedactionMatch {
            rule_name: "AnthropicApiKey".to_string(),
            kind: RedactionKind::Secret,
            snippet_preview: mask_snippet(m.as_str()),
            start: m.start(),
            end: m.end(),
        });
    }
    current = ANTHROPIC_KEY_REGEX
        .replace_all(&current, REDACTED_SECRET)
        .to_string();

    let redacted_count = redactions.len();
    let has_redactions = redacted_count > 0;

    SanitizationReport {
        has_redactions,
        redacted_count,
        redactions,
        sanitized_text: current,
    }
}

/// Sanitizes a structured prompt payload and evaluates its integrity.
pub fn sanitize_prompt_payload(
    payload: &PromptPayload,
) -> (PromptPayload, PromptIntegrityReport) {
    let sanitized_system = payload
        .system_prompt
        .as_ref()
        .map(|s| sanitize_prompt_text(s).sanitized_text);

    let sanitized_user = sanitize_prompt_text(&payload.user_query).sanitized_text;

    let sanitized_context: Vec<String> = payload
        .context_messages
        .iter()
        .map(|msg| sanitize_prompt_text(msg).sanitized_text)
        .collect();

    let sanitized_payload = PromptPayload {
        system_prompt: sanitized_system,
        user_query: sanitized_user,
        context_messages: sanitized_context,
    };

    let report = evaluate_prompt_integrity(payload, &sanitized_payload);
    (sanitized_payload, report)
}

/// Evaluates prompt integrity ensuring system prompts and user queries are never wiped.
pub fn evaluate_prompt_integrity(
    original: &PromptPayload,
    sanitized: &PromptPayload,
) -> PromptIntegrityReport {
    let mut violations = Vec::new();

    let original_system_len = original
        .system_prompt
        .as_ref()
        .map(|s| s.len())
        .unwrap_or(0);
    let sanitized_system_len = sanitized
        .system_prompt
        .as_ref()
        .map(|s| s.len())
        .unwrap_or(0);

    let original_user_len = original.user_query.len();
    let sanitized_user_len = sanitized.user_query.len();

    // 1. System Prompt Preservation Check
    let mut system_prompt_preserved = true;
    if let Some(ref orig_sys) = original.system_prompt {
        if !orig_sys.trim().is_empty() {
            match sanitized.system_prompt {
                Some(ref sanit_sys) => {
                    if sanit_sys.trim().is_empty() {
                        system_prompt_preserved = false;
                        violations.push("System prompt was wiped out to empty string".to_string());
                    } else if orig_sys.len() > 60 && sanit_sys.len() < orig_sys.len() / 5 {
                        system_prompt_preserved = false;
                        violations.push(format!(
                            "System prompt experienced excessive destruction (from {} to {} bytes)",
                            orig_sys.len(),
                            sanit_sys.len()
                        ));
                    }
                }
                None => {
                    system_prompt_preserved = false;
                    violations.push("System prompt was stripped entirely from payload".to_string());
                }
            }
        }
    }

    // 2. User Query Preservation Check
    let mut user_query_preserved = true;
    if !original.user_query.trim().is_empty() {
        if sanitized.user_query.trim().is_empty() {
            user_query_preserved = false;
            violations.push("User query was wiped out to empty string".to_string());
        } else if original.user_query.len() > 60
            && sanitized.user_query.len() < original.user_query.len() / 5
        {
            user_query_preserved = false;
            violations.push(format!(
                "User query experienced excessive destruction (from {} to {} bytes)",
                original.user_query.len(),
                sanitized.user_query.len()
            ));
        }
    }

    // 3. Context Messages Preservation Check
    let mut context_messages_preserved = true;
    if original.context_messages.len() != sanitized.context_messages.len() {
        context_messages_preserved = false;
        violations.push(format!(
            "Context message count mismatch: original={}, sanitized={}",
            original.context_messages.len(),
            sanitized.context_messages.len()
        ));
    }

    // 4. Destructive Keyword Replacements in System Prompt or User Query
    if let Some(ref s_sys) = sanitized.system_prompt {
        if let Some(ref o_sys) = original.system_prompt {
            let destructive = detect_destructive_keyword_replacement(o_sys, s_sys);
            for d in destructive {
                violations.push(format!("Destructive replacement in system prompt: {}", d.detail));
            }
        }
    }

    let user_destructive =
        detect_destructive_keyword_replacement(&original.user_query, &sanitized.user_query);
    for d in user_destructive {
        violations.push(format!("Destructive replacement in user query: {}", d.detail));
    }

    let is_intact = violations.is_empty()
        && system_prompt_preserved
        && user_query_preserved
        && context_messages_preserved;

    PromptIntegrityReport {
        is_intact,
        system_prompt_preserved,
        user_query_preserved,
        context_messages_preserved,
        original_system_len,
        sanitized_system_len,
        original_user_len,
        sanitized_user_len,
        violations,
    }
}

/// Detects destructive keyword replacements (e.g. naive string substitution
/// that mangled benign English words or stripped prompt bodies).
pub fn detect_destructive_keyword_replacement(
    original: &str,
    sanitized: &str,
) -> Vec<DestructiveKeywordViolation> {
    let mut violations = Vec::new();

    // 1. Detect mangled benign words (e.g. "keyboard" -> "REDACTED_SECRETboard")
    for m in MANGLED_WORD_REGEX.find_iter(sanitized) {
        let token = m.as_str();
        if token != REDACTED_SECRET && token != REDACTED_AUTH && token != "[REDACTED]" {
            violations.push(DestructiveKeywordViolation {
                violation_type: DestructiveViolationType::MangledBenignWord,
                keyword: token.to_string(),
                detail: format!(
                    "Benign identifier or word mangled by naive substring replacement: '{token}'"
                ),
            });
        }
    }

    // 2. Detect complete prompt wipeout
    let orig_trimmed = original.trim();
    let sanit_trimmed = sanitized.trim();
    if orig_trimmed.len() > 30 {
        if sanit_trimmed.is_empty() {
            violations.push(DestructiveKeywordViolation {
                violation_type: DestructiveViolationType::PromptWipeout,
                keyword: String::new(),
                detail: "Sanitization completely wiped out prompt content".to_string(),
            });
        } else if sanit_trimmed == REDACTED_SECRET || sanit_trimmed == REDACTED_AUTH {
            violations.push(DestructiveKeywordViolation {
                violation_type: DestructiveViolationType::PromptWipeout,
                keyword: sanit_trimmed.to_string(),
                detail: "Prompt was completely replaced by a lone redaction token".to_string(),
            });
        }
    }

    // 3. Detect JSON or structural quote corruption caused by naive replacement
    let orig_double_quotes = original.chars().filter(|&c| c == '"').count();
    let sanit_double_quotes = sanitized.chars().filter(|&c| c == '"').count();
    if orig_double_quotes % 2 == 0 && sanit_double_quotes % 2 != 0 {
        violations.push(DestructiveKeywordViolation {
            violation_type: DestructiveViolationType::SyntaxCorruption,
            keyword: "\"".to_string(),
            detail: "Unbalanced quotes detected after sanitization, corrupting syntax".to_string(),
        });
    }

    violations
}

/// Audits a sanitization rule pattern to prevent naive static keywords from being registered.
pub fn audit_sanitization_rule(rule_pattern: &str) -> Result<(), DestructiveKeywordViolation> {
    let trimmed = rule_pattern.trim();
    let lower = trimmed.to_ascii_lowercase();

    // Naive static keywords without regex delimiters or structural qualifiers
    let static_keywords = [
        "key", "keys", "token", "tokens", "auth", "secret", "secrets", "pass",
        "password", "pwd", "bearer", "basic",
    ];

    if static_keywords.contains(&lower.as_str()) {
        return Err(DestructiveKeywordViolation {
            violation_type: DestructiveViolationType::StaticBareKeywordRule,
            keyword: trimmed.to_string(),
            detail: format!(
                "Forbidden static bare keyword rule '{trimmed}'. Prioritize regex-based pattern matching per AGENTS.md"
            ),
        });
    }

    // Must contain regex word boundary or assignment structure
    if !trimmed.contains('\\') && !trimmed.contains(':') && !trimmed.contains('=') && !trimmed.contains('^') && !trimmed.contains('$') {
        if trimmed.len() < 8 {
            return Err(DestructiveKeywordViolation {
                violation_type: DestructiveViolationType::StaticBareKeywordRule,
                keyword: trimmed.to_string(),
                detail: format!(
                    "Rule '{trimmed}' lacks regex boundary anchors (\\b) or credential syntax delimiters"
                ),
            });
        }
    }

    Ok(())
}

/// Masks a secret snippet for safe preview in reports (e.g. "sk-a...3456").
fn mask_snippet(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.len() <= 8 {
        "***".to_string()
    } else {
        let prefix = &trimmed[..4];
        let suffix = &trimmed[trimmed.len() - 4..];
        format!("{prefix}...{suffix}")
    }
}
