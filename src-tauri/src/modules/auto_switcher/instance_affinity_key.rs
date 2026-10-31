//! Instance Affinity Sticky Key Generator.
//! Computes deterministic sticky affinity hashes from instance profile names and
//! workspace paths to minimize disruptive account switching between interrelated turns.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Deterministic affinity key binding an instance and project workspace.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InstanceAffinityKey {
    pub instance_id: String,
    pub workspace_hash: u64,
}

impl InstanceAffinityKey {
    /// Constructs a new affinity key by hashing the normalized workspace path.
    pub fn new(instance_id: &str, workspace_path: &str) -> Self {
        let normalized_path = workspace_path.trim().to_lowercase().replace('\\', "/");

        let mut hasher = DefaultHasher::new();
        normalized_path.hash(&mut hasher);
        let workspace_hash = hasher.finish();

        Self {
            instance_id: instance_id.trim().to_string(),
            workspace_hash,
        }
    }

    /// Generates a compact alphanumeric representation for log tracking.
    pub fn as_tag(&self) -> String {
        format!("{}:{:08x}", self.instance_id, self.workspace_hash as u32)
    }

    /// Evaluates affinity similarity score bonus (0.0 to 15.0 points)
    /// to favor sticking with the previously bound account when quota remains.
    pub fn compute_stickiness_bonus(
        &self,
        current_binding: Option<&str>,
        candidate_id: &str,
    ) -> f64 {
        if let Some(current) = current_binding {
            if current == candidate_id {
                return 15.0; // Significant stickiness bonus to prevent account churn
            }
        }
        0.0
    }
}
