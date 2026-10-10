use crate::modules::email_vault_db::{
    self, EmailAccount, EmailAccountInput, EmailNotificationSettings, NotifyRecipient,
    NotifyRecipientInput,
};

use super::*;

/// Export email accounts and recipients to CSV format
pub fn export_to_csv() -> Result<String, String> {
    let accounts = email_vault_db::list_email_accounts()?;
    let recipients = email_vault_db::list_notify_recipients()?;

    let mut csv = String::new();
    csv.push_str("# SECTION: ACCOUNTS\n");
    csv.push_str("alias,email,smtp_host,smtp_port,imap_host,imap_port,encryption_type,is_default,is_active\n");
    for a in &accounts {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            escape_csv_field(&a.alias),
            escape_csv_field(&a.email),
            escape_csv_field(&a.smtp_host),
            a.smtp_port,
            escape_csv_field(&a.imap_host),
            a.imap_port,
            escape_csv_field(&a.encryption_type),
            a.is_default,
            a.is_active
        ));
    }

    csv.push_str("\n# SECTION: RECIPIENTS\n");
    csv.push_str("email,group_name,is_active\n");
    for r in &recipients {
        csv.push_str(&format!(
            "{},{},{}\n",
            escape_csv_field(&r.email),
            escape_csv_field(&r.group_name),
            r.is_active
        ));
    }

    Ok(csv)
}

/// Helper to escape CSV fields containing commas or quotes
pub(crate) fn escape_csv_field(val: &str) -> String {
    if val.contains(',') || val.contains('"') || val.contains('\n') {
        format!("\"{}\"", val.replace('"', "\"\""))
    } else {
        val.to_string()
    }
}

/// Parse a single CSV line handling quotes
pub(crate) fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in line.chars() {
        if c == '"' {
            in_quotes = !in_quotes;
        } else if c == ',' {
            if !in_quotes {
                fields.push(current.trim().to_string());
                current.clear();
            } else {
                current.push(c);
            }
        } else {
            current.push(c);
        }
    }
    fields.push(current.trim().to_string());
    fields
}

/// Import accounts and recipients from CSV payload
pub fn import_from_csv(payload: &str) -> Result<ImportSummary, String> {
    let mut summary = ImportSummary {
        accounts_imported: 0,
        recipients_imported: 0,
        settings_updated: false,
        errors: Vec::new(),
    };

    let mut current_section = "accounts";

    for line in payload.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("# SECTION: RECIPIENTS") {
            current_section = "recipients";
            continue;
        } else if trimmed.starts_with("# SECTION: ACCOUNTS") {
            current_section = "accounts";
            continue;
        } else if trimmed.starts_with('#')
            || trimmed.starts_with("alias,")
            || trimmed.starts_with("email,")
        {
            continue;
        }

        let fields = parse_csv_line(trimmed);
        if current_section == "accounts" && fields.len() >= 7 {
            let input = EmailAccountInput {
                id: None,
                alias: fields[0].clone(),
                email: fields[1].clone(),
                password: None,
                smtp_host: fields[2].clone(),
                smtp_port: fields[3].parse::<u16>().unwrap_or(587),
                imap_host: fields[4].clone(),
                imap_port: fields[5].parse::<u16>().unwrap_or(993),
                encryption_type: fields[6].clone(),
                is_default: fields
                    .get(7)
                    .map(|v| v == "true" || v == "1")
                    .unwrap_or(false),
                is_active: fields
                    .get(8)
                    .map(|v| v != "false" && v != "0")
                    .unwrap_or(true),
            };

            if let Ok(_) = email_vault_db::upsert_email_account(input) {
                summary.accounts_imported += 1;
            }
        } else if current_section == "recipients" && fields.len() >= 2 {
            let input = NotifyRecipientInput {
                email: fields[0].clone(),
                group_name: Some(fields[1].clone()),
                is_active: fields.get(2).map(|v| v != "false" && v != "0"),
            };

            if let Ok(_) = email_vault_db::add_notify_recipient(input) {
                summary.recipients_imported += 1;
            }
        }
    }

    Ok(summary)
}
