//! Unit tests for Wildcard Model Routing & Future-Proof Pattern Guard.

use crate::modules::release_gate::wildcard_model_routing_guard::{
    detect_model_family, detect_static_model_matches, evaluate_model_match,
    has_future_proof_pattern, is_wildcard_pattern, lint_routing_code, match_wildcard,
    resolve_route, ModelFamily, ModelMatchReport, ModelRouteRule, ModelRoutingViolation,
    ResolvedRoute, RouteResolver, RoutingPatternType,
};

#[test]
fn test_wildcard_pattern_detection() {
    assert!(is_wildcard_pattern("gemini-*-flash-*"));
    assert!(is_wildcard_pattern("claude-3-5-*"));
    assert!(is_wildcard_pattern("gpt-4o-?"));
    assert!(!is_wildcard_pattern("gemini-1.5-flash-latest"));
    assert!(!is_wildcard_pattern("claude-3-5-sonnet"));

    assert!(has_future_proof_pattern("gemini-*-flash-*"));
    assert!(has_future_proof_pattern("^gpt-4o-.*$"));
    assert!(!has_future_proof_pattern("gemini-1.5-flash"));
}

#[test]
fn test_wildcard_glob_matching() {
    assert!(match_wildcard(
        "gemini-*-flash-*",
        "gemini-2.5-flash-preview"
    ));
    assert!(match_wildcard("gemini-*-flash-*", "gemini-1.5-flash-001"));
    assert!(match_wildcard("gemini-*-flash-*", "gemini-3.0-flash-exp"));
    assert!(match_wildcard("claude-3-5-*", "claude-3-5-sonnet-20241022"));
    assert!(match_wildcard("claude-3-5-*", "claude-3-5-haiku"));
    assert!(match_wildcard("gpt-4o-*", "gpt-4o-mini"));
    assert!(match_wildcard("gpt-4o-?", "gpt-4o-a"));
    assert!(!match_wildcard("gpt-4o-?", "gpt-4o-mini"));
    assert!(!match_wildcard("gemini-*-flash-*", "claude-3-5-sonnet"));
    assert!(!match_wildcard("gpt-4o-*", "gpt-3.5-turbo"));
}

#[test]
fn test_detect_model_family() {
    // Google Gemini
    assert_eq!(
        detect_model_family("gemini-1.5-pro"),
        ModelFamily::GoogleGemini
    );
    assert_eq!(
        detect_model_family("gemini-2.0-flash-exp"),
        ModelFamily::GoogleGemini
    );
    assert_eq!(
        detect_model_family("gemini-9.9-flash-nextgen"),
        ModelFamily::GoogleGemini
    );
    assert_eq!(
        detect_model_family("google-gemini-ultra"),
        ModelFamily::GoogleGemini
    );

    // Anthropic Claude
    assert_eq!(
        detect_model_family("claude-3-5-sonnet-20241022"),
        ModelFamily::AnthropicClaude
    );
    assert_eq!(
        detect_model_family("claude-3-opus"),
        ModelFamily::AnthropicClaude
    );
    assert_eq!(
        detect_model_family("claude-next-gen"),
        ModelFamily::AnthropicClaude
    );

    // OpenAI Chat
    assert_eq!(detect_model_family("gpt-4o"), ModelFamily::OpenAiChat);
    assert_eq!(detect_model_family("gpt-4o-mini"), ModelFamily::OpenAiChat);
    assert_eq!(detect_model_family("o1-preview"), ModelFamily::OpenAiChat);
    assert_eq!(detect_model_family("o3-mini"), ModelFamily::OpenAiChat);
    assert_eq!(
        detect_model_family("chatgpt-4o-latest"),
        ModelFamily::OpenAiChat
    );
    assert_eq!(
        detect_model_family("text-embedding-3-large"),
        ModelFamily::OpenAiChat
    );
    assert_eq!(detect_model_family("dall-e-3"), ModelFamily::OpenAiChat);

    // OpenAI Responses / Codex
    assert_eq!(
        detect_model_family("codex-davinci-002"),
        ModelFamily::OpenAiResponses
    );
    assert_eq!(
        detect_model_family("gpt-4o-responses"),
        ModelFamily::OpenAiResponses
    );

    // DeepSeek
    assert_eq!(detect_model_family("deepseek-chat"), ModelFamily::DeepSeek);
    assert_eq!(
        detect_model_family("deepseek-coder-v2"),
        ModelFamily::DeepSeek
    );

    // Unknown & Empty
    assert_eq!(
        detect_model_family("unknown-custom-model"),
        ModelFamily::Unknown
    );
    assert_eq!(detect_model_family("   "), ModelFamily::Unknown);
}

#[test]
fn test_route_resolver_with_wildcards() {
    let mut resolver = RouteResolver::new();
    resolver.add_rule(ModelRouteRule::wildcard(
        "gemini-*-flash-*",
        "google_gemini_adapter",
    ));
    resolver.add_rule(ModelRouteRule::wildcard(
        "claude-3-5-*",
        "anthropic_claude_adapter",
    ));
    resolver.add_rule(ModelRouteRule::regex(
        r"^o[1-9]-.*$",
        "openai_reasoning_adapter",
    ));

    let gemini_res = resolver
        .resolve("gemini-2.0-flash-exp")
        .expect("should resolve");
    assert_eq!(gemini_res.matched_adapter, "google_gemini_adapter");
    assert_eq!(gemini_res.family, ModelFamily::GoogleGemini);
    assert!(gemini_res.is_future_proof);
    assert_eq!(gemini_res.matched_pattern, "gemini-*-flash-*");

    let claude_res = resolver
        .resolve("claude-3-5-sonnet-latest")
        .expect("should resolve");
    assert_eq!(claude_res.matched_adapter, "anthropic_claude_adapter");
    assert_eq!(claude_res.family, ModelFamily::AnthropicClaude);

    let o1_res = resolver.resolve("o3-mini").expect("should resolve");
    assert_eq!(o1_res.matched_adapter, "openai_reasoning_adapter");
    assert_eq!(o1_res.family, ModelFamily::OpenAiChat);
}

