//! Unit tests for Session Trace UUID Generator.

use super::session_trace_uuid::SessionTraceUuid;

#[test]
fn test_generate_session_trace_default() {
    let trace_id = SessionTraceUuid::generate("");
    assert!(trace_id.starts_with("agm-inst-default-"));

    let parsed = SessionTraceUuid::parse(&trace_id);
    assert!(parsed.is_some());
    let p = parsed.unwrap();
    assert_eq!(p.instance_tag, "default");
    assert!(p.timestamp_epoch_ms > 0);
    assert!(!p.random_token.is_empty());
}

#[test]
fn test_generate_session_trace_custom_tag() {
    let trace_id = SessionTraceUuid::generate("worker-01");
    assert!(trace_id.starts_with("agm-inst-worker-01-"));

    let parsed = SessionTraceUuid::parse(&trace_id);
    assert!(parsed.is_some());
    let p = parsed.unwrap();
    assert_eq!(p.instance_tag, "worker-01");
}

#[test]
fn test_parse_invalid_trace_returns_none() {
    assert!(SessionTraceUuid::parse("not-a-trace").is_none());
    assert!(SessionTraceUuid::parse("agm-other-1234").is_none());
    assert!(SessionTraceUuid::parse("agm-inst-tag-notanumber-rand").is_none());
}
