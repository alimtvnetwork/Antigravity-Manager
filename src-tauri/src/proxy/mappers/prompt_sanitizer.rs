//! Prompt Sanitizer Pipeline Module
//!
//! Dedicated prompt sanitization pipeline node executing after proxy payload generation and before outbound dispatch.
//! Strictly and precisely strips billing metadata pseudo-headers (such as `x-anthropic-billing-header` and
//! `cc_version` / `cc_entrypoint` / `cch` signatures) and Claude Agent SDK leading fingerprint declarations
//! injected into system / user blocks by third-party clients (Claude Code CLI / VS Code CC / Cherry Studio etc.).
//! Prevents triggering Google Cloud Code upstream WAF feature interceptions (false 429 RESOURCE_EXHAUSTED)
//! that cause cascading 503 global account pool lockouts.
//!
//! Core design and defense rules:
//! 1. Deep code block protection: Automatically extracts and preserves fenced code blocks (```) and inline code (`), 100% exempt from modification;
//! 2. Zero Session Touch: Never cleans or filters any session-related fields (including
//!    `x-jeikcode-session-id`, `x-atomcode-session-id`, `x-session-id`, etc.); session uniqueness is
//!    governed by `SessionScope` through an independent priority chain, preserving all session identifiers 100% intact;
//! 3. Zero Delimiter Touch: Never deletes `=== ... ===`, `--- ... ---`, or
//!    XML tags (such as `<environment>`, `<workflow_and_execution_discipline>`, etc.), ensuring frameworks like
//!    JeikCode / AtomCode retain intact user and system instruction structures;
//! 4. Preserve user natural language and business instructions: Never make assumptions about IDE regurgitated text;
//! 5. Full pipeline audit consistency: Intervenes uniformly at pipeline level after proxy payload generation and thought block population;
//! 6. Thinking Block at Index 0: Must ensure thought block is strictly at index 0 of parts after cleaning,
//!    and thought text is strictly immutable to preserve digital signatures and hash integrity;
//! 7. Physical removal of empty Part and empty systemInstruction: Empty parts are removed from array; if systemInstruction
//!    is empty it is removed to eliminate Google API format validation errors.
//! 8. Two-Track Separation: Strictly differentiates Header track and System track to prevent rule pollution:
//!    - Header track (`clean_text`): Strips embedded billing pseudo-headers and risky client fingerprint declarations across ALL text
//!      (systemInstruction / contents / user) uniformly;
//!    - System track (`normalize_system_identity`): Only within the system prompt head window (first
//!      `IDENTITY_SCAN_MAX_SENTENCES` sentences of each system block), normalizes identity statements to neutral identity,
//!      WITHOUT touching user / tool text or pipeline internal system prompts.
//!    This separation allows all four protocols to share unified normalization (Pipeline First principle).

use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

/// Code block protection regex: isolates multiline fenced blocks ```...``` and inline code `...`
static RE_CODE_BLOCK: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?ms)(```[\s\S]*?```|`[^`\r\n]+`)").unwrap());

/// Regex matching high-risk client pseudo-headers that trigger upstream Google WAF (applied only to non-code blocks):
/// 1) Generalized matching of client-injected `*-billing*` pseudo-header lines (`x-anthropic-billing-header:`, `x-billing:`, `x-client-billing:`, `anthropic-billing-header:`, etc.)
/// 2) Matching any `x-` header carrying Claude Code CLI billing signatures (`cc_version`, `cc_entrypoint`, `cch=`)
/// 3) Matching mid-paragraph embedded `x-anthropic-billing-header:` and subsequent declarations
/// 4) Matching Claude Agent SDK leading fingerprint declarations (`You are a Claude agent, built on Anthropic's Claude Agent SDK.`)
/// Strictly excludes any `session` keywords to preserve user queries and conversation tracking.
static RE_WAF_TRIGGER_HEADERS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        r"(?im)^\s*(?:x-[a-z0-9_-]*billing[a-z0-9_-]*|[a-z0-9_-]+-billing-(?:header|metadata|token|info)):\s*[^\r\n]*(\r?\n)?",
        r"|^\s*x-[a-z0-9_-]+:\s*[^\r\n]*(?:cc_version|cc_entrypoint|cch=)[^\r\n]*(\r?\n)?",
        r"|(?i)x-anthropic-billing-header:\s*[^\r\n]*",
        r"|(?i)You are a Claude agent, built on Anthropic's Claude Agent SDK\.(\r?\n)?"
    ))
    .unwrap()
});

/// Regex to collapse redundant empty lines (collapsing 3+ consecutive newlines to 2 to maintain paragraph structure)
static RE_MULTI_NEWLINE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\n{3,}").unwrap());

