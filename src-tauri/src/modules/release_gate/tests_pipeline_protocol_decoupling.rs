//! Unit tests for Pipeline Decoupling & Protocol Normalization Guard.

use crate::modules::release_gate::pipeline_protocol_decoupling_guard::{
    lint_adapter_source_code, lint_pipeline_source_code, validate_architectural_barrier,
    validate_normalized_gemini_json, validate_output_protocol, AdapterOperationType, AiProtocol,
    DecouplingViolation, PipelineOperationType, ProcessingStage, ProtocolDivergenceJustification,
    CANONICAL_OUTPUT_PROTOCOL,
};

#[test]
fn test_ai_protocol_properties_and_parsing() {
    assert_eq!(AiProtocol::OpenAiResponses.as_str(), "openai_responses");
    assert_eq!(
        AiProtocol::OpenAiChatCompletions.as_str(),
        "openai_chat_completions"
    );
    assert_eq!(AiProtocol::AnthropicClaude.as_str(), "anthropic_claude");
    assert_eq!(AiProtocol::GoogleGemini.as_str(), "google_gemini");

    assert_eq!(
        AiProtocol::OpenAiResponses.display_name(),
        "OpenAI Responses"
    );
    assert_eq!(
        AiProtocol::OpenAiChatCompletions.display_name(),
        "OpenAI Chat Completions"
    );
    assert_eq!(
        AiProtocol::AnthropicClaude.display_name(),
        "Anthropic Claude"
    );
    assert_eq!(AiProtocol::GoogleGemini.display_name(), "Google Gemini");

    assert!(AiProtocol::GoogleGemini.is_gemini_native());
    assert!(!AiProtocol::AnthropicClaude.is_gemini_native());
    assert!(!AiProtocol::OpenAiChatCompletions.is_gemini_native());
    assert!(!AiProtocol::OpenAiResponses.is_gemini_native());

    assert_eq!(
        AiProtocol::from_str_loose("claude-3-5-sonnet"),
        Some(AiProtocol::AnthropicClaude)
    );
    assert_eq!(
        AiProtocol::from_str_loose("openai_responses_v1"),
        Some(AiProtocol::OpenAiResponses)
    );
    assert_eq!(
        AiProtocol::from_str_loose("chat-completions"),
        Some(AiProtocol::OpenAiChatCompletions)
    );
    assert_eq!(
        AiProtocol::from_str_loose("gemini-2.0-flash"),
        Some(AiProtocol::GoogleGemini)
    );
    assert_eq!(AiProtocol::from_str_loose("unknown_llm"), None);

    assert_eq!(AiProtocol::all().len(), 4);
}

#[test]
fn test_processing_stages() {
    let adapter_stage = ProcessingStage::AdapterNormalization;
    let pipeline_stage = ProcessingStage::PipelineUniformProcessing;

    assert!(adapter_stage.is_adapter());
    assert!(!adapter_stage.is_pipeline());
    assert_eq!(adapter_stage.as_str(), "adapter_normalization");

    assert!(pipeline_stage.is_pipeline());
    assert!(!pipeline_stage.is_adapter());
    assert_eq!(pipeline_stage.as_str(), "pipeline_uniform_processing");
}

#[test]
fn test_adapter_and_pipeline_operation_permissions() {
    let adapter_op = AdapterOperationType::PayloadTransformation;
    assert!(adapter_op.is_permitted_in_adapter());
    assert!(!adapter_op.is_permitted_in_pipeline());

    let edge_case_op = AdapterOperationType::UnsolvableEdgeCase;
    assert!(edge_case_op.is_permitted_in_adapter());
    assert!(!edge_case_op.is_permitted_in_pipeline());

    let pipe_op = PipelineOperationType::ThinkingBlockBackfilling;
    assert!(pipe_op.is_permitted_in_pipeline());
    assert!(!pipe_op.is_permitted_in_adapter());

    let budget_op = PipelineOperationType::ThinkingBudgetFilteringAndBackfilling;
    assert!(budget_op.is_permitted_in_pipeline());
    assert!(!budget_op.is_permitted_in_adapter());

    let prefix_op = PipelineOperationType::PrefixStability;
    assert!(prefix_op.is_permitted_in_pipeline());
    assert!(!prefix_op.is_permitted_in_adapter());
}

