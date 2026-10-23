//! Unit tests for Local Commit Convergence & Noise Elimination Auditor.

use crate::modules::release_gate::commit_convergence_auditor::{
    audit_commit_history, classify_commit, CommitEntry, NoiseClassification,
};

#[test]
fn test_clean_converged_history_passes() {
    let commits = vec![
        CommitEntry {
            hash: "a1b2c3d".to_string(),
            message: "feat(auth): implement OAuth2 token refresh with exponential backoff ladder"
                .to_string(),
            author: "aukgit".to_string(),
            files_changed: vec!["src-tauri/src/modules/auth.rs".to_string()],
        },
        CommitEntry {
            hash: "e4f5g6h".to_string(),
            message: "refactor(release_gate): extract risky path replacement fallback guard"
                .to_string(),
            author: "aukgit".to_string(),
            files_changed: vec![
                "src-tauri/src/modules/release_gate/risky_path_fallback_guard.rs".to_string(),
            ],
        },
    ];

    let report = audit_commit_history(&commits);
    assert!(report.is_converged);
    assert_eq!(report.total_commits, 2);
    assert_eq!(report.noise_commit_count, 0);
    assert!(report.flagged_commits.is_empty());
    assert!(report.suggested_action.contains("Ready for PR"));
}

#[test]
fn test_wip_commit_detected() {
    let commits = vec![CommitEntry {
        hash: "1111111".to_string(),
        message: "wip: initial draft for switcher".to_string(),
        author: "dev".to_string(),
        files_changed: vec!["src/switcher.rs".to_string()],
    }];

    let report = audit_commit_history(&commits);
    assert!(!report.is_converged);
    assert_eq!(report.noise_commit_count, 1);
    assert_eq!(
        report.flagged_commits[0].2,
        NoiseClassification::WipCheckpoint
    );
}

#[test]
fn test_wip2_checkpoint_detected() {
    let commits = vec![
        CommitEntry {
            hash: "2222222".to_string(),
            message: "checkpoint before refactoring settings".to_string(),
            author: "dev".to_string(),
            files_changed: vec!["src/settings.rs".to_string()],
        },
        CommitEntry {
            hash: "3333333".to_string(),
            message: "wip2: still working on state".to_string(),
            author: "dev".to_string(),
            files_changed: vec!["src/settings.rs".to_string()],
        },
    ];

    let report = audit_commit_history(&commits);
    assert!(!report.is_converged);
    assert_eq!(report.noise_commit_count, 2);
}

#[test]
fn test_try_fix_noise_detected() {
    let commit = CommitEntry {
        hash: "4444444".to_string(),
        message: "try fix database lock on windows".to_string(),
        author: "dev".to_string(),
        files_changed: vec!["src/db.rs".to_string()],
    };

    let classification = classify_commit(&commit, &[commit.clone()]);
    assert_eq!(
        classification,
        NoiseClassification::TrialAndErrorNoise("try fix".to_string())
    );
}

#[test]
fn test_temp_commit_noise_detected() {
    let commit = CommitEntry {
        hash: "5555555".to_string(),
        message: "temp bypass for authentication header".to_string(),
        author: "dev".to_string(),
        files_changed: vec!["src/net.rs".to_string()],
    };

    let classification = classify_commit(&commit, &[commit.clone()]);
    assert!(matches!(
        classification,
        NoiseClassification::TrialAndErrorNoise(ref s) if s == "temp"
    ));
}

#[test]
fn test_undo_test_noise_detected() {
    let commit = CommitEntry {
        hash: "6666666".to_string(),
        message: "undo test scripts in root".to_string(),
        author: "dev".to_string(),
        files_changed: vec!["run.ps1".to_string()],
    };

    let classification = classify_commit(&commit, &[commit.clone()]);
    assert_eq!(
        classification,
        NoiseClassification::TrialAndErrorNoise("undo test".to_string())
    );
}

