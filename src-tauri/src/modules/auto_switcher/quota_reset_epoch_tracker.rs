// Quota Reset Epoch Replenishment Tracker (Imp-517)
// Tracks historical quota replenishment timestamps per account to model
// recurring reset periods and anticipate available capacity windows.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaResetRecord {
    pub account_id: String,
    pub last_replenish_epoch_secs: u64,
    pub previous_replenish_epoch_secs: Option<u64>,
    pub estimated_cycle_secs: u64,
}

#[derive(Debug, Clone)]
pub struct QuotaResetEpochTracker {
    records: Arc<RwLock<HashMap<String, QuotaResetRecord>>>,
    default_cycle_secs: u64,
}

impl Default for QuotaResetEpochTracker {
    fn default() -> Self {
        Self::new(5 * 3600) // Default 5 hours
    }
}

impl QuotaResetEpochTracker {
    pub fn new(default_cycle_secs: u64) -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            default_cycle_secs,
        }
    }

    /// Records an observed quota reset event for an account.
    pub fn record_reset(&self, account_id: &str, current_epoch_secs: u64) {
        if let Ok(mut map) = self.records.write() {
            let entry = map.entry(account_id.to_string()).or_insert_with(|| QuotaResetRecord {
                account_id: account_id.to_string(),
                last_replenish_epoch_secs: current_epoch_secs,
                previous_replenish_epoch_secs: None,
                estimated_cycle_secs: self.default_cycle_secs,
            });

            if entry.last_replenish_epoch_secs != current_epoch_secs {
                entry.previous_replenish_epoch_secs = Some(entry.last_replenish_epoch_secs);
                let cycle = current_epoch_secs.saturating_sub(entry.last_replenish_epoch_secs);
                if cycle >= 3600 && cycle <= 86400 {
                    entry.estimated_cycle_secs = cycle;
                }
                entry.last_replenish_epoch_secs = current_epoch_secs;
            }
        }
    }

    /// Predicts the next expected reset epoch for an account.
    pub fn predict_next_reset(&self, account_id: &str, current_epoch: u64) -> u64 {
        if let Ok(map) = self.records.read() {
            if let Some(record) = map.get(account_id) {
                let next = record.last_replenish_epoch_secs.saturating_add(record.estimated_cycle_secs);
                if next > current_epoch {
                    return next;
                }
            }
        }
        current_epoch.saturating_add(self.default_cycle_secs)
    }
}