// ============================ System Track: Identity Declaration Normalization ============================
//
// What triggers upstream WAF pseudo-rate-limiting (false 429 RESOURCE_EXHAUSTED) is identity attribution declarations,
// rather than model capabilities or request size. Upstream applies stricter rules when identifying vendor / product /
// competitor model fingerprints in self-introductions, rejecting requests and falsely flagging accounts as exhausted.
// (See issues #3507 / #3506, and Codex `based on GPT-x` in #3444 / #3489).
//
// Client frameworks cannot be exhaustively enumerated, so normalization must be broad-spectrum:
// any statement containing opening + noun phrase + attribution/version is normalized to neutral identity.

/// Identity normalization scan window: scans only the first 4 sentences of each system prompt block.
/// Identity declarations appear at the start of prompts; restricting the window ensures
/// normal occurrences of 'created by ...' in body text/code are 100% unaffected.
const IDENTITY_SCAN_MAX_SENTENCES: usize = 4;

/// Neutral identity declaration after normalization (stripping vendor/product/competitor fingerprints)
const NEUTRAL_IDENTITY: &str = "You are an AI Agent.";

/// Identity declaration normalization regex (broad match, active only in system prompt head window).
///
/// Form = A Identity opening + B Identity noun phrase + C Attribution/version + D Subject + E Punctuation:
/// ```text
/// You are Hermes Agent, an intelligent AI assistant created by Nous Research.   → You are an AI Agent.
/// You are Codex, an advanced coding agent based on GPT-6.                       → You are an AI Agent.
/// You are an AI agent created by Example Corp.                                  → You are an AI Agent.
/// You are Antigravity, a powerful agentic AI coding assistant designed by the Google Deepmind team … → You are an AI Agent.
/// ```
/// Defense rules against false positives:
/// 1. Must have both identity noun and attribution declaration: 'You are a helpful assistant.', 'You are Claude Code, Anthropic's
///    official CLI for Claude.', 'You are JeikCode AI coding Agent by Jeik.' (no attribution verb) are preserved intact;
/// 2. Attribution subject bounded by punctuation ('.' ',' newline) and must never cross XML tags - structurally eliminating
///    cross-line consumption bugs while preserving Zero Delimiter Touch;
///    ensuring subsequent prompt instructions remain completely intact;
/// 3. Replaces only the declaration sentence itself: content following attribution (including comma-continued instructions) is verbatim preserved;
/// 4. Transition window <= 8 chars: bridge text between noun and verb must be very short to prevent clause-crossing false positives;
///    (e.g., '... agent, and the config was created by X' in body text must not be normalized).
static RE_IDENTITY_DECLARATION: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        // A. Identity opening (word boundary required to avoid matching inside phrases)
        r"(?i:\b(?:you\s+are|you're|i\s+am|i'm)\b[\s,，:：]*)",
        // B. Identity noun phrase: optional article + <=8 modifiers + optional AI + identity noun
        r"(?i:(?:an?\s+)?(?:[\w'\-]+[\s,，]+){0,8}(?:ai[\s,，]+)?(?:agent\b|assistant\b|ai\b))",
        // C. Attribution / version declaration (verb + preposition, or based on)
        //    Transition window permits comma (covering 'You are a Claude agent, built on Anthropic\'s ...'),
        //    but forbids crossing sentence ends, newlines, and XML tags, with max length of 8 characters.
        //    Narrowing the window prevents accidentally deleting cross-clause sentences.
        r"(?i:[^.!?。！？\r\n<>]{0,8}?\b(?:created|built|made|developed|designed|trained|powered|maintained|published|released|provided)\s+(?:by|on|upon|at)\s+",
        r"|[^.!?。！？\r\n<>]{0,8}?\bbased\s+on\s+)",
        // D. Attribution subject: matches up to punctuation boundary
        r"[^.,，。\r\n<>]{1,80}",
        // E. Trailing punctuation, used to determine replacement punctuation (comma-continued -> preserve comma)
        r"([.,，。]?)"
    ))
    .unwrap()
});

/// Fast pre-filter markers for identity declaration (matches any before running full regex on large prompts)
const IDENTITY_FAST_MARKERS: [&str; 13] = [
    "created by",
    "built by",
    "built on",
    "made by",
    "developed by",
    "designed by",
    "trained by",
    "powered by",
    "maintained by",
    "published by",
    "released by",
    "provided by",
    "based on",
];

/// Claude Agent SDK injected standalone identity block (exact match)
const CLAUDE_AGENT_SDK_IDENTITY: &str =
    "You are a Claude agent, built on Anthropic's Claude Agent SDK.";
/// Normalization target: Claude Code CLI identity. Deliberately preserved as mapping to known client identity -
/// upstream is more stable with this identity.
const CLAUDE_CODE_CLI_IDENTITY: &str = "You are Claude Code, Anthropic's official CLI for Claude.";

