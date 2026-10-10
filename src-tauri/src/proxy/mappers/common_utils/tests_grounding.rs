// common_utils tests (split from common_utils.rs).
// Test-only module, declared #[cfg(test)] in mod.rs.
use super::*;

#[test]
fn test_calculate_aspect_ratio_from_size() {
    // Test standard OpenAI sizes
    assert_eq!(calculate_aspect_ratio_from_size("1280x720"), "16:9");
    assert_eq!(calculate_aspect_ratio_from_size("1920x1080"), "16:9");
    assert_eq!(calculate_aspect_ratio_from_size("720x1280"), "9:16");
    assert_eq!(calculate_aspect_ratio_from_size("1080x1920"), "9:16");
    assert_eq!(calculate_aspect_ratio_from_size("1024x1024"), "1:1");
    assert_eq!(calculate_aspect_ratio_from_size("800x600"), "4:3");
    assert_eq!(calculate_aspect_ratio_from_size("600x800"), "3:4");
    assert_eq!(calculate_aspect_ratio_from_size("2560x1080"), "21:9");

    // [NEW] Test new aspect ratios
    assert_eq!(calculate_aspect_ratio_from_size("1500x1000"), "3:2");
    assert_eq!(calculate_aspect_ratio_from_size("1000x1500"), "2:3");
    assert_eq!(calculate_aspect_ratio_from_size("1250x1000"), "5:4");
    assert_eq!(calculate_aspect_ratio_from_size("1000x1250"), "4:5");

    // [NEW] Test direct aspect ratio strings
    assert_eq!(calculate_aspect_ratio_from_size("21:9"), "21:9");
    assert_eq!(calculate_aspect_ratio_from_size("16:9"), "16:9");
    assert_eq!(calculate_aspect_ratio_from_size("1:1"), "1:1");

    // Test edge cases
    assert_eq!(calculate_aspect_ratio_from_size("invalid"), "1:1");
    assert_eq!(calculate_aspect_ratio_from_size("1920x0"), "1:1");
    assert_eq!(calculate_aspect_ratio_from_size("0x1080"), "1:1");
    assert_eq!(calculate_aspect_ratio_from_size("abc x def"), "1:1");
}

#[test]
fn test_image_config_merging_priority() {
    // Case 1: Body contains empty/default imageSize, suffix contains -4k
    // Expected: Should KEEP 4K from suffix
    let body = json!({
        "generationConfig": {
            "imageConfig": {
                "aspectRatio": "1:1",
                "imageSize": "1K" // Simulated downgrade from client
            }
        }
    });
    let config = resolve_request_config(
        "gemini-3-pro-image-4k",
        "gemini-3-pro-image",
        &None,
        None,
        None,
        None,
        Some(&body),
    );
    let image_config = config.image_config.unwrap();
    assert_eq!(
        image_config["imageSize"], "4K",
        "Should shield inferred 4K from body downgrade"
    );
    assert_eq!(
        image_config["aspectRatio"], "1:1",
        "Should take aspectRatio from body"
    );

    // Case 2: Suffix contains -16-9, Body contains aspectRatio: 1:1
    // Expected: Body overrides suffix for aspectRatio (since it's not a 'downgrade' shield case yet, only size is shielded)
    let body_2 = json!({
        "generationConfig": {
            "imageConfig": {
                "aspectRatio": "1:1"
            }
        }
    });
    let config_2 = resolve_request_config(
        "gemini-3-pro-image-16x9",
        "gemini-3-pro-image",
        &None,
        None,
        None,
        None,
        Some(&body_2),
    );
    let image_config_2 = config_2.image_config.unwrap();
    assert_eq!(
        image_config_2["aspectRatio"], "1:1",
        "Body should be allowed to override aspectRatio"
    );
}

#[test]
fn test_image_size_priority() {
    // Case 1: imageSize param overrides quality
    // Expected: "4K" from imageSize param
    let (config_1, _) = parse_image_config_with_params(
        "gemini-3-pro-image",
        None,
        Some("standard"), // would be 1K
        Some("4K"),       // should override
    );
    assert_eq!(config_1["imageSize"], "4K");

    // Case 2: imageSize param overrides suffix
    // Expected: "2K" from imageSize param
    let (config_2, _) = parse_image_config_with_params(
        "gemini-3-pro-image-4k", // would be 4K
        None,
        None,
        Some("2K"), // should override
    );
    assert_eq!(config_2["imageSize"], "2K");

    // Case 3: imageSize param + size param + quality param
    // Expected: "4K" from imageSize, "16:9" from size
    let (config_3, _) = parse_image_config_with_params(
        "gemini-3-pro-image",
        Some("1920x1080"), // 16:9
        Some("standard"),  // 1K (ignored)
        Some("4K"),        // 4K (priority)
    );
    assert_eq!(config_3["imageSize"], "4K");
    assert_eq!(config_3["aspectRatio"], "16:9");
}

#[test]
fn image_quality_aliases_map_to_unified_image_sizes() {
    let cases = [
        ("low", "1K"),
        ("standard", "1K"),
        ("1k", "1K"),
        ("medium", "2K"),
        ("2k", "2K"),
        ("high", "4K"),
        ("hd", "4K"),
        ("4k", "4K"),
    ];
    for (quality, expected) in cases {
        let (config, _) =
            try_parse_image_config_with_params("gemini-3.1-flash-image", None, Some(quality), None)
                .expect("quality alias must parse");
        assert_eq!(config["imageSize"], expected, "quality={quality}");
    }
}

