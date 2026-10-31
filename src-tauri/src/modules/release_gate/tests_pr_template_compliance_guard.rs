//! Unit Tests for Pull Request Template Compliance Guard (`Imp-203`).

#[cfg(test)]
mod tests {
    use super::super::pr_template_compliance_guard::*;

    #[test]
    fn test_fully_compliant_pr_description() {
        let markdown = r#"
## Problem Classification
Memory leak detected in stream flusher during long-running sessions.

## Behavior Alterations
Buffered flusher output is now strictly bounded at 64 KiB.

## Unverified Paths
Load tested locally but need full cluster verification under high load.

## Rollback Strategy
Revert commit with git revert and roll back to previous stable release tag.
"#;
        let report = audit_pr_description(markdown);
        assert!(report.is_compliant);
        assert!(!report.has_findings());
        assert!(report.findings.is_empty());
        assert!(report.problem_classification.is_some());
        assert!(report.behavior_alterations.is_some());
        assert!(report.unverified_paths.is_some());
        assert!(report.rollback_strategy.is_some());
    }

    #[test]
    fn test_missing_single_section() {
        let markdown = r#"
## Problem Classification
Refactoring token bucket limiter to avoid deadlocks.

## Behavior Alterations
Acquires lock with timeout instead of blocking indefinitely.

## Unverified Paths
High concurrency stress testing with thousands of threads.
"#;
        let report = audit_pr_description(markdown);
        assert!(!report.is_compliant);
        assert!(report.has_findings());
        assert_eq!(report.findings.len(), 1);

        match &report.findings[0] {
            TemplateAuditFinding::MissingSection { section } => {
                assert_eq!(*section, PrTemplateSection::RollbackStrategy);
            }
            other => panic!("Expected MissingSection, got: {:?}", other),
        }
        assert_eq!(
            report.missing_sections(),
            vec![PrTemplateSection::RollbackStrategy]
        );
    }

    #[test]
    fn test_incomplete_section_with_trivial_word_count() {
        let markdown = r#"
## Problem Classification
Bug fix.

## Behavior Alterations
Fixed bad crash when parsing headers in proxy.

## Unverified Paths
All paths covered by comprehensive automated tests.

## Rollback Strategy
Revert the commit and rebuild artifacts.
"#;
        let report = audit_pr_description(markdown);
        assert!(!report.is_compliant);
        assert_eq!(report.findings.len(), 1);

        match &report.findings[0] {
            TemplateAuditFinding::IncompleteSection {
                section,
                minimum_words,
                found_words,
            } => {
                assert_eq!(*section, PrTemplateSection::ProblemClassification);
                assert_eq!(*minimum_words, 3);
                assert_eq!(*found_words, 2);
            }
            other => panic!("Expected IncompleteSection, got: {:?}", other),
        }
        assert_eq!(
            report.incomplete_sections(),
            vec![PrTemplateSection::ProblemClassification]
        );
    }

    #[test]
    fn test_empty_section_with_only_html_comment() {
        let markdown = r#"
## Problem Classification
<!-- describe the problem here -->

## Behavior Alterations
Behavior is updated to correctly parse input parameters.

## Unverified Paths
Local manual test with Edge browser and Safari.

## Rollback Strategy
Revert the commit and notify team immediately.
"#;
        let report = audit_pr_description(markdown);
        assert!(!report.is_compliant);
        match &report.findings[0] {
            TemplateAuditFinding::IncompleteSection {
                section,
                minimum_words,
                found_words,
            } => {
                assert_eq!(*section, PrTemplateSection::ProblemClassification);
                assert_eq!(*minimum_words, 3);
                assert_eq!(*found_words, 0);
            }
            other => panic!("Expected IncompleteSection with 0 words, got: {:?}", other),
        }
    }

    #[test]
    fn test_case_insensitive_and_h3_headers() {
        let markdown = r#"
### problem classification
A critical issue occurred during handshake negotiation between client and server.

### BEHAVIOR ALTERATIONS
Fallback ladder now attempts secondary cipher suite before disconnecting.

### unverified paths
Cross-platform compatibility across Windows ARM64 and macOS architectures.

### rollback strategy
Revert pull request on main branch and tag patch release.
"#;
        let report = audit_pr_description(markdown);
        assert!(report.is_compliant);
        assert!(report.findings.is_empty());
    }

    #[test]
    fn test_all_sections_missing_for_empty_markdown() {
        let report = audit_pr_description("");
        assert!(!report.is_compliant);
        assert_eq!(report.findings.len(), 4);
        assert_eq!(report.missing_sections().len(), 4);
    }

    #[test]
    fn test_display_formatting_and_error_trait() {
        let missing = TemplateAuditFinding::MissingSection {
            section: PrTemplateSection::RollbackStrategy,
        };
        assert_eq!(
            missing.to_string(),
            "Missing mandatory PR template section: Rollback Strategy"
        );

        let incomplete = TemplateAuditFinding::IncompleteSection {
            section: PrTemplateSection::BehaviorAlterations,
            minimum_words: 3,
            found_words: 1,
        };
        assert_eq!(
            incomplete.to_string(),
            "Incomplete PR template section: Behavior Alterations (found 1 words, minimum 3 required)"
        );

        // Verify std::error::Error trait implementation
        let err: &dyn std::error::Error = &missing;
        assert!(err.source().is_none());
    }

    #[test]
    fn test_unrelated_headers_do_not_bleed_into_sections() {
        let markdown = r#"
## Problem Classification
Stream connection drops unpredictably under network jitter.

## Reviewers
@aukgit and core team members.

## Behavior Alterations
Reconnection interval backoff increased from 1s to 5s.

## Unverified Paths
Satellite connection high latency simulation environment.

## Rollback Strategy
Revert commit cleanly using git revert and publish release.
"#;
        let report = audit_pr_description(markdown);
        assert!(report.is_compliant);
        assert!(report.findings.is_empty());
        assert_eq!(
            report.problem_classification.unwrap(),
            "Stream connection drops unpredictably under network jitter."
        );
    }
}