/// Claude Desktop injected single-line billing metadata prefix (issue #3452: causes upstream 429 when combined with many tools).
/// Model-agnostic: any client-injected risky metadata must be cleaned.
const BILLING_METADATA_PREFIX: &str = "x-anthropic-billing-header:";

/// Pipeline system prompt boundary markers (stripped if mixed in, avoiding upstream pollution)
const SYSTEM_PROMPT_END_MARKERS: [&str; 2] = ["--- [SYSTEM_PROMPT_END] ---", "[SYSTEM_PROMPT_END]"];

/// Pipeline official identity text (stripped if mixed in, avoiding duplicate identity declarations)
const SELF_IDENTITY_BLOCK: &str = "You are Antigravity, a powerful agentic AI coding assistant designed by the Google Deepmind team working on Advanced Agentic Coding.\nYou are pair programming with a USER to solve their coding task. The task may require creating a new codebase, modifying or debugging an existing codebase, or simply answering a question.\n**Absolute paths only**\n**Proactiveness**";

pub struct PromptSanitizer;

impl PromptSanitizer {
    /// High-precision prompt text sanitization:
    /// 1. Fast check for high-risk WAF trigger keywords (`billing`, `cc_version`, `cc_entrypoint`), returning early if absent;
    /// 2. Extract and protect all code blocks from regex modifications;
    /// 3. Strip only WAF-triggering billing pseudo-headers, preserving session IDs, XML tags, and `=== ... ===` delimiters;
    /// 4. Fully restore protected code blocks;
    /// 5. Normalize newlines, preserving multi-line paragraph layouts.
    pub fn clean_text(text: &str) -> String {
        // Fast pre-check: return original text with zero overhead if no high-risk features detected
        let lower = text.to_lowercase();
        let has_suspect = lower.contains("billing")
            || lower.contains("cc_version")
            || lower.contains("cc_entrypoint")
            || lower.contains("cch=")
            || lower.contains("claude agent sdk");

        if !has_suspect {
            return text.to_string();
        }

        // Step 1: Protect code blocks
        let mut placeholders: Vec<String> = Vec::new();
        let protected_text = RE_CODE_BLOCK.replace_all(text, |caps: &regex::Captures| {
            let idx = placeholders.len();
            placeholders.push(caps[0].to_string());
            format!("__PROMPT_SANITIZER_CODE_BLOCK_{}__", idx)
        });

        // Step 2: Strip WAF trigger pseudo-headers only from non-code regions
        let pass1 = RE_WAF_TRIGGER_HEADERS.replace_all(&protected_text, "");

        // Step 3: Restore protected code blocks
        let mut restored = pass1.into_owned();
        for (idx, original_code) in placeholders.iter().enumerate() {
            let ph = format!("__PROMPT_SANITIZER_CODE_BLOCK_{}__", idx);
            restored = restored.replace(&ph, original_code);
        }

        // Step 4: Collapse redundant consecutive empty lines to maintain 2-newline paragraph structure
        let normalized = RE_MULTI_NEWLINE.replace_all(&restored, "\n\n");

        normalized.trim().to_string()
    }

    /// Computes the system prompt head window: returns a slice covering the first `IDENTITY_SCAN_MAX_SENTENCES` sentences.
    /// Sentence boundaries = '.' '!' '?' or empty line (two consecutive newlines); normal line breaks do not count as sentence boundaries.
    fn identity_head_region(text: &str) -> &str {
        let mut sentence_count = 0usize;
        let mut prev_newline = false;

        for (idx, ch) in text.char_indices() {
            let is_newline = ch == '\n';
            let is_terminator = matches!(ch, '.' | '!' | '?' | '。' | '！' | '？');

            if is_terminator || (is_newline && prev_newline) {
                sentence_count += 1;
                if sentence_count >= IDENTITY_SCAN_MAX_SENTENCES {
                    return &text[..idx + ch.len_utf8()];
                }
            }

            if is_newline {
                prev_newline = true;
            } else if ch != '\r' {
                prev_newline = false;
            }
        }

        text
    }

