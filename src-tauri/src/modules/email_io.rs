//! Email IO Module
//! Two-way Import and Export Engine for JSON, CSV, Excel (XLSX/XML), and SQLite backup.

#![allow(dead_code)]

use crate::modules::email_vault_db::{
    self, EmailAccount, EmailAccountInput, EmailNotificationSettings, NotifyRecipient,
    NotifyRecipientInput,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Master export bundle for JSON serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailExportBundle {
    pub version: String,
    pub exported_at: i64,
    pub accounts: Vec<EmailAccount>,
    pub recipients: Vec<NotifyRecipient>,
    pub settings: EmailNotificationSettings,
}

/// Summary report returned after importing email configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSummary {
    pub accounts_imported: usize,
    pub recipients_imported: usize,
    pub settings_updated: bool,
    pub errors: Vec<String>,
}

/// Export email configuration as structured JSON
pub fn export_to_json() -> Result<String, String> {
    let accounts = email_vault_db::list_email_accounts()?;
    let recipients = email_vault_db::list_notify_recipients()?;
    let settings = email_vault_db::get_notification_settings()?;

    let bundle = EmailExportBundle {
        version: "4.18.0".to_string(),
        exported_at: Utc::now().timestamp(),
        accounts,
        recipients,
        settings,
    };

    serde_json::to_string_pretty(&bundle).map_err(|e| format!("Failed to serialize JSON: {}", e))
}