#[test]
fn test_validate_output_protocol_adherence() {
    assert!(validate_output_protocol(CANONICAL_OUTPUT_PROTOCOL).is_ok());
    assert!(validate_output_protocol("antigravity-gemini").is_ok());
    assert!(validate_output_protocol("gemini").is_ok());
    assert!(validate_output_protocol("google-gemini").is_ok());

    let res_claude = validate_output_protocol("anthropic-messages");
    assert!(res_claude.is_err());
    match res_claude.unwrap_err() {
        DecouplingViolation::NonGeminiOutputProtocolForbidden { attempted_format } => {
            assert_eq!(attempted_format, "anthropic-messages");
        }
        other => panic!("Unexpected violation: {other:?}"),
    }

    let res_openai = validate_output_protocol("openai-chat-completions");
    assert!(res_openai.is_err());
}

#[test]
fn test_validate_normalized_gemini_json_success() {
    let valid_payload = r#"{
        "contents": [
            {
                "role": "user",
                "parts": [{"text": "Hello Gemini!"}]
            },
            {
                "role": "model",
                "parts": [{"text": "Hello! How can I assist you today?"}]
            }
        ],
        "generationConfig": {
            "temperature": 0.7,
            "maxOutputTokens": 4096
        }
    }"#;

    let res = validate_normalized_gemini_json(valid_payload, AiProtocol::AnthropicClaude);
    assert!(res.is_ok(), "Valid Gemini payload must pass validation");
}

#[test]
fn test_validate_normalized_gemini_json_missing_contents() {
    let invalid_payload = r#"{
        "generationConfig": {
            "temperature": 0.5
        }
    }"#;

    let res = validate_normalized_gemini_json(invalid_payload, AiProtocol::OpenAiChatCompletions);
    assert!(res.is_err());
    let violations = res.unwrap_err();
    assert!(violations.iter().any(|v| matches!(
        v,
        DecouplingViolation::PayloadNotNormalizedToGemini {
            missing_canonical_field,
            ..
        } if missing_canonical_field == "contents"
    )));
}

#[test]
fn test_validate_normalized_gemini_json_rejects_raw_openai_messages() {
    let leaked_payload = r#"{
        "messages": [
            {"role": "user", "content": "What is the weather?"}
        ],
        "model": "gpt-4o"
    }"#;

    let res = validate_normalized_gemini_json(leaked_payload, AiProtocol::OpenAiChatCompletions);
    assert!(res.is_err());
    let violations = res.unwrap_err();
    assert!(violations.iter().any(|v| matches!(
        v,
        DecouplingViolation::ProtocolLeakInPipeline { leaked_symbol, .. } if leaked_symbol == "messages"
    )));
}

#[test]
fn test_validate_normalized_gemini_json_rejects_raw_claude_headers() {
    let leaked_claude = r#"{
        "contents": [
            {"role": "user", "parts": [{"text": "Explain quantum computing"}]}
        ],
        "anthropic_version": "2023-06-01"
    }"#;

    let res = validate_normalized_gemini_json(leaked_claude, AiProtocol::AnthropicClaude);
    assert!(res.is_err());
    let violations = res.unwrap_err();
    assert!(violations.iter().any(|v| matches!(
        v,
        DecouplingViolation::ProtocolLeakInPipeline { leaked_symbol, .. } if leaked_symbol == "anthropic-version"
    )));
}

#[test]
fn test_validate_normalized_gemini_json_turn_structure() {
    let bad_turn = r#"{
        "contents": [
            {"role": "user"}
        ]
    }"#;

    let res = validate_normalized_gemini_json(bad_turn, AiProtocol::GoogleGemini);
    assert!(res.is_err());
    let violations = res.unwrap_err();
    assert!(violations.iter().any(|v| matches!(
        v,
        DecouplingViolation::PayloadNotNormalizedToGemini { missing_canonical_field, .. } if missing_canonical_field == "contents[0].parts"
    )));
}

