use crate::modules::email_vault_db::{
    self, EmailAccount, EmailAccountInput, EmailNotificationSettings, NotifyRecipient,
    NotifyRecipientInput,
};

use super::*;

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
pub(crate) fn unescape_xml(val: &str) -> String {
    val.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// Helper to parse XML cells inside a <Row> block
pub(crate) fn parse_xml_row(row_str: &str) -> Vec<String> {
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
pub(crate) fn extract_worksheet<'a>(xml: &'a str, sheet_name: &str) -> Option<&'a str> {
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
