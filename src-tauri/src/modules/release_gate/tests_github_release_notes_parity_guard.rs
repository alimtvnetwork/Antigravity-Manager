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

    let report = validate_release_notes("v4.271.0", content, Some("## v4.271.0"));
    assert!(report.is_compliant);
    assert!(report.authorized_attribution);
    assert!(report.violations.is_empty());
    assert_eq!(report.tag, "v4.271.0");
}

#[test]
fn test_missing_aukgit_attribution() {
    let content = r#"## v4.271.0

- Some changes without author attribution.
"#;

    let report = validate_release_notes("v4.271.0", content, Some("## v4.271.0"));
    assert!(!report.is_compliant);
    assert!(!report.authorized_attribution);
    assert_eq!(report.violations.len(), 1);
    assert!(matches!(
        &report.violations[0],
        ReleaseNotesViolation::MissingAukgitAttribution
    ));
}

#[test]
fn test_unauthorized_contributor_handle() {
    let content = r#"## v4.271.0

- Contributed by @collaborator and @external-dev.
(Thanks to @aukgit)
"#;

    let report = validate_release_notes("v4.271.0", content, Some("## v4.271.0"));
    assert!(!report.is_compliant);
    assert!(report.authorized_attribution);
    assert_eq!(report.violations.len(), 2);

    let has_collab = report.violations.iter().any(|v| match v {
        ReleaseNotesViolation::UnauthorizedContributorHandle(h) => h == "@collaborator",
        _ => false,
    });
    let has_external = report.violations.iter().any(|v| match v {
        ReleaseNotesViolation::UnauthorizedContributorHandle(h) => h == "@external-dev",
        _ => false,
    });

    assert!(has_collab);
    assert!(has_external);
}

#[test]
fn test_detects_forbidden_windows_drive_absolute_path() {
    let content = r#"## v4.271.0

- Modified file at D:\work\Antigravity-Manager\src\index.ts
(Thanks to @aukgit)
"#;

    let report = validate_release_notes("v4.271.0", content, Some("## v4.271.0"));
    assert!(!report.is_compliant);
    assert_eq!(report.violations.len(), 1);
    match &report.violations[0] {
        ReleaseNotesViolation::AbsolutePathFound { line, path } => {
            assert_eq!(*line, 3);
            assert!(path.contains("D:\\work"));
        }
        other => panic!("Unexpected violation: {other:?}"),
    }
}

#[test]
fn test_detects_unix_absolute_path() {
    let content = r#"## v4.271.0

- Binary built at /home/runner/work/agm/agm
(Thanks to @aukgit)
"#;

    let report = validate_release_notes("v4.271.0", content, Some("## v4.271.0"));
    assert!(!report.is_compliant);
    assert_eq!(report.violations.len(), 1);
    match &report.violations[0] {
        ReleaseNotesViolation::AbsolutePathFound { line, path } => {
            assert_eq!(*line, 3);
            assert!(path.contains("/home/runner"));
        }
        other => panic!("Unexpected violation: {other:?}"),
    }
}

#[test]
fn test_detects_file_uri() {
    let content = r#"## v4.271.0

- See [log](file:///C:/Users/Administrator/app.log)
(Thanks to @aukgit)
"#;

    let report = validate_release_notes("v4.271.0", content, Some("## v4.271.0"));
    assert!(!report.is_compliant);
    assert_eq!(report.violations.len(), 1);
    match &report.violations[0] {
        ReleaseNotesViolation::AbsolutePathFound { line, path } => {
            assert_eq!(*line, 3);
            assert!(path.starts_with("file:///"));
        }
        other => panic!("Unexpected violation: {other:?}"),
    }
}

#[test]
fn test_tag_heading_mismatch() {
    let content = r#"## v4.270.0-beta.1

- Release details.
(Thanks to @aukgit)
"#;

    let report = validate_release_notes("v4.271.0", content, None);
    assert!(!report.is_compliant);
    assert!(report.violations.iter().any(|v| matches!(
        v,
        ReleaseNotesViolation::MissingChangelogHeadingMatch { tag } if tag == "v4.271.0"
    )));
}

#[test]
fn test_changelog_heading_mismatch() {
    let content = r#"## v4.271.0

- Release details.
(Thanks to @aukgit)
"#;

    let report = validate_release_notes("v4.271.0", content, Some("## v4.270.0"));
    assert!(!report.is_compliant);
    assert!(report.violations.iter().any(|v| matches!(
        v,
        ReleaseNotesViolation::MissingChangelogHeadingMatch { tag } if tag == "v4.271.0"
    )));
}

#[test]
fn test_backward_compatible_audit_wrapper() {
    let content = r#"## v4.271.0

- Release details in `src/index.ts`.
(Thanks to @aukgit)
"#;

    let report = audit_release_notes_parity(content, "v4.271.0");
    assert!(report.is_compliant);
    assert_eq!(report.tag, "v4.271.0");
}

#[test]
fn test_email_address_is_not_flagged_as_contributor_handle() {
    let content = r#"## v4.271.0

- For inquiries, contact alim.karim@riseup-asia.com.
(Thanks to @aukgit)
"#;

    let report = validate_release_notes("v4.271.0", content, Some("## v4.271.0"));
    assert!(report.is_compliant);
    assert!(report.authorized_attribution);
    assert!(report.violations.is_empty());
}

#[test]
fn test_display_formatting() {
    let v_handle = ReleaseNotesViolation::UnauthorizedContributorHandle("@badactor".to_string());
    let v_path = ReleaseNotesViolation::AbsolutePathFound {
        path: "C:\\secret".to_string(),
        line: 12,
    };
    let v_attr = ReleaseNotesViolation::MissingAukgitAttribution;
    let v_heading = ReleaseNotesViolation::MissingChangelogHeadingMatch {
        tag: "v1.0.0".to_string(),
    };

    assert!(format!("{v_handle}").contains("@badactor"));
    assert!(format!("{v_path}").contains("C:\\secret"));
    assert!(format!("{v_path}").contains("line 12"));
    assert!(format!("{v_attr}").contains("@aukgit"));
    assert!(format!("{v_heading}").contains("v1.0.0"));

    let report = ReleaseNotesParityReport {
        tag: "v1.0.0".to_string(),
        is_compliant: false,
        violations: vec![v_handle],
        authorized_attribution: true,
    };
    assert!(format!("{report}").contains("tag=v1.0.0"));
}
