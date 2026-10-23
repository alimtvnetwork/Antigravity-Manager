//! Unit tests for Verified Fallback Path & Risky Path Replacement Linter.

use super::verified_fallback_path_linter::{
    audit_fallback_paths, extract_block_window, has_fallback_invocation, has_silent_noop_return,
    is_comment_line, is_pattern_match_construct, is_zero_matches_check, FallbackAuditReport,
    FallbackLinterViolation,
};

#[test]
fn test_clean_replacement_with_explicit_fallback() {
    let source = r#"
fn query_fast_cache() -> Result<Vec<String>, AppError> {
    let matches = search_cache();
    if matches.is_empty() {
        return run_verified_fallback_restart();
    }
    Ok(matches)
}
"#;

    let report = audit_fallback_paths(source, "src-tauri/src/proxy/cache.rs");
    assert!(report.is_safe);
    assert_eq!(report.replacement_sites_scanned, 1);
    assert_eq!(report.verified_fallbacks_found, 1);
    assert!(report.violations.is_empty());
    assert!(!report.has_violations());
    assert!(report.summary().contains("Safe"));
}

#[test]
fn test_silent_noop_on_zero_matches() {
    let source = r#"
fn handle_request() -> Result<(), AppError> {
    let matches = scan_headers();
    if matches.is_empty() {
        return Ok(());
    }
    process(matches);
    Ok(())
}
"#;

    let report = audit_fallback_paths(source, "src-tauri/src/proxy/handler.rs");
    assert!(!report.is_safe);
    assert!(report.has_violations());
    assert_eq!(report.replacement_sites_scanned, 1);
    assert_eq!(report.verified_fallbacks_found, 0);
    assert_eq!(report.violations.len(), 1);

    match &report.violations[0] {
        FallbackLinterViolation::SilentNoopOnZeroMatches { file, line } => {
            assert_eq!(file, "src-tauri/src/proxy/handler.rs");
            assert_eq!(*line, 4);
        }
        _ => panic!("Expected SilentNoopOnZeroMatches violation"),
    }
}

#[test]
fn test_silent_noop_with_count_equal_zero() {
    let source = r#"
fn check_items() -> Result<(), AppError> {
    let count = count_items();
    if count == 0 {
        return Ok(());
    }
    Ok(())
}
"#;

    let report = audit_fallback_paths(source, "src-tauri/src/proxy/checker.rs");
    assert!(!report.is_safe);
    assert_eq!(report.violations.len(), 1);
    assert!(matches!(
        report.violations[0],
        FallbackLinterViolation::SilentNoopOnZeroMatches { line: 4, .. }
    ));
}

#[test]
fn test_missing_verified_fallback_branch_on_match() {
    let source = r#"
fn route_speculative() -> Result<Data, AppError> {
    match result {
        Ok(data) => Ok(data),
        Err(err) => Err(AppError::from(err)),
    }
}
"#;

    let report = audit_fallback_paths(source, "src-tauri/src/proxy/router.rs");
    assert!(!report.is_safe);
    assert_eq!(report.replacement_sites_scanned, 1);
    assert_eq!(report.violations.len(), 1);

    match &report.violations[0] {
        FallbackLinterViolation::MissingVerifiedFallbackBranch {
            file,
            line,
            construct,
        } => {
            assert_eq!(file, "src-tauri/src/proxy/router.rs");
            assert_eq!(*line, 3);
            assert!(construct.contains("match result"));
        }
        _ => panic!("Expected MissingVerifiedFallbackBranch violation"),
    }
}

#[test]
fn test_match_with_verified_fallback_arm() {
    let source = r#"
fn route_with_fallback() -> Result<Data, AppError> {
    match result {
        Ok(data) => Ok(data),
        Err(_) => fallback_to_verified_restart(),
    }
}
"#;

    let report = audit_fallback_paths(source, "src-tauri/src/proxy/router.rs");
    assert!(report.is_safe);
    assert_eq!(report.replacement_sites_scanned, 1);
    assert_eq!(report.verified_fallbacks_found, 1);
    assert!(report.violations.is_empty());
}