#[test]
fn test_architectural_barrier_compliant_adapter() {
    let ops = &["parameter_normalization", "payload_transformation"];
    let report = validate_architectural_barrier(
        ProcessingStage::AdapterNormalization,
        Some(AiProtocol::AnthropicClaude),
        ops,
        &[],
    );

    assert!(report.is_compliant);
    assert!(report.violations.is_empty());
}

#[test]
fn test_architectural_barrier_rejects_premature_pipeline_in_adapter() {
    let ops = &[
        "parameter_normalization",
        "thinking_block_backfill",
        "filter_budget",
        "prompt_sanitizing",
    ];
    let report = validate_architectural_barrier(
        ProcessingStage::AdapterNormalization,
        Some(AiProtocol::OpenAiChatCompletions),
        ops,
        &[],
    );

    assert!(!report.is_compliant);
    assert_eq!(report.violations.len(), 3);
    assert!(report.violations.iter().any(|v| matches!(
        v,
        DecouplingViolation::PrematurePipelineOperationInAdapter {
            operation: PipelineOperationType::ThinkingBlockBackfilling,
            ..
        }
    )));
    assert!(report.violations.iter().any(|v| matches!(
        v,
        DecouplingViolation::PrematurePipelineOperationInAdapter {
            operation: PipelineOperationType::ThinkingBudgetFilteringAndBackfilling,
            ..
        }
    )));
    assert!(report.violations.iter().any(|v| matches!(
        v,
        DecouplingViolation::PrematurePipelineOperationInAdapter {
            operation: PipelineOperationType::RiskyPromptSanitization,
            ..
        }
    )));
}

#[test]
fn test_architectural_barrier_rejects_unjustified_adapter_patch() {
    let ops = &["adapter_patch_for_streaming_quirk"];
    let report = validate_architectural_barrier(
        ProcessingStage::AdapterNormalization,
        Some(AiProtocol::AnthropicClaude),
        ops,
        &[], // No justification provided
    );

    assert!(!report.is_compliant);
    assert!(report.violations.iter().any(|v| matches!(
        v,
        DecouplingViolation::UnjustifiedAdapterModification { .. }
    )));
}

#[test]
fn test_architectural_barrier_accepts_justified_unsolvable_edge_case() {
    let ops = &["adapter_patch_quirk"];
    let justifications = vec![ProtocolDivergenceJustification::new(
        AiProtocol::AnthropicClaude,
        "Claude streaming signature parsing divergence cannot be resolved generically in pipeline stage",
        true,
        false,
    )];

    let report = validate_architectural_barrier(
        ProcessingStage::AdapterNormalization,
        Some(AiProtocol::AnthropicClaude),
        ops,
        &justifications,
    );

    assert!(report.is_compliant);
    assert!(report.violations.is_empty());
}

#[test]
fn test_architectural_barrier_pipeline_rejects_protocol_leaks() {
    let ops = &[
        "thinking_block_backfill",
        "anthropic_claude_special_header_parse",
    ];
    let report =
        validate_architectural_barrier(ProcessingStage::PipelineUniformProcessing, None, ops, &[]);

    assert!(!report.is_compliant);
    assert!(report.violations.iter().any(|v| matches!(
        v,
        DecouplingViolation::ProtocolLeakInPipeline {
            protocol: AiProtocol::AnthropicClaude,
            ..
        }
    )));
}

#[test]
fn test_lint_pipeline_source_code_leaks() {
    let bad_pipeline_code = r#"
        pub fn process_headers(headers: &HeaderMap) {
            if headers.contains_key("anthropic-version") {
                println!("Anthropic client detected");
            }
            if headers.contains_key("x-openai-organization") {
                println!("OpenAI client detected");
            }
        }
    "#;

    let violations = lint_pipeline_source_code(bad_pipeline_code, "pipeline_core.rs");
    assert_eq!(violations.len(), 2);
    assert!(violations.iter().any(|v| matches!(
        v,
        DecouplingViolation::ProtocolLeakInPipeline {
            protocol: AiProtocol::AnthropicClaude,
            ..
        }
    )));
    assert!(violations.iter().any(|v| matches!(
        v,
        DecouplingViolation::ProtocolLeakInPipeline {
            protocol: AiProtocol::OpenAiChatCompletions,
            ..
        }
    )));

    let clean_pipeline_code = r#"
        // Clean pipeline code operating strictly on normalized Gemini types
        pub fn process_pipeline_content(content: &GeminiContent) {
            for part in &content.parts {
                println!("Part processed: {:?}", part);
            }
        }
    "#;
    let clean_violations = lint_pipeline_source_code(clean_pipeline_code, "clean_pipeline.rs");
    assert!(clean_violations.is_empty());
}

