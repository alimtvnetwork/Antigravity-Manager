//! Unit tests for Individually Revertable Commit Guard (`Imp-684`).

use super::individually_revertable_commit_guard::*;

#[test]
fn test_evaluate_commit_compliant() {
    let commit = CommitMetadata {
        commit_id: "abc1234".to_string(),
        message: "Feature: proxy - implement streaming chunk boundary guard".to_string(),
        touched_files: vec![
            "src-tauri/src/proxy/handlers/common/streaming_chunk_boundary_guard.rs".to_string(),
            "src-tauri/src/proxy/handlers/common/tests_streaming_chunk_boundary_guard.rs".to_string(),
        ],
    };

    let record = evaluate_commit(&commit);
    assert!(record.is_revertable);
    assert!(record.violations.is_empty());
}

#[test]
fn test_evaluate_commit_vague_message() {
    let commit = CommitMetadata {
        commit_id: "def5678".to_string(),
        message: "fix".to_string(),
        touched_files: vec!["file.rs".to_string()],
    };

    let record = evaluate_commit(&commit);
    assert!(!record.is_revertable);
    assert!(record.violations.iter().any(|v| matches!(v, CommitQualityViolation::VagueOrEmptyMessage { .. })));
}

#[test]
fn test_evaluate_commit_missing_prefix() {
    let commit = CommitMetadata {
        commit_id: "ghi9012".to_string(),
        message: "add streaming boundary guard for proxies".to_string(),
        touched_files: vec!["file.rs".to_string()],
    };

    let record = evaluate_commit(&commit);
    assert!(!record.is_revertable);
    assert!(record.violations.iter().any(|v| matches!(v, CommitQualityViolation::MissingStandardPrefix { .. })));
}

#[test]
fn test_evaluate_commit_noise_keyword() {
    let commit = CommitMetadata {
        commit_id: "jkl3456".to_string(),
        message: "Fix: wip try fix tests again".to_string(),
        touched_files: vec!["file.rs".to_string()],
    };

    let record = evaluate_commit(&commit);
    assert!(!record.is_revertable);
    assert!(record.violations.iter().any(|v| matches!(v, CommitQualityViolation::TrialAndErrorNoise { .. })));
}

#[test]
fn test_audit_commit_series_end_to_end() {
    let commits = vec![
        CommitMetadata {
            commit_id: "c1".to_string(),
            message: "Feature: db - implement fifo queue drain monotonicity verifier".to_string(),
            touched_files: vec!["db.rs".to_string()],
        },
        CommitMetadata {
            commit_id: "c2".to_string(),
            message: "wip temp test".to_string(), // Violates prefix, length, noise
            touched_files: vec!["test.rs".to_string()],
        },
    ];

    let report = audit_commit_series(&commits);
    assert!(!report.is_compliant);
    assert_eq!(report.total_commits_count, 2);
    assert_eq!(report.compliant_commits_count, 1);
    assert!(report.records[0].is_revertable);
    assert!(!report.records[1].is_revertable);
}
