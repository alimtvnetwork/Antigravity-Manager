use crate::modules::email_vault_db::{
    self, AddRecipientOutcome, EmailAccount, EmailAccountInput, EmailNotificationSettings,
    NotifyRecipient, NotifyRecipientInput,
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
    /// Recipients skipped because the normalized email already existed (plan 88).
    #[serde(default)]
    pub recipients_skipped_duplicates: usize,
    /// Recipients that failed to import (plan 88).
    #[serde(default)]
    pub recipients_failed: usize,
    pub settings_updated: bool,
    pub errors: Vec<String>,
}

impl ImportSummary {
    /// Fold one recipient add outcome into the summary counts (plan 88, subtask 002).
    pub fn record_recipient_outcome(
        &mut self,
        outcome: &Result<AddRecipientOutcome, String>,
    ) {
        match outcome {
            Ok(o) if o.is_added() => self.recipients_imported += 1,
            Ok(_) => self.recipients_skipped_duplicates += 1,
            Err(e) => {
                self.recipients_failed += 1;
                self.errors.push(format!("Failed to import recipient: {}", e));
            }
        }
    }
}
