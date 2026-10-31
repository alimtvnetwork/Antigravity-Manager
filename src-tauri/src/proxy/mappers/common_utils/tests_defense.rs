// Defense tests (split from common_utils.rs).
// Test-only module, declared #[cfg(test)] in mod.rs.
use super::*;

#[test]
fn test_ensure_gemini_payload_ends_with_user_empty() {
    let mut payload = json!({
        "contents": []
    });
    assert!(ensure_gemini_payload_ends_with_user(&mut payload));
    let contents = payload["contents"].as_array().unwrap();
    assert_eq!(contents.len(), 1);
    assert_eq!(contents[0]["role"], "user");
    assert_eq!(
        contents[0]["parts"][0]["text"],
        TRANSIT_DEFENSE_FALLBACK_TEXT
    );
}

#[test]
fn test_ensure_gemini_payload_ends_with_user_model_ending() {
    let mut payload = json!({
        "request": {
            "contents": [
                { "role": "user", "parts": [{ "text": "hello" }] },
                { "role": "model", "parts": [{ "text": "hi there" }] }
            ]
        }
    });
    assert!(ensure_gemini_payload_ends_with_user(&mut payload));
    let contents = payload["request"]["contents"].as_array().unwrap();
    assert_eq!(contents.len(), 3);
    assert_eq!(contents[2]["role"], "user");
    assert_eq!(
        contents[2]["parts"][0]["text"],
        TRANSIT_DEFENSE_FALLBACK_TEXT
    );
}

#[test]
fn test_ensure_gemini_payload_ends_with_user_no_content() {
    let mut payload = json!({
        "contents": [
            { "role": "user", "parts": [{ "text": "(no content)" }] }
        ]
    });
    assert!(ensure_gemini_payload_ends_with_user(&mut payload));
    let contents = payload["contents"].as_array().unwrap();
    assert_eq!(contents.len(), 1);
    assert_eq!(
        contents[0]["parts"][0]["text"],
        TRANSIT_DEFENSE_FALLBACK_TEXT
    );
}

#[test]
fn test_ensure_gemini_payload_ends_with_user_valid_untouched() {
    let mut payload = json!({
        "contents": [
            { "role": "user", "parts": [{ "text": "valid message" }] }
        ]
    });
    assert!(!ensure_gemini_payload_ends_with_user(&mut payload));
    let contents = payload["contents"].as_array().unwrap();
    assert_eq!(contents[0]["parts"][0]["text"], "valid message");
}

#[test]
fn test_ensure_gemini_payload_ends_with_user_tool_turn_not_injected() {
    // The model wheel at the end is the tool wheel (functionCall) → legal intermediate state, no fake user words are injected
    let mut payload = json!({
        "contents": [
            { "role": "user", "parts": [{ "text": "run the tool" }] },
            {
                "role": "model",
                "parts": [
                    { "thought": true, "text": "I should call a tool." },
                    {
                        "functionCall": {
                            "id": "call_1",
                            "name": "run_command",
                            "args": { "CommandLine": "echo hi" }
                        },
                        "thoughtSignature": "AABBCC"
                    }
                ]
            }
        ]
    });
    assert!(!ensure_gemini_payload_ends_with_user(&mut payload));
    let contents = payload["contents"].as_array().unwrap();
    assert_eq!(contents.len(), 2);

    // The model wheel at the end is pure text (not a tool wheel) → still injected (original semantics retained)
    let mut payload2 = json!({
        "contents": [
            { "role": "user", "parts": [{ "text": "hello" }] },
            { "role": "model", "parts": [{ "text": "hi there" }] }
        ]
    });
    assert!(ensure_gemini_payload_ends_with_user(&mut payload2));
    let contents2 = payload2["contents"].as_array().unwrap();
    assert_eq!(contents2.len(), 3);
    assert_eq!(contents2[2]["role"], "user");
}

#[test]
fn test_wrap_in_system_reminder() {
    use super::wrap_in_system_reminder;

    // Empty content returns empty string
    assert_eq!(wrap_in_system_reminder("   "), "");

    // Raw text gets wrapped with English reminder header
    let wrapped = wrap_in_system_reminder("Current date: 2026-09-19");
    assert!(wrapped.starts_with("<system-reminder>\nBefore the user's request for this turn"));
    assert!(wrapped.contains("Current date: 2026-09-19"));
    assert!(wrapped.ends_with("</system-reminder>"));

    // Already wrapped content is untouched (no double wrapping)
    let already = "<system-reminder>\nsome text\n</system-reminder>";
    assert_eq!(wrap_in_system_reminder(already), already);
}

