//! Candidate Account List Deduplicator
//! Eliminates duplicate candidate account entries originating from multiple binding
//! references, retaining the candidate record with the highest composite score.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CandidateRecord {
    pub account_id: String,
    pub email: String,
    pub composite_score: f64,
}

pub struct CandidateDedup;

impl CandidateDedup {
    /// Deduplicates candidate accounts by both account_id and email,
    /// keeping the record with the maximum composite score.
    pub fn deduplicate(candidates: Vec<CandidateRecord>) -> Vec<CandidateRecord> {
        let mut by_id: HashMap<String, CandidateRecord> = HashMap::new();
        let mut by_email: HashMap<String, String> = HashMap::new(); // email -> account_id

        for candidate in candidates {
            let email_lower = candidate.email.to_ascii_lowercase();

            // Check if email was already seen under a different ID
            if let Some(existing_id) = by_email.get(&email_lower).cloned() {
                if let Some(existing) = by_id.get_mut(&existing_id) {
                    if candidate.composite_score > existing.composite_score {
                        *existing = candidate;
                    }
                    continue;
                }
            }

            // Check by account_id
            if let Some(existing) = by_id.get_mut(&candidate.account_id) {
                if candidate.composite_score > existing.composite_score {
                    *existing = candidate;
                }
            } else {
                by_email.insert(email_lower, candidate.account_id.clone());
                by_id.insert(candidate.account_id.clone(), candidate);
            }
        }

        let mut result: Vec<CandidateRecord> = by_id.into_values().collect();
        result.sort_by(|a, b| {
            b.composite_score
                .partial_cmp(&a.composite_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        result
    }
}
