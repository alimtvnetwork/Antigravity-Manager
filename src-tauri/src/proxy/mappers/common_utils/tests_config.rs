// common_utils tests (split from common_utils.rs).
// Test-only module, declared #[cfg(test)] in mod.rs.
use super::*;

use super::*;

// ============ requestId 与 session_id 的关系 ============

/// 会话段必须**稳定**（同一 session 多次请求共享，贴近官方 conversationId 语义），
/// 但整体 ID 每次唯一（含 unixMs + 轨迹段 → 幂等隔离）。
#[test]
fn test_official_request_id_stable_conversation_unique_overall() {
    let a = build_official_request_id("sess-aaaabbbbccccdddd", 3);
    let b = build_official_request_id("sess-aaaabbbbccccdddd", 3);
    assert_eq!(
        a.split('/').nth(1),
        b.split('/').nth(1),
        "同一 session 的会话段必须稳定"
    );
    assert_ne!(a, b, "整体 requestId 必须每次唯一（幂等隔离）");
    assert_eq!(
        a.split('/').count(),
        5,
        "官方形态为 5 段：agent/conversation/unixMs/trajectory/step"
    );
    assert!(a.starts_with("agent/"));
    assert!(a.ends_with("/3"));
}

/// requestId **不得**泄露网关内部 blended session_id 的原文。
#[test]
fn test_official_request_id_never_exposes_raw_session() {
    let sid = "sess-deadbeefcafe1234";
    let id = build_official_request_id(sid, 1);
    assert!(
        !id.contains(sid),
        "requestId 不得包含 session_id 原文（应为单向哈希派生）"
    );
}

/// 不同会话（主 agent / 子 agent 并发）必须产出不同会话段 —— 隔离性保持。
#[test]
fn test_official_request_id_separates_concurrent_sessions() {
    let main = build_official_request_id("sess-main-00000001", 1);
    let sub = build_official_request_id("sess-sub-00000002", 1);
    assert_ne!(
        main.split('/').nth(1),
        sub.split('/').nth(1),
        "不同 session 的会话段必须不同（主子 agent 隔离）"
    );
}

/// 空 session 时退化为随机会话段，不 panic 且仍为 5 段。
#[test]
fn test_official_request_id_handles_empty_session() {
    let id = build_official_request_id("", 7);
    assert_eq!(id.split('/').count(), 5);
    assert!(id.ends_with("/7"));
}

#[test]
fn test_high_quality_model_auto_grounding() {
    // Auto-grounding is currently disabled by default due to conflict with image gen
    let config =
        resolve_request_config("gpt-4o", "gemini-2.5-flash", &None, None, None, None, None);
    assert_eq!(config.request_type, "agent");
    assert!(!config.inject_google_search);
}

#[test]
fn test_gemini_native_tool_detection() {
    let tools = Some(vec![json!({
        "functionDeclarations": [
            { "name": "web_search", "parameters": {} }
        ]
    })]);
    assert!(detects_networking_tool(&tools));
}

#[test]
fn test_online_suffix_force_grounding() {
    let config = resolve_request_config(
        "gemini-3-flash-online",
        "gemini-3-flash",
        &None,
        None,
        None,
        None,
        None,
    );
    assert_eq!(config.request_type, "web_search");
    assert!(config.inject_google_search);
    assert_eq!(config.final_model, "gemini-3-flash");
}

#[test]
fn test_default_no_grounding() {
    let config = resolve_request_config(
        "claude-sonnet",
        "gemini-3-flash",
        &None,
        None,
        None,
        None,
        None,
    );
    assert_eq!(config.request_type, "agent");
    assert!(!config.inject_google_search);
}

#[test]
fn test_image_model_excluded() {
    let config = resolve_request_config(
        "gemini-3-pro-image",
        "gemini-3-pro-image",
        &None,
        None,
        None,
        None,
        None,
    );
    assert_eq!(config.request_type, "image_gen");
    assert!(!config.inject_google_search);
}

#[test]
fn test_image_2k_and_ultrawide_config() {
    // Test 2K
    let (config_2k, _) = parse_image_config("gemini-3-pro-image-2k");
    assert_eq!(config_2k["imageSize"], "2K");

    // Test 21:9
    let (config_21x9, _) = parse_image_config("gemini-3-pro-image-21x9");
    assert_eq!(config_21x9["aspectRatio"], "21:9");

    // Test Combined (if logic allows, though suffix parsing is greedy)
    let (config_combined, _) = parse_image_config("gemini-3-pro-image-2k-21x9");
    assert_eq!(config_combined["imageSize"], "2K");
    assert_eq!(config_combined["aspectRatio"], "21:9");

    // Test 4K + 21:9
    let (config_4k_wide, _) = parse_image_config("gemini-3-pro-image-4k-21x9");
    assert_eq!(config_4k_wide["imageSize"], "4K");
    assert_eq!(config_4k_wide["aspectRatio"], "21:9");
}

