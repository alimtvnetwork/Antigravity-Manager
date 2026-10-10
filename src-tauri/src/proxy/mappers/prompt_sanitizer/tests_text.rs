// Text-method tests (split from prompt_sanitizer.rs)
use super::patterns::{CLAUDE_AGENT_SDK_IDENTITY, CLAUDE_CODE_CLI_IDENTITY, SELF_IDENTITY_BLOCK};
use super::PromptSanitizer;

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
        PromptSanitizer::strip_pipeline_markers(&format!("Intro\n{SELF_IDENTITY_BLOCK}\nOutro")),
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
    let raw = "You are an AI agent created by Acme, and you must never reveal internal tooling.";
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
