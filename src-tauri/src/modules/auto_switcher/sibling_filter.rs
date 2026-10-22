//! Sibling Machine Collision Exclusion Filter.
//! Excludes accounts actively leased by sibling machines in the multi-machine matrix from local selection candidate lists.

use serde::{Deserialize, Serialize};

/// Lease record representing an account reservation by a specific machine node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiblingMachineLease {
    pub machine_id: String,
    pub account_id: String,
    pub leased_until_epoch: i64,
}

/// Filter outcome separating eligible accounts from sibling-leased accounts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SiblingFilterResult {
    pub eligible_candidate_ids: Vec<String>,
    pub excluded_sibling_candidates: Vec<(String, String)>, // (account_id, holding_machine_id)
}

/// Filters candidate account IDs, eliminating accounts held by any sibling node whose lease is still active.
pub fn filter_sibling_leased_candidates(
    candidate_account_ids: &[String],
    active_leases: &[SiblingMachineLease],
    current_machine_id: &str,
    now_epoch: i64,
) -> SiblingFilterResult {
    let mut eligible_candidate_ids = Vec::new();
    let mut excluded_sibling_candidates = Vec::new();

    for id in candidate_account_ids {
        // Find if this account is actively leased by another machine
        let leased_by_sibling = active_leases.iter().find(|lease| {
            lease.account_id == *id
                && lease.machine_id != current_machine_id
                && lease.leased_until_epoch > now_epoch
        });

        if let Some(lease) = leased_by_sibling {
            excluded_sibling_candidates.push((id.clone(), lease.machine_id.clone()));
        } else {
            eligible_candidate_ids.push(id.clone());
        }
    }

    SiblingFilterResult {
        eligible_candidate_ids,
        excluded_sibling_candidates,
    }
}
