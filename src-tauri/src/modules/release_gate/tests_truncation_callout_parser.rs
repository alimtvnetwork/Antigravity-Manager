#[cfg(test)]
mod tests {
    use super::super::truncation_callout_parser_guard::{
        extract_truncation_callouts, format_dark_glass_html, format_omission_banner_text,
        has_raw_truncation_markers, parse_and_transform_truncations,
        replace_raw_truncation_markers_with_banners, replace_raw_truncation_markers_with_html,
        validate_no_raw_truncation_markers, TruncationUnit,
    };

    #[test]
    fn test_extract_byte_truncation_callout() {
        let text = "Context snippet: <truncated 2048 bytes> remaining conversation.";
        let callouts = extract_truncation_callouts(text);

        assert_eq!(callouts.len(), 1);
        let callout = &callouts[0];
        assert_eq!(callout.raw_marker, "<truncated 2048 bytes>");
        assert_eq!(callout.quantity, 2048);
        assert_eq!(callout.unit, TruncationUnit::Bytes);
        assert_eq!(
            callout.banner_text,
            "⚡ [Omitted 2048 bytes of context - Click to inspect/expand]"
        );
        assert!(callout
            .html_representation
            .contains("data-omitted-quantity=\"2048\""));
        assert!(callout
            .html_representation
            .contains("data-omitted-unit=\"bytes\""));
    }

    #[test]
    fn test_extract_line_truncation_callout() {
        let text = "Log tail follows:\n<truncated 75 lines>\nProcess exit code 0.";
        let callouts = extract_truncation_callouts(text);

        assert_eq!(callouts.len(), 1);
        let callout = &callouts[0];
        assert_eq!(callout.raw_marker, "<truncated 75 lines>");
        assert_eq!(callout.quantity, 75);
        assert_eq!(callout.unit, TruncationUnit::Lines);
        assert_eq!(
            callout.banner_text,
            "⚡ [Omitted 75 lines of context - Click to inspect/expand]"
        );
    }

    #[test]
    fn test_singular_byte_and_line_labels() {
        assert_eq!(
            format_omission_banner_text(1, TruncationUnit::Bytes),
            "⚡ [Omitted 1 byte of context - Click to inspect/expand]"
        );
        assert_eq!(
            format_omission_banner_text(1, TruncationUnit::Lines),
            "⚡ [Omitted 1 line of context - Click to inspect/expand]"
        );
    }

    #[test]
    fn test_case_insensitive_matching() {
        let text = "First: <TRUNCATED 1024 BYTES>, Second: <Truncated 12 Lines>";
        let callouts = extract_truncation_callouts(text);

        assert_eq!(callouts.len(), 2);
        assert_eq!(callouts[0].quantity, 1024);
        assert_eq!(callouts[0].unit, TruncationUnit::Bytes);
        assert_eq!(callouts[1].quantity, 12);
        assert_eq!(callouts[1].unit, TruncationUnit::Lines);
    }

    #[test]
    fn test_range_marker_variation() {
        let text = "Log excerpt: <truncated 923 of 10767 bytes from the middle of the output>";
        let callouts = extract_truncation_callouts(text);

        assert_eq!(callouts.len(), 1);
        let callout = &callouts[0];
        assert_eq!(callout.quantity, 923);
        assert_eq!(callout.unit, TruncationUnit::Bytes);
        assert_eq!(
            callout.banner_text,
            "⚡ [Omitted 923 bytes of context - Click to inspect/expand]"
        );
    }

    #[test]
    fn test_replace_raw_truncation_markers_with_banners() {
        let raw = "Header\n<truncated 50 lines>\nMiddle\n<truncated 4096 bytes>\nFooter";
        let replaced = replace_raw_truncation_markers_with_banners(raw);

        assert!(!has_raw_truncation_markers(&replaced));
        assert!(replaced.contains("⚡ [Omitted 50 lines of context - Click to inspect/expand]"));
        assert!(replaced.contains("⚡ [Omitted 4096 bytes of context - Click to inspect/expand]"));
    }

    #[test]
    fn test_replace_raw_truncation_markers_with_html() {
        let raw = "Warning: <truncated 300 bytes> in transcript.";
        let html = replace_raw_truncation_markers_with_html(raw);

        assert!(!has_raw_truncation_markers(&html));
        assert!(html.contains("interactive-dark-glass-omission-banner"));
        assert!(html.contains("data-omitted-quantity=\"300\""));
        assert!(html.contains("data-omitted-unit=\"bytes\""));
        assert!(html.contains("backdrop-blur-md"));
    }

    #[test]
    fn test_validate_no_raw_truncation_markers() {
        let clean = "Clean text without any truncation markers.";
        assert!(validate_no_raw_truncation_markers(clean).is_ok());

        let dirty = "System prompt: <truncated 120 lines>";
        let validation_err = validate_no_raw_truncation_markers(dirty);
        assert!(validation_err.is_err());
        let violations = validation_err.unwrap_err();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("<truncated 120 lines>"));
    }

    #[test]
    fn test_parse_and_transform_truncations_pipeline() {
        let text = "Log start\n<truncated 500 bytes>\nLog end";
        let result = parse_and_transform_truncations(text);

        assert!(result.has_truncations);
        assert_eq!(result.callouts.len(), 1);
        assert_eq!(result.raw_text, text);
        assert!(result
            .formatted_text
            .contains("⚡ [Omitted 500 bytes of context - Click to inspect/expand]"));
        assert!(result
            .html_text
            .contains("interactive-dark-glass-omission-banner"));
    }

    #[test]
    fn test_no_truncation_clean_text() {
        let clean = "Standard transcript turn with complete content.";
        assert!(!has_raw_truncation_markers(clean));
        let result = parse_and_transform_truncations(clean);
        assert!(!result.has_truncations);
        assert!(result.callouts.is_empty());
        assert_eq!(result.formatted_text, clean);
        assert_eq!(result.html_text, clean);
    }
}