#[test]
fn test_route_resolver_automatic_family_fallback() {
    let resolver = RouteResolver::new(); // empty rules, default fallback enabled

    // Future unlisted Gemini variant automatically routes to Gemini adapter
    let res = resolver
        .resolve("gemini-9.0-pro-quantum")
        .expect("should fallback to family");
    assert_eq!(res.matched_adapter, "google_gemini_adapter");
    assert_eq!(res.family, ModelFamily::GoogleGemini);
    assert!(res.is_future_proof);

    // Future unlisted Claude variant automatically routes to Claude adapter
    let claude_res = resolver
        .resolve("claude-5-sonnet")
        .expect("should fallback to family");
    assert_eq!(claude_res.matched_adapter, "anthropic_claude_adapter");
    assert_eq!(claude_res.family, ModelFamily::AnthropicClaude);

    // Future unlisted OpenAI variant automatically routes to OpenAI Chat adapter
    let gpt_res = resolver
        .resolve("gpt-5-omni")
        .expect("should fallback to family");
    assert_eq!(gpt_res.matched_adapter, "openai_chat_adapter");
    assert_eq!(gpt_res.family, ModelFamily::OpenAiChat);

    // Completely unknown model fails because no adapter or family matches
    let err = resolver.resolve("xyz-transformer-9000").unwrap_err();
    assert!(matches!(
        err,
        ModelRoutingViolation::UnmatchedModelNoFallback { .. }
    ));
}

#[test]
fn test_route_resolver_disabled_fallback() {
    let resolver = RouteResolver::new().with_fallback_to_family(false);

    let err = resolver.resolve("gemini-2.0-flash").unwrap_err();
    assert!(matches!(
        err,
        ModelRoutingViolation::UnmatchedModelNoFallback { .. }
    ));
}

#[test]
fn test_reject_brittle_exact_rule() {
    let mut resolver = RouteResolver::new();
    resolver.add_rule(ModelRouteRule::exact(
        "gemini-1.5-flash",
        "google_gemini_adapter",
    ));

    let res = resolver.resolve("gemini-1.5-flash");
    assert!(res.is_err());
    match res.unwrap_err() {
        ModelRoutingViolation::BrittleExactMatchForbidden { pattern } => {
            assert_eq!(pattern, "gemini-1.5-flash");
        }
        other => panic!("Expected BrittleExactMatchForbidden, got {:?}", other),
    }
}

#[test]
fn test_empty_model_name_rejected() {
    let resolver = RouteResolver::new();
    let res = resolver.resolve("   ");
    assert_eq!(res, Err(ModelRoutingViolation::EmptyModelPattern));
}

#[test]
fn test_evaluate_model_match_backwards_compatible() {
    let rules = vec![
        ModelRouteRule::wildcard("gemini-*-flash-*", "google_gemini_adapter"),
        ModelRouteRule::wildcard("claude-3-5-*", "anthropic_claude_adapter"),
    ];

    let res = evaluate_model_match("gemini-2.0-flash-exp", &rules);
    assert!(res.is_ok());
    let report = res.unwrap();
    assert_eq!(
        report.matched_adapter.as_deref(),
        Some("google_gemini_adapter")
    );
    assert!(report.is_wildcard_matched);
    assert_eq!(report.rule_pattern, "gemini-*-flash-*");
    assert_eq!(report.model_family, Some(ModelFamily::GoogleGemini));
}

#[test]
fn test_lint_routing_code_detects_brittle_equality() {
    let source = r#"
        if model == "gemini-1.5-flash" {
            route_gemini();
        }
        match model.as_str() {
            "claude-3-5-sonnet" => route_claude(),
            _ => route_default(),
        }
    "#;

    let violations = lint_routing_code(source);
    assert_eq!(violations.len(), 2);
    assert!(matches!(
        &violations[0],
        ModelRoutingViolation::StaticModelEqualityDetected { .. }
    ));
    assert!(matches!(
        &violations[1],
        ModelRoutingViolation::StaticModelEqualityDetected { .. }
    ));
}

#[test]
fn test_lint_routing_code_detects_static_sanitization() {
    let source = r#"
        let clean_prompt = prompt.replace("system:", "");
    "#;

    let violations = detect_static_model_matches(source);
    assert_eq!(violations.len(), 1);
    assert!(matches!(
        &violations[0],
        ModelRoutingViolation::StaticKeywordSanitizationForbidden { .. }
    ));
}

#[test]
fn test_lint_routing_code_clean_code() {
    let source = r#"
        if match_wildcard("gemini-*-flash-*", model) {
            route_gemini();
        }
        let clean_prompt = SANITIZER_REGEX.replace_all(&prompt, "");
    "#;

    let violations = lint_routing_code(source);
    assert!(violations.is_empty());
}
