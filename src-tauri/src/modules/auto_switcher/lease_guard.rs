//! Machine Lease Expiry Guard.
//! Verifies distributed workspace profile leases and prevents cross-machine account contention.

use serde::{Deserialize, Serialize};

pub const DEFAULT_LEASE_TTL_SECONDS: u64 = 300; // 5 minutes

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseStatus {
    pub is_lease_valid: bool,
    pub holder_machine_id: Option<String>,
    pub remaining_seconds: u64,
    pub is_held_by_this_node: bool,
}

/// Evaluates if an existing lease is active and held by a foreign node.
pub fn evaluate_lease_validity(
    lease_machine_id: Option<&str>,
    lease_updated_at: Option<i64>,
    current_node_id: &str,
    now_sec: i64,
) -> LeaseStatus {
    let holder = match lease_machine_id {
        Some(h) if !h.trim().is_empty() => h.trim(),
        _ => {
            return LeaseStatus {
                is_lease_valid: false,
                holder_machine_id: None,
                remaining_seconds: 0,
                is_held_by_this_node: false,
            };
        }
    };

    let updated_at = lease_updated_at.unwrap_or(0);
    if updated_at <= 0 {
        return LeaseStatus {
            is_lease_valid: false,
            holder_machine_id: Some(holder.to_string()),
            remaining_seconds: 0,
            is_held_by_this_node: holder == current_node_id,
        };
    }

    let elapsed = now_sec.saturating_sub(updated_at) as u64;
    let is_held_by_this_node = holder == current_node_id;

    if elapsed < DEFAULT_LEASE_TTL_SECONDS {
        let remaining_seconds = DEFAULT_LEASE_TTL_SECONDS - elapsed;
        LeaseStatus {
            is_lease_valid: true,
            holder_machine_id: Some(holder.to_string()),
            remaining_seconds,
            is_held_by_this_node,
        }
    } else {
        LeaseStatus {
            is_lease_valid: false,
            holder_machine_id: Some(holder.to_string()),
            remaining_seconds: 0,
            is_held_by_this_node,
        }
    }
}
