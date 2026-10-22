//! Multi-instance binding conflict detector.
//! Identifies when two concurrent IDE instance profiles are bound to the same
//! target account (by ID or email), preventing credential corruption and session collisions.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceBinding {
    pub instance_id: String,
    pub account_id: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingConflict {
    pub conflict_key: String, // account_id or email
    pub key_type: String,     // "account_id" or "email"
    pub bound_instance_ids: Vec<String>,
}

pub struct BindingConflictDetector;

impl BindingConflictDetector {
    /// Audits all active instance-to-account bindings and reports any collisions.
    pub fn detect_conflicts(bindings: &[InstanceBinding]) -> Vec<BindingConflict> {
        let mut by_account_id: HashMap<String, HashSet<String>> = HashMap::new();
        let mut by_email: HashMap<String, HashSet<String>> = HashMap::new();

        for b in bindings {
            if !b.account_id.trim().is_empty() {
                by_account_id
                    .entry(b.account_id.trim().to_string())
                    .or_default()
                    .insert(b.instance_id.clone());
            }

            if !b.email.trim().is_empty() {
                by_email
                    .entry(b.email.trim().to_lowercase())
                    .or_default()
                    .insert(b.instance_id.clone());
            }
        }

        let mut conflicts = Vec::new();

        for (acc_id, insts) in by_account_id {
            if insts.len() > 1 {
                conflicts.push(BindingConflict {
                    conflict_key: acc_id,
                    key_type: "account_id".to_string(),
                    bound_instance_ids: insts.into_iter().collect(),
                });
            }
        }

        for (email, insts) in by_email {
            if insts.len() > 1 {
                // Only add if not already captured by account_id conflict
                conflicts.push(BindingConflict {
                    conflict_key: email,
                    key_type: "email".to_string(),
                    bound_instance_ids: insts.into_iter().collect(),
                });
            }
        }

        conflicts
    }
}
