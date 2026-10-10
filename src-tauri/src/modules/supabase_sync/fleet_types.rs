use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Deserialize)]
pub(crate) struct PostgrestNodeRow {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) alias: String,
    #[serde(default)]
    pub(crate) ip_address: String,
    #[serde(default)]
    pub(crate) uptime_seconds: u64,
    #[serde(default)]
    pub(crate) last_heartbeat_at: i64,
    #[serde(default)]
    pub(crate) status: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PostgrestProfileRow {
    pub(crate) id: String,
    pub(crate) node_id: String,
    #[serde(default)]
    pub(crate) profile_name: String,
    #[serde(default)]
    pub(crate) active_account_id: String,
    #[serde(default)]
    pub(crate) active_account_email: String,
    #[serde(default)]
    pub(crate) is_active: bool,
    #[serde(default)]
    pub(crate) status: String,
    #[serde(default)]
    pub(crate) running_prompts_count: Option<usize>,
    #[serde(default)]
    pub(crate) updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PostgrestLeaseRow {
    #[serde(default)]
    pub(crate) account_id: String,
    #[serde(default)]
    pub(crate) account_email: String,
    #[serde(default)]
    pub(crate) node_id: String,
    #[serde(default)]
    pub(crate) profile_name: String,
    #[serde(default)]
    pub(crate) expires_at: i64,
}
