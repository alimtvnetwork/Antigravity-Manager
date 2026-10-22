//! Instance child process tree enumerator.
//! Recursively identifies child and grandchild process IDs spawned by
//! an IDE instance (such as language servers, terminal shells, git workers)
//! before performing clean lifecycle termination.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessTreeNode {
    pub pid: u32,
    pub parent_pid: u32,
    pub name: String,
}

pub struct ChildProcessTreeEnumerator;

impl ChildProcessTreeEnumerator {
    /// Enumerates all descendant PIDs from a list of system processes for a given root PID.
    pub fn collect_descendants(root_pid: u32, all_processes: &[ProcessTreeNode]) -> Vec<u32> {
        let mut descendants = Vec::new();
        let mut current_level = vec![root_pid];

        while !current_level.is_empty() {
            let mut next_level = Vec::new();
            for parent in current_level {
                for proc in all_processes {
                    if proc.parent_pid == parent && proc.pid != root_pid && !descendants.contains(&proc.pid) {
                        descendants.push(proc.pid);
                        next_level.push(proc.pid);
                    }
                }
            }
            current_level = next_level;
        }

        descendants
    }

    /// Reverse order of descendants (deepest children first) for safe bottom-up termination.
    pub fn bottom_up_termination_order(root_pid: u32, all_processes: &[ProcessTreeNode]) -> Vec<u32> {
        let mut list = Self::collect_descendants(root_pid, all_processes);
        list.reverse();
        list.push(root_pid);
        list
    }
}
