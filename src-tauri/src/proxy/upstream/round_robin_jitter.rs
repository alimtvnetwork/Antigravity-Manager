//! Weighted Round Robin with Randomized Jitter Load Balancer
//!
//! Provides upstream endpoint selection using smooth weighted round-robin distribution
//! enhanced with bounded pseudo-random jitter to prevent synchronous harmonic routing spikes
//! across identical weight nodes.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use tracing::debug;

/// Node descriptor for upstream load balancing.
#[derive(Debug, Clone)]
pub struct WeightedUpstreamNode {
    /// Endpoint identifier or base URL.
    pub endpoint: String,
    /// Base configured weight (1..100).
    pub base_weight: u32,
    /// Current effective weight used for smooth WRR.
    pub current_weight: i32,
}

/// Thread-safe Weighted Round Robin balancer with jitter.
pub struct JitteredWeightedRoundRobin {
    nodes: Mutex<Vec<WeightedUpstreamNode>>,
    request_counter: AtomicUsize,
}

impl JitteredWeightedRoundRobin {
    /// Initializes a new balancer with target endpoints and initial weights.
    pub fn new(endpoints: Vec<(String, u32)>) -> Self {
        let nodes = endpoints
            .into_iter()
            .map(|(endpoint, weight)| WeightedUpstreamNode {
                endpoint,
                base_weight: weight.max(1),
                current_weight: 0,
            })
            .collect();

        Self {
            nodes: Mutex::new(nodes),
            request_counter: AtomicUsize::new(0),
        }
    }

    /// Selects the next upstream node using smooth WRR with counter-derived jitter.
    pub fn next_node(&self) -> Option<String> {
        let mut nodes = match self.nodes.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };

        if nodes.is_empty() {
            return None;
        }

        if nodes.len() == 1 {
            return Some(nodes[0].endpoint.clone());
        }

        let req_id = self.request_counter.fetch_add(1, Ordering::Relaxed);
        let total_weight: i32 = nodes.iter().map(|n| n.base_weight as i32).sum();

        let mut best_idx = 0;
        let mut max_current = i32::MIN;

        // Apply smooth WRR with minor deterministic jitter derived from request sequence
        for (idx, node) in nodes.iter_mut().enumerate() {
            // Jitter modifier between -1 and +1 derived from sequence and index
            let jitter = ((req_id.wrapping_add(idx * 7)) % 3) as i32 - 1;
            let effective_weight = (node.base_weight as i32 + jitter).max(1);

            node.current_weight += effective_weight;

            if node.current_weight > max_current {
                max_current = node.current_weight;
                best_idx = idx;
            }
        }

        nodes[best_idx].current_weight -= total_weight;
        let chosen_endpoint = nodes[best_idx].endpoint.clone();

        debug!(
            endpoint = %chosen_endpoint,
            req_id = req_id,
            "Selected upstream node via jittered WRR"
        );

        Some(chosen_endpoint)
    }

    /// Updates or replaces the active node list.
    pub fn update_nodes(&self, new_nodes: Vec<(String, u32)>) {
        let mut nodes = match self.nodes.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };

        *nodes = new_nodes
            .into_iter()
            .map(|(endpoint, weight)| WeightedUpstreamNode {
                endpoint,
                base_weight: weight.max(1),
                current_weight: 0,
            })
            .collect();
    }
}