#[test]
fn test_multi_line_file_auditing() {
    let source = r#"
// Clean function with explicit restart fallback
fn proc_a() -> Result<(), AppError> {
    let matches = search_a();
    if matches.is_empty() {
        return reload_and_restart();
    }
    Ok(())
}

// Violating function with silent no-op
fn proc_b() -> Result<(), AppError> {
    let items = search_b();
    if items.is_empty() {
        return Ok(());
    }
    Ok(())
}

// Clean function with explicit fallback invocation
fn proc_c() -> Result<(), AppError> {
    match replacement_result {
        Ok(val) => Ok(val),
        Err(_) => run_fallback_path(),
    }
}
"#;

    let report = audit_fallback_paths(source, "src-tauri/src/proxy/complex.rs");
    assert!(!report.is_safe);
    assert_eq!(report.replacement_sites_scanned, 3);
    assert_eq!(report.verified_fallbacks_found, 2);
    assert_eq!(report.violations.len(), 1);
    assert!(matches!(
        report.violations[0],
        FallbackLinterViolation::SilentNoopOnZeroMatches { line: 14, .. }
    ));
}

#[test]
fn test_comments_are_ignored() {
    let source = r#"
// if matches.is_empty() { return Ok(()); }
/* match result { Ok(_) => return Ok(()), } */
fn normal_code() -> Result<(), AppError> {
    Ok(())
}
"#;

    let report = audit_fallback_paths(source, "src-tauri/src/proxy/commented.rs");
    assert!(report.is_safe);
    assert_eq!(report.replacement_sites_scanned, 0);
    assert!(report.violations.is_empty());
}

#[test]
fn test_violation_display_and_error_trait() {
    let violation_branch = FallbackLinterViolation::MissingVerifiedFallbackBranch {
        file: "src-tauri/src/proxy/router.rs".to_string(),
        line: 12,
        construct: "match result".to_string(),
    };
    let formatted_branch = format!("{}", violation_branch);
    assert!(formatted_branch.contains("src-tauri/src/proxy/router.rs:12:"));
    assert!(formatted_branch.contains("lacks verified fallback branch"));

    let violation_noop = FallbackLinterViolation::SilentNoopOnZeroMatches {
        file: "src-tauri/src/proxy/cache.rs".to_string(),
        line: 25,
    };
    let formatted_noop = format!("{}", violation_noop);
    assert!(formatted_noop.contains("src-tauri/src/proxy/cache.rs:25:"));
    assert!(formatted_noop.contains("Forbidden silent no-op return"));

    let err_ref: &dyn std::error::Error = &violation_noop;
    assert!(!err_ref.to_string().is_empty());
}

#[test]
fn test_fallback_audit_report_display() {
    let mut report = FallbackAuditReport::new();
    report.replacement_sites_scanned = 2;
    report.verified_fallbacks_found = 1;
    report.violations.push(FallbackLinterViolation::SilentNoopOnZeroMatches {
        file: "src-tauri/src/proxy/test.rs".to_string(),
        line: 10,
    });
    report.is_safe = false;

    let formatted = format!("{}", report);
    assert!(formatted.contains("Unsafe: 1 violation(s)"));
    assert!(formatted.contains("src-tauri/src/proxy/test.rs:10:"));
}

#[test]
fn test_helper_predicates() {
    assert!(is_comment_line("// comment"));
    assert!(is_comment_line("/* comment */"));
    assert!(is_comment_line("* continuation"));
    assert!(!is_comment_line("let a = 1;"));

    assert!(is_zero_matches_check("if matches.is_empty() {"));
    assert!(is_zero_matches_check("if count == 0 {"));
    assert!(!is_zero_matches_check("let is_empty = false;"));

    assert!(is_pattern_match_construct("match result {"));
    assert!(is_pattern_match_construct("match replacement {"));
    assert!(!is_pattern_match_construct("let result = 1;"));

    assert!(has_fallback_invocation("run_fallback()"));
    assert!(has_fallback_invocation("full_restart()"));
    assert!(has_fallback_invocation("reload_config()"));
    assert!(!has_fallback_invocation("execute_normal()"));

    assert!(has_silent_noop_return("return Ok(());"));
    assert!(has_silent_noop_return("return Ok(vec![]);"));
    assert!(!has_silent_noop_return("return Err(err);"));

    let lines = vec!["line 0", "line 1", "line 2", "line 3"];
    let window = extract_block_window(&lines, 1);
    assert!(window.contains("line 1"));
    assert!(window.contains("line 3"));
}
