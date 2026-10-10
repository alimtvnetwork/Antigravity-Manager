#![cfg(test)]

use super::*;

use super::*;

#[test]
pub(crate) fn test_debounce_rate_limiter() {
    let now = 1000;
    let sender = "test@example.com";
    let cmd = "status";
    let target = "VM3";

    assert!(check_debounce_rate_limit(sender, cmd, target, now));
    assert!(check_debounce_rate_limit(sender, cmd, target, now + 2));
    // 3rd call in window: throttled
    assert!(!check_debounce_rate_limit(sender, cmd, target, now + 4));
    // After 10s: resets
    assert!(check_debounce_rate_limit(sender, cmd, target, now + 11));
}

#[test]
pub(crate) fn test_parse_vm3_instance_help_command() {
    // User's exact subject: "VM3 | 1 | help"
    let action = parse_email_command("VM3 | 1 | help", "");
    assert_eq!(
        action,
        InboundAction::HelpRequest {
            target: "VM3".to_string(),
            instance_id: Some("1".to_string()),
        }
    );

    // With Re: prefix: "Re: VM3 | 1 | help"
    let action_re = parse_email_command("Re: VM3 | 1 | help", "");
    assert_eq!(
        action_re,
        InboundAction::HelpRequest {
            target: "VM3".to_string(),
            instance_id: Some("1".to_string()),
        }
    );

    // Without instance part: "VM3 | help"
    let action_simple = parse_email_command("VM3 | help", "");
    assert_eq!(
        action_simple,
        InboundAction::HelpRequest {
            target: "VM3".to_string(),
            instance_id: None,
        }
    );
}

#[test]
pub(crate) fn test_rfc2047_decoding() {
    let raw = "=?UTF-8?B?Vk0zIHwgMSB8IGhlbHA=?=";
    assert_eq!(decode_rfc2047(raw), "VM3 | 1 | help");

    let plain = "VM3 | 1 | help";
    assert_eq!(decode_rfc2047(plain), "VM3 | 1 | help");
}

#[test]
pub(crate) fn test_extract_clean_reply_body_strips_quotes() {
    let gmail_reply = "VM3 | 1 | help\r\n\r\nOn Thu, Sep 24, 2026 at 7:45 PM ai-agm-tool-v1 <...> wrote:\r\n> ================================================================================\r\n> [AGM Help] Inbound Remote Mailbox Instructions Cheat Sheet\r\n> ...";
    let (first_cmd, remaining) = extract_clean_reply_body(gmail_reply);
    assert_eq!(first_cmd, "VM3 | 1 | help");
    assert_eq!(remaining, "");

    let outlook_reply = "status\r\n\r\n-----Original Message-----\r\nFrom: ai-agm-tool-v1\r\nSent: Thursday, September 24, 2026\r\nTo: user\r\nSubject: [AGM Help]";
    let (first_cmd2, remaining2) = extract_clean_reply_body(outlook_reply);
    assert_eq!(first_cmd2, "status");
    assert_eq!(remaining2, "");
}

#[test]
pub(crate) fn test_parse_reply_with_body_command() {
    // User clicks "Reply" to Cheat Sheet in Gmail:
    // Subject: "Re: [AGM Help] Inbound Remote Mailbox Instructions Cheat Sheet"
    // Body: "VM3 | 1 | help\n\nOn Thu, Sep 24, 2026... wrote:\n> ..."
    let action = parse_email_command(
            "Re: [AGM Help] Inbound Remote Mailbox Instructions Cheat Sheet",
            "VM3 | 1 | help\r\n\r\nOn Thu, Sep 24, 2026 at 7:45 PM ai-agm-tool-v1 <...> wrote:\r\n> [AGM Help] ...",
        );
    assert_eq!(
        action,
        InboundAction::HelpRequest {
            target: "VM3".to_string(),
            instance_id: Some("1".to_string()),
        }
    );

    // User clicks "Reply" and types just "status" in body:
    let action_status = parse_email_command(
        "Re: [AGM Help] Inbound Remote Mailbox Instructions Cheat Sheet",
        "status\r\n\r\n> [AGM Help] ...",
    );
    assert_eq!(
        action_status,
        InboundAction::StatusQuery {
            target: "*".to_string(),
        }
    );

    // User clicks "Reply" and types "accounts" in body:
    let action_acc = parse_email_command(
        "Re: [AGM Result] SUCCESS: help",
        "accounts\r\n\r\nOn Wed, Sep 24... wrote:\n> ...",
    );
    assert_eq!(
        action_acc,
        InboundAction::ListAccounts {
            target: "*".to_string(),
        }
    );
}

