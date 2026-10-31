//! Switch Event Audit Log Enricher.
//! Decorates rotation events with node identities, OS telemetry, and quota delta snapshots.

use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedSwitchContext {
    pub instance_id: String,
    pub from_account_id: String,
    pub to_account_id: String,
    pub from_quota: Option<f64>,
    pub to_quota: Option<f64>,
    pub switch_reason: String,
    pub node_alias: String,
    pub platform: String,
    pub timestamp: i64,
}

/// Enriches a raw profile rotation event with machine and platform context.
pub fn enrich_switch_event(
    instance_id: &str,
    from_account_id: &str,
    to_account_id: &str,
    from_quota: Option<f64>,
    to_quota: Option<f64>,
    switch_reason: &str,
) -> EnrichedSwitchContext {
    let node_alias = match std::env::var("AGM_NODE_ALIAS") {
        Ok(alias) if !alias.trim().is_empty() => alias,
        _ => match hostname::get() {
            Ok(h) => h.to_string_lossy().to_string(),
            Err(_) => "node-unknown".to_string(),
        },
    };

    let platform = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    let timestamp = Utc::now().timestamp();

    EnrichedSwitchContext {
        instance_id: instance_id.to_string(),
        from_account_id: from_account_id.to_string(),
        to_account_id: to_account_id.to_string(),
        from_quota,
        to_quota,
        switch_reason: switch_reason.to_string(),
        node_alias,
        platform,
        timestamp,
    }
}
