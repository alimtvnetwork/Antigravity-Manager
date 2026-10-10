use crate::modules::email_vault_db::{
    self, EmailAccount, EmailAccountInput, EmailNotificationSettings, NotifyRecipient,
    NotifyRecipientInput,
};
use serde::{Deserialize, Serialize};

use super::*;

/// Encrypted credential record bundled in universal export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedCredential {
    pub account_id: String,
    pub auth_type: String,
    pub multi_encoded_secret: String,
    pub passes: usize,
}

/// Master export bundle for JSON serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailExportBundle {
    pub version: String,
    pub exported_at: i64,
    pub accounts: Vec<EmailAccount>,
    pub recipients: Vec<NotifyRecipient>,
    pub settings: EmailNotificationSettings,
    #[serde(default)]
    pub credentials: Vec<ExportedCredential>,
}

/// Summary report returned after importing email configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSummary {
    pub accounts_imported: usize,
    pub recipients_imported: usize,
    pub settings_updated: bool,
    pub errors: Vec<String>,
}