/// Import email configuration from structured JSON
pub fn import_from_json(payload: &str) -> Result<ImportSummary, String> {
    let bundle: EmailExportBundle =
        serde_json::from_str(payload).map_err(|e| format!("Invalid JSON format: {}", e))?;

    let mut summary = ImportSummary {
        accounts_imported: 0,
        recipients_imported: 0,
        settings_updated: false,
        errors: Vec::new(),
    };

    for acc in bundle.accounts {
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
            summary.errors.push(format!("Failed to import account '{}'", acc.email));
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
fn escape_csv_field(val: &str) -> String {
    if val.contains(',') || val.contains('"') || val.contains('\n') {
        format!("\"{}\"", val.replace('"', "\"\""))
    } else {
        val.to_string()
    }
}

/// Parse a single CSV line handling quotes
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in line.chars() {
        if c == '"' {
            in_quotes = !in_quotes;
        } else if c == ',' && !in_quotes {
            fields.push(current.trim().to_string());
            current.clear();
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
        } else if trimmed.starts_with('#') || trimmed.starts_with("alias,") || trimmed.starts_with("email,") {
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
                is_default: fields.get(7).map(|v| v == "true" || v == "1").unwrap_or(false),
                is_active: fields.get(8).map(|v| v != "false" && v != "0").unwrap_or(true),
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

/// Export configuration to native Excel XML Spreadsheet format (.xlsx / .xml)
pub fn export_to_excel() -> Result<String, String> {
    let accounts = email_vault_db::list_email_accounts()?;
    let recipients = email_vault_db::list_notify_recipients()?;

    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<?mso-application progid=\"Excel.Sheet\"?>\n");
    xml.push_str("<Workbook xmlns=\"urn:schemas-microsoft-com:office:spreadsheet\"\n");
    xml.push_str(" xmlns:ss=\"urn:schemas-microsoft-com:office:spreadsheet\">\n");

    // Worksheet 1: Mailboxes
    xml.push_str(" <Worksheet ss:Name=\"Mailboxes\">\n");
    xml.push_str("  <Table>\n");
    xml.push_str("   <Row>\n");
    xml.push_str("    <Cell><Data ss:Type=\"String\">Alias</Data></Cell>\n");
    xml.push_str("    <Cell><Data ss:Type=\"String\">Email</Data></Cell>\n");
    xml.push_str("    <Cell><Data ss:Type=\"String\">SMTP Host</Data></Cell>\n");
    xml.push_str("    <Cell><Data ss:Type=\"String\">SMTP Port</Data></Cell>\n");
    xml.push_str("    <Cell><Data ss:Type=\"String\">IMAP Host</Data></Cell>\n");
    xml.push_str("    <Cell><Data ss:Type=\"String\">IMAP Port</Data></Cell>\n");
    xml.push_str("    <Cell><Data ss:Type=\"String\">Encryption</Data></Cell>\n");
    xml.push_str("    <Cell><Data ss:Type=\"String\">Is Default</Data></Cell>\n");
    xml.push_str("    <Cell><Data ss:Type=\"String\">Is Active</Data></Cell>\n");
    xml.push_str("   </Row>\n");

    for a in &accounts {
        xml.push_str("   <Row>\n");
        xml.push_str(&format!("    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n", a.alias));
        xml.push_str(&format!("    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n", a.email));
        xml.push_str(&format!("    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n", a.smtp_host));
        xml.push_str(&format!("    <Cell><Data ss:Type=\"Number\">{}</Data></Cell>\n", a.smtp_port));
        xml.push_str(&format!("    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n", a.imap_host));
        xml.push_str(&format!("    <Cell><Data ss:Type=\"Number\">{}</Data></Cell>\n", a.imap_port));
        xml.push_str(&format!("    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n", a.encryption_type));
        xml.push_str(&format!("    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n", a.is_default));
        xml.push_str(&format!("    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n", a.is_active));
        xml.push_str("   </Row>\n");
    }

    xml.push_str("  </Table>\n");
    xml.push_str(" </Worksheet>\n");

    // Worksheet 2: Recipients
    xml.push_str(" <Worksheet ss:Name=\"Recipients\">\n");
    xml.push_str("  <Table>\n");
    xml.push_str("   <Row>\n");
    xml.push_str("    <Cell><Data ss:Type=\"String\">Email</Data></Cell>\n");
    xml.push_str("    <Cell><Data ss:Type=\"String\">Group</Data></Cell>\n");
    xml.push_str("    <Cell><Data ss:Type=\"String\">Is Active</Data></Cell>\n");
    xml.push_str("   </Row>\n");

    for r in &recipients {
        xml.push_str("   <Row>\n");
        xml.push_str(&format!("    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n", r.email));
        xml.push_str(&format!("    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n", r.group_name));
        xml.push_str(&format!("    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n", r.is_active));
        xml.push_str("   </Row>\n");
    }

    xml.push_str("  </Table>\n");
    xml.push_str(" </Worksheet>\n");
    xml.push_str("</Workbook>\n");

    Ok(xml)
}

/// Backup email_vault.db to target path
pub fn backup_vault_db(target_path: &Path) -> Result<(), String> {
    let source_path = email_vault_db::get_email_vault_db_path()?;
    if !source_path.exists() {
        return Err("Email vault database does not exist yet".to_string());
    }

    fs::copy(&source_path, target_path)
        .map_err(|e| format!("Failed to backup email vault database: {}", e))?;
    Ok(())
}

/// Restore email_vault.db from source backup file
pub fn restore_vault_db(source_path: &Path) -> Result<(), String> {
    if !source_path.exists() {
        return Err("Source database file does not exist".to_string());
    }

    let target_path = email_vault_db::get_email_vault_db_path()?;
    fs::copy(source_path, &target_path)
        .map_err(|e| format!("Failed to restore email vault database: {}", e))?;

    // Re-verify connectivity and run pragmas
    let _ = email_vault_db::connect_vault_db()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_line_parser() {
        let line = "My Alias,user@example.com,smtp.example.com,587";
        let fields = parse_csv_line(line);
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], "My Alias");
        assert_eq!(fields[1], "user@example.com");
    }

    #[test]
    fn test_escape_csv_field() {
        assert_eq!(escape_csv_field("simple"), "simple");
        assert_eq!(escape_csv_field("has,comma"), "\"has,comma\"");
    }
}
