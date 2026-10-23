//! Unit Tests for Protocol Adapter Divergence Auditor (`Imp-256`).

#[cfg(test)]
mod tests {
    use super::protocol_adapter_divergence_auditor::*;

    #[test]
    fn test_clean_adapter_payload_transformation_passes() {
        let clean_source = r#"
            pub fn convert_openai_to_gemini(req: OpenAiRequest) -> GeminiRequest {
                let model = normalize_model_name(&req.model);
                let contents = map_messages_to_contents(&req.messages);
                GeminiRequest { model, contents }
            }
        "#;

        let report = audit_adapter_source_code(
            clean_source,
            ProtocolAdapterKind::OpenAiResponses,
            "src/proxy/adapters/openai_responses.rs",
        );

        assert!(report.is_decoupled);
        assert!(!report.has_leaks());
        assert_eq!(report.leak_count(), 0);
        assert_eq!(report.adapter_kind, ProtocolAdapterKind::OpenAiResponses);
        assert_eq!(report.lines_scanned, 7);
    }

    #[test]
    fn test_thinking_budget_filtering_leak_detected() {
        let leaking_source = r#"
            pub fn handle_claude_adapter(req: ClaudeRequest) {
                let budget = filter_thinking_budget(&req);
                forward_to_pipeline(budget);
            }
        "#;

        let report = audit_adapter_source_code(
            leaking_source,
            ProtocolAdapterKind::AnthropicClaude,
            "src/proxy/adapters/claude.rs",
        );

        assert!(!report.is_decoupled);
        assert!(report.has_leaks());
        assert_eq!(report.leak_count(), 1);

        let leak = &report.leaks[0];
        assert_eq!(leak.line(), 3);
        assert_eq!(leak.file(), "src/proxy/adapters/claude.rs");
        assert_eq!(leak.leak_category(), "thinking_budget_filtering");

        match leak {
            ForbiddenPipelineLeak::ThinkingBudgetFilteringInAdapter { file, line } => {
                assert_eq!(file, "src/proxy/adapters/claude.rs");
                assert_eq!(*line, 3);
            }
            _ => panic!("Expected ThinkingBudgetFilteringInAdapter variant"),
        }
    }

    #[test]
    fn test_prefix_stability_enforcement_leak_detected() {
        let leaking_source = r#"
            pub fn transform_chat_completions(req: ChatCompletionRequest) {
                enforce_prefix_stability(&req.messages);
            }
        "#;

        let report = audit_adapter_source_code(
            leaking_source,
            ProtocolAdapterKind::OpenAiChatCompletions,
            "src/proxy/adapters/openai_chat.rs",
        );

        assert!(!report.is_decoupled);
        assert_eq!(report.leak_count(), 1);

        let leak = &report.leaks[0];
        assert_eq!(leak.line(), 3);
        assert_eq!(leak.leak_category(), "prefix_stability_enforcement");

        match leak {
            ForbiddenPipelineLeak::PrefixStabilityEnforcementInAdapter { file, line } => {
                assert_eq!(file, "src/proxy/adapters/openai_chat.rs");
                assert_eq!(*line, 3);
            }
            _ => panic!("Expected PrefixStabilityEnforcementInAdapter variant"),
        }
    }

    #[test]
    fn test_context_structural_alignment_leak_detected() {
        let leaking_source = r#"
            pub fn prepare_gemini_payload(req: GeminiRequest) {
                let aligned = align_context_structure(&req.contents);
            }
        "#;

        let report = audit_adapter_source_code(
            leaking_source,
            ProtocolAdapterKind::GoogleGemini,
            "src/proxy/adapters/gemini.rs",
        );

        assert!(!report.is_decoupled);
        assert_eq!(report.leak_count(), 1);

        let leak = &report.leaks[0];
        assert_eq!(leak.line(), 3);
        assert_eq!(leak.leak_category(), "context_structural_alignment");

        match leak {
            ForbiddenPipelineLeak::ContextStructuralAlignmentInAdapter { file, line } => {
                assert_eq!(file, "src/proxy/adapters/gemini.rs");
                assert_eq!(*line, 3);
            }
            _ => panic!("Expected ContextStructuralAlignmentInAdapter variant"),
        }
    }

