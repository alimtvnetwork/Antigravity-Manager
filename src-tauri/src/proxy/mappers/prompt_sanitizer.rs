//! Prompt Sanitizer Pipeline Module
//!
//! Provides a dedicated prompt cleaning pipeline node after proxy request generation
//! and before outbound dispatch to upstream endpoints.
//! Strictly and precisely strips client billing metadata pseudo-headers (such as
//! `x-anthropic-billing-header`, `cc_version`, `cc_entrypoint` signatures) injected into
//! system or user blocks by third-party clients (Claude Code CLI, Cherry Studio, etc.).
//! This prevents triggering Google Cloud Code upstream WAF rule interceptions (false
//! 429 RESOURCE_EXHAUSTED) that cause cascading 503 global account lockouts.
//!
//! Core design rules:
//! 1. Code block protection: Fenced code blocks (```) and inline code (`) are automatically
//!    preserved without modification.
//! 2. Zero session touch: Never sanitize or filter session-related identifiers (including
//!    `x-jeikcode-session-id`, `x-atomcode-session-id`, `x-session-id`, etc.).
//! 3. Zero delimiter touch: Never delete `=== ... ===`, `--- ... ---`, or XML tags (`<environment>`,
//!    `<workflow_and_execution_discipline>`, etc.), preserving AI agent structures intact.
//! 4. Preserve user natural language and business instructions.
//! 5. Full pipeline consistency across storage and outbound payloads.

use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

