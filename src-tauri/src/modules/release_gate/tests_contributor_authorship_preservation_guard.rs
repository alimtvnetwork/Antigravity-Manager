//! Unit tests for Contributor Authorship Preservation & Cost Disclosure Guard (`Imp-699`).

#[cfg(test)]
mod tests {
    use super::super::contributor_authorship_preservation_guard::{
        attach_coauthor_trailers, format_coauthor_trailer, format_coauthor_trailers,
        format_contributor_coauthor_trailer, parse_coauthor_trailers_from_msg,
        parse_cost_disclosure_from_text, validate_contributor_authorship, AuthorshipPreservationReport,
        CommitType, CostDisclosure,
    };

    #[test]
    fn test_format_coauthor_trailer_and_helpers() {
        let single = format_coauthor_trailer("Alice Smith", "alice@example.com");
        assert_eq!(single, "Co-authored-by: Alice Smith <alice@example.com>");

        let alias = format_contributor_coauthor_trailer("Bob Vance", "<bob@vance.com>");
        assert_eq!(alias, "Co-authored-by: Bob Vance <bob@vance.com>");

        let list = format_coauthor_trailers(&[
            ("Alice", "alice@example.com"),
            ("Bob", "bob@example.com"),
        ]);
        assert_eq!(list.len(), 2);
        assert_eq!(list[0], "Co-authored-by: Alice <alice@example.com>");
        assert_eq!(list[1], "Co-authored-by: Bob <bob@example.com>");

        let body = "feat: optimize pipeline dispatcher";
        let with_trailers = attach_coauthor_trailers(body, &list);
        assert!(with_trailers.contains("feat: optimize pipeline dispatcher\n\n"));
        assert!(with_trailers.contains("Co-authored-by: Alice <alice@example.com>\n"));
        assert!(with_trailers.contains("Co-authored-by: Bob <bob@example.com>\n"));
    }

    #[test]
    fn test_commit_type_properties() {
        assert!(CommitType::SquashFromContributorPr.is_squash());
        assert!(!CommitType::SquashFromContributorPr.is_merge());
        assert!(!CommitType::SquashFromContributorPr.is_proxy_pr());
        assert_eq!(
            CommitType::SquashFromContributorPr.as_str(),
            "squash_from_contributor_pr"
        );

        assert!(CommitType::MergeCommit.is_merge());
        assert_eq!(CommitType::MergeCommit.as_str(), "merge_commit");

        assert!(CommitType::ProxyPrCommit.is_proxy_pr());
        assert_eq!(CommitType::ProxyPrCommit.as_str(), "proxy_pr_commit");
    }

    #[test]
    fn test_parse_coauthor_trailers_from_msg() {
        let commit_msg = r#"
feat: add telemetry streaming

Co-authored-by: Sarah Connor <sarah@resistance.org>
Co-authored-by: John Connor <john@resistance.org>
Co-authored-by: Malformed Bare Handle
"#;

        let (valid, invalid) = parse_coauthor_trailers_from_msg(commit_msg);
        assert_eq!(valid.len(), 2);
        assert_eq!(
            valid[0],
            "Co-authored-by: Sarah Connor <sarah@resistance.org>"
        );
        assert_eq!(
            valid[1],
            "Co-authored-by: John Connor <john@resistance.org>"
        );
        assert_eq!(invalid.len(), 1);
        assert_eq!(invalid[0], "Co-authored-by: Malformed Bare Handle");
    }

