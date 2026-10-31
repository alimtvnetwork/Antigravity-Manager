//! Stale Account Binding Sweeper Sentinel.
//! Sweeps orphaned instance-to-account bindings where the associated instance IDE process
//! has terminated without an orderly unbinding or shutdown handshake.

use std::collections::{HashMap, HashSet};

/// Result summary of an account binding sweep pass.
#[derive(Debug, Clone, Default)]
pub struct BindingSweepSummary {
    pub total_bindings_checked: usize,
    pub stale_bindings_released: usize,
    pub active_bindings_retained: usize,
}

/// Identifies stale bindings given a map of (instance_id -> account_id) and a set of verified live instance IDs.
pub fn sweep_stale_instance_bindings(
    bindings: &mut HashMap<String, String>,
    live_instance_ids: &HashSet<String>,
) -> BindingSweepSummary {
    let mut summary = BindingSweepSummary {
        total_bindings_checked: bindings.len(),
        stale_bindings_released: 0,
        active_bindings_retained: 0,
    };

    let initial_len = bindings.len();
    bindings.retain(|instance_id, _| {
        let is_live = live_instance_ids.contains(instance_id);
        if is_live {
            summary.active_bindings_retained += 1;
        }
        is_live
    });

    summary.stale_bindings_released = initial_len.saturating_sub(bindings.len());
    summary
}
