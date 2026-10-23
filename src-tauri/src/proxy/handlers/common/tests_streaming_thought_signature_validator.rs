//! Unit tests for Streaming Thought Signature & Delimiter Validator (`Imp-710`).

use super::streaming_thought_signature_validator::*;

#[test]
fn test_default_state_inactive() {
    let state = ThoughtTagState::default();
    assert_eq!(state, ThoughtTagState::Inactive);
    assert!(state.is_inactive());
    assert!(!state.is_inside());
    assert_eq!(format!("{state}"), "Inactive");
}

#[test]
fn test_compliant_balanced_thought_single_chunk() {
    let mut state = ThoughtTagState::Inactive;
    let delta = "<thought>Calculating token budget and headroom</thought>Final answer.";

    let report = validate_thought_stream_chunk(delta, &mut state);

    assert!(report.is_compliant());
    assert_eq!(report.total_thought_bytes, 37);
    assert!(report.findings.iter().any(|f| f.is_balanced()));
    assert_eq!(state, ThoughtTagState::ClosingTagEncountered);
    assert!(!report.has_unclosed());
    assert!(!report.has_unexpected_closing());
    assert!(!report.has_mismatched_flag());
}

#[test]
fn test_streaming_multi_chunk_lifecycle() {
    let mut state = ThoughtTagState::Inactive;

    // Chunk 1: Starts thought block
    let chunk1 = "data: <thought>Step 1: Inspect repository structure";
    let report1 = validate_thought_stream_chunk(chunk1, &mut state);
    assert!(!report1.is_compliant()); // unclosed chunk is detected
    assert!(report1.has_unclosed());
    assert_eq!(state, ThoughtTagState::InsideThoughtBlock);
    assert!(state.is_inside());

    // Chunk 2: Mid-thought continuation
    let chunk2 = " and verify pipeline invariants";
    let report2 = validate_thought_stream_chunk(chunk2, &mut state);
    assert!(!report2.is_compliant());
    assert!(report2.has_unclosed());
    assert_eq!(state, ThoughtTagState::InsideThoughtBlock);
    assert_eq!(report2.total_thought_bytes, chunk2.len());

    // Chunk 3: Closes thought block and outputs response
    let chunk3 = "</thought>\nHere is the validated solution.";
    let report3 = validate_thought_stream_chunk(chunk3, &mut state);
    assert!(report3.is_compliant());
    assert!(report3.findings.iter().any(|f| f.is_balanced()));
    assert_eq!(state, ThoughtTagState::ClosingTagEncountered);

    // Chunk 4: Following text chunk outside thought
    let chunk4 = " Continuation after thought.";
    let report4 = validate_thought_stream_chunk(chunk4, &mut state);
    assert!(report4.is_compliant());
    assert_eq!(state, ThoughtTagState::Inactive);
    assert_eq!(report4.total_thought_bytes, 0);
}

#[test]
fn test_unexpected_closing_tag() {
    let mut state = ThoughtTagState::Inactive;
    let delta = "Spurious closing tag </thought> without prior open tag.";

    let report = validate_thought_stream_chunk(delta, &mut state);

    assert!(!report.is_compliant());
    assert!(report.has_unexpected_closing());
    assert!(report
        .findings
        .iter()
        .any(|f| matches!(f, ThoughtValidationFinding::UnexpectedClosingTag)));
}

#[test]
fn test_mismatched_thought_flag_false_inside() {
    let mut state = ThoughtTagState::InsideThoughtBlock;
    let delta = r#"{"text": "still thinking", "thought": false}"#;

    let report = validate_thought_stream_chunk(delta, &mut state);

    assert!(!report.is_compliant());
    assert!(report.has_mismatched_flag());
}

#[test]
fn test_mismatched_thought_flag_true_outside() {
    let mut state = ThoughtTagState::Inactive;
    let delta = r#"{"text": "regular response", "thought": true}"#;

    let report = validate_thought_stream_chunk(delta, &mut state);

    assert!(!report.is_compliant());
    assert!(report.has_mismatched_flag());
}

#[test]
fn test_sanitization_of_internal_markers() {
    let mut state = ThoughtTagState::Inactive;
    let delta = "[Thought] [Thinking] <thought>Analyzing user intent</thought>";

    let report = validate_thought_stream_chunk(delta, &mut state);

    assert!(!report.sanitized_payload.contains("[Thought]"));
    assert!(!report.sanitized_payload.contains("[Thinking]"));
    assert!(report.sanitized_payload.contains("<thought>"));
}

#[test]
fn test_normalization_of_alt_thinking_tags() {
    let mut state = ThoughtTagState::Inactive;
    let delta = "<thinking>Analyzing user intent</thinking>";

    let report = validate_thought_stream_chunk(delta, &mut state);

    assert!(report.sanitized_payload.contains("<thought>"));
    assert!(report.sanitized_payload.contains("</thought>"));
    assert!(!report.sanitized_payload.contains("<thinking>"));
    assert!(!report.sanitized_payload.contains("</thinking>"));
    assert!(report.is_compliant());
}

#[test]
fn test_repair_truncated_thought_block_unclosed() {
    let unclosed = "<thought>Step 1: Check dependencies and build parameters";
    let repaired = repair_truncated_thought_block(unclosed);

    assert_eq!(
        repaired,
        "<thought>Step 1: Check dependencies and build parameters</thought>"
    );
}

#[test]
fn test_repair_truncated_thought_block_partial_tag() {
    let partial = "<thought>Reasoning completed.</thou";
    let repaired = repair_truncated_thought_block(partial);

    assert_eq!(repaired, "<thought>Reasoning completed.</thought>");
}

#[test]
fn test_repair_truncated_thought_block_missing_open() {
    let missing_open = "Finished reasoning about the architectural plan</thought>";
    let repaired = repair_truncated_thought_block(missing_open);

    assert_eq!(
        repaired,
        "<thought>Finished reasoning about the architectural plan</thought>"
    );
}

#[test]
fn test_repair_truncated_thought_block_already_balanced() {
    let balanced = "<thought>All checks passed</thought>\nReady to proceed.";
    let repaired = repair_truncated_thought_block(balanced);

    assert_eq!(repaired, balanced);
}

#[test]
fn test_repair_truncated_thought_block_empty_and_plain() {
    assert_eq!(repair_truncated_thought_block(""), "");
    assert_eq!(
        repair_truncated_thought_block("Hello world without thought"),
        "Hello world without thought"
    );
}

#[test]
fn test_report_display_formatting() {
    let report = ThoughtStreamValidationReport::new(
        true,
        42,
        vec![ThoughtValidationFinding::Balanced],
        "test payload".to_string(),
    );

    let display_str = format!("{report}");
    assert!(display_str.contains("compliant=true"));
    assert!(display_str.contains("thought_bytes=42"));
    assert!(display_str.contains("findings=1"));
}
