//! Unit tests for Candidate Score Variance & Outlier Dampener (Imp-559).

use super::score_variance_guard::{dampen_score_variance, CandidateScoreItem, MAX_ALLOWED_Z_SCORE};

#[test]
fn test_dampen_score_variance_small_pool() {
    let mut pool = vec![
        CandidateScoreItem {
            account_id: "acc-1".to_string(),
            raw_score: 50.0,
            effective_score: 50.0,
        },
        CandidateScoreItem {
            account_id: "acc-2".to_string(),
            raw_score: 100.0,
            effective_score: 100.0,
        },
    ];

    // Less than 3 items: untouched
    dampen_score_variance(&mut pool);
    assert_eq!(pool[0].effective_score, 50.0);
    assert_eq!(pool[1].effective_score, 100.0);
}

#[test]
fn test_dampen_score_variance_uniform_pool() {
    let mut pool = vec![
        CandidateScoreItem {
            account_id: "acc-1".to_string(),
            raw_score: 75.0,
            effective_score: 75.0,
        },
        CandidateScoreItem {
            account_id: "acc-2".to_string(),
            raw_score: 75.0,
            effective_score: 75.0,
        },
        CandidateScoreItem {
            account_id: "acc-3".to_string(),
            raw_score: 75.0,
            effective_score: 75.0,
        },
    ];

    // Zero variance: untouched
    dampen_score_variance(&mut pool);
    for item in &pool {
        assert_eq!(item.effective_score, 75.0);
    }
}

#[test]
fn test_dampen_score_variance_extreme_outlier() {
    // 9 accounts with scores ~50.0, 1 outlier with score 10_000.0
    let mut pool = Vec::new();
    for i in 0..9 {
        pool.push(CandidateScoreItem {
            account_id: format!("acc-{}", i),
            raw_score: 50.0,
            effective_score: 50.0,
        });
    }
    pool.push(CandidateScoreItem {
        account_id: "acc-outlier".to_string(),
        raw_score: 10_000.0,
        effective_score: 10_000.0,
    });

    dampen_score_variance(&mut pool);

    // The outlier score should be dampened significantly
    let outlier = &pool[9];
    assert!(outlier.effective_score < 10_000.0);
    assert!(outlier.effective_score > 50.0);
}

#[test]
fn test_max_allowed_z_score_constant() {
    assert_eq!(MAX_ALLOWED_Z_SCORE, 2.5);
}
