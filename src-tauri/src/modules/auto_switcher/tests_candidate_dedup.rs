#[cfg(test)]
mod tests {
    use super::super::candidate_dedup::{CandidateDedup, CandidateRecord};

    #[test]
    fn test_deduplicate_empty() {
        let result = CandidateDedup::deduplicate(vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_deduplicate_no_duplicates() {
        let candidates = vec![
            CandidateRecord {
                account_id: "acc-1".to_string(),
                email: "acc1@test.com".to_string(),
                composite_score: 50.0,
            },
            CandidateRecord {
                account_id: "acc-2".to_string(),
                email: "acc2@test.com".to_string(),
                composite_score: 85.0,
            },
        ];

        let result = CandidateDedup::deduplicate(candidates);
        assert_eq!(result.len(), 2);
        // Sorted descending: acc-2 first
        assert_eq!(result[0].account_id, "acc-2");
        assert_eq!(result[1].account_id, "acc-1");
    }

    #[test]
    fn test_deduplicate_by_account_id_keeps_highest_score() {
        let candidates = vec![
            CandidateRecord {
                account_id: "acc-1".to_string(),
                email: "acc1@test.com".to_string(),
                composite_score: 40.0,
            },
            CandidateRecord {
                account_id: "acc-1".to_string(),
                email: "acc1@test.com".to_string(),
                composite_score: 95.0,
            },
            CandidateRecord {
                account_id: "acc-1".to_string(),
                email: "acc1@test.com".to_string(),
                composite_score: 70.0,
            },
        ];

        let result = CandidateDedup::deduplicate(candidates);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].composite_score, 95.0);
    }

    #[test]
    fn test_deduplicate_by_email_case_insensitive() {
        let candidates = vec![
            CandidateRecord {
                account_id: "acc-1".to_string(),
                email: "SHARED@TEST.COM".to_string(),
                composite_score: 30.0,
            },
            CandidateRecord {
                account_id: "acc-2".to_string(),
                email: "shared@test.com".to_string(),
                composite_score: 90.0,
            },
        ];

        let result = CandidateDedup::deduplicate(candidates);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].composite_score, 90.0);
    }
}
