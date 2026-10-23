#[cfg(test)]
mod tests {
    use super::super::transcript_deep_inspection_scanner::{
        is_ignorable_telemetry_record, scan_transcript_lines, select_transcript_file_source,
        TRANSCRIPT_COMPACT_FILENAME, TRANSCRIPT_FULL_FILENAME,
    };

    #[test]
    fn test_select_transcript_file_prefers_full_log() {
        assert_eq!(
            select_transcript_file_source(true, true),
            Some(TRANSCRIPT_FULL_FILENAME)
        );
        assert_eq!(
            select_transcript_file_source(false, true),
            Some(TRANSCRIPT_COMPACT_FILENAME)
        );
        assert_eq!(select_transcript_file_source(false, false), None);
    }

    #[test]
    fn test_is_ignorable_telemetry_record() {
        assert!(is_ignorable_telemetry_record("{\"type\": \"TOKEN_USAGE\", \"tokens\": 120}"));
        assert!(is_ignorable_telemetry_record("{\"type\": \"HEARTBEAT\", \"alive\": true}"));
        assert!(is_ignorable_telemetry_record("{\"type\": \"TELEMETRY\", \"latency\": 12}"));
        assert!(!is_ignorable_telemetry_record("{\"type\": \"PLANNER_RESPONSE\", \"tool_calls\": []}"));
    }

    #[test]
    fn test_scan_transcript_lines_bypasses_telemetry_extracts_tools_and_thoughts() {
        let lines = vec![
            "{\"type\": \"USER_INPUT\", \"content\": \"Implement feature\"}",
            "{\"type\": \"PLANNER_RESPONSE\", \"thinking\": \"Planning steps...\", \"tool_calls\": [{\"name\": \"write_file\"}]}",
            "{\"type\": \"TOKEN_USAGE\", \"count\": 500}",
            "{\"type\": \"HEARTBEAT\", \"ts\": 12345}",
            "{\"type\": \"TELEMETRY\", \"event\": \"turn_end\"}",
        ];

        let report = scan_transcript_lines(&lines, 10, TRANSCRIPT_FULL_FILENAME);
        assert_eq!(report.total_scanned, 5);
        assert_eq!(report.bypassed_telemetry_count, 3);
        assert_eq!(report.tool_calls_count, 1);
        assert_eq!(report.thoughts_count, 1);
        assert!(report.has_genuine_turn);
        assert_eq!(report.selected_file_source, TRANSCRIPT_FULL_FILENAME);
    }
}
