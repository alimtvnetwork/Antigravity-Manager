use crate::modules::email_vault_db::{self, EmailAccount};
use base64::prelude::*;
use std::io::{Read, Write};

use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    pub(crate) fn test_template_rendering() {
        let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
        let (subj, text) =
            render_quota_drop_email("test@example.com", 12.5, 15, "my-pc", "192.168.1.50");
        assert!(subj.contains("12.5%"));
        assert!(subj.starts_with(&format!("[AGM {} | my-pc | 192.168.1.50]", pkg_ver)));
        assert!(!subj.contains("] [AGM]"));
        assert!(!subj.contains("] [Antigravity]"));
        assert!(text.contains("192.168.1.50"));
        assert!(text.contains("<html"));
        assert!(text.contains("<div"));
        assert!(text.contains("Ubuntu"));
    }

    #[test]
    pub(crate) fn test_format_subject_with_telemetry() {
        let ver = "v4.71.4";
        let node = "VM3";
        let ip = "192.168.1.12";

        // Plain subject without duplicate [Antigravity]
        let s1 = format_subject_with_telemetry("[Antigravity] Account Switched", ver, node, ip);
        assert_eq!(s1, "[AGM v4.71.4 | VM3 | 192.168.1.12] Account Switched");

        // Old tag upgrade
        let s2 = format_subject_with_telemetry("[VM3 | 192.168.1.12] Alert", ver, node, ip);
        assert_eq!(s2, "[AGM v4.71.4 | VM3 | 192.168.1.12] Alert");

        // Reply subject with old tag
        let s3 = format_subject_with_telemetry("Re: [VM3 | 192.168.1.12] Result", ver, node, ip);
        assert_eq!(s3, "[AGM v4.71.4 | VM3 | 192.168.1.12] Re: Result");

        // Reply subject without tag
        let s4 = format_subject_with_telemetry("Re: help", ver, node, ip);
        assert_eq!(s4, "[AGM v4.71.4 | VM3 | 192.168.1.12] Re: help");

        // Already tagged cleanly
        let s5 = format_subject_with_telemetry(
            "[AGM v4.71.4 | VM3 | 192.168.1.12] Status",
            ver,
            node,
            ip,
        );
        assert_eq!(s5, "[AGM v4.71.4 | VM3 | 192.168.1.12] Status");

        // User request sample upgrade: strips redundant [Antigravity] before [JSON]
        let s6 = format_subject_with_telemetry(
            "[Antigravity | v4.75.0 | VM3 | 192.168.1.12] [Antigravity] [JSON] Node & Credits Status",
            ver,
            node,
            ip,
        );
        assert_eq!(
            s6,
            "[AGM v4.71.4 | VM3 | 192.168.1.12] [JSON] Node & Credits Status"
        );

        // Other subject with redundant tag
        let s7 = format_subject_with_telemetry(
            "[v4.75.0 | VM3 | 192.168.1.12] [Antigravity] Write the subject",
            ver,
            node,
            ip,
        );
        assert_eq!(s7, "[AGM v4.71.4 | VM3 | 192.168.1.12] Write the subject");

        // Deduplicate node alias when subject has "W2 | prompt | proj-..."
        let s8 =
            format_subject_with_telemetry("VM3 | prompt | proj-Antigravity-Manager", ver, node, ip);
        assert_eq!(
            s8,
            "[AGM v4.71.4 | VM3 | 192.168.1.12] prompt | proj-Antigravity-Manager"
        );

        // Deduplicate wildcard prefix "* | prompt | ..."
        let s9 =
            format_subject_with_telemetry("* | prompt | proj-Antigravity-Manager", ver, node, ip);
        assert_eq!(
            s9,
            "[AGM v4.71.4 | VM3 | 192.168.1.12] prompt | proj-Antigravity-Manager"
        );

        // Subject with old telemetry tag and duplicate node alias
        let s10 = format_subject_with_telemetry(
            "[v4.75.0 | VM3 | 192.168.1.7] VM3 | prompt | proj-Antigravity-Manager",
            ver,
            node,
            ip,
        );
        assert_eq!(
            s10,
            "[AGM v4.71.4 | VM3 | 192.168.1.12] prompt | proj-Antigravity-Manager"
        );
    }

    #[test]
    pub(crate) fn test_build_mime_message_json_plain_no_html() {
        let account = EmailAccount {
            id: "acc-1".to_string(),
            alias: "Test Node".to_string(),
            email: "test@example.com".to_string(),
            smtp_host: "smtp.example.com".to_string(),
            smtp_port: 587,
            imap_host: "imap.example.com".to_string(),
            imap_port: 993,
            encryption_type: "TLS".to_string(),
            is_default: true,
            is_active: true,
            created_at: 0,
            updated_at: 0,
        };
        let json_body = r#"{"event":"node_and_credits_status","immediate_credits":85.5}"#;
        let mime = build_mime_message(
            &account,
            "[JSON] Node & Credits Status",
            json_body,
            &["user@example.com".to_string()],
            None,
        );

        assert!(mime.contains("Content-Type: text/plain; charset=UTF-8"));
        assert!(!mime.contains("Content-Type: text/html"));
        assert!(!mime.contains("multipart/alternative"));
        assert!(mime.contains("Subject: [AGM "));
        assert!(mime.contains("[JSON] Node & Credits Status"));
    }

    #[test]
    pub(crate) fn test_build_mime_message_html_multipart_ubuntu() {
        let account = EmailAccount {
            id: "acc-1".to_string(),
            alias: "Test Node".to_string(),
            email: "test@example.com".to_string(),
            smtp_host: "smtp.example.com".to_string(),
            smtp_port: 587,
            imap_host: "imap.example.com".to_string(),
            imap_port: 993,
            encryption_type: "TLS".to_string(),
            is_default: true,
            is_active: true,
            created_at: 0,
            updated_at: 0,
        };
        let text_body =
            "Active Account: default@example.com\nImmediate Credits: 95.0%\nWeekly Credits: 98.0%";
        let mime = build_mime_message(
            &account,
            "Node & Credits Status",
            text_body,
            &["user@example.com".to_string()],
            None,
        );

        assert!(mime.contains("Content-Type: multipart/alternative; boundary="));
        assert!(mime.contains("Content-Type: text/plain; charset=UTF-8"));
        assert!(mime.contains("Content-Type: text/html; charset=UTF-8"));
        assert!(mime.contains("Content-Transfer-Encoding: base64"));
        assert!(!mime.contains("[JSON]"));
    }
}
