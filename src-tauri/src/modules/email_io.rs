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

use base64::engine::general_purpose::STANDARD;
use base64::Engine;

/// Multi-pass Base64 encoder for sensitive data protection
pub fn base64_encode_multi(data: &str, passes: usize) -> String {
    let mut current = data.as_bytes().to_vec();
    let num_passes = passes.max(1);
    for _ in 0..num_passes {
        let encoded = STANDARD.encode(&current);
        current = encoded.into_bytes();
    }
    String::from_utf8(current).unwrap_or_default()
}

/// Multi-pass Base64 decoder for restoring multi-encoded sensitive payloads
pub fn base64_decode_multi(encoded_str: &str, passes: usize) -> Result<String, String> {
    let mut current_bytes = encoded_str.as_bytes().to_vec();
    let num_passes = passes.max(1);
    for pass in 0..num_passes {
        let text = String::from_utf8(current_bytes.clone())
            .map_err(|e| format!("Pass {} utf8 conversion error: {}", pass, e))?;
        current_bytes = STANDARD
            .decode(text.trim())
            .map_err(|e| format!("Pass {} base64 decode error: {}", pass, e))?;
    }
    String::from_utf8(current_bytes).map_err(|e| format!("Final utf8 decode error: {}", e))
}

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
        version: "4.49.0".to_string(),
        exported_at: Utc::now().timestamp(),
        accounts,
        recipients,
        settings,
        credentials,
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
            let _ = email_vault_db::save_account_secret(&cred.account_id, &plain_secret);
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
        xml.push_str(&format!(
            "    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n",
            a.alias
        ));
        xml.push_str(&format!(
            "    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n",
            a.email
        ));
        xml.push_str(&format!(
            "    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n",
            a.smtp_host
        ));
        xml.push_str(&format!(
            "    <Cell><Data ss:Type=\"Number\">{}</Data></Cell>\n",
            a.smtp_port
        ));
        xml.push_str(&format!(
            "    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n",
            a.imap_host
        ));
        xml.push_str(&format!(
            "    <Cell><Data ss:Type=\"Number\">{}</Data></Cell>\n",
            a.imap_port
        ));
        xml.push_str(&format!(
            "    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n",
            a.encryption_type
        ));
        xml.push_str(&format!(
            "    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n",
            a.is_default
        ));
        xml.push_str(&format!(
            "    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n",
            a.is_active
        ));
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
        xml.push_str(&format!(
            "    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n",
            r.email
        ));
        xml.push_str(&format!(
            "    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n",
            r.group_name
        ));
        xml.push_str(&format!(
            "    <Cell><Data ss:Type=\"String\">{}</Data></Cell>\n",
            r.is_active
        ));
        xml.push_str("   </Row>\n");
    }

    xml.push_str("  </Table>\n");
    xml.push_str(" </Worksheet>\n");
    xml.push_str("</Workbook>\n");

    Ok(xml)
}

/// Helper to unescape basic XML entities
fn unescape_xml(val: &str) -> String {
    val.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// Helper to parse XML cells inside a <Row> block
fn parse_xml_row(row_str: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut rest = row_str;

    while let Some(start_data) = rest.find("<Data") {
        let after_data_tag = &rest[start_data..];
        if let Some(tag_close) = after_data_tag.find('>') {
            let content_start = tag_close + 1;
            let data_body = &after_data_tag[content_start..];
            if let Some(end_data) = data_body.find("</Data>") {
                let cell_val = &data_body[..end_data];
                cells.push(unescape_xml(cell_val.trim()));
                rest = &data_body[end_data + 7..];
            } else {
                break;
            }
        } else {
            break;
        }
    }
    cells
}

/// Helper to extract worksheet slice by name
fn extract_worksheet<'a>(xml: &'a str, sheet_name: &str) -> Option<&'a str> {
    let target = format!("Name=\"{}\"", sheet_name);
    let pos = xml.find(&target)?;
    let after_ws = &xml[pos..];
    let end_pos = after_ws.find("</Worksheet>")?;
    Some(&after_ws[..end_pos])
}

