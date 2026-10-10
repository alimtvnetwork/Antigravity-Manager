use base64::prelude::*;

use super::*;

/// Extract clean user content from email body, discarding quoted reply text and signatures
pub fn extract_clean_reply_body(body: &str) -> (String, String) {
    let mut clean_lines: Vec<&str> = Vec::new();

    for line in body.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();

        // Detect quotation block boundaries common in Gmail, Outlook, Apple Mail, Thunderbird
        if lower.starts_with('>')
            || (lower.starts_with("on ") && (lower.ends_with("wrote:") || lower.contains("wrote:")))
            || lower.starts_with("-----original message-----")
            || lower.starts_with("--- original message ---")
            || lower.starts_with("________________________________")
            || (lower.starts_with("from:") && !clean_lines.is_empty())
            || (lower.starts_with("sent:") && !clean_lines.is_empty())
        {
            break;
        }

        clean_lines.push(trimmed);
    }

    let mut first_cmd = String::new();
    let mut remaining = Vec::new();
    let mut found_first = false;

    for line in clean_lines {
        if !found_first {
            if !line.is_empty() {
                first_cmd = line.to_string();
                found_first = true;
            }
        } else {
            remaining.push(line);
        }
    }

    (first_cmd, remaining.join("\n").trim().to_string())
}

/// Parse subject and body into strongly typed InboundAction with bidirectional reply support
pub fn parse_email_command(subject: &str, body: &str) -> InboundAction {
    let lower_subj_raw = subject.trim().to_lowercase();
    let is_reply_or_fwd = lower_subj_raw.starts_with("re:")
        || lower_subj_raw.starts_with("re :")
        || lower_subj_raw.starts_with("fwd:")
        || lower_subj_raw.starts_with("fw:");

    // 1. If this is an email reply, user's command is usually typed in the body
    if is_reply_or_fwd {
        let (first_cmd, remaining_body) = extract_clean_reply_body(body);
        if !first_cmd.is_empty() {
            let body_action = parse_single_command_string(&first_cmd, &remaining_body);
            if !matches!(body_action, InboundAction::Ignored { .. }) {
                return body_action;
            }
        }
    }

    // 2. Try parsing the subject line
    let subject_action = parse_single_command_string(subject, body);
    if !matches!(subject_action, InboundAction::Ignored { .. }) {
        return subject_action;
    }

    // 3. Fallback: Check the body even if subject wasn't explicitly marked as reply
    if !is_reply_or_fwd {
        let (first_cmd, remaining_body) = extract_clean_reply_body(body);
        if !first_cmd.is_empty() {
            let body_action = parse_single_command_string(&first_cmd, &remaining_body);
            if !matches!(body_action, InboundAction::Ignored { .. }) {
                return body_action;
            }
        }
    }

    subject_action
}

/// Format an RFC 5322 reply subject that strictly preserves threading in Gmail / Outlook
/// while prepending [v<VERSION> | <VM_ALIAS> | <LOCAL_IP>] to identify the originating VM node clearly.
pub fn format_reply_subject(
    original_subject: Option<&str>,
    fallback_prefix: &str,
    command_name: &str,
    local_name: &str,
    local_ip: &str,
) -> String {
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let node_tag = format!("[AGM {} | {} | {}]", pkg_ver, local_name, local_ip);
    if let Some(orig) = original_subject {
        let trimmed = orig.trim();
        if !trimmed.is_empty() {
            let clean_orig = strip_email_prefixes(trimmed);
            // If clean_orig starts with an existing bracketed tag with a pipe, strip it
            let mut final_orig = if clean_orig.starts_with('[') && clean_orig.contains(']') {
                if let Some(end_idx) = clean_orig.find(']') {
                    let inside = &clean_orig[1..end_idx];
                    if inside.contains('|') {
                        clean_orig[end_idx + 1..].trim()
                    } else {
                        clean_orig.as_str()
                    }
                } else {
                    clean_orig.as_str()
                }
            } else {
                clean_orig.as_str()
            };

            // Strip redundant leading node alias or wildcard prefix
            let m_prefix = format!("{} |", local_name.to_lowercase());
            let m_prefix_no_space = format!("{}|", local_name.to_lowercase());
            if final_orig.to_lowercase().starts_with(&m_prefix) {
                final_orig = final_orig[m_prefix.len()..].trim();
            } else if final_orig.to_lowercase().starts_with(&m_prefix_no_space) {
                final_orig = final_orig[m_prefix_no_space.len()..].trim();
            } else if final_orig.starts_with("* |") {
                final_orig = final_orig[3..].trim();
            } else if final_orig.starts_with("*|") {
                final_orig = final_orig[2..].trim();
            }

            if !final_orig.is_empty() {
                return format!("{} Re: {}", node_tag, final_orig);
            }
        }
    }
    format!("{} {} {}", node_tag, fallback_prefix, command_name)
}