    /// System track: Normalizes identity attribution declaration sentences within system prompt head window to neutral identity.
    ///
    /// Normalization rules: Matching declaration sentence is replaced with
    /// `NEUTRAL_IDENTITY`; if followed by a comma continuation, preserves comma and keeps following text intact.
    /// Code blocks are 100% exempt, and text following the head window is preserved verbatim.
    pub fn normalize_system_identity(text: &str) -> String {
        // Fast pre-check: zero-overhead return if no attribution markers present
        let lower = text.to_lowercase();
        if !IDENTITY_FAST_MARKERS
            .iter()
            .any(|marker| lower.contains(marker))
        {
            return text.to_string();
        }

        // Only head window participates in identity normalization (body text references exempt)
        let head = Self::identity_head_region(text);
        if !RE_IDENTITY_DECLARATION.is_match(head) {
            return text.to_string();
        }

        // Step 1: Protect code blocks in head window from identity normalization
        let mut placeholders: Vec<String> = Vec::new();
        let protected_head = RE_CODE_BLOCK.replace_all(head, |caps: &regex::Captures| {
            let idx = placeholders.len();
            placeholders.push(caps[0].to_string());
            format!("__PROMPT_SANITIZER_CODE_BLOCK_{}__", idx)
        });

        // Step 2: Normalize identity statement (preserving comma if present)
        let normalized = RE_IDENTITY_DECLARATION
            .replace_all(&protected_head, |caps: &regex::Captures| {
                match caps.get(1).map(|m| m.as_str()).unwrap_or("") {
                    "," | "，" => "You are an AI Agent,".to_string(),
                    _ => NEUTRAL_IDENTITY.to_string(),
                }
            })
            .into_owned();

        // Step 3: Restore protected code blocks and append remaining text after head window
        let mut restored = normalized;
        for (idx, original_code) in placeholders.iter().enumerate() {
            let ph = format!("__PROMPT_SANITIZER_CODE_BLOCK_{}__", idx);
            restored = restored.replace(&ph, original_code);
        }

        restored.push_str(&text[head.len()..]);
        restored
    }

    /// Sanitizes all text nodes within parts array while strictly guaranteeing:
    /// 1. Thought blocks are protected by thoughtSignature and remain byte-level immutable;
    /// 2. Physically prunes empty text parts (preventing upstream 400/429 errors);
    /// 3. Enforces thought block at index 0 if present;
    /// 4. `system_scope` determines whether System track (identity normalization) is applied - true only for systemInstruction.
    /// Client identity normalization: Exact whole-string match mapping to target identity.
    /// Different semantics from `normalize_system_identity`: deliberately preserves mapping to known client identity.
    pub fn normalize_client_identity(text: &str) -> &str {
        if text == CLAUDE_AGENT_SDK_IDENTITY {
            CLAUDE_CODE_CLI_IDENTITY
        } else {
            text
        }
    }

    /// Checks if text is client-injected single-line billing metadata (issue #3452).
    /// Model-agnostic: risky metadata comes from client injection, treated as risky under any model or protocol.
    pub fn is_billing_metadata(text: &str) -> bool {
        let t = text.trim();
        t.starts_with(BILLING_METADATA_PREFIX) && !t.contains('\n') && !t.contains('\r')
    }

    /// Strips pipeline internal prompt boundary markers and official identity text.
    /// Note: does not include `clean_text` as Header track is executed in `sanitize_parts_core`.
    pub fn strip_pipeline_markers(text: &str) -> String {
        let mut s = text.to_string();
        for marker in SYSTEM_PROMPT_END_MARKERS {
            if s.contains(marker) {
                s = s.replace(marker, "");
            }
        }
        if s.contains(SELF_IDENTITY_BLOCK) {
            s = s.replace(SELF_IDENTITY_BLOCK, "");
        }
        s.trim().to_string()
    }

    fn sanitize_parts_core(parts: &mut Vec<Value>, system_scope: bool) -> usize {
        let mut cleaned_count = 0;

        // 0. Physically prune client-injected billing metadata parts (issue #3452)
        //    Model-agnostic: removed completely if matched.
        let before = parts.len();
        parts.retain(|part| {
            let is_thought = part
                .get("thought")
                .and_then(Value::as_bool)
                .unwrap_or(false)
                || part.get("thoughtSignature").is_some()
                || part.get("thought_signature").is_some();
            if is_thought {
                return true;
            }
            match part.get("text").and_then(Value::as_str) {
                Some(t) => !Self::is_billing_metadata(t),
                None => true,
            }
        });
        cleaned_count += before - parts.len();

        for part in parts.iter_mut() {
            if let Some(obj) = part.as_object_mut() {
                // Thought blocks are strictly protected by thoughtSignature and must remain byte-level immutable
                if obj.get("thought").and_then(Value::as_bool).unwrap_or(false)
                    || obj.contains_key("thoughtSignature")
                {
                    continue;
                }

                if let Some(text_val) = obj.get("text").and_then(Value::as_str) {
                    // Client identity mapping (exact match) - system prompt only, user turn text preserved
                    let mapped = if system_scope {
                        Self::normalize_client_identity(text_val)
                    } else {
                        text_val
                    };
                    // Header track (applies to all text): billing pseudo-headers and risky client fingerprint declarations
                    let cleaned = Self::clean_text(mapped);
                    // System track (system prompt only): strip boundary markers / official identity + normalize attribution
                    let cleaned = if system_scope {
                        Self::normalize_system_identity(&Self::strip_pipeline_markers(&cleaned))
                    } else {
                        cleaned
                    };
                    if cleaned != text_val {
                        obj.insert("text".to_string(), Value::String(cleaned));
                        cleaned_count += 1;
                    }
                }
            }
        }

        // Physically prune empty text parts generated after sanitization
        // Note: thought blocks and non-text parts (inlineData, functionCall, etc.) must remain intact
        parts.retain(|part| {
            let is_thought = part
                .get("thought")
                .and_then(Value::as_bool)
                .unwrap_or(false)
                || part.get("thoughtSignature").is_some()
                || part.get("thought_signature").is_some();
            if is_thought {
                return true;
            }
            if let Some(text) = part.get("text").and_then(Value::as_str) {
                !text.trim().is_empty()
            } else {
                true
            }
        });

        // Rule: ensure thought block is strictly at index 0 if present in current turn
        Self::ensure_thought_block_first(parts);

        cleaned_count
    }

