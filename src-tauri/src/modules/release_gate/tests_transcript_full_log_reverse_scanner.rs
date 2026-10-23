#[cfg(test)]
mod tests {
    use super::super::transcript_full_log_reverse_scanner::{
        is_noise_record, reverse_scan_transcript, reverse_scan_transcript_str,
        select_transcript_file, ExtractedExecutionArtifact, ReverseScanReport,
        TranscriptRecordType, MINIMUM_REVERSE_SCAN_LINES, NOISE_RECORD_TYPES,
        TRANSCRIPT_COMPACT_FILENAME, TRANSCRIPT_FULL_FILENAME,
    };

    #[test]
    fn test_select_transcript_file_prefers_full_log() {
        assert_eq!(
            select_transcript_file(true, true),
            Some(TRANSCRIPT_FULL_FILENAME)
        );
        assert_eq!(
            select_transcript_file(true, false),
            Some(TRANSCRIPT_FULL_FILENAME)
        );
        assert_eq!(
            select_transcript_file(false, true),
            Some(TRANSCRIPT_COMPACT_FILENAME)
        );
        assert_eq!(select_transcript_file(false, false), None);
    }

    #[test]
    fn test_is_noise_record_detection() {
        for &noise in NOISE_RECORD_TYPES {
            let json = format!("{{\"type\": \"{}\", \"val\": 123}}", noise);
            assert!(is_noise_record(&json), "Failed to detect noise: {}", noise);
            let lower_json = format!("{{\"type\": \"{}\"}}", noise.to_ascii_lowercase());
            assert!(
                is_noise_record(&lower_json),
                "Failed to detect lower noise: {}",
                noise
            );
        }
        assert!(!is_noise_record("{\"type\": \"PLANNER_RESPONSE\"}"));
        assert!(!is_noise_record("{\"type\": \"USER_INPUT\", \"content\": \"hello\"}"));
    }

    #[test]
    fn test_reverse_scan_skips_trailing_noise_and_extracts_genuine_execution() {
        let lines = vec![
            "{\"type\": \"USER_INPUT\", \"content\": \"Refactor pipeline\"}",
            "{\"type\": \"PLANNER_RESPONSE\", \"thinking\": \"Step 1: Check invariants.\", \"tool_calls\": [{\"name\": \"run_command\"}]}",
            "{\"type\": \"TOKEN_USAGE\", \"prompt_tokens\": 1500, \"completion_tokens\": 320}",
            "{\"type\": \"HEARTBEAT\", \"status\": \"alive\", \"timestamp\": 1700000000}",
            "{\"type\": \"TELEMETRY\", \"event\": \"turn_latency\", \"ms\": 450}",
            "{\"type\": \"METRICS\", \"cpu\": 12.5}",
            "{\"type\": \"PING\", \"seq\": 99}",
        ];

        let report = reverse_scan_transcript_str(&lines, 10);
        assert!(report.is_full_log_preferred);
        assert_eq!(report.total_lines_scanned, 7);
        assert_eq!(report.noise_records_skipped, 5);
        assert!(report.found_genuine_execution);

        let tool_artifacts: Vec<_> = report
            .extracted_artifacts
            .iter()
            .filter(|a| matches!(a.artifact_type, TranscriptRecordType::ToolCalls(_)))
            .collect();
        assert_eq!(tool_artifacts.len(), 1);
        if let TranscriptRecordType::ToolCalls(names) = &tool_artifacts[0].artifact_type {
            assert_eq!(names, &vec!["run_command".to_string()]);
        }

        let thinking_artifacts: Vec<_> = report
            .extracted_artifacts
            .iter()
            .filter(|a| matches!(a.artifact_type, TranscriptRecordType::ThinkingContent(_)))
            .collect();
        assert_eq!(thinking_artifacts.len(), 1);
    }

    #[test]
    fn test_minimum_scan_lines_enforced() {
        let lines: Vec<String> = (0..15)
            .map(|i| {
                if i == 5 {
                    "{\"type\": \"PLANNER_RESPONSE\", \"tool_calls\": [{\"name\": \"exec\"}]}"
                        .to_string()
                } else {
                    format!("{{\"type\": \"TELEMETRY\", \"index\": {}}}", i)
                }
            })
            .collect();

        // Pass 3 lines scan limit, minimum 10 must be enforced
        let report = reverse_scan_transcript(&lines, 3);
        assert_eq!(report.total_lines_scanned, MINIMUM_REVERSE_SCAN_LINES);
        assert!(report.found_genuine_execution);
    }

    #[test]
    fn test_display_and_artifact_helpers() {
        let artifact = ExtractedExecutionArtifact::new(
            42,
            TranscriptRecordType::ThinkingContent("Reasoning about bug".to_string()),
        );
        assert!(artifact.is_genuine());
        assert_eq!(
            format!("{}", artifact),
            "Line 42: ThinkingContent(18 chars)"
        );

        let noise_artifact = ExtractedExecutionArtifact::new(
            10,
            TranscriptRecordType::IgnoredNoise("HEARTBEAT".to_string()),
        );
        assert!(!noise_artifact.is_genuine());
        assert_eq!(format!("{}", noise_artifact), "Line 10: IgnoredNoise(HEARTBEAT)");

        let mut report = ReverseScanReport::new(true);
        report.total_lines_scanned = 12;
        report.noise_records_skipped = 4;
        report.found_genuine_execution = true;
        let display = format!("{}", report);
        assert!(display.contains("scanned=12"));
        assert!(display.contains("skipped_noise=4"));
        assert!(display.contains("genuine=true"));
    }

    #[test]
    fn test_empty_and_whitespace_lines() {
        let lines = vec!["   ", "", "\t\n"];
        let report = reverse_scan_transcript_str(&lines, 10);
        assert_eq!(report.total_lines_scanned, 0);
        assert_eq!(report.noise_records_skipped, 0);
        assert!(!report.found_genuine_execution);
    }
}
