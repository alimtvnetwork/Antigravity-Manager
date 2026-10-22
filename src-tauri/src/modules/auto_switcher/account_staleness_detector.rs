// Inactive Account Staleness Detector (Imp-519)
// Identifies accounts whose cached quota data has aged significantly,
// flagging them for priority synchronization before selection evaluation.

use serde::{Deserialize, Serialize};

pub const DEFAULT_STALENESS_THRESHOLD_SECS: u64 = 1800; // 30 minutes

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStalenessReport {
    pub account_id: String,
    pub last_synced_epoch_secs: u64,
    pub elapsed_secs: u64,
    pub is_stale: bool,
    pub requires_force_refresh: bool,
}

pub struct AccountStalenessDetector {
    staleness_threshold_secs: u64,
}

impl Default for AccountStalenessDetector {
    fn default() -> Self {
        Self::new(DEFAULT_STALENESS_THRESHOLD_SECS)
    }
}

impl AccountStalenessDetector {
    pub fn new(staleness_threshold_secs: u64) -> Self {
        Self {
            staleness_threshold_secs: if staleness_threshold_secs == 0 {
                DEFAULT_STALENESS_THRESHOLD_SECS
            } else {
                staleness_threshold_secs
            },
        }
    }

    /// Evaluates staleness of an account given the last synchronization timestamp.
    pub fn evaluate_staleness(&self, account_id: &str, last_synced: u64, current_epoch: u64) -> AccountStalenessReport {
        let elapsed = current_epoch.saturating_sub(last_synced);
        let is_stale = elapsed >= self.staleness_threshold_secs;
        let requires_force_refresh = elapsed >= (self.staleness_threshold_secs * 2);

        AccountStalenessReport {
            account_id: account_id.to_string(),
            last_synced_epoch_secs: last_synced,
            elapsed_secs: elapsed,
            is_stale,
            requires_force_refresh,
        }
    }
}