    /// Content track sanitization: Header track only, NO identity normalization.
    /// Used for `contents` (user / model turns) - strictly preserves user queries.
    pub fn sanitize_parts(parts: &mut Vec<Value>) -> usize {
        Self::sanitize_parts_core(parts, false)
    }

    /// System track sanitization: Header track + System track, used only for `systemInstruction`.
    pub fn sanitize_system_parts(parts: &mut Vec<Value>) -> usize {
        Self::sanitize_parts_core(parts, true)
    }

    /// Ensures thought block is strictly at index 0 of parts array
    pub fn ensure_thought_block_first(parts: &mut Vec<Value>) {
        if parts.len() <= 1 {
            return;
        }

        let thought_idx = parts.iter().position(|p| {
            p.get("thought").and_then(Value::as_bool).unwrap_or(false)
                || ((p.get("thoughtSignature").is_some() || p.get("thought_signature").is_some())
                    && p.get("functionCall").is_none()
                    && p.get("functionResponse").is_none())
        });

        if let Some(idx) = thought_idx {
            if idx != 0 {
                let thought_part = parts.remove(idx);
                parts.insert(0, thought_part);
            }
        }
    }

    /// Pipeline node: Sanitizes systemInstruction and contents within proxy payload
    /// Supports top-level Gemini body as well as payload wrapped under `request` key.
    pub fn sanitize_gemini_payload(body: &mut Value) -> usize {
        let mut total_cleaned = 0;

        // 0. Deep clean '[undefined]' placeholder strings (injected by clients like Cherry Studio)
        //    Executed uniformly across all protocols.
        crate::proxy::mappers::common_utils::deep_clean_undefined(body, 0);

        // Compatibility: sanitize inner object if wrapped under 'request'
        if let Some(inner) = body.get_mut("request").and_then(Value::as_object_mut) {
            let mut inner_val = Value::Object(inner.clone());
            let count = Self::sanitize_gemini_payload_inner(&mut inner_val);
            if let Value::Object(new_inner) = inner_val {
                *inner = new_inner;
            }
            total_cleaned += count;
        }

        // Sanitize current level
        total_cleaned += Self::sanitize_gemini_payload_inner(body);
        total_cleaned
    }