#[test]
fn test_lint_adapter_source_code_premature_pipeline() {
    let bad_adapter_code = r#"
        pub fn convert_claude_to_gemini(req: ClaudeRequest) {
            // Prematurely backfilling thinking block inside adapter
            backfill_thinking_block(&req);
            // Localized prompt sanitization instead of pipeline
            sanitize_risky_prompts(&req);
        }
    "#;

    let violations = lint_adapter_source_code(
        bad_adapter_code,
        "claude_adapter.rs",
        AiProtocol::AnthropicClaude,
    );
    assert_eq!(violations.len(), 2);
    assert!(violations.iter().any(|v| matches!(
        v,
        DecouplingViolation::PrematurePipelineOperationInAdapter {
            operation: PipelineOperationType::ThinkingBlockBackfilling,
            ..
        }
    )));
    assert!(violations.iter().any(|v| matches!(
        v,
        DecouplingViolation::PrematurePipelineOperationInAdapter {
            operation: PipelineOperationType::RiskyPromptSanitization,
            ..
        }
    )));

    let clean_adapter_code = r#"
        // Clean adapter normalizing parameters and transforming payload
        pub fn convert_openai_payload(req: OpenAiRequest) -> GeminiPayload {
            GeminiPayload {
                contents: map_messages_to_contents(&req.messages),
                generation_config: map_parameters(&req),
            }
        }
    "#;
    let clean_violations = lint_adapter_source_code(
        clean_adapter_code,
        "clean_adapter.rs",
        AiProtocol::OpenAiChatCompletions,
    );
    assert!(clean_violations.is_empty());
}

#[test]
fn test_decoupling_violation_display_formatting() {
    let v1 = DecouplingViolation::ProtocolLeakInPipeline {
        protocol: AiProtocol::AnthropicClaude,
        leaked_symbol: "anthropic-version".to_string(),
        context: "proxy/pipeline/core.rs:42".to_string(),
    };
    let s1 = v1.to_string();
    assert!(s1.contains("Protocol leak in pipeline"));
    assert!(s1.contains("anthropic-version"));
    assert!(s1.contains("Anthropic Claude"));

    let v2 = DecouplingViolation::PrematurePipelineOperationInAdapter {
        protocol: AiProtocol::OpenAiChatCompletions,
        operation: PipelineOperationType::ThinkingBudgetFilteringAndBackfilling,
        context: "adapters/openai.rs:18".to_string(),
    };
    let s2 = v2.to_string();
    assert!(s2.contains("Premature pipeline operation"));
    assert!(s2.contains("thinking_budget_filtering_and_backfilling"));

    let v3 = DecouplingViolation::NonGeminiOutputProtocolForbidden {
        attempted_format: "openai".to_string(),
    };
    let s3 = v3.to_string();
    assert!(s3.contains("antigravity-gemini"));
}

#[test]
fn test_report_serde_roundtrip() {
    let report = PipelineDecouplingReport {
        is_compliant: false,
        stage: ProcessingStage::AdapterNormalization,
        protocol: Some(AiProtocol::AnthropicClaude),
        violations: vec![DecouplingViolation::UnjustifiedAdapterModification {
            protocol: AiProtocol::AnthropicClaude,
            description: "Ad-hoc regex patch in adapter".to_string(),
        }],
        warnings: vec!["Consider moving patch to pipeline stage".to_string()],
    };

    let serialized = serde_json::to_string(&report).expect("Serialization succeeds");
    let deserialized: PipelineDecouplingReport =
        serde_json::from_str(&serialized).expect("Deserialization succeeds");

    assert_eq!(report, deserialized);
}