#[test]
pub(crate) fn test_single_word_commands() {
    assert_eq!(
        parse_email_command("help", ""),
        InboundAction::HelpRequest {
            target: "*".to_string(),
            instance_id: None,
        }
    );
    assert_eq!(
        parse_email_command("status", ""),
        InboundAction::StatusQuery {
            target: "*".to_string(),
        }
    );
    assert_eq!(
        parse_email_command("accounts", ""),
        InboundAction::ListAccounts {
            target: "*".to_string(),
        }
    );
    assert_eq!(
        parse_email_command("instances", ""),
        InboundAction::ListInstances {
            target: "*".to_string(),
        }
    );
    assert_eq!(
        parse_email_command("doctor", ""),
        InboundAction::DoctorDiagnostic {
            target: "*".to_string(),
        }
    );
    assert_eq!(
        parse_email_command("rotate", ""),
        InboundAction::AccountRotate {
            target: "*".to_string(),
            instance_id: None,
        }
    );
    assert_eq!(
        parse_email_command("agm rotate", ""),
        InboundAction::AccountRotate {
            target: "*".to_string(),
            instance_id: None,
        }
    );
}

#[test]
pub(crate) fn test_format_reply_subject_preserves_threading() {
    let local_name = "VM3";
    let local_ip = "192.168.1.12";
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let node_tag = format!("[AGM {} | {} | {}]", pkg_ver, local_name, local_ip);

    // Direct subject: should prepend node tag and Re: (with VM3 | stripped to prevent stuttering)
    assert_eq!(
        format_reply_subject(
            Some("VM3 | 1 | help"),
            "[AGM ACK]",
            "help",
            local_name,
            local_ip
        ),
        format!("{} Re: 1 | help", node_tag)
    );

    // Subject already has Re: should NOT duplicate Re:, and should strip VM3 |
    assert_eq!(
        format_reply_subject(
            Some("Re: VM3 | 1 | help"),
            "[AGM ACK]",
            "help",
            local_name,
            local_ip
        ),
        format!("{} Re: 1 | help", node_tag)
    );

    // Replying to prompt task: should strip VM3 |
    assert_eq!(
        format_reply_subject(
            Some("VM3 | prompt | proj-Antigravity-Manager"),
            "[AGM ACK]",
            "prompt",
            local_name,
            local_ip
        ),
        format!("{} Re: prompt | proj-Antigravity-Manager", node_tag)
    );

    // Replying to existing notification:
    assert_eq!(
        format_reply_subject(
            Some("Re: [AGM Help] Inbound Remote Mailbox Instructions Cheat Sheet"),
            "[AGM ACK]",
            "help",
            local_name,
            local_ip
        ),
        format!(
            "{} Re: Inbound Remote Mailbox Instructions Cheat Sheet",
            node_tag
        )
    );

    // Fallback when no original subject:
    assert_eq!(
        format_reply_subject(None, "[AGM ACK] Running:", "help", local_name, local_ip),
        format!("{} [AGM ACK] Running: help", node_tag)
    );
}

#[test]
pub(crate) fn test_parse_flexible_pipe_spacing_and_instances() {
    // 0 spaces
    let zero_space_help = parse_email_command("VM3|1|help", "");
    assert_eq!(
        zero_space_help,
        InboundAction::HelpRequest {
            target: "VM3".to_string(),
            instance_id: Some("1".to_string()),
        }
    );

    let zero_space_switch = parse_email_command("VM3|1|switch|user@gmail.com", "");
    assert_eq!(
        zero_space_switch,
        InboundAction::AccountSwitch {
            target: "VM3".to_string(),
            email_query: "user@gmail.com".to_string(),
            instance_id: Some("1".to_string()),
        }
    );

    // 1 space
    let one_space_switch = parse_email_command("VM3 | 1 | switch | user@gmail.com", "");
    assert_eq!(
        one_space_switch,
        InboundAction::AccountSwitch {
            target: "VM3".to_string(),
            email_query: "user@gmail.com".to_string(),
            instance_id: Some("1".to_string()),
        }
    );

    // Multiple spaces
    let multi_space_help = parse_email_command("VM3   |   1   |   help", "");
    assert_eq!(
        multi_space_help,
        InboundAction::HelpRequest {
            target: "VM3".to_string(),
            instance_id: Some("1".to_string()),
        }
    );

    let multi_space_switch =
        parse_email_command("VM3   |   1   |   switch   |   user@gmail.com", "");
    assert_eq!(
        multi_space_switch,
        InboundAction::AccountSwitch {
            target: "VM3".to_string(),
            email_query: "user@gmail.com".to_string(),
            instance_id: Some("1".to_string()),
        }
    );

    // Fast forward and rotate on specific instance
    let ff_inst = parse_email_command("VM3 | 2 | ff", "");
    assert_eq!(
        ff_inst,
        InboundAction::FastForward {
            target_node: "VM3".to_string(),
            instance_id: Some("2".to_string()),
        }
    );

    let rotate_inst = parse_email_command("VM3 | 2 | rotate", "");
    assert_eq!(
        rotate_inst,
        InboundAction::AccountRotate {
            target: "VM3".to_string(),
            instance_id: Some("2".to_string()),
        }
    );
}