    fn sanitize_gemini_payload_inner(body: &mut Value) -> usize {
        let mut cleaned_count = 0;

        // 1. Sanitize system prompt (systemInstruction)
        //    Executes System track: Header track + identity normalization
        if let Some(sys) = body
            .get_mut("systemInstruction")
            .and_then(Value::as_object_mut)
        {
            if let Some(parts) = sys.get_mut("parts").and_then(Value::as_array_mut) {
                cleaned_count += Self::sanitize_system_parts(parts);
            }

            // If parts is empty after sanitization, remove systemInstruction to avoid sending empty structure to Google
            let is_parts_empty = sys
                .get("parts")
                .and_then(Value::as_array)
                .map(|p| p.is_empty())
                .unwrap_or(true);

            if is_parts_empty {
                body.as_object_mut().map(|b| b.remove("systemInstruction"));
            }
        }

        // 2. Sanitize all dialogue turns (contents, including user and model turns)
        if let Some(contents) = body.get_mut("contents").and_then(Value::as_array_mut) {
            for turn in contents.iter_mut() {
                if let Some(parts) = turn.get_mut("parts").and_then(Value::as_array_mut) {
                    cleaned_count += Self::sanitize_parts(parts);
                }
            }

            // Turn protection: prune turns where parts became empty to prevent malformed payload
            contents.retain(|turn| {
                turn.get("parts")
                    .and_then(Value::as_array)
                    .map(|p| !p.is_empty())
                    .unwrap_or(true)
            });
        }

        cleaned_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_client_identity_mapping_is_preserved_not_neutralized() {
        // Exact match mapping: must become Claude Code CLI identity
        assert_eq!(
            PromptSanitizer::normalize_client_identity(CLAUDE_AGENT_SDK_IDENTITY),
            CLAUDE_CODE_CLI_IDENTITY
        );
        assert_eq!(
            PromptSanitizer::normalize_client_identity("You are an expert coder."),
            "You are an expert coder."
        );
    }

    #[test]
    fn test_strip_pipeline_markers_and_self_identity() {
        assert_eq!(
            PromptSanitizer::strip_pipeline_markers("Rule A\n--- [SYSTEM_PROMPT_END] ---\nRule B"),
            "Rule A\n\nRule B"
        );
        assert_eq!(
            PromptSanitizer::strip_pipeline_markers(&format!(
                "Intro\n{SELF_IDENTITY_BLOCK}\nOutro"
            )),
            "Intro\n\nOutro"
        );
    }

    #[test]
    fn test_billing_metadata_detection_is_model_agnostic() {
        assert!(PromptSanitizer::is_billing_metadata(
            "x-anthropic-billing-header: cc_version=2.1.270.ffc;"
        ));
        // Multi-line (real prompts) must not be flagged as risky
        assert!(!PromptSanitizer::is_billing_metadata(
            "x-anthropic-billing-header: a;\nSecond line prompt instruction"
        ));
        assert!(!PromptSanitizer::is_billing_metadata(
            "You are an expert coder."
        ));
    }

    #[test]
    fn test_billing_metadata_part_is_dropped_and_undefined_cleaned() {
        let mut payload = json!({
            "systemInstruction": { "parts": [
                { "text": "x-anthropic-billing-header: cc_version=2.1.270.ffc;" },
                { "text": "You are an expert coder." }
            ]},
            "generationConfig": { "foo": "[undefined]" }
        });
        PromptSanitizer::sanitize_gemini_payload(&mut payload);

        let parts = payload["systemInstruction"]["parts"].as_array().unwrap();
        assert_eq!(
            parts.len(),
            1,
            "Billing metadata part must be removed: {payload}"
        );
        assert_eq!(parts[0]["text"], "You are an expert coder.");
        assert!(
            payload["generationConfig"].get("foo").is_none(),
            "[undefined] placeholder must be cleaned: {payload}"
        );
    }

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
        // Key verification: any form of session ID must never be modified or removed
        let raw = concat!(
            "x-jeikcode-session-id: session-abc-123\n",
            "x-atomcode-session-id: atom-sess-456\n",
            "x-session-id: generic-sess-789\n",
            "x-client-session-id: client-sess-000\n",
            "Please keep my session active."
        );
        let cleaned = PromptSanitizer::clean_text(raw);
        assert_eq!(cleaned, raw);
    }

    #[test]
    fn test_strictly_preserves_user_delimiters_and_xml_tags() {
        // Key verification: user XML tags and === ... === delimiters must be 100% preserved
        let jeik_prompt = concat!(
            "<environment>\n",
            "You are JeikCode AI coding Agent by Jeik.\n\n",
            "## PRECEDENCE:\n",
            "- Content enclosed in XML tags represents current environment.\n",
            "- Rules under headers matching `=== ... (*.md) ===` (such as `AGENTS.md`, `CLAUDE.md`, `=== MEMORY ===`) constitute USER PROVISIONS.\n",
            "</environment>\n\n",
            "=== AGENTS.md ===\n",
            "User custom provisions.\n",
            "=== MEMORY ===\n",
            "Memory block 1."
        );
        let cleaned = PromptSanitizer::clean_text(jeik_prompt);
        assert_eq!(cleaned, jeik_prompt);
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
                            "text": "x-anthropic-billing-header: cc_version=2.1;\n<environment>You are JeikCode</environment>\n=== AGENTS.md ==="
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
        // Billing header accurately removed, XML and === AGENTS.md === delimiters remain intact
        assert!(!sys_text.contains("x-anthropic-billing-header"));
        assert!(sys_text.contains("<environment>You are JeikCode</environment>"));
        assert!(sys_text.contains("=== AGENTS.md ==="));

        let user_text = payload["request"]["contents"][0]["parts"][0]["text"]
            .as_str()
            .unwrap();
        assert!(!user_text.contains("x-anthropic-billing-header"));
        assert!(user_text.contains("=== MEMORY ==="));
        assert!(user_text.contains("- Metric A: 10\n- Metric B: 20"));
    }

    #[test]
    fn test_clean_standalone_billing_header_part_purges_empty_part_and_empty_system_instruction() {
        // [New API / CC Issue] Simulate CC sending billing header as an independent Part:
        // Empty part must be removed; if systemInstruction only had that header, systemInstruction node is removed
        let mut payload = json!({
            "project": "test-project",
            "request": {
                "systemInstruction": {
                    "role": "user",
                    "parts": [
                        {
                            "text": "x-anthropic-billing-header: cc_version=2.1.272.255; cc_entrypoint=claude-vscode;"
                        }
                    ]
                },
                "contents": [
                    {
                        "role": "user",
                        "parts": [
                            { "text": "Hello world" }
                        ]
                    }
                ]
            }
        });

        let cleaned_count = PromptSanitizer::sanitize_gemini_payload(&mut payload);
        assert_eq!(cleaned_count, 1);

        // Verify: systemInstruction node completely removed when parts is empty, avoiding {"text": ""} anomaly
        assert!(payload["request"].get("systemInstruction").is_none());
        assert_eq!(
            payload["request"]["contents"][0]["parts"][0]["text"],
            "Hello world"
        );
    }