#[test]
fn image_size_priority_and_auto_fallback_are_enforced() {
    let (explicit, _) = try_parse_image_config_with_params(
        "gemini-3.1-flash-image-1k",
        None,
        Some("high"),
        Some("2k"),
    )
    .expect("case-insensitive explicit image size");
    assert_eq!(explicit["imageSize"], "2K");

    let (quality, _) =
        try_parse_image_config_with_params("gemini-3.1-flash-image-1k", None, Some("medium"), None)
            .expect("quality overrides suffix");
    assert_eq!(quality["imageSize"], "2K");

    for quality in [Some("auto"), Some(""), None] {
        let (fallback, _) = try_parse_image_config_with_params(
            "gemini-3.1-flash-image-4k",
            None,
            quality,
            Some("auto"),
        )
        .expect("auto values fall back to suffix");
        assert_eq!(fallback["imageSize"], "4K");
    }

    let (upstream_default, _) =
        try_parse_image_config_with_params("gemini-3.1-flash-image", None, Some("auto"), None)
            .expect("auto without suffix uses upstream default");
    assert!(upstream_default.get("imageSize").is_none());

    assert!(
        try_parse_image_config_with_params("gemini-3.1-flash-image", None, None, Some("8K"),)
            .is_err()
    );
}

#[test]
fn test_detect_mime_from_bytes() {
    assert_eq!(
        detect_mime_from_bytes(b"\x89PNG\r\n\x1a\n\0\0\0"),
        Some("image/png")
    );
    assert_eq!(
        detect_mime_from_bytes(b"\xff\xd8\xff\xe0\0\x10JFIF"),
        Some("image/jpeg")
    );
    assert_eq!(
        detect_mime_from_bytes(b"GIF89a\x01\0\x01\0"),
        Some("image/gif")
    );
    assert_eq!(
        detect_mime_from_bytes(b"RIFF\0\0\0\0WEBPVP8 "),
        Some("image/webp")
    );
    assert_eq!(
        detect_mime_from_bytes(b"%PDF-1.7\n%"),
        Some("application/pdf")
    );
    assert_eq!(detect_mime_from_bytes(b"invalid"), None);
}

#[test]
fn test_validate_and_sanitize_inline_data() {
    // 1. Empty data
    assert_eq!(
        validate_and_sanitize_inline_data(Some("image/png"), ""),
        None
    );
    assert_eq!(validate_and_sanitize_inline_data(None, "   "), None);

    // 2. Corrupted short data (like the +A== in the incident)
    assert_eq!(
        validate_and_sanitize_inline_data(Some("image/png"), "+A=="),
        None
    );
    assert_eq!(validate_and_sanitize_inline_data(None, "AQ=="), None);

    // 3. Invalid base64 characters
    assert_eq!(
        validate_and_sanitize_inline_data(Some("image/png"), "not-valid-base64!@#$"),
        None
    );

    // 4. Valid PNG base64 (8 bytes magic header)
    let valid_png_b64 = "iVBORw0KGgo=";
    let res = validate_and_sanitize_inline_data(Some("image/png"), valid_png_b64);
    assert!(res.is_some());
    let (mime, data) = res.unwrap();
    assert_eq!(mime, "image/png");
    assert_eq!(data, valid_png_b64);

    // 5. Valid PNG with omitted mime type (should auto-detect from magic bytes)
    let res_no_mime = validate_and_sanitize_inline_data(None, valid_png_b64);
    assert!(res_no_mime.is_some());
    assert_eq!(res_no_mime.unwrap().0, "image/png");
}

#[test]
fn test_create_gemini_inline_part() {
    let valid_png_b64 = "iVBORw0KGgo=";
    let valid_part = create_gemini_inline_part(Some("image/png"), valid_png_b64, "Image");
    assert!(valid_part.get("inlineData").is_some());
    assert_eq!(valid_part["inlineData"]["mimeType"], "image/png");

    let bad_part = create_gemini_inline_part(Some("image/png"), "+A==", "Image");
    assert!(bad_part.get("inlineData").is_none());
    assert_eq!(
        bad_part["text"],
        "[Image: invalid or corrupted data omitted]"
    );
}

#[test]
fn test_sanitize_gemini_payload_inline_data() {
    let valid_png_b64 = "iVBORw0KGgo=";
    let mut payload = json!({
        "contents": [
            {
                "role": "user",
                "parts": [
                    { "text": "Hello" },
                    { "inlineData": { "mimeType": "image/png", "data": "+A==" } }, // corrupt
                    { "inlineData": { "mimeType": "image/png", "data": "" } },     // empty
                    { "inlineData": { "mimeType": "image/png", "data": valid_png_b64 } } // valid
                ]
            }
        ]
    });

    let sanitized_count = sanitize_gemini_payload_inline_data(&mut payload);
    assert_eq!(sanitized_count, 2);

    let parts = payload["contents"][0]["parts"].as_array().unwrap();
    assert_eq!(parts.len(), 4);
    assert_eq!(parts[0]["text"], "Hello");
    assert_eq!(
        parts[1]["text"],
        "[Image/Data: invalid or corrupted inline payload omitted]"
    );
    assert_eq!(
        parts[2]["text"],
        "[Image/Data: invalid or corrupted inline payload omitted]"
    );
    assert!(parts[3].get("inlineData").is_some());
    assert_eq!(parts[3]["inlineData"]["data"], valid_png_b64);
}
