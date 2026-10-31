//! Unit tests for Rejection Breakdown Generator (`Imp-624`).

use super::rejection_breakdown_generator::{
    generate_markdown_rejection_breakdown, is_relative_git_path, parse_and_classify_changeset,
    pattern_matches, validate_rejection_has_actionable_path, FileBreakdownItem, FileDisposition,
    RejectionBreakdownReport,
};

#[test]
fn test_relative_git_path_validation() {
    assert!(is_relative_git_path("src-tauri/src/modules/foo.rs"));
    assert!(is_relative_git_path("src/components/common/Button.tsx"));
    assert!(is_relative_git_path("02-spec/21-app/spec.md"));
    assert!(is_relative_git_path(".ai-memory/plans/readme.md"));

    // Forbidden absolute or non-relative paths
    assert!(!is_relative_git_path("C:\\Users\\admin\\repo\\file.rs"));
    assert!(!is_relative_git_path("D:/work/repo/file.rs"));
    assert!(!is_relative_git_path("/home/user/project/file.rs"));
    assert!(!is_relative_git_path("~/.bashrc"));
    assert!(!is_relative_git_path("../outside/file.rs"));
    assert!(!is_relative_git_path(""));
}

#[test]
fn test_pattern_matching() {
    assert!(pattern_matches("package-lock.json", "*.json"));
    assert!(pattern_matches("target/debug/build.log", "target/*"));
    assert!(pattern_matches("foo/bar/baz.txt", "bar"));
    assert!(!pattern_matches("src/main.rs", "*.js"));
}

#[test]
fn test_validation_rejects_empty_breakdown() {
    let empty_report = RejectionBreakdownReport::new("Empty", "author");
    let res = validate_rejection_has_actionable_path(&empty_report);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("cannot be empty"));
}

#[test]
fn test_validation_rejects_missing_actionable_path_on_drop() {
    let item = FileBreakdownItem::new(
        "src-tauri/src/main.rs",
        FileDisposition::Drop,
        "Violates pipeline first architecture",
        None,
    );
    let mut report = RejectionBreakdownReport::new("Review", "author").with_file_breakdown(item);

    let res = validate_rejection_has_actionable_path(&report);
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .contains("lacks an actionable path forward"));

    // Add actionable step and verify success
    report
        .actionable_path_forward
        .push("Move logic to pipeline common handler".to_string());
    assert!(validate_rejection_has_actionable_path(&report).is_ok());
}

#[test]
fn test_validation_rejects_absolute_path() {
    let item = FileBreakdownItem::new(
        "C:\\repo\\src\\main.rs",
        FileDisposition::Accept,
        "Clean logic",
        None,
    );
    let report = RejectionBreakdownReport::new("Review", "author").with_file_breakdown(item);
    let res = validate_rejection_has_actionable_path(&report);
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .contains("not a repository-relative Git path"));
}

#[test]
fn test_parse_and_classify_changeset() {
    let files = [
        "src-tauri/src/proxy.rs",
        "target/output.tmp",
        "package-lock.json",
    ];
    let drop_patterns = ["target/*", "*.tmp"];

    let report = parse_and_classify_changeset(&files, &drop_patterns, |path| {
        if path == "src-tauri/src/proxy.rs" {
            (
                FileDisposition::Accept,
                "Verified proxy changes".to_string(),
                None,
            )
        } else if path == "package-lock.json" {
            (
                FileDisposition::PartialAccept,
                "Lockfile update needs version sync check".to_string(),
                Some("Re-generate lockfile with npm install --package-lock-only".to_string()),
            )
        } else {
            (FileDisposition::Accept, "".to_string(), None)
        }
    });

    assert_eq!(report.file_breakdowns.len(), 3);
    assert_eq!(report.accepted_count(), 1);
    assert_eq!(report.dropped_count(), 1); // target/output.tmp matched drop pattern
    assert_eq!(report.partial_count(), 1);
}

#[test]
fn test_generate_markdown_rejection_breakdown() {
    let item1 = FileBreakdownItem::new(
        "src-tauri/src/modules/instance.rs",
        FileDisposition::Accept,
        "Good type definitions.",
        None,
    );
    let item2 = FileBreakdownItem::new(
        "src-tauri/src/proxy/handlers/claude.rs",
        FileDisposition::Drop,
        "Local adapter fix violates pipeline-first decoupling guideline.",
        Some(
            "Refactor generic prompt sanitization logic into common/request_header_sanitizer.rs"
                .to_string(),
        ),
    );

    let report = RejectionBreakdownReport::new("Refactor pipeline", "contributor_jane")
        .with_pr_number(42)
        .with_file_breakdown(item1)
        .with_file_breakdown(item2)
        .with_actionable_step("Move adapter logic into common pipeline")
        .with_co_author("Jane Contributor", "jane@example.com");

    let md = generate_markdown_rejection_breakdown(&report);
    assert!(md.contains("Review Feedback for PR #42 (`Refactor pipeline`)"));
    assert!(md.contains("Thank you @contributor_jane"));
    assert!(md.contains("🟢 **Accept**"));
    assert!(md.contains("🔴 **Drop**"));
    assert!(md.contains("#### 🧭 Actionable Path Forward"));
    assert!(md.contains("- [ ] Move adapter logic into common pipeline"));
    assert!(md.contains("Co-authored-by: Jane Contributor <jane@example.com>"));
}
