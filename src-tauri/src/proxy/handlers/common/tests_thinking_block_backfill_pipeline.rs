//! Unit Tests for Thinking Block Backfill Pipeline (`Imp-252`).

#[cfg(test)]
mod tests {
    use super::super::thinking_block_backfill_pipeline::*;
    use serde_json::json;

    #[test]
    fn test_detect_thinking_format_tagged_thought() {
        let content = "<thought>\nAnalyzing user request...\n</thought>\nHere is the answer.";
        let format = detect_thinking_format(content);

        assert!(format.is_tagged());
        assert!(!format.is_candidate_part());
        assert!(!format.is_raw_reasoning());

        match format {
            ThinkingPayloadFormat::TaggedThought {
                tag_open,
                tag_close,
            } => {
                assert_eq!(tag_open, "<thought>");
                assert_eq!(tag_close, "</thought>");
            }
            _ => panic!("Expected TaggedThought"),
        }
    }

    #[test]
    fn test_detect_thinking_format_candidate_flag() {
        let json_content = r#"{"parts": [{"text": "thought text", "thought": true}]}"#;
        let format = detect_thinking_format(json_content);

        assert!(format.is_candidate_part());
        assert!(!format.is_tagged());
        assert!(!format.is_raw_reasoning());
    }

    #[test]
    fn test_detect_thinking_format_raw_reasoning() {
        let plain_content = "Just plain text without any thought tags or json indicators.";
        let format = detect_thinking_format(plain_content);

        assert!(format.is_raw_reasoning());
        assert!(!format.is_tagged());
        assert!(!format.is_candidate_part());
    }

    #[test]
    fn test_backfill_missing_thought_tags_with_reasoning_prefix() {
        let raw = "Reasoning: First, let us check the input parameters.\nSecond, evaluate edge cases.";
        let report = backfill_gemini_thinking_blocks(raw, true);

        assert!(report.is_modified());
        assert!(report.action.is_injected());
        assert!(report.format.is_tagged());

        let processed = report.processed_content();
        assert!(processed.starts_with("<thought>"));
        assert!(processed.contains("First, let us check the input parameters."));
        assert!(processed.ends_with("</thought>"));
        assert!(report.estimated_tokens() > 0);
    }

    #[test]
    fn test_backfill_with_answer_transition_marker() {
        let raw = "Reasoning: Step by step thinking here.\n\nFinal Answer: 42 is the answer.";
        let report = backfill_gemini_thinking_blocks(raw, true);

        assert!(report.is_modified());
        assert!(report.action.is_injected());

        let processed = report.processed_content();
        assert!(processed.contains("<thought>\nStep by step thinking here.\n</thought>"));
        assert!(processed.contains("42 is the answer."));
        assert!(!processed.contains("<thought>\nStep by step thinking here.\n\nFinal Answer:"));
    }

    #[test]
    fn test_backfill_when_require_thought_tags_plain_text() {
        let raw = "I am pondering this deeply.";
        let report = backfill_gemini_thinking_blocks(raw, true);

        assert!(report.is_modified());
        assert!(report.action.is_injected());
        assert_eq!(
            report.processed_content(),
            "<thought>\nI am pondering this deeply.\n</thought>"
        );
    }

    #[test]
    fn test_content_already_tagged_remains_untouched() {
        let raw = "<thought>\nAlready tagged thinking.\n</thought>\nRegular response.";
        let report = backfill_gemini_thinking_blocks(raw, true);

        assert!(!report.is_modified());
        assert!(report.action.is_no_action());
        assert!(report.format.is_tagged());
        assert_eq!(report.processed_content(), raw);
        assert!(report.estimated_tokens() > 0);
    }

    #[test]
    fn test_empty_content_returns_no_action() {
        let report = backfill_gemini_thinking_blocks("", true);
        assert!(!report.is_modified());
        assert!(report.action.is_no_action());
        assert_eq!(report.estimated_tokens(), 0);
        assert_eq!(report.processed_content(), "");
    }

    #[test]
    fn test_strip_empty_thought_block() {
        let raw = "<thought>   </thought>\nHere is the real content.";
        let report = backfill_gemini_thinking_blocks(raw, true);

        assert!(report.is_modified());
        assert!(report.action.is_stripped());
        assert_eq!(report.processed_content(), "Here is the real content.");
    }

    #[test]
    fn test_ensure_thinking_part_flags_text_tag() {
        let mut parts = vec![
            json!({
                "text": "<thought>\nCalculating factorial...\n</thought>"
            }),
            json!({
                "text": "The answer is 120."
            }),
        ];

        let modified_count = ensure_thinking_part_flags(&mut parts);
        assert_eq!(modified_count, 1);
        assert_eq!(parts[0]["thought"], true);
        assert_eq!(parts[1].get("thought"), None);

        // Repeated run should make 0 modifications (idempotent)
        let modified_again = ensure_thinking_part_flags(&mut parts);
        assert_eq!(modified_again, 0);
    }

    #[test]
    fn test_ensure_thinking_part_flags_type_reasoning() {
        let mut parts = vec![
            json!({
                "type": "reasoning",
                "text": "Deep thinking in progress"
            }),
            json!({
                "type": "text",
                "text": "Output text"
            }),
        ];

        let modified_count = ensure_thinking_part_flags(&mut parts);
        assert_eq!(modified_count, 1);
        assert_eq!(parts[0]["thought"], true);
        assert_eq!(parts[1].get("thought"), None);
    }

    #[test]
    fn test_estimate_thinking_tokens() {
        assert_eq!(estimate_thinking_tokens(""), 0);
        assert_eq!(estimate_thinking_tokens("   "), 0);
        assert_eq!(estimate_thinking_tokens("hi"), 1); // min 1 for non-empty

        let longer_thought = "This is a detailed reasoning step that spans across multiple sentences and tokens.";
        let tokens = estimate_thinking_tokens(longer_thought);
        assert!(tokens >= 15);
    }

    #[test]
    fn test_display_and_serialization() {
        let format = ThinkingPayloadFormat::CandidatePartThoughtFlag;
        assert_eq!(format.to_string(), "CandidatePartThoughtFlag");

        let action = BackfillAction::ThoughtTagInjected {
            character_offset: 12,
        };
        assert!(action.to_string().contains("offset 12"));

        let report = ThinkingBackfillReport {
            is_modified: true,
            format: ThinkingPayloadFormat::TaggedThought {
                tag_open: "<thought>".to_string(),
                tag_close: "</thought>".to_string(),
            },
            action,
            processed_content: "content".to_string(),
            estimated_thinking_tokens: 42,
        };

        let report_str = report.to_string();
        assert!(report_str.contains("modified=true"));
        assert!(report_str.contains("tokens=42"));

        // Verify JSON serialization roundtrip
        let serialized = serde_json::to_string(&report).expect("must serialize");
        let deserialized: ThinkingBackfillReport =
            serde_json::from_str(&serialized).expect("must deserialize");
        assert_eq!(report, deserialized);
    }
}