#[test]
fn test_fixed_typo_noise_detected() {
    let commit = CommitEntry {
        hash: "7777777".to_string(),
        message: "fixed typo in logging statement".to_string(),
        author: "dev".to_string(),
        files_changed: vec!["src/logger.rs".to_string()],
    };

    let classification = classify_commit(&commit, &[commit.clone()]);
    assert_eq!(
        classification,
        NoiseClassification::TrialAndErrorNoise("fixed typo".to_string())
    );
}

#[test]
fn test_revert_previous_noise_detected() {
    let commit = CommitEntry {
        hash: "8888888".to_string(),
        message: "revert previous broken patch".to_string(),
        author: "dev".to_string(),
        files_changed: vec!["src/engine.rs".to_string()],
    };

    let classification = classify_commit(&commit, &[commit.clone()]);
    assert_eq!(
        classification,
        NoiseClassification::TrialAndErrorNoise("revert previous".to_string())
    );
}

#[test]
fn test_test_again_noise_detected() {
    let commit = CommitEntry {
        hash: "9999999".to_string(),
        message: "test again on CI runner".to_string(),
        author: "dev".to_string(),
        files_changed: vec![".github/workflows/ci.yml".to_string()],
    };

    let classification = classify_commit(&commit, &[commit.clone()]);
    assert_eq!(
        classification,
        NoiseClassification::TrialAndErrorNoise("test again".to_string())
    );
}

#[test]
fn test_fix_fix_noise_detected() {
    let commit = CommitEntry {
        hash: "aaaaaaa".to_string(),
        message: "fix fix broken test assertion".to_string(),
        author: "dev".to_string(),
        files_changed: vec!["tests/suite.rs".to_string()],
    };

    let classification = classify_commit(&commit, &[commit.clone()]);
    assert_eq!(
        classification,
        NoiseClassification::TrialAndErrorNoise("fix fix".to_string())
    );
}

#[test]
fn test_oops_noise_detected() {
    let commit = CommitEntry {
        hash: "bbbbbbb".to_string(),
        message: "oops forgot to remove debug println".to_string(),
        author: "dev".to_string(),
        files_changed: vec!["src/main.rs".to_string()],
    };

    let classification = classify_commit(&commit, &[commit.clone()]);
    assert_eq!(
        classification,
        NoiseClassification::TrialAndErrorNoise("oops".to_string())
    );
}

#[test]
fn test_scattered_iterative_fixes_detected() {
    let commits = vec![
        CommitEntry {
            hash: "c1".to_string(),
            message: "fix api endpoint".to_string(),
            author: "dev".to_string(),
            files_changed: vec!["src/api/handler.rs".to_string()],
        },
        CommitEntry {
            hash: "c2".to_string(),
            message: "tweak handler".to_string(),
            author: "dev".to_string(),
            files_changed: vec!["src/api/handler.rs".to_string()],
        },
        CommitEntry {
            hash: "c3".to_string(),
            message: "adjust response".to_string(),
            author: "dev".to_string(),
            files_changed: vec!["src/api/handler.rs".to_string()],
        },
    ];

    let report = audit_commit_history(&commits);
    assert!(!report.is_converged);
    assert_eq!(report.noise_commit_count, 3);
    assert_eq!(
        report.flagged_commits[0].2,
        NoiseClassification::ScatteredFixes(3)
    );
    assert_eq!(
        report.flagged_commits[1].2,
        NoiseClassification::ScatteredFixes(3)
    );
    assert_eq!(
        report.flagged_commits[2].2,
        NoiseClassification::ScatteredFixes(3)
    );
}

#[test]
fn test_empty_commit_history_is_converged() {
    let report = audit_commit_history(&[]);
    assert!(report.is_converged);
    assert_eq!(report.total_commits, 0);
    assert_eq!(report.noise_commit_count, 0);
}

#[test]
fn test_serde_serialization_roundtrip() {
    let report = audit_commit_history(&[CommitEntry {
        hash: "test_hash".to_string(),
        message: "wip test".to_string(),
        author: "dev".to_string(),
        files_changed: vec!["file.rs".to_string()],
    }]);

    let json = serde_json::to_string(&report).unwrap();
    let deserialized: super::super::commit_convergence_auditor::CommitConvergenceReport =
        serde_json::from_str(&json).unwrap();
    assert_eq!(report, deserialized);
}
