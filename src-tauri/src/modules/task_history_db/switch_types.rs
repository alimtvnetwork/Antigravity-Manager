use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwitchBackupStep {
    #[serde(default)]
    pub prompt_count: usize,
    #[serde(default)]
    pub project_names: Vec<String>,
    #[serde(default)]
    pub project_paths: Vec<String>,
    #[serde(default)]
    pub backup_batch_id: String,
    #[serde(default)]
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwitchResetStep {
    #[serde(default)]
    pub terminated_pids: Vec<u32>,
    #[serde(default)]
    pub auth_swapped: bool,
    #[serde(default)]
    pub credentials_injected: bool,
    #[serde(default)]
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwitchRestoreStep {
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub restored_count: usize,
    #[serde(default)]
    pub dispatched_count: usize,
    #[serde(default)]
    pub prompt_channel_waited: bool,
    #[serde(default)]
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwitchVerificationStep {
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwitchAuditSteps {
    #[serde(default)]
    pub backup: Option<SwitchBackupStep>,
    #[serde(default)]
    pub reset: Option<SwitchResetStep>,
    #[serde(default)]
    pub restore: Option<SwitchRestoreStep>,
    #[serde(default)]
    pub verification: Option<SwitchVerificationStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstanceSwitchRecord {
    pub id: String,
    pub action_code: i32,
    pub action: String,
    pub action_label: String,
    pub status: String,
    pub subject: String,
    pub detail: String,
    pub instance_id: String,
    #[serde(default)]
    pub instance_name: String,
    pub from_email: String,
    pub to_email: String,
    #[serde(default)]
    pub from_account_email: String,
    #[serde(default)]
    pub to_account_email: String,
    #[serde(default)]
    pub switch_reason: String,
    pub created_at: i64,
    pub finished_at: Option<i64>,
    #[serde(default)]
    pub duration_ms: i64,
    #[serde(default)]
    pub steps: Option<SwitchAuditSteps>,
    #[serde(default)]
    pub payload_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstanceSwitchHistoryResponse {
    pub total: usize,
    pub instance_id: String,
    pub records: Vec<InstanceSwitchRecord>,
    #[serde(default)]
    pub items: Vec<InstanceSwitchRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwitchFacts {
    pub from_email: String,
    pub to_email: String,
    pub reason: String,
    pub how: String,
    pub prompt_id: String,
    pub prompt_text: String,
    pub conversation_id: String,
    pub prompt_reinjected: bool,
    pub switch_ok: bool,
    #[serde(default)]
    pub instance_id: String,
    #[serde(default)]
    pub ide_type: String,
    #[serde(default)]
    pub idc_machine_alias: String,
    #[serde(default)]
    pub ide_path: String,
    #[serde(default)]
    pub switch_reason: String,
    #[serde(default)]
    pub steps: Option<SwitchAuditSteps>,
}
