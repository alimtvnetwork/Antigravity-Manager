use crate::modules::email_vault_db::{
    self, EmailAccount, EmailAccountInput, EmailNotificationSettings, NotifyRecipient,
    NotifyRecipientInput,
};
use chrono::Utc;

use super::*;

/// Export email configuration as structured JSON
pub fn export_to_json() -> Result<String, String> {
    let accounts = email_vault_db::list_email_accounts()?;
    let recipients = email_vault_db::list_notify_recipients()?;
    let settings = email_vault_db::get_notification_settings()?;

    let mut credentials = Vec::new();
    for acc in &accounts {
        if let Ok(secret) = email_vault_db::get_account_secret(&acc.id) {
            if !secret.trim().is_empty() {
                let encoded = base64_encode_multi(&secret, 3);
                credentials.push(ExportedCredential {
                    account_id: acc.id.clone(),
                    auth_type: "PASSWORD".to_string(),
                    multi_encoded_secret: encoded,
                    passes: 3,
                });
            }
        }
    }

    let bundle = EmailExportBundle {
        version: "2.0".to_string(),
        exported_at: Utc::now().timestamp(),
        accounts,
        recipients,
        settings,
        credentials,
    };

    let envelope =
        crate::modules::json_envelope::JsonEnvelope::new("agm/email-credentials", bundle);
    serde_json::to_string_pretty(&envelope).map_err(|e| format!("Failed to serialize JSON: {}", e))
}

/// Import email configuration from structured JSON
pub fn import_from_json(payload: &str) -> Result<ImportSummary, String> {
    let bundle: EmailExportBundle =
        crate::modules::json_envelope::extract_payload::<EmailExportBundle>(payload)
            .map(|(b, _)| b)
            .or_else(|_| serde_json::from_str::<EmailExportBundle>(payload))
            .map_err(|e| format!("Invalid JSON format: {}", e))?;

    let mut summary = ImportSummary {
        accounts_imported: 0,
        recipients_imported: 0,
        settings_updated: false,
        errors: Vec::new(),
    };

    for acc in bundle.accounts {
        let acc_email = acc.email.clone();
        let input = EmailAccountInput {
            id: Some(acc.id),
            alias: acc.alias,
            email: acc.email,
            password: None,
            smtp_host: acc.smtp_host,
            smtp_port: acc.smtp_port,
            imap_host: acc.imap_host,
            imap_port: acc.imap_port,
            encryption_type: acc.encryption_type,
            is_default: acc.is_default,
            is_active: acc.is_active,
        };

        if let Ok(_) = email_vault_db::upsert_email_account(input) {
            summary.accounts_imported += 1;
        } else {
            summary
                .errors
                .push(format!("Failed to import account '{}'", acc_email));
        }
    }

    // Restore multi-pass encoded secrets into vault
    for cred in bundle.credentials {
        if let Ok(plain_secret) = base64_decode_multi(&cred.multi_encoded_secret, cred.passes) {
            // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
            crate::error::record_ignored(
                email_vault_db::save_account_secret(&cred.account_id, &plain_secret),
                "save_account_secret",
            );
        }
    }

    for rec in bundle.recipients {
        let input = NotifyRecipientInput {
            email: rec.email,
            group_name: Some(rec.group_name),
            is_active: Some(rec.is_active),
        };

        if let Ok(_) = email_vault_db::add_notify_recipient(input) {
            summary.recipients_imported += 1;
        }
    }

    if let Ok(_) = email_vault_db::save_notification_settings(bundle.settings) {
        summary.settings_updated = true;
    }

    Ok(summary)
}