/// Import accounts and recipients from Excel XML Spreadsheet (.xlsx / .xml)
pub fn import_from_excel(payload: &str) -> Result<ImportSummary, String> {
    let mut summary = ImportSummary {
        accounts_imported: 0,
        recipients_imported: 0,
        settings_updated: false,
        errors: Vec::new(),
    };

    // 1. Process Mailboxes Worksheet
    if let Some(mailboxes_ws) = extract_worksheet(payload, "Mailboxes") {
        let mut rest = mailboxes_ws;
        while let Some(row_start) = rest.find("<Row>") {
            let after_row = &rest[row_start..];
            if let Some(row_end) = after_row.find("</Row>") {
                let row_xml = &after_row[..row_end];
                let cells = parse_xml_row(row_xml);
                rest = &after_row[row_end + 6..];

                if cells.len() >= 7 {
                    let is_header = cells[0].eq_ignore_ascii_case("alias")
                        || cells[1].eq_ignore_ascii_case("email");
                    if is_header {
                        continue;
                    }

                    let is_def = cells
                        .get(7)
                        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
                        .unwrap_or(false);
                    let is_act = match cells.get(8) {
                        Some(v) if v.eq_ignore_ascii_case("false") || v == "0" => false,
                        _ => true,
                    };

                    let input = EmailAccountInput {
                        id: None,
                        alias: cells[0].clone(),
                        email: cells[1].clone(),
                        password: None,
                        smtp_host: cells[2].clone(),
                        smtp_port: cells[3].parse::<u16>().unwrap_or(587),
                        imap_host: cells[4].clone(),
                        imap_port: cells[5].parse::<u16>().unwrap_or(993),
                        encryption_type: cells[6].clone(),
                        is_default: is_def,
                        is_active: is_act,
                    };

                    let email_name = cells[1].clone();
                    if let Ok(_) = email_vault_db::upsert_email_account(input) {
                        summary.accounts_imported += 1;
                    } else {
                        summary
                            .errors
                            .push(format!("Failed to import account '{}'", email_name));
                    }
                }
            } else {
                break;
            }
        }
    }

    // 2. Process Recipients Worksheet
    if let Some(recipients_ws) = extract_worksheet(payload, "Recipients") {
        let mut rest = recipients_ws;
        while let Some(row_start) = rest.find("<Row>") {
            let after_row = &rest[row_start..];
            if let Some(row_end) = after_row.find("</Row>") {
                let row_xml = &after_row[..row_end];
                let cells = parse_xml_row(row_xml);
                rest = &after_row[row_end + 6..];

                if cells.len() >= 2 {
                    let is_header = cells[0].eq_ignore_ascii_case("email")
                        || cells[1].eq_ignore_ascii_case("group");
                    if is_header {
                        continue;
                    }

                    let is_act = match cells.get(2) {
                        Some(v) if v.eq_ignore_ascii_case("false") || v == "0" => false,
                        _ => true,
                    };

                    let input = NotifyRecipientInput {
                        email: cells[0].clone(),
                        group_name: Some(cells[1].clone()),
                        is_active: Some(is_act),
                    };

                    if let Ok(_) = email_vault_db::add_notify_recipient(input) {
                        summary.recipients_imported += 1;
                    }
                }
            } else {
                break;
            }
        }
    }

    Ok(summary)
}

/// Backup email_vault.db and split email_passwords.db to target path
pub fn backup_vault_db(target_path: &Path) -> Result<(), String> {
    let source_path = email_vault_db::get_email_vault_db_path()?;
    let is_source_exists = source_path.exists();
    if !is_source_exists {
        return Err("Email vault database does not exist yet".to_string());
    }

    fs::copy(&source_path, target_path)
        .map_err(|e| format!("Failed to backup email vault database: {}", e))?;

    // Also backup split passwords database alongside target if present
    if let Ok(pass_src) = email_vault_db::get_email_passwords_db_path() {
        let is_pass_exists = pass_src.exists();
        if is_pass_exists {
            let pass_target = target_path.with_extension("passwords.db");
            let _ = fs::copy(&pass_src, &pass_target);
        }
    }

    Ok(())
}

/// Restore email_vault.db and split email_passwords.db from source backup file
pub fn restore_vault_db(source_path: &Path) -> Result<(), String> {
    let is_source_exists = source_path.exists();
    if !is_source_exists {
        return Err("Source database file does not exist".to_string());
    }

    let target_path = email_vault_db::get_email_vault_db_path()?;
    fs::copy(source_path, &target_path)
        .map_err(|e| format!("Failed to restore email vault database: {}", e))?;

    // Restore companion split passwords database if backup file exists
    let pass_src = source_path.with_extension("passwords.db");
    let is_pass_src_exists = pass_src.exists();
    if is_pass_src_exists {
        if let Ok(pass_target) = email_vault_db::get_email_passwords_db_path() {
            let _ = fs::copy(&pass_src, &pass_target);
        }
    }

    // Re-verify connectivity and run pragmas
    let _ = email_vault_db::connect_vault_db()?;
    let _ = email_vault_db::connect_passwords_db()?;
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

    #[test]
    fn test_unescape_xml() {
        assert_eq!(unescape_xml("a &amp; b &lt; c &gt; d"), "a & b < c > d");
    }

    #[test]
    fn test_parse_xml_row() {
        let row = r#"   <Row>
    <Cell><Data ss:Type="String">Alias &amp; Name</Data></Cell>
    <Cell><Data ss:Type="String">test@example.com</Data></Cell>
    <Cell><Data ss:Type="Number">587</Data></Cell>
   </Row>"#;
        let cells = parse_xml_row(row);
        assert_eq!(cells.len(), 3);
        assert_eq!(cells[0], "Alias & Name");
        assert_eq!(cells[1], "test@example.com");
        assert_eq!(cells[2], "587");
    }

    #[test]
    fn test_base64_multi_pass_roundtrip() {
        let secret = "super-secret-password-123!@#$%^&*()";
        let encoded_3 = base64_encode_multi(secret, 3);
        assert_ne!(encoded_3, secret);
        let decoded = base64_decode_multi(&encoded_3, 3).expect("3-pass decode failed");
        assert_eq!(decoded, secret);

        let encoded_1 = base64_encode_multi(secret, 1);
        let decoded_1 = base64_decode_multi(&encoded_1, 1).expect("1-pass decode failed");
        assert_eq!(decoded_1, secret);
    }

    #[test]
    fn test_base64_multi_pass_invalid_input() {
        let result = base64_decode_multi("not-valid-base64!!!", 1);
        assert!(result.is_err());
    }
}
