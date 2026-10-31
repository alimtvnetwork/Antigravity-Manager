// Stale Instance PID Process Reaper (Imp-530)
// Cross-references recorded instance PIDs against the OS active process table,
// identifying dead or zombie instances and updating their state to Stopped.

use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReapedInstance {
    pub instance_id: String,
    pub stale_pid: u32,
}

pub struct StalePidReaper;

impl StalePidReaper {
    /// Evaluates a list of (instance_id, pid) pairs against a liveness probe function.
    pub fn reap_dead_instances<F>(tracked: &[(String, u32)], is_alive_fn: F) -> Vec<ReapedInstance>
    where
        F: Fn(u32) -> bool,
    {
        let mut reaped = Vec::new();

        for (id, pid) in tracked {
            if *pid > 0 && !is_alive_fn(*pid) {
                info!(instance_id = %id, stale_pid = pid, "Reaped dead instance PID");
                reaped.push(ReapedInstance {
                    instance_id: id.clone(),
                    stale_pid: *pid,
                });
            }
        }

        reaped
    }
}