    #[test]
    fn test_compliant_contributor_squash_commit() {
        let msg = "feat: reduce idle memory overhead";
        let cost = CostDisclosure::new(
            vec!["Socket pool connection timeout changed from 30s to 15s".to_string()],
            vec!["Windows ARM64 execution unverified in CI".to_string()],
        );

        let report = validate_contributor_authorship(
            msg,
            CommitType::SquashFromContributorPr,
            Some("Sarah Connor <sarah@resistance.org>"),
            Some(&cost),
        );

        assert!(report.is_compliant);
        assert!(report.has_valid_author);
        assert!(report.cost_disclosure.has_disclosed_costs);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_squash_commit_missing_original_contributor() {
        let msg = "fix: edge case parsing";
        let cost = CostDisclosure::new(
            vec!["Regex parser updated".to_string()],
            vec!["None".to_string()],
        );

        let report = validate_contributor_authorship(
            msg,
            CommitType::SquashFromContributorPr,
            None,
            Some(&cost),
        );

        assert!(!report.is_compliant);
        assert!(!report.has_valid_author);
        assert!(report
            .violations
            .iter()
            .any(|v| v.contains("requires identified original contributor")));
    }

    #[test]
    fn test_compliant_merge_commit_with_coauthor_trailer() {
        let msg = r#"
Merge branch 'feature/speedup' from contributor

Co-authored-by: Bob Builder <bob@example.com>
"#;
        let cost = CostDisclosure::new(
            vec!["Cache eviction policy now LRU".to_string()],
            vec!["Multi-node cluster unverified".to_string()],
        );

        let report = validate_contributor_authorship(
            msg,
            CommitType::MergeCommit,
            Some("Bob Builder"),
            Some(&cost),
        );

        assert!(report.is_compliant);
        assert!(report.has_valid_author);
        assert_eq!(report.trailer_count(), 1);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_merge_commit_missing_coauthor_trailer_rejected() {
        let msg = "Merge branch 'feature/speedup' from contributor";
        let cost = CostDisclosure::new(
            vec!["Database schema updated".to_string()],
            vec![],
        );

        let report = validate_contributor_authorship(
            msg,
            CommitType::MergeCommit,
            Some("Bob Builder"),
            Some(&cost),
        );

        assert!(!report.is_compliant);
        assert!(!report.has_valid_author);
        assert!(report
            .violations
            .iter()
            .any(|v| v.contains("Merge commit requires explicit `Co-authored-by:` trailer")));
    }

    #[test]
    fn test_merge_commit_trailer_mismatch_original_contributor() {
        let msg = r#"
Merge branch 'feature/speedup'

Co-authored-by: Alice Smith <alice@example.com>
"#;
        let cost = CostDisclosure::new(vec!["Affected logic".to_string()], vec![]);

        let report = validate_contributor_authorship(
            msg,
            CommitType::MergeCommit,
            Some("Bob Builder"),
            Some(&cost),
        );

        assert!(!report.is_compliant);
        assert!(!report.has_valid_author);
        assert!(report
            .violations
            .iter()
            .any(|v| v.contains("do not attribute original contributor 'Bob Builder'")));
    }

    #[test]
    fn test_proxy_pr_commit_with_valid_trailer_and_handle_normalization() {
        let msg = r#"
refactor: proxy submission for external fix

Co-authored-by: Carol Danvers <carol@marvel.com>
"#;
        let cost = CostDisclosure::new(
            vec!["Telemetry payload schema changed".to_string()],
            vec!["FreeBSD runtime untested".to_string()],
        );

        let report = validate_contributor_authorship(
            msg,
            CommitType::ProxyPrCommit,
            Some("@carol"), // Handle format stripped and matched
            Some(&cost),
        );

        assert!(report.is_compliant);
        assert!(report.has_valid_author);
        assert_eq!(report.trailer_count(), 1);
    }

    #[test]
    fn test_proxy_pr_commit_missing_coauthor_trailer_rejected() {
        let msg = "refactor: proxy submission without attribution";
        let cost = CostDisclosure::new(vec!["Behavior changed".to_string()], vec![]);

        let report = validate_contributor_authorship(
            msg,
            CommitType::ProxyPrCommit,
            Some("Carol Danvers"),
            Some(&cost),
        );

        assert!(!report.is_compliant);
        assert!(!report.has_valid_author);
        assert!(report
            .violations
            .iter()
            .any(|v| v.contains("Proxy PR commit missing mandatory `Co-authored-by:` trailer")));
    }

    #[test]
    fn test_cost_disclosure_extracted_from_commit_message_text() {
        let msg = r#"
feat: improve stream deserializer

## Cost Disclosure
### Affected Existing Behaviors
- Fixed delay reduced to adaptive queue interval
- Max buffer size capped at 16MB

### Unverified Paths
- Windows ARM64 continuous streaming under 10k connections

Co-authored-by: Diana Prince <diana@themyscira.gov>
"#;

        // Pass None for cost_disclosure so it extracts from commit msg body
        let report = validate_contributor_authorship(
            msg,
            CommitType::MergeCommit,
            Some("Diana Prince"),
            None,
        );

        assert!(report.is_compliant);
        assert!(report.has_valid_author);
        assert!(report.cost_disclosure.has_disclosed_costs);
        assert_eq!(
            report.cost_disclosure.affected_existing_behaviors.len(),
            2
        );
        assert_eq!(report.cost_disclosure.unverified_paths.len(), 1);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_cost_disclosure_missing_both_in_struct_and_text_rejected() {
        let msg = r#"
feat: minor tweak

Co-authored-by: Diana Prince <diana@themyscira.gov>
"#;

        let report = validate_contributor_authorship(
            msg,
            CommitType::MergeCommit,
            Some("Diana Prince"),
            None,
        );

        assert!(!report.is_compliant);
        assert!(report
            .violations
            .iter()
            .any(|v| v.contains("Cost disclosure missing")));
    }

    #[test]
    fn test_cost_disclosure_trivial_placeholders_rejected() {
        let text = r#"
### Affected Existing Behaviors
- None
- N/A

### Unverified Paths
- nil
"#;
        let cd = parse_cost_disclosure_from_text(text);
        assert!(!cd.has_disclosed_costs);
        assert!(cd.is_empty());
    }
}
