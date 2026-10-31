//! Instance Workspace Hash Collision Sentinel.
//! Computes deterministic hashes of project workspace root directories and detects
//! collisions across registered multi-instance configurations.

use std::collections::{HashMap, HashSet};

/// Collision detection outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceCollisionCheck {
    Unique,
    CollisionDetected {
        first_instance: String,
        second_instance: String,
        workspace_hash: String,
    },
}

/// Normalizes and hashes a workspace directory path string into an 8-character hex string.
pub fn hash_workspace_path(path: &str) -> String {
    let normalized = path.trim().to_lowercase().replace('\\', "/");
    let mut hash: u64 = 0xcbf29ce484222325; // FNV-1a 64-bit offset basis
    for byte in normalized.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
    }
    format!("{:08x}", (hash & 0xFFFFFFFF) as u32)
}

/// Validates that no two active instances share identical hashed workspace roots.
pub fn check_workspace_hash_collisions(
    instance_workspaces: &[(String, Vec<String>)],
) -> WorkspaceCollisionCheck {
    // Map: workspace_hash -> instance_id
    let mut hash_owner: HashMap<String, String> = HashMap::new();

    for (instance_id, workspaces) in instance_workspaces {
        let mut unique_hashes = HashSet::new();
        for ws in workspaces {
            let h = hash_workspace_path(ws);
            if unique_hashes.insert(h.clone()) {
                if let Some(existing_owner) = hash_owner.get(&h) {
                    if existing_owner != instance_id {
                        return WorkspaceCollisionCheck::CollisionDetected {
                            first_instance: existing_owner.clone(),
                            second_instance: instance_id.clone(),
                            workspace_hash: h,
                        };
                    }
                } else {
                    hash_owner.insert(h, instance_id.clone());
                }
            }
        }
    }

    WorkspaceCollisionCheck::Unique
}
