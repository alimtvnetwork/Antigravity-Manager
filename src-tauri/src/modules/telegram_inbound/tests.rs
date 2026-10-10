use chrono::Utc;

use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    pub(crate) fn test_chunk_telegram_text_short() {
        let text = "Hello world";
        let chunks = chunk_telegram_text(text, 100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Hello world");
    }

    #[test]
    pub(crate) fn test_chunk_telegram_text_splits_at_newline() {
        let text = "Line 1\nLine 2\nLine 3\nLine 4";
        let chunks = chunk_telegram_text(text, 14);
        assert!(chunks.len() >= 2);
        for chunk in &chunks {
            assert!(chunk.len() <= 14);
        }
    }

    #[test]
    pub(crate) fn test_shorten_project_name() {
        assert_eq!(shorten_project_name("Antigravity-Manager"), "AGM");
        assert_eq!(shorten_project_name("antigravity_manager_v2"), "AGM");
        assert_eq!(shorten_project_name("gitmap-cli"), "GitMap");
        assert_eq!(shorten_project_name("frontend-a8b21c"), "frontend");
        assert_eq!(shorten_project_name("custom-tool"), "custom-tool");
    }

    #[test]
    pub(crate) fn test_clean_for_telegram_html_truncation() {
        let input = "This is a very long prompt line that should be cleanly truncated without line break artifacts";
        let cleaned = clean_for_telegram_html(input, 20);
        assert!(cleaned.ends_with("..."));
        assert!(!cleaned.contains("\n... [truncated]"));
        assert!(!cleaned.ends_with("...."));

        // Verify that strings ending with dots do not produce stacked dots (e.g. .....)
        let with_dots = "Prompt ending with dots...";
        let cleaned_dots = clean_for_telegram_html(with_dots, 25);
        assert!(cleaned_dots.ends_with("..."));
        assert!(!cleaned_dots.ends_with("...."));
    }

    #[test]
    pub(crate) fn test_format_running_duration() {
        let now = chrono::Utc::now().timestamp();
        assert_eq!(format_running_duration(now - 30), "(running 30s)");
        assert_eq!(format_running_duration(now - 125), "(running 2m 5s)");
        assert_eq!(format_running_duration(0), "");
    }

    #[test]
    pub(crate) fn test_format_observe_report_structure() {
        let report = format_observe_report();
        assert!(report.contains("🤖 <b>AGM v"));
        assert!(report.contains("Status</b>"));
        assert!(report.contains("• <b>Machine:</b> "));
        assert!(report.contains("• <b>Alias:</b> "));
        assert!(report.contains("• <b>IP:</b> "));
        assert!(report.contains("• <b>Build:</b> v"));
        assert!(report.contains("• <b>Active Account:</b> "));
        assert!(report.contains("• <b>Quota / Tier:</b> "));
        assert!(report.contains("switch threshold:"));
        assert!(report.contains("🟢 <b>Running:</b>"));
        assert!(!report.contains("Backup Batches:"));
        assert!(report.contains("Send <code>/expand"));
    }
}
