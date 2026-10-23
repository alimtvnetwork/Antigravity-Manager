//! Unit tests for GitHub Release Notes Parity Guard (`Imp-689`).

use super::github_release_notes_parity_guard::*;

#[test]
fn test_compliant_release_notes_passes() {
    let content = r#"## v4.271.0

### Highlights
- Implemented pipeline response header sanitizer in `src-tauri/src/proxy/handlers/common/`.
- Enforced window control capsule styling in `src-tauri/src/modules/instance/`.
- Documented in `02-spec/21-app/212-implementing-next-five-tasks/spec.md`.

(Thanks to @aukgit)
"#;

    let report = audit_release_notes_parity(content, "v4.271.0");
    assert!(report.is_compliant);
    assert_eq!(report.detected_heading, Some("v4.271.0".to_string()));
    assert!(report.violations.is_empty());
}

#[test]
fn test_detects_forbidden_absolute_path() {
    let content = r#"## v4.271.0

- Modified file at D:\work\Antigravity-Manager\src\index.ts
(Thanks to @aukgit)
"#;

    let report = audit_release_notes_parity(content, "v4.271.0");
    assert!(!report.is_compliant);
    assert_eq!(report.violations.len(), 1);
    match &report.violations[0] {
        ReleaseNotesViolation::AbsolutePathDetected { line_number, detected_path } => {
            assert_eq!(*line_number, 3);
            assert!(detected_path.contains("D:\\work"));
        }
        other => panic!("Unexpected violation: {other:?}"),
    }
}

#[test]
fn test_detects_foreign_contributor_handle() {
    let content = r#"## v4.271.0

- Thanks to @collaborator and @aukgit for fixes.
"#;

    let report = audit_release_notes_parity(content, "v4.271.0");
    assert!(!report.is_compliant);
    assert_eq!(report.violations.len(), 1);
    match &report.violations[0] {
        ReleaseNotesViolation::ForeignContributorHandle { line_number, detected_handle } => {
            assert_eq!(*line_number, 3);
            assert_eq!(detected_handle, "@collaborator");
        }
        other => panic!("Unexpected violation: {other:?}"),
    }
}

#[test]
fn test_detects_mismatched_tag_heading() {
    let content = r#"## v4.270.0-beta.1

- Release details.
(Thanks to @aukgit)
"#;

    let report = audit_release_notes_parity(content, "v4.271.0");
    assert!(!report.is_compliant);
    assert!(report.violations.iter().any(|v| matches!(v, ReleaseNotesViolation::TagHeadingMismatch { .. })));
}