#[test]
fn test_safe_truncate_str_utf8_boundaries() {
    // "Hello World" Each Chinese character is 3 bytes, totaling 12 bytes:
    // 'you': 0..3, '好': 3..6, '世': 6..9, '世界': 9..12
    let text = "hello world";
    assert_eq!(safe_truncate_str(text, 0), "");
    assert_eq!(safe_truncate_str(text, 1), ""); // falls between 'you', falls back to 0
    assert_eq!(safe_truncate_str(text, 2), ""); // falls between 'you', falls back to 0
    assert_eq!(safe_truncate_str(text, 3), "你");
    assert_eq!(safe_truncate_str(text, 4), "你"); // 落在 '好' 中间，回退到 3
    assert_eq!(safe_truncate_str(text, 5), "你");
    assert_eq!(safe_truncate_str(text, 6), "hello");
    assert_eq!(safe_truncate_str(text, 12), "hello world");
    assert_eq!(safe_truncate_str(text, 100), "hello world");

    // Verification Issue #3493 Scenario: The 57th byte falls inside a 3-byte Chinese character
    // Construct 55 bytes ASCII + "Chinese Test" (3 bytes each)
    // "Chinese test" starts at index 55: '中' (55..58)
    // Index 57 falls right in the middle of 'middle' (55..58)
    let mut s3493 = "a".repeat(55);
    s3493.push_str("Chinese test");
    assert!(!s3493.is_char_boundary(57));
    let truncated = safe_truncate_str(&s3493, 57);
    assert_eq!(truncated.len(), 55);
    assert_eq!(truncated, "a".repeat(55));

    // Emoji test (4 bytes: 🦀 0..4)
    let emoji = "🦀🦀";
    assert_eq!(safe_truncate_str(emoji, 2), "");
    assert_eq!(safe_truncate_str(emoji, 4), "🦀");
    assert_eq!(safe_truncate_str(emoji, 6), "🦀");
    assert_eq!(safe_truncate_str(emoji, 8), "🦀🦀");
}

#[test]
fn test_safe_truncate_chars_utf8() {
    let text = "Hello world, Rust programming!";
    assert_eq!(safe_truncate_chars(text, 0), "");
    assert_eq!(safe_truncate_chars(text, 2), "hello");
    assert_eq!(safe_truncate_chars(text, 4), "hello world");
    assert_eq!(safe_truncate_chars(text, 5), "hello world,");
    assert_eq!(safe_truncate_chars(text, 100), text);

    let emoji_text = "🎉Hello World🦀";
    assert_eq!(safe_truncate_chars(emoji_text, 1), "🎉");
    assert_eq!(safe_truncate_chars(emoji_text, 6), "🎉Hello");
    assert_eq!(safe_truncate_chars(emoji_text, 8), "🎉Hello world");
    assert_eq!(safe_truncate_chars(emoji_text, 9), "🎉Hello World🦀");
}

#[test]
fn test_extract_multimodal_strictly_respects_image_whitelist_and_rejects_audio_and_files() {
    let fake_png_b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

    // 1. Legal whitelist images (PNG): Normal extraction
    let mut extra_parts = Vec::new();
    let valid_img_text = format!("![screen](data:image/png;base64,{})", fake_png_b64);
    let res = extract_multimodal_from_tool_text(&valid_img_text, &mut extra_parts);
    assert_eq!(extra_parts.len(), 1);
    assert_eq!(extra_parts[0]["inlineData"]["mimeType"], "image/png");
    assert!(res.contains("[Image: forwarded to visual input (image/png)]"));

    // 2. Audio files (data:audio/mp3): absolutely not extracted, kept as is and transparently transmitted
    let mut audio_parts = Vec::new();
    let audio_text = "Audio output: data:audio/mp3;base64,SUQzBAAAAAAAI1RTU0UAAAAPAAADTGF2ZjU4Ljc2LjEwMAAAAAAAAAAAAAAA";
    let res_audio = extract_multimodal_from_tool_text(audio_text, &mut audio_parts);
    assert_eq!(
        audio_parts.len(),
        0,
        "Audio files must not be extracted as inlineData multimodal"
    );
    assert_eq!(
        res_audio, audio_text,
        "Audio text must be 100% original and transparent"
    );

    // 3. PDF/document file (data:application/pdf): Absolutely no extraction, keep the original and transparent transmission
    let mut pdf_parts = Vec::new();
    let pdf_text = "PDF doc: data:application/pdf;base64,JVBERi0xLjQKJcOkw7zDtsOfCjIgMCBvYmoKPDwKL0xlbmd0aCAzIDA";
    let res_pdf = extract_multimodal_from_tool_text(pdf_text, &mut pdf_parts);
    assert_eq!(
        pdf_parts.len(),
        0,
        "PDF/document files must not be extracted as inlineData multimodal"
    );
    assert_eq!(
        res_pdf, pdf_text,
        "PDF text must be 100% original and transparent"
    );

    // 4. Non-whitelisted image formats (SVG/TIFF): Absolutely not extracted and transparently transmitted as they are.
    let mut svg_parts = Vec::new();
    let svg_text = "Vector icon: data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjwvc3ZnPg==";
    let res_svg = extract_multimodal_from_tool_text(svg_text, &mut svg_parts);
    assert_eq!(
        svg_parts.len(),
        0,
        "SVG non-standard raster images must not be extracted as inlineData"
    );
    assert_eq!(res_svg, svg_text, "SVG must maintain original passthrough");

    // 5. Audio or unknown files in JSON format tool output: absolutely not extracted
    let mut json_audio_parts = Vec::new();
    let json_audio = r#"{"type":"audio","image":"data:audio/wav;base64,UklGRiQAAABXQVZFZm10IBAAAAABAAEAQB8AAEAfAAABAAgAZGF0YQAAAAA="}"#;
    let res_json = extract_multimodal_from_tool_text(json_audio, &mut json_audio_parts);
    assert_eq!(
        json_audio_parts.len(),
        0,
        "Audio fields in JSON must not be mistakenly extracted as multi-modal images"
    );
    assert_eq!(res_json, json_audio);
}
