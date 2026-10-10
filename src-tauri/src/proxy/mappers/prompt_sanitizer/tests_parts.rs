// Parts-method tests (split from prompt_sanitizer.rs)
use super::PromptSanitizer;
use serde_json::json;

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

    // Justification: non-Result return value intentionally discarded — no error channel to track
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
