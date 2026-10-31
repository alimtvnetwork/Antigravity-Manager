//! Unit tests for Capsule Segmented Control Validator (`Imp-262`).

use super::capsule_segmented_control_validator::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compliant_capsule_with_rounded_full_and_transparent_select() {
        let markup = r#"
            <div className="flex items-center rounded-full border border-white/10 dark-glass">
                <button title="Minimize" onClick={minimize}>-</button>
                <button title="Maximize" onClick={maximize}>+</button>
                <button title="Close" onClick={close}>x</button>
                <select className="bg-transparent border-0 focus:ring-0 focus:outline-none">
                    <option value="default">Default Profile</option>
                </select>
            </div>
        "#;

        let report = validate_component_markup(markup);

        assert!(report.is_compliant);
        assert!(!report.has_violations());
        assert_eq!(report.capsules_checked, 1);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn test_disjoint_window_controls_without_capsule() {
        let markup = r#"
            <div className="flex items-center gap-1">
                <button title="Minimize">-</button>
                <button title="Maximize">+</button>
                <button title="Close">x</button>
            </div>
        "#;

        let report = validate_component_markup(markup);

        assert!(!report.is_compliant);
        assert!(report.has_violations());
        assert_eq!(report.capsules_checked, 0);
        assert_eq!(report.violations.len(), 3);

        for violation in &report.violations {
            assert!(violation.is_disjoint_controls());
            assert!(!violation.is_nested_select_defect());
            assert!(!violation.is_missing_rounded_full());
        }
    }

    #[test]
    fn test_non_transparent_inline_select_defect() {
        let markup = r#"
            <div className="flex items-center rounded-full border border-white/10">
                <select className="bg-white border rounded text-black">
                    <option value="profile">Profile A</option>
                </select>
            </div>
        "#;

        let report = validate_component_markup(markup);

        assert!(!report.is_compliant);
        assert!(report.has_violations());
        assert_eq!(report.capsules_checked, 1);
        assert_eq!(report.violations.len(), 1);

        let defect = &report.violations[0];
        assert!(defect.is_nested_select_defect());
        assert!(!defect.is_disjoint_controls());
        assert_eq!(defect.line(), 3);
    }

    #[test]
    fn test_missing_contiguous_rounded_full_on_container() {
        let markup = r#"
            <div className="window-controls flex items-center border">
                <span>Title</span>
            </div>
        "#;

        let report = validate_component_markup(markup);

        assert!(!report.is_compliant);
        assert!(report.has_violations());
        assert_eq!(report.violations.len(), 1);

        let violation = &report.violations[0];
        assert!(violation.is_missing_rounded_full());
        assert_eq!(violation.line(), 2);
    }

    #[test]
    fn test_violation_display_and_error_formatting() {
        let v1 = CapsuleStylingViolation::DisjointWindowControlsWithoutCapsule {
            line: 12,
            snippet: "<button title=\"Close\">x</button>".to_string(),
        };
        let formatted1 = v1.to_string();
        assert!(formatted1.contains("Line 12"));
        assert!(formatted1.contains("Disjoint window control without capsule container"));
        assert!(formatted1.contains("<button title=\"Close\">x</button>"));
        assert_eq!(v1.line(), 12);

        let v2 = CapsuleStylingViolation::NestedSelectPillInPillDefect {
            line: 45,
            tag: "<select className=\"bg-white\">".to_string(),
        };
        let formatted2 = v2.to_string();
        assert!(formatted2.contains("Line 45"));
        assert!(formatted2.contains("Nested select pill-in-pill defect"));
        assert_eq!(v2.line(), 45);

        let v3 = CapsuleStylingViolation::MissingContiguousRoundedFull {
            line: 7,
            container: "<div className=\"toolbar-actions\">".to_string(),
        };
        let formatted3 = v3.to_string();
        assert!(formatted3.contains("Line 7"));
        assert!(formatted3.contains("Missing contiguous rounded-full on container"));
        assert_eq!(v3.line(), 7);

        // Verify std::error::Error trait implementation
        let err: &dyn std::error::Error = &v1;
        assert_eq!(err.to_string(), formatted1);
    }

    #[test]
    fn test_plain_markup_without_controls_is_compliant() {
        let markup = r#"
            <div className="container mx-auto p-4">
                <h1 className="text-xl font-bold">Hello World</h1>
                <p>Normal text without window controls.</p>
            </div>
        "#;

        let report = validate_component_markup(markup);

        assert!(report.is_compliant);
        assert!(!report.has_violations());
        assert_eq!(report.capsules_checked, 0);
        assert!(report.violations.is_empty());
    }
}