/// Code block protection regex: matches multiline fenced blocks ```...``` and inline code `...`
static RE_CODE_BLOCK: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?ms)(```[\s\S]*?```|`[^`\r\n]+`)").unwrap());

/// Regex matching high-risk client pseudo-headers that trigger upstream Google WAF (applied only to non-code blocks):
/// 1) Matches client-injected `*-billing*` headers (`x-anthropic-billing-header:`, `x-billing:`, etc.)
/// 2) Matches `x-` headers carrying Claude Code CLI billing signatures (`cc_version`, `cc_entrypoint`)
/// Strictly excludes `session` keywords to preserve conversation tracking.
static RE_WAF_TRIGGER_HEADERS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        r"(?im)^\s*(?:x-[a-z0-9_-]*billing[a-z0-9_-]*|[a-z0-9_-]+-billing-(?:header|metadata|token|info)):\s*[^\r\n]*(\r?\n)?",
        r"|^\s*x-[a-z0-9_-]+:\s*[^\r\n]*(?:cc_version|cc_entrypoint)[^\r\n]*(\r?\n)?"
    ))
    .unwrap()
});

/// Regex collapsing 3 or more consecutive newlines into 2 to preserve clean paragraphs
static RE_MULTI_NEWLINE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\n{3,}").unwrap());

pub struct PromptSanitizer;

impl PromptSanitizer {
    /// High-precision prompt text sanitization:
    /// 1. Fast pre-check: if text does not contain suspect trigger terms (`billing`, `cc_version`, `cc_entrypoint`), returns early.
    /// 2. Extracts and shields code blocks with placeholders.
    /// 3. Strips WAF-triggering pseudo-headers from non-code segments.
    /// 4. Restores shielded code blocks.
    /// 5. Normalizes consecutive newlines.
    pub fn clean_text(text: &str) -> String {
        let lower = text.to_lowercase();
        let has_suspect = lower.contains("billing")
            || lower.contains("cc_version")
            || lower.contains("cc_entrypoint");

        if !has_suspect {
            return text.to_string();
        }

        // Step 1: Shield code blocks
        let mut placeholders: Vec<String> = Vec::new();
        let protected_text = RE_CODE_BLOCK.replace_all(text, |caps: &regex::Captures| {
            let idx = placeholders.len();
            placeholders.push(caps[0].to_string());
            format!("__PROMPT_SANITIZER_CODE_BLOCK_{}__", idx)
        });

        // Step 2: Strip suspect headers from non-code regions
        let pass1 = RE_WAF_TRIGGER_HEADERS.replace_all(&protected_text, "");

        // Step 3: Restore protected code blocks
        let mut restored = pass1.into_owned();
        for (idx, original_code) in placeholders.iter().enumerate() {
            let ph = format!("__PROMPT_SANITIZER_CODE_BLOCK_{}__", idx);
            restored = restored.replace(&ph, original_code);
        }

        // Step 4: Collapse redundant empty lines
        let normalized = RE_MULTI_NEWLINE.replace_all(&restored, "\n\n");

        normalized.trim().to_string()
    }

    /// Sanitizes all text nodes within a parts array
    pub fn sanitize_parts(parts: &mut Vec<Value>) -> usize {
        let mut cleaned_count = 0;
        for part in parts.iter_mut() {
            if let Some(obj) = part.as_object_mut() {
                // Thinking blocks are strictly protected by thoughtSignature, their text must remain byte-level immutable
                if obj.get("thought").and_then(Value::as_bool).unwrap_or(false)
                    || obj.contains_key("thoughtSignature")
                {
                    continue;
                }

                if let Some(text_val) = obj.get("text").and_then(Value::as_str) {
                    let cleaned = Self::clean_text(text_val);
                    if cleaned != text_val {
                        obj.insert("text".to_string(), Value::String(cleaned));
                        cleaned_count += 1;
                    }
                }
            }
        }
        cleaned_count
    }

    /// Pipeline node: sanitizes system instructions and contents within a Gemini payload body.
    /// Supports top-level Gemini body as well as payloads wrapped under the `request` key.
    pub fn sanitize_gemini_payload(body: &mut Value) -> usize {
        let mut total_cleaned = 0;

        if let Some(inner) = body.get_mut("request").and_then(Value::as_object_mut) {
            let mut inner_val = Value::Object(inner.clone());
            let count = Self::sanitize_gemini_payload_inner(&mut inner_val);
            if let Value::Object(new_inner) = inner_val {
                *inner = new_inner;
            }
            total_cleaned += count;
        }

        total_cleaned += Self::sanitize_gemini_payload_inner(body);
        total_cleaned
    }

    fn sanitize_gemini_payload_inner(body: &mut Value) -> usize {
        let mut cleaned_count = 0;

        // 1. Sanitize system instructions
        if let Some(sys) = body
            .get_mut("systemInstruction")
            .and_then(Value::as_object_mut)
        {
            if let Some(parts) = sys.get_mut("parts").and_then(Value::as_array_mut) {
                cleaned_count += Self::sanitize_parts(parts);
            }
        }

        // 2. Sanitize dialogue turns in contents
        if let Some(contents) = body.get_mut("contents").and_then(Value::as_array_mut) {
            for turn in contents.iter_mut() {
                if let Some(parts) = turn.get_mut("parts").and_then(Value::as_array_mut) {
                    cleaned_count += Self::sanitize_parts(parts);
                }
            }
        }

        cleaned_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_clean_text_multiline_system_prompt_with_billing() {
        let raw = concat!(
            "x-anthropic-billing-header: cc_version=2.1.220.04c; cc_entrypoint=sdk-ts;\n",
            "You are Claude Code, Anthropic's official CLI for Claude.\n\n",
            "Please follow these instructions:\n",
            "1. Assist with coding tasks."
        );
        let cleaned = PromptSanitizer::clean_text(raw);
        assert!(!cleaned.contains("x-anthropic-billing-header"));
        assert!(cleaned.starts_with("You are Claude Code"));
        assert!(cleaned.contains("1. Assist with coding tasks."));
    }

    #[test]
    fn test_clean_waf_trigger_cc_entrypoint_header() {
        let raw = concat!(
            "x-custom-billing: cc_version=2.0; cc_entrypoint=cli;\n",
            "Actual user instructions."
        );
        let cleaned = PromptSanitizer::clean_text(raw);
        assert_eq!(cleaned, "Actual user instructions.");
    }

    #[test]
    fn test_clean_generic_wildcard_billing_headers() {
        let raw = concat!(
            "x-billing: enabled\n",
            "x-custom-billing-info: token123\n",
            "x-client-billing: active\n",
            "anthropic-billing-header: cc_version=2.1\n",
            "Actual user instructions."
        );
        let cleaned = PromptSanitizer::clean_text(raw);
        assert_eq!(cleaned, "Actual user instructions.");
    }

    #[test]
    fn test_strictly_preserves_session_headers_and_tokens() {
        let raw = concat!(
            "x-session-identifier: session-abc-123\n",
            "x-custom-session-id: atom-sess-456\n",
            "x-session-id: generic-sess-789\n",
            "x-client-session-id: client-sess-000\n",
            "Please keep my session active."
        );
        let cleaned = PromptSanitizer::clean_text(raw);
        assert_eq!(cleaned, raw);
    }

    #[test]
    fn test_strictly_preserves_user_delimiters_and_xml_tags() {
        let prompt = concat!(
            "<environment>\n",
            "You are an AI coding Agent.\n\n",
            "## PRECEDENCE:\n",
            "- Content enclosed in XML tags represents current environment.\n",
            "- Rules under headers matching `=== ... (*.md) ===` constitute USER PROVISIONS.\n",
            "</environment>\n\n",
            "=== AGENTS.md ===\n",
            "User custom provisions.\n",
            "=== MEMORY ===\n",
            "Memory block 1."
        );
        let cleaned = PromptSanitizer::clean_text(prompt);
        assert_eq!(cleaned, prompt);
    }

    #[test]
    fn test_protects_code_blocks_containing_waf_signatures() {
        let code = concat!(
            "Here is my code:\n",
            "```python\n",
            "headers = {'x-anthropic-billing-header': 'cc_version=1.0'}\n",
            "print(headers)\n",
            "```\n",
            "Does this look right?"
        );
        let cleaned = PromptSanitizer::clean_text(code);
        assert_eq!(cleaned, code);
    }

    #[test]
    fn test_protects_normal_user_prompts_and_http_headers() {
        let normal_text = concat!(
            "How do I set Authorization: Bearer <token> in curl?\n",
            "- Step 1: Add -H flag\n",
            "- Step 2: Test endpoint\n\n",
            "Total steps: 2"
        );
        let cleaned = PromptSanitizer::clean_text(normal_text);
        assert_eq!(cleaned, normal_text);
    }

    #[test]
    fn test_sanitize_gemini_payload_system_and_user_blocks() {
        let mut payload = json!({
            "project": "test-project",
            "model": "gemini-3.8-flash-high",
            "request": {
                "systemInstruction": {
                    "role": "user",
                    "parts": [
                        {
                            "text": "x-anthropic-billing-header: cc_version=2.1;\n<environment>You are an AI assistant</environment>\n=== AGENTS.md ==="
                        }
                    ]
                },
                "contents": [
                    {
                        "role": "user",
                        "parts": [
                            {
                                "text": "x-anthropic-billing-header: cc_entrypoint=cli;\nPlease analyze my data:\n=== MEMORY ===\n- Metric A: 10\n- Metric B: 20"
                            }
                        ]
                    },
                    {
                        "role": "model",
                        "parts": [
                            {
                                "text": "Model output response."
                            }
                        ]
                    }
                ]
            }
        });

        let cleaned_count = PromptSanitizer::sanitize_gemini_payload(&mut payload);
        assert_eq!(cleaned_count, 2);

        let sys_text = payload["request"]["systemInstruction"]["parts"][0]["text"]
            .as_str()
            .unwrap();
        assert!(!sys_text.contains("x-anthropic-billing-header"));
        assert!(sys_text.contains("<environment>You are an AI assistant</environment>"));
        assert!(sys_text.contains("=== AGENTS.md ==="));

        let user_text = payload["request"]["contents"][0]["parts"][0]["text"]
            .as_str()
            .unwrap();
        assert!(!user_text.contains("x-anthropic-billing-header"));
        assert!(user_text.contains("=== MEMORY ==="));
        assert!(user_text.contains("- Metric A: 10\n- Metric B: 20"));
    }
}
