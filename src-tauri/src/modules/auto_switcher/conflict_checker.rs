//! Dual-key sibling conflict checker.
//! Detects and prevents account collisions across running instances and sandboxes
//! by checking both account_id and bound_email invariants.

use crate::models::instance::InstanceConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictCheckResult {
    pub has_conflict: bool,
    pub conflicting_instance_ids: Vec<String>,
    pub target_account_id: String,
    pub target_email: String,
    pub message: String,
}

/// Evaluates if a candidate account (by ID or Email) is currently bound/active in any sibling instance.
pub fn check_candidate_conflict(
    target_account_id: &str,
    target_email: &str,
    target_instance_id: &str,
    instances: &[InstanceConfig],
) -> ConflictCheckResult {
    let clean_acc_id = target_account_id.trim();
    let clean_email = target_email.trim().to_lowercase();
    let mut conflicting_instance_ids = Vec::new();

    for inst in instances {
        // Skip target instance itself
        if inst.id == target_instance_id {
            continue;
        }

        let mut matches = false;

        // 1. Check bound_account_id match
        if let Some(ref bound_id) = inst.bound_account_id {
            if !clean_acc_id.is_empty() && bound_id.trim() == clean_acc_id {
                matches = true;
            }
        }

        // 2. Check bound_email match
        if let Some(ref bound_em) = inst.bound_email {
            if !clean_email.is_empty() && bound_em.trim().to_lowercase() == clean_email {
                matches = true;
            }
        }

        if matches {
            conflicting_instance_ids.push(inst.id.clone());
        }
    }

    let has_conflict = !conflicting_instance_ids.is_empty();
    let message = if has_conflict {
        format!(
            "Conflict detected: Account '{}' ({}) is actively used by sibling instance(s): [{}]",
            target_account_id,
            target_email,
            conflicting_instance_ids.join(", ")
        )
    } else {
        "No sibling instance conflict detected".to_string()
    };

    ConflictCheckResult {
        has_conflict,
        conflicting_instance_ids,
        target_account_id: target_account_id.to_string(),
        target_email: target_email.to_string(),
        message,
    }
}
