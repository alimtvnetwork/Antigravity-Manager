#[cfg(test)]
mod tests {
    use super::super::contributor_cost_disclosure_linter::{
        format_rejection_breakdown, lint_contributor_pr, lint_rejection_response,
        FileDecision, FileDisposition, RejectionBreakdown,
    };

    #[test]
    fn test_lint_contributor_pr_fully_compliant() {
        let pr = r#"
## Cost Disclosure
This PR refactors the streaming buffer for upstream proxy responses.

### Improvements
Reduces memory consumption by 40% during large token bursts and prevents out-of-memory panics.

### Affected Existing Behaviors
Stream buffer flush interval changes from fixed 100ms to adaptive 20ms-250ms based on queue depth.

### Unverified Paths
High-throughput socket connections under Windows ARM64 hardware cannot be verified in current CI.

Co-authored-by: Sarah Connor <sarah@resistance.org>
"#;

        let report = lint_contributor_pr(pr, false);
        assert!(report.is_merge_ready);
        assert!(report.has_cost_disclosure);
        assert!(report.has_improvements);
        assert!(report.has_affected_behaviors);
        assert!(report.has_unverified_paths);
        assert!(report.has_valid_coauthors);
        assert_eq!(report.coauthors.len(), 1);
        assert_eq!(report.coauthors[0].name, "Sarah Connor");
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_lint_contributor_pr_missing_cost_disclosure() {
        let pr = r#"
## Improvements
Added a new button in UI toolbar.
"#;
        let report = lint_contributor_pr(pr, false);
        assert!(!report.is_merge_ready);
        assert!(!report.has_cost_disclosure);
        assert!(report.violations.iter().any(|v| v.contains("Cost Disclosure")));
    }

    #[test]
    fn test_lint_contributor_pr_trivial_affected_behaviors() {
        let pr = r#"
## Cost Disclosure

### Improvements
Optimized database connection pooling.

### Affected Existing Behaviors
N/A

### Unverified Paths
Production cluster deployment under multi-node master election.
"#;
        let report = lint_contributor_pr(pr, false);
        assert!(!report.is_merge_ready);
        assert!(!report.has_affected_behaviors);
        assert!(report.violations.iter().any(|v| v.contains("trivial placeholder")));
    }

    #[test]
    fn test_lint_contributor_pr_proxy_pr_with_valid_trailer() {
        let pr = r#"
## Cost Disclosure
Proxy PR submitted on behalf of @external_contributor.

### Improvements
Fixes UTF-8 truncation in log stream deserializer.

### Affected Existing Behaviors
Truncated multi-byte characters are now preserved until the next chunk rather than replaced with U+FFFD.

### Unverified Paths
Non-UTF8 legacy encodings (GBK/Shift-JIS) in external terminals.

Co-authored-by: Alice Smith <alice@example.com>
"#;
        let report = lint_contributor_pr(pr, true);
        assert!(report.is_merge_ready);
        assert!(!report.proxy_missing_coauthor);
        assert!(report.has_valid_coauthors);
        assert_eq!(report.coauthors[0].name, "Alice Smith");
    }

    #[test]
    fn test_lint_contributor_pr_proxy_pr_missing_trailer() {
        let pr = r#"
## Cost Disclosure
This is a proxy PR for @external_contributor who submitted a patch via email.

### Improvements
Cleaned up dead code in route dispatcher.

### Affected Existing Behaviors
Removed deprecated legacy route endpoint /v1/old-stream.

### Unverified Paths
Third-party clients calling deprecated /v1/old-stream.
"#;
        let report = lint_contributor_pr(pr, true);
        assert!(!report.is_merge_ready);
        assert!(report.proxy_missing_coauthor);
        assert!(report.violations.iter().any(|v| v.contains("Proxy PR submitted on behalf of contributor must attach")));
    }

    #[test]
    fn test_lint_contributor_pr_malformed_trailer() {
        let pr = r#"
## Cost Disclosure

### Improvements
Added helper functions.

### Affected Existing Behaviors
No change to public API contracts.

### Unverified Paths
None tested outside Linux x86_64.

Co-authored-by: Bob Jones without email
"#;
        let report = lint_contributor_pr(pr, false);
        assert!(!report.is_merge_ready);
        assert!(report.violations.iter().any(|v| v.contains("Malformed Co-authored-by trailer")));
    }

    #[test]
    fn test_format_and_lint_rejection_breakdown() {
        let breakdown = RejectionBreakdown {
            contributor_handle: "@contributor_dev".to_string(),
            pr_reference: "#412".to_string(),
            file_decisions: vec![
                FileDecision {
                    file_path: "src-tauri/src/modules/proxy.rs".to_string(),
                    disposition: FileDisposition::Accept,
                    rationale: "Clean socket boundary refactoring with proper error propagation.".to_string(),
                },
                FileDecision {
                    file_path: "src-tauri/src/modules/legacy.rs".to_string(),
                    disposition: FileDisposition::Drop,
                    rationale: "Dead code file already deprecated in sequence 180.".to_string(),
                },
                FileDecision {
                    file_path: "src/components/Header.tsx".to_string(),
                    disposition: FileDisposition::NeedsRevision,
                    rationale: "Please wrap toolbar actions in segmented pill capsule per AGENTS.md.".to_string(),
                },
            ],
            actionable_path_forward: "1. Rebase on origin/beta.\n2. Apply segmented pill styling to Header.tsx.\n3. Resubmit updated PR.".to_string(),
            owned_tooling_or_doc_gaps: vec![
                "Our capsule component documentation lacked explicit examples in AGENTS.md.".to_string(),
                "CI linter did not catch legacy file references prior to manual review.".to_string(),
            ],
        };

        let formatted = format_rejection_breakdown(&breakdown);
        assert!(formatted.contains("### Contributor Review Feedback for PR #412"));
        assert!(formatted.contains("| `src-tauri/src/modules/proxy.rs` | **Accept** |"));
        assert!(formatted.contains("| `src-tauri/src/modules/legacy.rs` | **Drop** |"));
        assert!(formatted.contains("| `src/components/Header.tsx` | **Needs Revision** |"));
        assert!(formatted.contains("Tooling & Documentation Gaps (Maintainer Owned)"));
        assert!(formatted.contains("Actionable Path Forward"));

        let review_report = lint_rejection_response(&formatted);
        assert!(review_report.is_compliant);
        assert!(review_report.has_file_breakdown);
        assert!(review_report.has_actionable_path_forward);
        assert!(review_report.has_owned_tooling_gaps);
        assert!(review_report.missing_requirements.is_empty());
    }

    #[test]
    fn test_lint_rejection_response_non_compliant() {
        let simple_rejection = "Thanks, but we decided not to merge this PR.";
        let review_report = lint_rejection_response(simple_rejection);

        assert!(!review_report.is_compliant);
        assert!(!review_report.has_file_breakdown);
        assert!(!review_report.has_actionable_path_forward);
        assert!(!review_report.has_owned_tooling_gaps);
        assert_eq!(review_report.missing_requirements.len(), 3);
    }
}
