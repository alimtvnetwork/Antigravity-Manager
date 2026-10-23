//! Unit tests for Remote Branch Base Verifier.

use crate::modules::release_gate::remote_branch_base_verifier::{
    evaluate_branch_base, normalize_git_ref, parse_remote_and_branch, verify_branch_base,
    BranchBaseReport, BranchBaseVerification, BranchBaseViolation, ReleaseChannel,
};

#[test]
fn test_valid_stable_main_origin_main() {
    let req = BranchBaseVerification::new("hotfix/auth-leak", "origin/main", ReleaseChannel::StableMain);
    let res = verify_branch_base(&req);
    assert!(res.is_ok(), "Expected origin/main for StableMain to be valid");

    let report = res.unwrap();
    assert!(report.is_base_valid);
    assert!(report.is_remote_tracking);
    assert_eq!(report.detected_remote.as_deref(), Some("origin"));
    assert_eq!(report.detected_branch.as_deref(), Some("main"));
    assert!(report.violations.is_empty());
}

#[test]
fn test_valid_stable_main_upstream_main() {
    let req = BranchBaseVerification::new("hotfix/crash-fix", "upstream/main", ReleaseChannel::StableMain);
    let report = verify_branch_base(&req).expect("upstream/main is allowed for StableMain");
    assert!(report.is_base_valid);
    assert_eq!(report.detected_remote.as_deref(), Some("upstream"));
    assert_eq!(report.detected_branch.as_deref(), Some("main"));
}

#[test]
fn test_valid_preview_beta_origin_beta() {
    let req = BranchBaseVerification::new("feat/thinking-modal", "origin/beta", ReleaseChannel::PreviewBeta);
    let res = verify_branch_base(&req);
    assert!(res.is_ok(), "Expected origin/beta for PreviewBeta to be valid");

    let report = res.unwrap();
    assert!(report.is_base_valid);
    assert!(report.is_remote_tracking);
    assert_eq!(report.detected_remote.as_deref(), Some("origin"));
    assert_eq!(report.detected_branch.as_deref(), Some("beta"));
    assert!(report.violations.is_empty());
}

#[test]
fn test_valid_preview_beta_upstream_beta() {
    let req = BranchBaseVerification::new("feat/proxy-router", "upstream/beta", ReleaseChannel::PreviewBeta);
    let report = verify_branch_base(&req).expect("upstream/beta is allowed for PreviewBeta");
    assert!(report.is_base_valid);
    assert_eq!(report.detected_remote.as_deref(), Some("upstream"));
    assert_eq!(report.detected_branch.as_deref(), Some("beta"));
}

#[test]
fn test_reject_local_main_base() {
    let req = BranchBaseVerification::new("hotfix/patch-1", "main", ReleaseChannel::StableMain);
    let res = verify_branch_base(&req);
    match res {
        Err(BranchBaseViolation::LocalBranchBaseForbidden { base_ref, guidance }) => {
            assert_eq!(base_ref, "main");
            assert!(guidance.contains("origin/main"));
        }
        other => panic!("Expected LocalBranchBaseForbidden, got {other:?}"),
    }
}

#[test]
fn test_reject_local_beta_base() {
    let req = BranchBaseVerification::new("feat/ui-glow", "beta", ReleaseChannel::PreviewBeta);
    let res = verify_branch_base(&req);
    match res {
        Err(BranchBaseViolation::LocalBranchBaseForbidden { base_ref, guidance }) => {
            assert_eq!(base_ref, "beta");
            assert!(guidance.contains("origin/beta"));
        }
        other => panic!("Expected LocalBranchBaseForbidden, got {other:?}"),
    }
}

#[test]
fn test_reject_local_feature_branch() {
    let req = BranchBaseVerification::new("feat/subtask", "feature/local-work", ReleaseChannel::PreviewBeta);
    let res = verify_branch_base(&req);
    match res {
        Err(BranchBaseViolation::LocalBranchBaseForbidden { base_ref, .. }) => {
            assert_eq!(base_ref, "feature/local-work");
        }
        other => panic!("Expected LocalBranchBaseForbidden, got {other:?}"),
    }
}

