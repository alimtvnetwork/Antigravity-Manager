//! Unit tests for Streaming Payload Accumulator (`Imp-695`).

use super::streaming_payload_accumulator::*;

#[test]
fn test_normal_text_delta_stream_aggregation() {
    let mut accumulator = StreamingPayloadAccumulator::default();

    let res1 = accumulator.push_text_delta("Hello ").unwrap();
    assert_eq!(res1.action, AccumulatorAction::Appended);
    assert_eq!(res1.bytes_buffered, 6);

    let res2 = accumulator.push_text_delta("world, ").unwrap();
    assert_eq!(res2.action, AccumulatorAction::Appended);

    let res3 = accumulator.push_text_delta("welcome to Gemini!").unwrap();
    assert_eq!(res3.action, AccumulatorAction::Appended);

    let turn = accumulator.finish_turn(Some("STOP".to_string()));
    assert!(turn.is_turn_complete());
    assert_eq!(turn.finish_reason(), Some("STOP"));
    assert_eq!(turn.full_text(), "Hello world, welcome to Gemini!");
    assert_eq!(turn.parts_count(), 1);
    assert!(matches!(&turn.parts()[0], AccumulatedPart::Text(t) if t == "Hello world, welcome to Gemini!"));
}

#[test]
fn test_detect_thought_tags_into_thought_part() {
    let config = StreamingAccumulatorConfig::default().with_detect_thought_tags(true);
    let mut accumulator = StreamingPayloadAccumulator::new(config);

    let input = "<thought>Analyzing the query and calculating response...</thought>Here is the computed answer.";
    let res = accumulator.push_text_delta(input).unwrap();
    assert_eq!(res.action, AccumulatorAction::PartCompleted);

    let turn = accumulator.finish_turn(Some("STOP".to_string()));
    assert_eq!(turn.parts_count(), 2);
    assert!(turn.has_thought());
    assert_eq!(turn.full_thought(), "Analyzing the query and calculating response...");
    assert_eq!(turn.full_text(), "Here is the computed answer.");
}

#[test]
fn test_split_thought_tags_across_chunks() {
    let config = StreamingAccumulatorConfig::default().with_detect_thought_tags(true);
    let mut accumulator = StreamingPayloadAccumulator::new(config);

    // Split open tag across chunks
    accumulator.push_text_delta("<th").unwrap();
    accumulator.push_text_delta("ought>Thinking step 1; ").unwrap();
    accumulator.push_text_delta("Thinking step 2.</th").unwrap();
    accumulator.push_text_delta("ought>Now final response.").unwrap();

    let turn = accumulator.finish_turn(Some("STOP".to_string()));
    assert_eq!(turn.parts_count(), 2);
    assert_eq!(turn.full_thought(), "Thinking step 1; Thinking step 2.");
    assert_eq!(turn.full_text(), "Now final response.");
}

#[test]
fn test_function_call_argument_chunk_accumulation_and_completion() {
    let mut accumulator = StreamingPayloadAccumulator::default();

    let res1 = accumulator
        .push_function_call_chunk("get_weather", "{\"location\": ")
        .unwrap();
    assert_eq!(res1.action, AccumulatorAction::Appended);
    match &res1.current_turn.parts()[0] {
        AccumulatedPart::FunctionCall {
            name,
            args,
            is_complete,
        } => {
            assert_eq!(name, "get_weather");
            assert_eq!(args, "{\"location\": ");
            assert!(!*is_complete);
        }
        _ => panic!("Expected FunctionCall part"),
    }

    let res2 = accumulator
        .push_function_call_chunk("get_weather", "\"Tokyo\", \"unit\": \"celsius\"}")
        .unwrap();
    assert_eq!(res2.action, AccumulatorAction::PartCompleted);
    match &res2.current_turn.parts()[0] {
        AccumulatedPart::FunctionCall {
            name,
            args,
            is_complete,
        } => {
            assert_eq!(name, "get_weather");
            assert_eq!(args, "{\"location\": \"Tokyo\", \"unit\": \"celsius\"}");
            assert!(*is_complete);
        }
        _ => panic!("Expected completed FunctionCall part"),
    }
}

#[test]
fn test_buffer_overflow_guard() {
    let config = StreamingAccumulatorConfig::default().with_max_buffer_bytes(50);
    let mut accumulator = StreamingPayloadAccumulator::new(config);

    assert!(accumulator.push_text_delta("12345678901234567890").is_ok()); // 20 bytes
    assert!(accumulator.push_text_delta("12345678901234567890").is_ok()); // 40 bytes

    // 40 + 20 = 60 > 50 -> must error
    let err = accumulator.push_text_delta("12345678901234567890");
    assert!(err.is_err());
    let err_msg = err.unwrap_err();
    assert!(err_msg.contains("Streaming buffer overflow"));
}

#[test]
fn test_auto_repair_json_on_finish() {
    let config = StreamingAccumulatorConfig::default()
        .with_auto_repair_json(true);
    let mut accumulator = StreamingPayloadAccumulator::new(config);

    // Incomplete JSON with open bracket and brace
    accumulator
        .push_function_call_chunk("search_database", "{\"query\": \"users\", \"filters\": [\"active\"")
        .unwrap();

    let turn = accumulator.finish_turn(Some("STOP".to_string()));
    assert_eq!(turn.parts_count(), 1);
    match &turn.parts()[0] {
        AccumulatedPart::FunctionCall {
            args,
            is_complete,
            ..
        } => {
            assert!(*is_complete);
            assert!(serde_json::from_str::<serde_json::Value>(args).is_ok());
        }
        _ => panic!("Expected FunctionCall part"),
    }
}

#[test]
fn test_inline_data_accumulation() {
    let mut accumulator = StreamingPayloadAccumulator::default();

    let res = accumulator
        .push_inline_data("image/png", "iVBORw0KGgoAAAANSUhEUgAA...")
        .unwrap();
    assert_eq!(res.action, AccumulatorAction::PartCompleted);

    let turn = accumulator.finish_turn(None);
    assert_eq!(turn.parts_count(), 1);
    match &turn.parts()[0] {
        AccumulatedPart::InlineData { mime_type, data } => {
            assert_eq!(mime_type, "image/png");
            assert_eq!(data, "iVBORw0KGgoAAAANSUhEUgAA...");
        }
        _ => panic!("Expected InlineData part"),
    }
}

#[test]
fn test_reset_clears_turn_state() {
    let mut accumulator = StreamingPayloadAccumulator::default();
    accumulator.push_text_delta("Temporary data").unwrap();
    assert_eq!(accumulator.bytes_buffered(), 14);

    accumulator.reset();
    assert_eq!(accumulator.bytes_buffered(), 0);
    assert!(accumulator.current_turn().is_empty());
    assert!(!accumulator.is_in_thought());
}

#[test]
fn test_legacy_ingest_chunk_and_finalize_turn() {
    let mut accumulator = StreamingPayloadAccumulator::default();

    accumulator.ingest_chunk("Hello ").unwrap();
    accumulator.ingest_chunk("world!").unwrap();

    let turn = accumulator.finalize_turn();
    assert_eq!(turn.text, "Hello world!");
    assert_eq!(turn.thought, None);
    assert!(turn.has_content());
    assert_eq!(turn.total_bytes, 12);
}
