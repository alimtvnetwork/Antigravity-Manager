#[cfg(test)]
mod tests {
    use super::super::pr_template_compliance_linter::{
        lint_pr_description, PrTemplateComplianceReport,
    };

    #[test]
    fn test_pr_template_fully_compliant() {
        let desc = r#"
## Problem Classification
Refactoring proxy stream flusher to eliminate potential memory leakage.

## Behavior Alterations
Flusher now bounds maximum buffered bytes to 64 KiB.

## Unverified Paths
Load testing with 10,000 concurrent streaming connections in production environment.

## Rollback Strategy
Revert commit with git revert and redeploy previous stable version.
"#;
        let report = lint_pr_description(desc);
        assert!(report.is_compliant);
        assert!(report.missing_sections.is_empty());
    }

    #[test]
    fn test_pr_template_missing_rollback_strategy() {
        let desc = r#"
## Problem Classification
Bug fix.

## Behavior Alterations
Fixed crash on empty header.

## Unverified Paths
None.
"#;
        let report = lint_pr_description(desc);
        assert!(!report.is_compliant);
        assert_eq!(report.missing_sections, vec!["Rollback Strategy"]);
    }

    #[test]
    fn test_pr_template_empty_description() {
        let report = lint_pr_description("");
        assert!(!report.is_compliant);
        assert_eq!(report.missing_sections.len(), 4);
    }
}
