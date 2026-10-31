//! Unit tests for Truncation Omission Banner Formatter (`Imp-683`).

use super::truncation_omission_banner_formatter::*;

#[test]
fn test_format_bytes_human() {
    assert_eq!(format_bytes_human(500), "500 B");
    assert_eq!(format_bytes_human(1024), "1.0 KB");
    assert_eq!(format_bytes_human(2560), "2.5 KB");
    assert_eq!(format_bytes_human(1048576), "1.0 MB");
    assert_eq!(format_bytes_human(5242880), "5.0 MB");
}

#[test]
fn test_parse_truncation_markers_bytes() {
    let content = "System initial context... <truncated 2048 bytes> trailing thoughts.";
    let records = parse_truncation_markers(content);

    assert_eq!(records.len(), 1);
    let rec = &records[0];
    assert_eq!(rec.raw_token, "<truncated 2048 bytes>");
    assert_eq!(rec.kind, TruncationKind::Bytes(2048));
    assert_eq!(rec.formatted_label, "2.0 KB");
    assert_eq!(
        rec.banner_text,
        "⚡ [Omitted 2.0 KB of context - Click to inspect/expand]"
    );
    assert!(rec.banner_html.contains("text-amber-300"));
}

#[test]
fn test_parse_truncation_markers_lines() {
    let content = "Log trace output:\n<truncated 50 lines>\nFinal error.";
    let records = parse_truncation_markers(content);

    assert_eq!(records.len(), 1);
    let rec = &records[0];
    assert_eq!(rec.raw_token, "<truncated 50 lines>");
    assert_eq!(rec.kind, TruncationKind::Lines(50));
    assert_eq!(rec.formatted_label, "50 lines");
    assert_eq!(
        rec.banner_text,
        "⚡ [Omitted 50 lines of context - Click to inspect/expand]"
    );
}

#[test]
fn test_replace_markers_with_banners() {
    let content = "Pre <truncated 1024 bytes> mid <truncated 10 lines> post.";
    let (replaced, count) = replace_markers_with_banners(content);

    assert_eq!(count, 2);
    assert!(replaced.contains("⚡ [Omitted 1.0 KB of context - Click to inspect/expand]"));
    assert!(replaced.contains("⚡ [Omitted 10 lines of context - Click to inspect/expand]"));
    assert!(!replaced.contains("<truncated"));
}
