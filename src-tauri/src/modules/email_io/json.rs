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

/// Inspect JSON payload to extract primary account email and whether a secret/password is already embedded.
pub fn extract_email_and_has_password(payload: &str) -> (Option<String>, bool) {
    let clean = payload.trim_start_matches('\u{feff}');
    let Ok(raw_val) = serde_json::from_str::<serde_json::Value>(clean) else {
        return (None, false);
    };
    let (_, val) = crate::modules::json_envelope::unpack_envelope(raw_val);

    if let Some(accounts) = val.get("accounts").and_then(|v| v.as_array()) {
        let email = accounts
            .first()
            .and_then(|a| a.get("email"))
            .and_then(|e| e.as_str())
            .map(|s| s.to_string());
        let has_creds = val
            .get("credentials")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter().any(|c| {
                    c.get("multi_encoded_secret")
                        .and_then(|s| s.as_str())
                        .map(|s| !s.trim().is_empty())
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);
        return (email, has_creds);
    }

    let email = val
        .get("email")
        .and_then(|e| e.as_str())
        .map(|s| s.to_string());
    let has_pwd = val
        .get("password")
        .and_then(|p| p.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
        || val
            .get("multi_encoded_secret")
            .and_then(|p| p.as_str())
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
    (email, has_pwd)
}

/// Import email configuration from structured JSON
pub fn import_from_json(payload: &str) -> Result<ImportSummary, String> {
    import_from_json_with_password(payload, None)
}

/// Import email configuration from structured JSON (full bundle or compact credential spec) with optional password override
pub fn import_from_json_with_password(
    payload: &str,
    password_override: Option<&str>,
) -> Result<ImportSummary, String> {
    let clean = payload.trim_start_matches('\u{feff}');
    if let Ok(bundle) = crate::modules::json_envelope::extract_payload::<EmailExportBundle>(clean)
        .map(|(b, _)| b)
        .or_else(|_| serde_json::from_str::<EmailExportBundle>(clean))
    {
        let mut summary = ImportSummary {
            accounts_imported: 0,
            recipients_imported: 0,
            recipients_skipped_duplicates: 0,
            recipients_failed: 0,
            settings_updated: false,
            errors: Vec::new(),
        };

        let mut imported_ids = Vec::new();
        for acc in bundle.accounts {
            let acc_email = acc.email.clone();
            let acc_id = acc.id.clone();
            let input = EmailAccountInput {
                id: Some(acc.id),
                alias: acc.alias,
                email: acc.email,
                password: password_override
                    .filter(|p| !p.trim().is_empty())
                    .map(|p| p.to_string()),
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
                imported_ids.push(acc_id);
            } else {
                summary
                    .errors
                    .push(format!("Failed to import account '{}'", acc_email));
            }
        }

        // Restore multi-pass encoded secrets into vault (unless overridden by explicit password)
        if password_override.filter(|p| !p.trim().is_empty()).is_none() {
            for cred in bundle.credentials {
                if let Ok(plain_secret) =
                    base64_decode_multi(&cred.multi_encoded_secret, cred.passes)
                {
                    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                    crate::error::record_ignored(
                        email_vault_db::save_account_secret(&cred.account_id, &plain_secret),
                        "save_account_secret",
                    );
                }
            }
        } else if let Some(pwd) = password_override.filter(|p| !p.trim().is_empty()) {
            for acc_id in &imported_ids {
                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                crate::error::record_ignored(
                    email_vault_db::save_account_secret(acc_id, pwd),
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

            let outcome = email_vault_db::add_notify_recipient_outcome(input);
            summary.record_recipient_outcome(&outcome);
        }

        if let Ok(_) = email_vault_db::save_notification_settings(bundle.settings) {
            summary.settings_updated = true;
        }

        return Ok(summary);
    }

    // Fallback: Compact single-account credential spec JSON (e.g. email-credentials.json or inline JSON one-liner)
    let raw_val: serde_json::Value =
        serde_json::from_str(clean).map_err(|e| format!("Invalid JSON format: {}", e))?;
    let (_, val) = crate::modules::json_envelope::unpack_envelope(raw_val);

    let email = val
        .get("email")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "Invalid JSON format: missing 'email' or 'accounts' field".to_string())?;

    let account_id = val
        .get("account_id")
        .or_else(|| val.get("id"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let alias = val
        .get("alias")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| email.clone());

    let smtp_obj = val.get("smtp");
    let imap_obj = val.get("imap");

    let domain = email.split('@').nth(1).unwrap_or("gmail.com");
    let smtp_host = smtp_obj
        .and_then(|o| o.get("host"))
        .or_else(|| val.get("smtp_host"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("mail.{}", domain));

    let smtp_port = smtp_obj
        .and_then(|o| o.get("port"))
        .or_else(|| val.get("smtp_port"))
        .and_then(|v| v.as_u64())
        .map(|p| p as u16)
        .unwrap_or(465);

    let imap_host = imap_obj
        .and_then(|o| o.get("host"))
        .or_else(|| val.get("imap_host"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| smtp_host.clone());

    let imap_port = imap_obj
        .and_then(|o| o.get("port"))
        .or_else(|| val.get("imap_port"))
        .and_then(|v| v.as_u64())
        .map(|p| p as u16)
        .unwrap_or(993);

    let encryption_type = smtp_obj
        .and_then(|o| o.get("security").or_else(|| o.get("encryption")))
        .or_else(|| val.get("encryption_type"))
        .or_else(|| val.get("security"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            if smtp_port == 465 || imap_port == 993 {
                "SSL".to_string()
            } else {
                "TLS".to_string()
            }
        });

    let effective_password = password_override
        .filter(|p| !p.trim().is_empty())
        .map(|p| p.to_string())
        .or_else(|| {
            val.get("password")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .filter(|s| !s.trim().is_empty())
        })
        .or_else(|| {
            let passes = val.get("passes").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
            val.get("multi_encoded_secret")
                .and_then(|v| v.as_str())
                .and_then(|enc| base64_decode_multi(enc, passes).ok())
        });

    let mut summary = ImportSummary {
        accounts_imported: 0,
        recipients_imported: 0,
        recipients_skipped_duplicates: 0,
        recipients_failed: 0,
        settings_updated: false,
        errors: Vec::new(),
    };

    let input = EmailAccountInput {
        id: account_id,
        alias,
        email: email.clone(),
        password: effective_password,
        smtp_host,
        smtp_port,
        imap_host,
        imap_port,
        encryption_type,
        is_default: val
            .get("is_default")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        is_active: val
            .get("is_active")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
    };

    if let Ok(_) = email_vault_db::upsert_email_account(input) {
        summary.accounts_imported += 1;
    } else {
        summary
            .errors
            .push(format!("Failed to import account '{}'", email));
    }

    if let Some(rcpt) = val
        .get("notify_recipient")
        .or_else(|| val.get("recipient"))
        .and_then(|v| v.as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        let rec_input = NotifyRecipientInput {
            email: rcpt.to_string(),
            group_name: Some("default".to_string()),
            is_active: Some(true),
        };
        let outcome = email_vault_db::add_notify_recipient_outcome(rec_input);
        summary.record_recipient_outcome(&outcome);
    }

    if let Some(rcpts) = val.get("recipients").and_then(|v| v.as_array()) {
        for r in rcpts {
            let r_email = r
                .get("email")
                .and_then(|v| v.as_str())
                .or_else(|| r.as_str())
                .map(|s| s.trim())
                .unwrap_or("");
            if !r_email.is_empty() {
                let grp = r
                    .get("group_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default")
                    .to_string();
                let act = r.get("is_active").and_then(|v| v.as_bool()).unwrap_or(true);
                let rec_input = NotifyRecipientInput {
                    email: r_email.to_string(),
                    group_name: Some(grp),
                    is_active: Some(act),
                };
                let outcome = email_vault_db::add_notify_recipient_outcome(rec_input);
                summary.record_recipient_outcome(&outcome);
            }
        }
    }

    let mut settings = email_vault_db::get_notification_settings().unwrap_or_default();
    settings.is_enabled = true;
    settings.quota_drop_threshold_percent = 15;
    if let Some(s_obj) = val.get("settings").and_then(|v| v.as_object()) {
        if let Some(b) = s_obj.get("is_enabled").and_then(|v| v.as_bool()) {
            settings.is_enabled = b;
        }
        if let Some(n) = s_obj
            .get("polling_interval_minutes")
            .and_then(|v| v.as_u64())
        {
            settings.polling_interval_minutes = n as u32;
        }
        if let Some(n) = s_obj
            .get("inbox_check_interval_minutes")
            .and_then(|v| v.as_u64())
        {
            settings.inbox_check_interval_minutes = n as u32;
        }
        if let Some(b) = s_obj.get("notify_on_quota_drop").and_then(|v| v.as_bool()) {
            settings.notify_on_quota_drop = b;
        }
        if let Some(n) = s_obj
            .get("quota_drop_threshold_percent")
            .and_then(|v| v.as_u64())
        {
            settings.quota_drop_threshold_percent = n as u32;
        }
        if let Some(b) = s_obj
            .get("notify_on_workspace_switch")
            .and_then(|v| v.as_bool())
        {
            settings.notify_on_workspace_switch = b;
        }
        if let Some(b) = s_obj
            .get("notify_on_idle_workspace")
            .and_then(|v| v.as_bool())
        {
            settings.notify_on_idle_workspace = b;
        }
        if let Some(b) = s_obj
            .get("notify_on_system_update")
            .and_then(|v| v.as_bool())
        {
            settings.notify_on_system_update = b;
        }
        if let Some(b) = s_obj
            .get("allow_remote_prompt_execution")
            .and_then(|v| v.as_bool())
        {
            settings.allow_remote_prompt_execution = b;
        }
        if let Some(b) = s_obj
            .get("allow_remote_cli_execution")
            .and_then(|v| v.as_bool())
        {
            settings.allow_remote_cli_execution = b;
        }
        if let Some(b) = s_obj
            .get("allow_remote_instance_rotation")
            .and_then(|v| v.as_bool())
        {
            settings.allow_remote_instance_rotation = b;
        }
    }
    if email_vault_db::save_notification_settings(settings).is_ok() {
        summary.settings_updated = true;
    }

    Ok(summary)
}