    #[test]
    fn test_ensure_thought_block_always_first_after_sanitization() {
        // [Rule verification] Thinking block must be strictly at index 0 of parts after sanitization
        let mut payload = json!({
            "project": "test-project",
            "request": {
                "contents": [
                    {
                        "role": "model",
                        "parts": [
                            {
                                "text": "x-anthropic-billing-header: cc_version=2.1.272.255; cc_entrypoint=cli;\nSome commentary."
                            },
                            {
                                "text": "I am thinking deeply about the problem...",
                                "thought": true,
                                "thoughtSignature": "valid_hmac_signature_123456"
                            },
                            {
                                "text": "Final answer text."
                            }
                        ]
                    }
                ]
            }
        });

        let _ = PromptSanitizer::sanitize_gemini_payload(&mut payload);

        let parts = payload["request"]["contents"][0]["parts"]
            .as_array()
            .expect("parts should be array");

        // Thinking block must be reordered to index 0
        assert_eq!(parts[0]["thought"], true);
        assert_eq!(
            parts[0]["text"],
            "I am thinking deeply about the problem..."
        );
        assert_eq!(parts[0]["thoughtSignature"], "valid_hmac_signature_123456");

        // Other non-thought parts follow after and are properly sanitized
        assert_eq!(parts[1]["text"], "Some commentary.");
        assert_eq!(parts[2]["text"], "Final answer text.");
    }

    #[test]
    fn test_clean_claude_agent_sdk_preamble() {
        // [WAF 429 Bypass] Verify Claude Agent SDK leading fingerprint declaration is stripped
        let raw = concat!(
            "You are a Claude agent, built on Anthropic's Claude Agent SDK.\n\n",
            "You have access to a variety of tools."
        );
        let cleaned = PromptSanitizer::clean_text(raw);
        assert!(!cleaned.contains("Claude Agent SDK"));
        assert_eq!(cleaned, "You have access to a variety of tools.");
    }

    #[test]
    fn test_clean_mid_paragraph_billing_header() {
        // [WAF 429 Bypass] Verify billing header embedded mid-paragraph is stripped
        let raw = "Intro text x-anthropic-billing-header: cc_version=2.1.278; cc_entrypoint=cli; cch=fa690; and more text";
        let cleaned = PromptSanitizer::clean_text(raw);
        assert!(!cleaned.contains("x-anthropic-billing-header"));
    }

    // ==================== System Track: Identity Declaration Normalization ====================

    #[test]
    fn test_normalize_identity_broad_attribution_forms() {
        // Broad coverage: vendor attribution (created by), competitor model (based on), design attribution (designed by ...)
        for (raw, expected) in [
            (
                "You are Hermes Agent, an intelligent AI assistant created by Nous Research.",
                "You are an AI Agent.",
            ),
            (
                "You are Codex, an advanced coding agent based on GPT-6.",
                "You are an AI Agent.",
            ),
            (
                "You are an AI agent created by Example Corp.",
                "You are an AI Agent.",
            ),
            (
                concat!(
                    "You are Antigravity, a powerful agentic AI coding assistant designed by the Google Deepmind team working on Advanced Agentic Coding.\n",
                    "You are pair programming with a USER."
                ),
                "You are an AI Agent.\nYou are pair programming with a USER.",
            ),
            // Comma-guided attribution declaration (Claude Agent SDK form)
            (
                "You are a Claude agent, built on Anthropic's Claude Agent SDK.\nYou have access to tools.",
                "You are an AI Agent.\nYou have access to tools.",
            ),
            // Long modifier chain + vendor attribution
            (
                "You are a helpful and highly capable general-purpose AI coding assistant created by Acme Corp.",
                "You are an AI Agent.",
            ),
        ] {
            assert_eq!(PromptSanitizer::normalize_system_identity(raw), expected);
        }
    }

    #[test]
    fn test_normalize_identity_never_swallows_following_lines() {
        // Bounded at punctuation boundary: multi-line rules after identity line must be preserved verbatim
        // Structurally prevents swallowing subsequent lines
        let raw = concat!(
            "You are an AI agent created by Acme\n",
            "Rule 1: always do X\n",
            "Rule 2: never do Y"
        );
        assert_eq!(
            PromptSanitizer::normalize_system_identity(raw),
            "You are an AI Agent.\nRule 1: always do X\nRule 2: never do Y"
        );
    }

