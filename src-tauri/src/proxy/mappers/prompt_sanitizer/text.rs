// Text-level sanitization methods (split from prompt_sanitizer.rs)
use super::patterns::{
    BILLING_METADATA_PREFIX, CLAUDE_AGENT_SDK_IDENTITY, CLAUDE_CODE_CLI_IDENTITY,
    IDENTITY_FAST_MARKERS, IDENTITY_SCAN_MAX_SENTENCES, NEUTRAL_IDENTITY, RE_CODE_BLOCK,
    RE_IDENTITY_DECLARATION, RE_MULTI_NEWLINE, RE_WAF_TRIGGER_HEADERS, SELF_IDENTITY_BLOCK,
    SYSTEM_PROMPT_END_MARKERS,
};
use super::PromptSanitizer;
use serde_json::Value;

impl PromptSanitizer {
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
}