    #[test]
    fn test_request_header_sanitization_leak_detected() {
        let leaking_source = r#"
            pub fn filter_inbound_headers(headers: &HeaderMap) {
                strip_hop_by_hop(headers);
            }
        "#;

        let report = audit_adapter_source_code(
            leaking_source,
            ProtocolAdapterKind::OpenAiResponses,
            "src/proxy/adapters/openai_headers.rs",
        );

        assert!(!report.is_decoupled);
        assert_eq!(report.leak_count(), 1);

        let leak = &report.leaks[0];
        assert_eq!(leak.line(), 3);
        assert_eq!(leak.leak_category(), "request_header_sanitization");

        match leak {
            ForbiddenPipelineLeak::RequestHeaderSanitizationInAdapter { file, line } => {
                assert_eq!(file, "src/proxy/adapters/openai_headers.rs");
                assert_eq!(*line, 3);
            }
            _ => panic!("Expected RequestHeaderSanitizationInAdapter variant"),
        }
    }

    #[test]
    fn test_multiple_leaks_detected_in_single_file() {
        let multi_leak_source = r#"
            pub fn dirty_adapter() {
                clamp_thinking_budget(4096);
                check_prefix_stability();
                let _ = unified_context_structural_aligner();
                sanitize_request_headers();
            }
        "#;

        let report = audit_adapter_source_code(
            multi_leak_source,
            ProtocolAdapterKind::AnthropicClaude,
            "src/proxy/adapters/dirty.rs",
        );

        assert!(!report.is_decoupled);
        assert_eq!(report.leak_count(), 4);
        assert_eq!(
            report.leaks[0].leak_category(),
            "thinking_budget_filtering"
        );
        assert_eq!(
            report.leaks[1].leak_category(),
            "prefix_stability_enforcement"
        );
        assert_eq!(
            report.leaks[2].leak_category(),
            "context_structural_alignment"
        );
        assert_eq!(
            report.leaks[3].leak_category(),
            "request_header_sanitization"
        );
    }

    #[test]
    fn test_forbidden_pipeline_leak_display_and_error_impl() {
        let leak1 = ForbiddenPipelineLeak::ThinkingBudgetFilteringInAdapter {
            file: "mod.rs".to_string(),
            line: 12,
        };
        let leak2 = ForbiddenPipelineLeak::PrefixStabilityEnforcementInAdapter {
            file: "mod.rs".to_string(),
            line: 24,
        };
        let leak3 = ForbiddenPipelineLeak::ContextStructuralAlignmentInAdapter {
            file: "mod.rs".to_string(),
            line: 36,
        };
        let leak4 = ForbiddenPipelineLeak::RequestHeaderSanitizationInAdapter {
            file: "mod.rs".to_string(),
            line: 48,
        };

        assert_eq!(
            leak1.to_string(),
            "Thinking budget filtering leaked in adapter at mod.rs:12"
        );
        assert_eq!(
            leak2.to_string(),
            "Prefix stability enforcement leaked in adapter at mod.rs:24"
        );
        assert_eq!(
            leak3.to_string(),
            "Context structural alignment leaked in adapter at mod.rs:36"
        );
        assert_eq!(
            leak4.to_string(),
            "Request header sanitization leaked in adapter at mod.rs:48"
        );

        // Verify std::error::Error trait bound
        let err: &dyn std::error::Error = &leak1;
        assert!(err.source().is_none());
    }

    #[test]
    fn test_adapter_name_mapping() {
        assert_eq!(
            adapter_name(ProtocolAdapterKind::OpenAiResponses),
            "openai_responses"
        );
        assert_eq!(
            adapter_name(ProtocolAdapterKind::OpenAiChatCompletions),
            "openai_chat_completions"
        );
        assert_eq!(
            adapter_name(ProtocolAdapterKind::AnthropicClaude),
            "anthropic_claude"
        );
        assert_eq!(
            adapter_name(ProtocolAdapterKind::GoogleGemini),
            "google_gemini"
        );
    }
}
