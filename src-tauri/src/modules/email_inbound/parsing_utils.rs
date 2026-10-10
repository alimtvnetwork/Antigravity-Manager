use base64::prelude::*;

use super::*;

/// Clean subject by removing reply/forward/subject prefixes
pub fn strip_email_prefixes(subject: &str) -> String {
    let mut clean = subject.trim();
    loop {
        let lower = clean.to_lowercase();
        if lower.starts_with("re:") {
            clean = clean[3..].trim();
        } else if lower.starts_with("re :") {
            clean = clean[4..].trim();
        } else if lower.starts_with("fwd:") {
            clean = clean[4..].trim();
        } else if lower.starts_with("fwd :") {
            clean = clean[5..].trim();
        } else if lower.starts_with("fw:") {
            clean = clean[3..].trim();
        } else if lower.starts_with("fw :") {
            clean = clean[4..].trim();
        } else if lower.starts_with("sub:") {
            clean = clean[4..].trim();
        } else if lower.starts_with("subject:") {
            clean = clean[8..].trim();
        } else if lower.starts_with("[agm]:") {
            clean = clean[6..].trim();
        } else if lower.starts_with("[agm]") {
            clean = clean[5..].trim();
        } else if lower.starts_with("[agm-task]") {
            clean = clean[10..].trim();
        } else if lower.starts_with("[agm ack]") {
            clean = clean[9..].trim();
        } else if lower.starts_with("[agm result]") {
            clean = clean[12..].trim();
        } else if lower.starts_with("[agm alert]") {
            clean = clean[11..].trim();
        } else if lower.starts_with("[agm help]:") {
            clean = clean[11..].trim();
        } else if lower.starts_with("[agm help]") {
            clean = clean[10..].trim();
        } else if lower.starts_with("[agm execution report]") {
            clean = clean[22..].trim();
        } else if lower.starts_with("[agm test ping]") {
            clean = clean[15..].trim();
        } else if lower.starts_with("[agm status]") {
            clean = clean[12..].trim();
        } else if clean.starts_with('[') {
            if let Some(end) = clean.find(']') {
                let inside = &clean[1..end];
                let lower_inside = inside.to_lowercase();
                if lower_inside.starts_with("agm")
                    || lower_inside.contains('|')
                    || lower_inside.contains('.')
                    || lower_inside.starts_with("antigravity")
                    || lower_inside.starts_with("vm")
                {
                    clean = clean[end + 1..].trim();
                    continue;
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Support responses to "[Node: AI-MAIN] [Type: prompt]"
    let lower_clean = clean.to_lowercase();
    if lower_clean.starts_with("[node:") {
        if let Some(node_end) = clean.find(']') {
            let node_name = clean["[node:".len()..node_end].trim();
            let after_node = clean[node_end + 1..].trim();
            let lower_after = after_node.to_lowercase();
            if lower_after.starts_with("[type:") {
                if let Some(type_end) = after_node.find(']') {
                    let task_type = after_node["[type:".len()..type_end].trim();
                    return format!("{} | {}", node_name, task_type);
                }
            }
        }
    }

    clean.to_string()
}

/// Check if a pipe segment is an instance specifier (e.g. "1", "#1", "ins-1", "default")
pub fn is_instance_specifier(part: &str) -> bool {
    let lower = part.trim().to_lowercase();
    if lower.starts_with("ins-") || lower.starts_with("instance-") || lower.starts_with("instance:")
    {
        return true;
    }
    if lower.starts_with('#') && lower[1..].chars().all(|c| c.is_ascii_digit()) && lower.len() > 1 {
        return true;
    }
    if !lower.is_empty() && lower.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    if lower == "default" || lower == "active" {
        return true;
    }
    false
}

/// Extract clean instance name/number from instance specifier
pub fn extract_clean_instance_id(part: &str) -> String {
    let trimmed = part.trim();
    let lower = trimmed.to_lowercase();
    if lower.starts_with("ins-") {
        trimmed["ins-".len()..].trim().to_string()
    } else if lower.starts_with("instance-") {
        trimmed["instance-".len()..].trim().to_string()
    } else if lower.starts_with("instance:") {
        trimmed["instance:".len()..].trim().to_string()
    } else if lower.starts_with('#') {
        trimmed["#".len()..].trim().to_string()
    } else {
        trimmed.to_string()
    }
}

/// Check if a pipe segment represents a known AGM command
pub fn is_known_command(cmd: &str) -> bool {
    let lower = cmd.trim().to_lowercase();
    matches!(
        lower.as_str(),
        "help"
            | "status"
            | "agm status"
            | "cmd"
            | "update"
            | "agm update"
            | "gitmap update"
            | "ls"
            | "agm ls"
            | "instances"
            | "agm instances"
            | "ff"
            | "agm ff"
            | "smart-switch"
            | "agm smart-switch"
            | "agm ff/smart-switch"
            | "prompts"
            | "agm prompts"
            | "agm prompts ls"
            | "agy prompts ls"
            | "gitmap prompts ls"
            | "doctor"
            | "agm doctor"
            | "check"
            | "agm check"
            | "accounts"
            | "agm accounts"
            | "acc"
            | "agm acc"
            | "rotate"
            | "agm rotate"
            | "proxy"
            | "agm proxy"
            | "agm proxy status"
            | "proxy test"
            | "agm proxy test"
            | "clean"
            | "agm clean"
            | "purge"
            | "agm purge"
            | "sync"
            | "agm sync"
            | "prompt"
    ) || lower.starts_with("gitmap")
        || lower.starts_with("switch")
        || lower.starts_with("agm switch")
}

/// Decode RFC 2047 encoded words in email headers (e.g. =?UTF-8?B?...?= or =?UTF-8?Q?...?=)
pub fn decode_rfc2047(input: &str) -> String {
    if !input.contains("=?") || !input.contains("?=") {
        return input.to_string();
    }
    let mut result = String::new();
    let mut remaining = input;
    while let Some(start) = remaining.find("=?") {
        result.push_str(&remaining[..start]);
        let after_start = &remaining[start + 2..];
        if let Some(end) = after_start.find("?=") {
            let token = &after_start[..end];
            let parts: Vec<&str> = token.split('?').collect();
            if parts.len() == 3 {
                let enc = parts[1].to_uppercase();
                let payload = parts[2];
                let decoded_opt = if enc == "B" {
                    BASE64_STANDARD
                        .decode(payload.trim())
                        .ok()
                        .and_then(|bytes| String::from_utf8(bytes).ok())
                } else if enc == "Q" {
                    let mut bytes = Vec::new();
                    let mut chars = payload.chars().peekable();
                    while let Some(c) = chars.next() {
                        if c == '_' {
                            bytes.push(b' ');
                        } else if c == '=' {
                            let hex1 = chars.next();
                            let hex2 = chars.next();
                            if let (Some(h1), Some(h2)) = (hex1, hex2) {
                                let hex_str = format!("{}{}", h1, h2);
                                if let Ok(b) = u8::from_str_radix(&hex_str, 16) {
                                    bytes.push(b);
                                }
                            }
                        } else {
                            bytes.push(c as u8);
                        }
                    }
                    String::from_utf8(bytes).ok()
                } else {
                    None
                };

                if let Some(decoded) = decoded_opt {
                    result.push_str(&decoded);
                } else {
                    result.push_str(&format!("=?{}?=", token));
                }
            } else {
                result.push_str(&format!("=?{}?=", token));
            }
            remaining = &after_start[end + 2..];
        } else {
            result.push_str("=?");
            remaining = after_start;
        }
    }
    result.push_str(remaining);
    result
}

/// Check if target matches local machine name, IP, partial octet, or wildcard
pub fn matches_target_node_or_ip(target: &str, local_ip: &str, local_name: &str) -> bool {
    let t = target.trim();
    if t.is_empty() || t == "*" || t.eq_ignore_ascii_case("all") || t.eq_ignore_ascii_case("any") {
        return true;
    }
    if t.eq_ignore_ascii_case("local") || t.eq_ignore_ascii_case("localhost") || t == "127.0.0.1" {
        return true;
    }
    if t.eq_ignore_ascii_case(local_name) {
        return true;
    }
    if t == local_ip {
        return true;
    }

    // IP Octet match: e.g. "12" matches "192.168.1.12"
    let ip_octets: Vec<&str> = local_ip.split('.').collect();
    if ip_octets.contains(&t) {
        return true;
    }
    if local_ip.ends_with(&format!(".{}", t)) {
        return true;
    }

    // Host environment variables check
    if let Ok(comp) = std::env::var("COMPUTERNAME") {
        if t.eq_ignore_ascii_case(&comp) {
            return true;
        }
    }
    if let Ok(host) = std::env::var("HOSTNAME") {
        if t.eq_ignore_ascii_case(&host) {
            return true;
        }
    }

    false
}

/// Extract prompt name and prompt instruction from email body (stripping any footer after ---)
pub fn parse_prompt_body(body: &str) -> (String, String) {
    let body_before_footer = body.split("\n---").next().unwrap_or(body).trim();
    let mut prompt_name = String::new();
    let mut prompt_instruction = String::new();
    let mut in_instruction = false;

    for line in body_before_footer.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();
        if lower.starts_with("prompt-name:") {
            prompt_name = trimmed["prompt-name:".len()..].trim().to_string();
            in_instruction = false;
        } else if lower.starts_with("prompt instruction:") {
            prompt_instruction = trimmed["prompt instruction:".len()..].trim().to_string();
            in_instruction = true;
        } else if in_instruction {
            if !prompt_instruction.is_empty() {
                prompt_instruction.push('\n');
            }
            prompt_instruction.push_str(line);
        }
    }

    if prompt_instruction.is_empty() && prompt_name.is_empty() {
        prompt_instruction = body_before_footer.to_string();
    }

    (prompt_name, prompt_instruction)
}