#[test]
fn test_parse_image_config_with_openai_params() {
    // Test quality parameter mapping
    let (config_hd, model_hd) =
        parse_image_config_with_params("gemini-3-pro-image", None, Some("hd"), None);
    assert_eq!(config_hd["imageSize"], "4K");
    assert_eq!(config_hd["aspectRatio"], "1:1");
    assert_eq!(model_hd, "gemini-3-pro-image");

    let (config_medium, model_medium) =
        parse_image_config_with_params("gemini-3-pro-image", None, Some("medium"), None);
    assert_eq!(config_medium["imageSize"], "2K");
    assert_eq!(model_medium, "gemini-3-pro-image");

    let (config_standard, model_standard) =
        parse_image_config_with_params("gemini-3-pro-image", None, Some("standard"), None);
    assert_eq!(config_standard["imageSize"], "1K");
    assert_eq!(model_standard, "gemini-3-pro-image");

    // Test size parameter mapping with dynamic calculation
    let (config_16_9, model_16_9) =
        parse_image_config_with_params("gemini-3-pro-image", Some("1280x720"), None, None);
    assert_eq!(config_16_9["aspectRatio"], "16:9");
    assert_eq!(model_16_9, "gemini-3-pro-image");

    let (config_9_16, model_9_16) =
        parse_image_config_with_params("gemini-3-pro-image", Some("720x1280"), None, None);
    assert_eq!(config_9_16["aspectRatio"], "9:16");
    assert_eq!(model_9_16, "gemini-3-pro-image");

    let (config_4_3, model_4_3) =
        parse_image_config_with_params("gemini-3-pro-image", Some("800x600"), None, None);
    assert_eq!(config_4_3["aspectRatio"], "4:3");
    assert_eq!(model_4_3, "gemini-3-pro-image");

    // Test combined size + quality
    let (config_combined, model_combined) =
        parse_image_config_with_params("gemini-3-pro-image", Some("1920x1080"), Some("hd"), None);
    assert_eq!(config_combined["aspectRatio"], "16:9");
    assert_eq!(config_combined["imageSize"], "4K");
    assert_eq!(model_combined, "gemini-3-pro-image");

    // Test backward compatibility: model suffix takes precedence when no params
    let (config_compat, model_compat) =
        parse_image_config_with_params("gemini-3-pro-image-16x9-4k", None, None, None);
    assert_eq!(config_compat["aspectRatio"], "16:9");
    assert_eq!(config_compat["imageSize"], "4K");
    assert_eq!(model_compat, "gemini-3-pro-image");

    // Test parameter priority: params override model suffix
    let (config_override, model_override) = parse_image_config_with_params(
        "gemini-3-pro-image-1x1-2k",
        Some("1280x720"),
        Some("hd"),
        None,
    );
    assert_eq!(config_override["aspectRatio"], "16:9"); // from size param, not model suffix
    assert_eq!(config_override["imageSize"], "4K"); // from quality param, not model suffix
    assert_eq!(model_override, "gemini-3-pro-image");
}

#[test]
fn test_clean_image_model_name() {
    assert_eq!(
        clean_image_model_name("gemini-3.1-flash-image"),
        "gemini-3.1-flash-image"
    );
    assert_eq!(
        clean_image_model_name("gemini-3.1-flash-image-4k"),
        "gemini-3.1-flash-image"
    );
    assert_eq!(
        clean_image_model_name("gemini-3-pro-image-16x9"),
        "gemini-3-pro-image"
    );
    assert_eq!(
        clean_image_model_name("gemini-3-pro-image-16x9-4k"),
        "gemini-3-pro-image"
    );
    // Test varying order
    assert_eq!(
        clean_image_model_name("gemini-3.1-flash-image-4k-16x9"),
        "gemini-3.1-flash-image"
    );
    assert_eq!(
        clean_image_model_name("gemini-3.1-flash-image-16-9-hd"),
        "gemini-3.1-flash-image"
    );
    assert_eq!(
        clean_image_model_name("gemini-3.1-flash-image-2k-9x16"),
        "gemini-3.1-flash-image"
    );
    assert_eq!(
        clean_image_model_name("gemini-3.1-flash-image-1x1"),
        "gemini-3.1-flash-image"
    );
    assert_eq!(
        clean_image_model_name("gemini-3.1-flash-image-standard"),
        "gemini-3.1-flash-image"
    );
    assert_eq!(
        clean_image_model_name("gemini-3.1-flash-image-medium"),
        "gemini-3.1-flash-image"
    );
    assert_eq!(
        clean_image_model_name("gemini-3.1-flash-image-21-9-4k"),
        "gemini-3.1-flash-image"
    );
}
