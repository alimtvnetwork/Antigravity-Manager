//! Unit tests for Co-Author Trailer Alignment Guard (`AGENTS.md`).

use super::coauthor_trailer_alignment_guard::{
    audit_commit_message_trailers, format_coauthor_trailer, parse_coauthor_line, CoauthorTrailer,
    TrailerValidationFinding,
};

#[test]
fn test_valid_trailer_parsed_cleanly() {
    let line = "Co-authored-by: Jane Doe <jane@example.com>";
    let result = parse_coauthor_line(line, 1);

    assert!(result.is_ok());
    let opt = result.unwrap();
    assert!(opt.is_some());

    let trailer = opt.unwrap();
    assert_eq!(trailer.name, "Jane Doe");
    assert_eq!(trailer.email, "jane@example.com");
    assert_eq!(trailer.raw_line, line);
}

#[test]
fn test_multiple_valid_trailers_in_multi_author_commit() {
    let commit_message = r#"Merge branch 'feature/worker-swarm' into beta

Implement multi-agent tree swarm architecture.

Co-authored-by: Alice Smith <alice@example.com>
Co-authored-by: Bob Builder <bob@example.org>
Co-authored-by: Charlie Day <charlie@corp.local>
"#;

    let report = audit_commit_message_trailers(commit_message);

    assert!(report.is_valid);
    assert!(report.is_clean());
    assert!(report.has_trailers());
    assert_eq!(report.total_trailers_found, 3);
    assert_eq!(report.trailers.len(), 3);
    assert!(report.findings.is_empty());

    assert_eq!(report.trailers[0].name, "Alice Smith");
    assert_eq!(report.trailers[0].email, "alice@example.com");
    assert_eq!(report.trailers[1].name, "Bob Builder");
    assert_eq!(report.trailers[1].email, "bob@example.org");
    assert_eq!(report.trailers[2].name, "Charlie Day");
    assert_eq!(report.trailers[2].email, "charlie@corp.local");
}

#[test]
fn test_malformed_trailer_missing_email_bracket() {
    let line = "Co-authored-by: Jane Doe jane@example.com";
    let result = parse_coauthor_line(line, 4);

    assert!(result.is_err());
    match result.unwrap_err() {
        TrailerValidationFinding::MissingEmailBracket { line, raw } => {
            assert_eq!(line, 4);
            assert_eq!(raw, "Co-authored-by: Jane Doe jane@example.com");
        }
        other => panic!("Expected MissingEmailBracket, got: {:?}", other),
    }
}

#[test]
fn test_malformed_trailer_inverted_email_bracket() {
    let line = "Co-authored-by: Jane Doe >jane@example.com<";
    let result = parse_coauthor_line(line, 2);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TrailerValidationFinding::MissingEmailBracket { .. }
    ));
}

#[test]
fn test_invalid_email_missing_at() {
    let line = "Co-authored-by: Jane Doe <jane.example.com>";
    let result = parse_coauthor_line(line, 5);

    assert!(result.is_err());
    match result.unwrap_err() {
        TrailerValidationFinding::InvalidEmailFormat { line, email } => {
            assert_eq!(line, 5);
            assert_eq!(email, "jane.example.com");
        }
        other => panic!("Expected InvalidEmailFormat, got: {:?}", other),
    }
}

#[test]
fn test_invalid_email_missing_domain_dot() {
    let line = "Co-authored-by: Jane Doe <jane@localhost>";
    let result = parse_coauthor_line(line, 6);

    assert!(result.is_err());
    match result.unwrap_err() {
        TrailerValidationFinding::InvalidEmailFormat { line, email } => {
            assert_eq!(line, 6);
            assert_eq!(email, "jane@localhost");
        }
        other => panic!("Expected InvalidEmailFormat, got: {:?}", other),
    }
}

#[test]
fn test_disallowed_handle_in_contributor_name() {
    let line = "Co-authored-by: @janedoe <jane@example.com>";
    let result = parse_coauthor_line(line, 10);

    assert!(result.is_err());
    match result.unwrap_err() {
        TrailerValidationFinding::DisallowedHandleInName { line, handle } => {
            assert_eq!(line, 10);
            assert_eq!(handle, "@janedoe");
        }
        other => panic!("Expected DisallowedHandleInName, got: {:?}", other),
    }

    let line_embedded = "Co-authored-by: Jane @janedoe Doe <jane@example.com>";
    let result_embedded = parse_coauthor_line(line_embedded, 11);
    assert!(matches!(
        result_embedded.unwrap_err(),
        TrailerValidationFinding::DisallowedHandleInName { .. }
    ));
}

#[test]
fn test_malformed_syntax_missing_colon_or_empty_name() {
    let line_missing_colon = "Co-authored-by Jane Doe <jane@example.com>";
    let result_colon = parse_coauthor_line(line_missing_colon, 1);
    assert!(matches!(
        result_colon.unwrap_err(),
        TrailerValidationFinding::MalformedSyntax { .. }
    ));

    let line_empty_name = "Co-authored-by: <jane@example.com>";
    let result_empty_name = parse_coauthor_line(line_empty_name, 2);
    assert!(matches!(
        result_empty_name.unwrap_err(),
        TrailerValidationFinding::MalformedSyntax { .. }
    ));

    let line_trailing_text = "Co-authored-by: Jane Doe <jane@example.com> unexpected trailing text";
    let result_trailing = parse_coauthor_line(line_trailing_text, 3);
    assert!(matches!(
        result_trailing.unwrap_err(),
        TrailerValidationFinding::MalformedSyntax { .. }
    ));
}

#[test]
fn test_formatting_helper_produces_correct_git_trailer() {
    let formatted = format_coauthor_trailer("  Alice Cooper  ", "  alice@example.com  ");
    assert_eq!(
        formatted,
        "Co-authored-by: Alice Cooper <alice@example.com>"
    );
}

#[test]
fn test_display_and_error_formatting_of_findings() {
    let f1 = TrailerValidationFinding::MalformedSyntax {
        line: 1,
        raw: "Co-authored-by invalid".to_string(),
    };
    assert!(f1
        .to_string()
        .contains("Line 1: Malformed Co-authored-by trailer syntax"));

    let f2 = TrailerValidationFinding::MissingEmailBracket {
        line: 2,
        raw: "Co-authored-by: John Doe".to_string(),
    };
    assert!(f2
        .to_string()
        .contains("Line 2: Missing '<email>' brackets"));

    let f3 = TrailerValidationFinding::InvalidEmailFormat {
        line: 3,
        email: "bad-email".to_string(),
    };
    assert!(f3
        .to_string()
        .contains("Line 3: Invalid contributor email format"));

    let f4 = TrailerValidationFinding::DisallowedHandleInName {
        line: 4,
        handle: "@badhandle".to_string(),
    };
    assert!(f4
        .to_string()
        .contains("Line 4: Disallowed handle '@badhandle'"));

    // Verify std::error::Error trait
    let err_box: Box<dyn std::error::Error> = Box::new(f1);
    assert!(!err_box.to_string().is_empty());
}

#[test]
fn test_audit_mixed_commit_message_accumulates_findings_and_trailers() {
    let msg = r#"fix: stabilize token streaming parser

Regular commit body here.

Co-authored-by: Valid Contributor <valid@example.com>
Co-authored-by: Invalid Contributor missing-email
"#;

    let report = audit_commit_message_trailers(msg);
    assert!(!report.is_valid);
    assert_eq!(report.total_trailers_found, 1);
    assert_eq!(report.trailers.len(), 1);
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.trailers[0].name, "Valid Contributor");
}