#[test]
fn test_channel_base_mismatch_preview_branching_from_main() {
    let req = BranchBaseVerification::new("feat/new-pipeline", "origin/main", ReleaseChannel::PreviewBeta);
    let res = verify_branch_base(&req);
    match res {
        Err(BranchBaseViolation::ChannelBaseMismatch { channel, base_ref, expected, guidance }) => {
            assert_eq!(channel, ReleaseChannel::PreviewBeta);
            assert_eq!(base_ref, "origin/main");
            assert!(expected.contains("origin/beta"));
            assert!(guidance.contains("preview releases must base from origin/beta"));
        }
        other => panic!("Expected ChannelBaseMismatch, got {other:?}"),
    }
}

#[test]
fn test_channel_base_mismatch_stable_branching_from_beta() {
    let req = BranchBaseVerification::new("hotfix/p0-prod", "origin/beta", ReleaseChannel::StableMain);
    let res = verify_branch_base(&req);
    match res {
        Err(BranchBaseViolation::ChannelBaseMismatch { channel, base_ref, expected, guidance }) => {
            assert_eq!(channel, ReleaseChannel::StableMain);
            assert_eq!(base_ref, "origin/beta");
            assert!(expected.contains("origin/main"));
            assert!(guidance.contains("stable releases must base from origin/main"));
        }
        other => panic!("Expected ChannelBaseMismatch, got {other:?}"),
    }
}

#[test]
fn test_normalize_git_ref_strips_heads_and_remotes() {
    assert_eq!(normalize_git_ref("refs/remotes/origin/main"), "origin/main");
    assert_eq!(normalize_git_ref("refs/heads/main"), "main");
    assert_eq!(normalize_git_ref("origin/beta"), "origin/beta");
    assert_eq!(normalize_git_ref("  refs/remotes/upstream/beta  "), "upstream/beta");
}

#[test]
fn test_parse_remote_and_branch() {
    let (rem1, br1, ok1) = parse_remote_and_branch("origin/main");
    assert_eq!(rem1.as_deref(), Some("origin"));
    assert_eq!(br1.as_deref(), Some("main"));
    assert!(ok1);

    let (rem2, br2, ok2) = parse_remote_and_branch("upstream/beta");
    assert_eq!(rem2.as_deref(), Some("upstream"));
    assert_eq!(br2.as_deref(), Some("beta"));
    assert!(ok2);

    let (rem3, br3, ok3) = parse_remote_and_branch("local-branch");
    assert_eq!(rem3, None);
    assert_eq!(br3.as_deref(), Some("local-branch"));
    assert!(!ok3);
}

#[test]
fn test_empty_branch_name_and_base_ref_rejected() {
    let empty_name = BranchBaseVerification::new("", "origin/main", ReleaseChannel::StableMain);
    assert_eq!(verify_branch_base(&empty_name), Err(BranchBaseViolation::EmptyBranchName));

    let empty_base = BranchBaseVerification::new("feat/x", "   ", ReleaseChannel::PreviewBeta);
    assert_eq!(verify_branch_base(&empty_base), Err(BranchBaseViolation::EmptyBaseRef));
}

#[test]
fn test_invalid_branch_name_characters() {
    let bad_space = BranchBaseVerification::new("feat/my branch", "origin/beta", ReleaseChannel::PreviewBeta);
    assert!(matches!(verify_branch_base(&bad_space), Err(BranchBaseViolation::InvalidBranchName { .. })));

    let bad_dots = BranchBaseVerification::new("feat/foo..bar", "origin/beta", ReleaseChannel::PreviewBeta);
    assert!(matches!(verify_branch_base(&bad_dots), Err(BranchBaseViolation::InvalidBranchName { .. })));
}

#[test]
fn test_evaluate_branch_base_report() {
    let req = BranchBaseVerification::new("feat/ui", "origin/beta", ReleaseChannel::PreviewBeta);
    let report = evaluate_branch_base(&req);
    assert!(report.is_base_valid);
    assert!(report.is_remote_tracking);
    assert!(report.violations.is_empty());
}