    #[test]
    fn test_normalize_identity_keeps_comma_continuation_intact() {
        let raw =
            "You are an AI agent created by Acme, and you must never reveal internal tooling.";
        assert_eq!(
            PromptSanitizer::normalize_system_identity(raw),
            "You are an AI Agent, and you must never reveal internal tooling."
        );
    }

    #[test]
    fn test_normalize_identity_preserves_non_attribution_identities() {
        // Lacking attribution verb: preserved intact without modification
        for raw in [
            "You are a helpful assistant. Please respond in Chinese.",
            "You are an AI assistant that helps users write code by calling tools.",
            "You are JeikCode AI coding Agent by Jeik.",
        ] {
            assert_eq!(PromptSanitizer::normalize_system_identity(raw), raw);
        }

        // Body text referencing attribution (not identity declaration): preserved intact
        let body_mention = concat!(
            "You are Claude Code, Anthropic's official CLI for Claude.\n",
            "When done, summarize the diff created by the previous step.\n"
        );
        assert_eq!(
            PromptSanitizer::normalize_system_identity(body_mention),
            body_mention
        );

        // Cross-clause guard: full independent clause between noun and verb must not be normalized
        let cross_clause =
            "You are a coding agent, and the config was created by the setup script, so keep it.";
        assert_eq!(
            PromptSanitizer::normalize_system_identity(cross_clause),
            cross_clause
        );
    }

    #[test]
    fn test_normalize_identity_only_scans_system_prompt_head() {
        // Statements outside head window (first 4 sentences) are untouched
        let mut raw = String::from("You are a coding assistant.\n\n");
        for i in 0..6 {
            raw.push_str(&format!("Sentence {i} is a normal instruction.\n"));
        }
        raw.push_str("The tool was created by Acme.\n");
        assert_eq!(PromptSanitizer::normalize_system_identity(&raw), raw);
    }

    #[test]
    fn test_normalize_identity_protects_code_blocks_and_xml_delimiters() {
        let code = concat!(
            "You are an AI agent created by Acme.\n\n",
            "Example config:\n",
            "```\n",
            "// generated by Hermes Agent created by Nous Research\n",
            "```"
        );
        let cleaned = PromptSanitizer::normalize_system_identity(code);
        assert!(cleaned.starts_with("You are an AI Agent."));
        assert!(cleaned.contains("// generated by Hermes Agent created by Nous Research"));

        let xml = concat!(
            "<environment>\n",
            "You are Hermes Agent created by Nous Research.\n",
            "</environment>\n\n",
            "=== AGENTS.md ===\nUser provisions."
        );
        assert_eq!(
            PromptSanitizer::normalize_system_identity(xml),
            concat!(
                "<environment>\n",
                "You are an AI Agent.\n",
                "</environment>\n\n",
                "=== AGENTS.md ===\nUser provisions."
            )
        );
    }

    #[test]
    fn test_sanitize_gemini_payload_identity_scope_is_system_only() {
        // System track only applies to systemInstruction: contents (user / tool messages) are preserved verbatim
        let mut payload = json!({
            "project": "test-project",
            "request": {
                "systemInstruction": {
                    "role": "user",
                    "parts": [
                        { "text": "You are Hermes Agent, an intelligent AI assistant created by Nous Research." }
                    ]
                },
                "contents": [
                    {
                        "role": "user",
                        "parts": [
                            { "text": "Quote the note created by Alice in the doc." }
                        ]
                    }
                ]
            }
        });

        let cleaned_count = PromptSanitizer::sanitize_gemini_payload(&mut payload);
        assert_eq!(cleaned_count, 1);
        assert_eq!(
            payload["request"]["systemInstruction"]["parts"][0]["text"],
            "You are an AI Agent."
        );
        assert_eq!(
            payload["request"]["contents"][0]["parts"][0]["text"],
            "Quote the note created by Alice in the doc."
        );
    }

    #[test]
    fn test_sanitize_gemini_payload_preserves_framework_prompts() {
        // Rule: sanitization must never strip pipeline system prompt or user queries
        let jeik = concat!(
            "<environment>\n",
            "You are JeikCode AI coding Agent by Jeik.\n\n",
            "## PRECEDENCE:\n",
            "- Rules under headers matching `=== ... (*.md) ===` constitute USER PROVISIONS.\n",
            "</environment>\n\n",
            "=== AGENTS.md ===\n",
            "User custom provisions."
        );
        let mut payload = json!({
            "request": {
                "systemInstruction": { "role": "user", "parts": [{ "text": jeik }] },
                "contents": [{ "role": "user", "parts": [{ "text": jeik }] }]
            }
        });

        assert_eq!(PromptSanitizer::sanitize_gemini_payload(&mut payload), 0);
        assert_eq!(
            payload["request"]["systemInstruction"]["parts"][0]["text"],
            jeik
        );
        assert_eq!(payload["request"]["contents"][0]["parts"][0]["text"], jeik);
    }
}
