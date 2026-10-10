use base64::prelude::*;
use chrono::Utc;

use super::*;

/// Detect if content contains HTML markup
pub(crate) fn is_html_content(content: &str) -> bool {
    let lower = content.trim().to_lowercase();
    lower.contains("<html")
        || lower.contains("<!doctype")
        || lower.contains("<div")
        || lower.contains("<table")
        || lower.contains("<body")
        || lower.contains("<span style=")
}

/// Strip HTML tags and <style>/<script> contents for clean RFC 2046 plaintext fallback
pub(crate) fn strip_html_tags(html: &str) -> String {
    // 1. Strip entire <style>...</style> and <script>...</script> blocks
    let mut cleaned = String::with_capacity(html.len());
    let mut remaining = html;
    while let Some(style_start) = remaining.to_lowercase().find("<style") {
        cleaned.push_str(&remaining[..style_start]);
        let after_start = &remaining[style_start..];
        if let Some(style_end) = after_start.to_lowercase().find("</style>") {
            remaining = &after_start[style_end + "</style>".len()..];
        } else {
            remaining = "";
            break;
        }
    }
    cleaned.push_str(remaining);

    let mut no_script = String::with_capacity(cleaned.len());
    let mut remaining_script = cleaned.as_str();
    while let Some(sc_start) = remaining_script.to_lowercase().find("<script") {
        no_script.push_str(&remaining_script[..sc_start]);
        let after_start = &remaining_script[sc_start..];
        if let Some(sc_end) = after_start.to_lowercase().find("</script>") {
            remaining_script = &after_start[sc_end + "</script>".len()..];
        } else {
            remaining_script = "";
            break;
        }
    }
    no_script.push_str(remaining_script);

    let mut out = String::with_capacity(no_script.len());
    let mut in_tag = false;
    for c in no_script.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            out.push(c);
        }
    }
    let lines: Vec<&str> = out
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    lines.join("\r\n")
}

/// Encode string as RFC 2045 76-character CRLF-wrapped base64 payload
pub(crate) fn encode_mime_base64_body(input: &str) -> String {
    let b64 = BASE64_STANDARD.encode(input.as_bytes());
    let mut wrapped = String::with_capacity(b64.len() + (b64.len() / 76) * 2 + 4);
    for (idx, ch) in b64.chars().enumerate() {
        if idx > 0 && idx % 76 == 0 {
            wrapped.push_str("\r\n");
        }
        wrapped.push(ch);
    }
    wrapped
}

/// Escape basic HTML entities for safe inclusion in <pre> or table cells
pub(crate) fn escape_html_entities(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Format an RFC 5322 subject with standard telemetry prefix: `[AGM v<VERSION> | <VM_ALIAS> | <LOCAL_IP>]`
/// Replaces any obsolete or unversioned prefix like `[Antigravity | ...]`, `[VM | IP]`, `[vX | VM | IP]`, or duplicate `[AGM]`,
/// preserves `Re:` prefix, and handles arbitrary whitespace around pipes.
pub fn format_subject_with_telemetry(
    subject: &str,
    pkg_ver: &str,
    machine_name: &str,
    machine_ip: &str,
) -> String {
    let ver_tag = if pkg_ver.starts_with('v') || pkg_ver.starts_with('V') {
        pkg_ver.to_string()
    } else {
        format!("v{}", pkg_ver)
    };
    let prefix = format!("[AGM {} | {} | {}]", ver_tag, machine_name, machine_ip);
    let mut trimmed = subject.trim();

    // Strip any leading telemetry tag like [AGM v... | ...], [Antigravity | ...], [v4.75.0 | ...], or [VM | IP]
    if trimmed.starts_with('[') {
        if let Some(end_idx) = trimmed.find(']') {
            let inside = &trimmed[1..end_idx];
            if inside.contains('|') {
                trimmed = trimmed[end_idx + 1..].trim();
            }
        }
    }

    let is_reply = trimmed.to_lowercase().starts_with("re:");
    if is_reply {
        trimmed = trimmed[3..].trim();
        // Check if reply had a secondary telemetry tag after Re:
        if trimmed.starts_with('[') {
            if let Some(end_idx) = trimmed.find(']') {
                let inside = &trimmed[1..end_idx];
                if inside.contains('|') {
                    trimmed = trimmed[end_idx + 1..].trim();
                }
            }
        }
    }

    // Strip leading [Antigravity] or [AGM] if present (case-insensitive)
    while trimmed.to_lowercase().starts_with("[antigravity]") {
        trimmed = trimmed["[antigravity]".len()..].trim();
    }
    while trimmed.to_lowercase().starts_with("[agm]") {
        trimmed = trimmed["[agm]".len()..].trim();
    }

    // Strip redundant leading node alias or wildcard prefix like "W2 |", "w2 |", or "* |"
    let m_prefix = format!("{} |", machine_name.to_lowercase());
    let m_prefix_no_space = format!("{}|", machine_name.to_lowercase());
    if trimmed.to_lowercase().starts_with(&m_prefix) {
        trimmed = trimmed[m_prefix.len()..].trim();
    } else if trimmed.to_lowercase().starts_with(&m_prefix_no_space) {
        trimmed = trimmed[m_prefix_no_space.len()..].trim();
    } else if trimmed.starts_with("* |") {
        trimmed = trimmed[3..].trim();
    } else if trimmed.starts_with("*|") {
        trimmed = trimmed[2..].trim();
    }

    if is_reply {
        if trimmed.is_empty() {
            format!("{} Re:", prefix)
        } else {
            format!("{} Re: {}", prefix, trimmed)
        }
    } else if trimmed.is_empty() {
        prefix
    } else {
        format!("{} {}", prefix, trimmed)
    }
}

/// Helper to extract clean pretty-printed JSON from raw text or strip HTML wrappers
pub fn extract_clean_json_body(raw: &str) -> String {
    let trimmed = raw.trim();

    // 1. Check if the string directly parses as a JSON Value
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
        return serde_json::to_string_pretty(&val).unwrap_or_else(|_| trimmed.to_string());
    }

    // 2. Check if there is a JSON block inside <pre>...</pre> or <code>...</code>
    if let (Some(pre_start), Some(pre_end)) = (trimmed.find("<pre"), trimmed.rfind("</pre>")) {
        if let Some(tag_end) = trimmed[pre_start..].find('>') {
            let inner = &trimmed[pre_start + tag_end + 1..pre_end];
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(inner.trim()) {
                return serde_json::to_string_pretty(&val).unwrap_or_else(|_| inner.to_string());
            }
        }
    }

    // 3. Search for balanced JSON object `{ ... }`
    if let (Some(start), Some(end)) = (trimmed.find('{'), trimmed.rfind('}')) {
        if end > start {
            let candidate = &trimmed[start..=end];
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(candidate) {
                return serde_json::to_string_pretty(&val)
                    .unwrap_or_else(|_| candidate.to_string());
            }
        }
    }

    // 4. Search for balanced JSON array `[ ... ]`
    if let (Some(start), Some(end)) = (trimmed.find('['), trimmed.rfind(']')) {
        if end > start {
            let candidate = &trimmed[start..=end];
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(candidate) {
                return serde_json::to_string_pretty(&val)
                    .unwrap_or_else(|_| candidate.to_string());
            }
        }
    }

    // 5. Fallback: sanitize plain text into a valid JSON envelope so email with [JSON] header NEVER has HTML/CSS
    let clean_text = strip_html_tags(trimmed);
    let fallback_json = serde_json::json!({
        "message": clean_text
    });
    serde_json::to_string_pretty(&fallback_json).unwrap_or_else(|_| "{}".to_string())
}

/// Helper to convert key-value lines into rich table rows with Ubuntu styling and better coloring
pub(crate) fn format_content_as_table_rows(content: &str) -> (bool, String) {
    let lines: Vec<&str> = content.lines().collect();
    let mut table_rows = String::new();
    let mut non_kv_lines = Vec::new();
    let mut kv_count = 0;

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check if line is Key: Value
        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[..colon_pos].trim();
            let val = trimmed[colon_pos + 1..].trim();
            if !key.is_empty()
                && !val.is_empty()
                && key.len() <= 45
                && !key.contains("://")
                && !key.starts_with("http")
            {
                kv_count += 1;
                let val_badge = if val.eq_ignore_ascii_case("yes")
                    || val.starts_with("Yes")
                    || val.eq_ignore_ascii_case("true")
                    || val.eq_ignore_ascii_case("success")
                    || val.eq_ignore_ascii_case("pass")
                {
                    format!(
                        r#"<span style="background: #ecfdf5; color: #059669; padding: 4px 10px; border-radius: 6px; font-weight: 700; border: 1px solid #a7f3d0; font-size: 14px;">{}</span>"#,
                        escape_html_entities(val)
                    )
                } else if val.eq_ignore_ascii_case("no")
                    || val.starts_with("No")
                    || val.eq_ignore_ascii_case("none")
                {
                    format!(
                        r#"<span style="color: #64748b; font-weight: 500;">{}</span>"#,
                        escape_html_entities(val)
                    )
                } else if val.contains('%') {
                    format!(
                        r#"<span style="color: #0284c7; font-weight: 700; font-family: 'Ubuntu Mono', monospace;">{}</span>"#,
                        escape_html_entities(val)
                    )
                } else if val.contains('@') {
                    format!(
                        r#"<span style="color: #0f172a; font-weight: 700; font-family: 'Ubuntu Mono', monospace;">{}</span>"#,
                        escape_html_entities(val)
                    )
                } else {
                    format!(
                        r#"<span style="color: #0f172a; font-weight: 600;">{}</span>"#,
                        escape_html_entities(val)
                    )
                };

                table_rows.push_str(&format!(
                    r#"<tr>
  <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #334155; font-weight: 600; font-size: 15px; width: 190px; background: #f8fafc;">{}</td>
  <td style="padding: 12px 18px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-size: 15px; line-height: 1.5;">{}</td>
</tr>"#,
                    escape_html_entities(key),
                    val_badge
                ));
                continue;
            }
        }

        non_kv_lines.push(escape_html_entities(trimmed));
    }

    if kv_count >= 2 {
        let mut out = format!(
            r#"<table style="width: 100%; border-collapse: collapse; font-size: 15px; margin-bottom: 20px; background: #ffffff; border-radius: 10px; overflow: hidden; border: 1px solid #e2e8f0; box-shadow: 0 1px 3px rgba(0,0,0,0.04);">
  {}
</table>"#,
            table_rows
        );
        if !non_kv_lines.is_empty() {
            out.push_str(&format!(
                r#"<div style="font-weight: 700; font-size: 13px; text-transform: uppercase; color: #475569; margin: 16px 0 8px 0; letter-spacing: 0.08em;">Details</div>
<div style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 14px 18px; color: #334155; font-size: 15px; line-height: 1.6;">{}</div>"#,
                non_kv_lines.join("<br>")
            ));
        }
        (true, out)
    } else {
        (false, String::new())
    }
}

/// Build a rich, responsive HTML card for any notification or command output
pub fn wrap_html_email_card(
    title: &str,
    content: &str,
    machine_name: &str,
    machine_ip: &str,
) -> String {
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let git_hash = crate::modules::git_info::get_git_hash();
    let git_full_hash = crate::modules::git_info::get_git_full_hash();
    let git_branch = crate::modules::git_info::get_git_branch();
    let last_release = crate::modules::git_info::get_last_release();
    let db_path = crate::modules::repo_db::get_repo_db_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "~/.antigravity_tools/repo_prompts.db".to_string());
    let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

    let details_section = if content.trim().starts_with('<') {
        content.to_string()
    } else {
        let (is_table_format, rendered_table) = format_content_as_table_rows(content);
        if is_table_format {
            rendered_table
        } else {
            let escaped_content = escape_html_entities(content.trim());
            format!(
                r#"<div style="font-weight: 800; font-size: 18px; text-transform: uppercase; color: #475569; margin-bottom: 12px; letter-spacing: 0.08em;">Message Details</div>
<pre style="background: #0f172a; color: #e2e8f0; padding: 22px; border-radius: 12px; font-family: 'Ubuntu Mono', 'Consolas', monospace; font-size: 16px; line-height: 1.7; white-space: pre-wrap; word-break: break-word; margin: 0; border: 1px solid #1e293b;">{}</pre>"#,
                escaped_content
            )
        }
    };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Ubuntu:ital,wght@0,400;0,500;0,700;1,400&family=Ubuntu+Mono:wght@400;700&display=swap" rel="stylesheet">
<style>
  body, table, td, p, div, span {{ font-family: 'Ubuntu', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 18px; line-height: 1.7; }}
  code, pre {{ font-family: 'Ubuntu Mono', 'Consolas', 'Courier New', monospace; font-size: 16px; }}
  a {{ color: #0284c7 !important; text-decoration: underline; font-weight: bold; }}
  a:visited {{ color: #0369a1 !important; }}
  a:hover {{ color: #0284c7 !important; }}
  .btn-link, td a.btn, p a.btn {{ background: #0284c7; color: #ffffff !important; padding: 6px 14px; border-radius: 8px; text-decoration: none; display: inline-block; font-size: 16px; font-weight: bold; border: 1px solid #0369a1; }}
  .btn-link:hover, td a.btn:hover, p a.btn:hover {{ background: #0369a1; color: #ffffff !important; }}
  @media (prefers-color-scheme: dark) {{
    body {{ background-color: #0f172a !important; color: #f1f5f9 !important; }}
    td {{ color: #e2e8f0 !important; }}
    a {{ color: #f8fafc !important; }}
    a:visited {{ color: #e2e8f0 !important; }}
  }}
</style>
</head>
<body style="margin: 0; padding: 28px; background-color: #f1f5f9; font-family: 'Ubuntu', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 18px; line-height: 1.7; color: #0f172a;">
  <div style="max-width: 860px; margin: 0 auto; background: #ffffff; border-radius: 16px; overflow: hidden; box-shadow: 0 10px 25px -5px rgba(15,23,42,0.1), 0 8px 10px -6px rgba(15,23,42,0.1); border: 1px solid #cbd5e1;">
    <div style="background: linear-gradient(135deg, #0f172a 0%, #1e1b4b 55%, #1e293b 100%); border-top: 5px solid #38bdf8; padding: 28px 32px; color: #ffffff;">
      <div style="margin-bottom: 14px;">
        <span style="background: rgba(56, 189, 248, 0.15); color: #f8fafc; padding: 8px 16px; border-radius: 8px; font-family: 'Ubuntu Mono', monospace; font-size: 16px; font-weight: 700; border: 1px solid rgba(56, 189, 248, 0.35);">[{} | {} ({}) | {} | {}]</span>
        <span style="background: linear-gradient(135deg, #2563eb 0%, #1d4ed8 100%); color: #ffffff; padding: 6px 14px; border-radius: 9999px; font-size: 14px; font-weight: 700; text-transform: uppercase; margin-left: 10px; letter-spacing: 0.06em;">AGM TELEMETRY</span>
      </div>
      <h2 style="margin: 10px 0 0 0; font-size: 32px; color: #ffffff; font-weight: 800; line-height: 1.35;">{}</h2>
    </div>
    <div style="padding: 32px;">
      <div style="font-weight: 800; font-size: 18px; text-transform: uppercase; color: #475569; margin-bottom: 12px; letter-spacing: 0.08em;">Node &amp; Git Telemetry (GitMap Parity)</div>
      <table style="width: 100%; border-collapse: collapse; font-size: 16px; margin-bottom: 26px; background: #ffffff; border-radius: 12px; overflow: hidden; border: 1px solid #e2e8f0; box-shadow: 0 1px 3px rgba(0,0,0,0.04);">
        <tr>
          <td style="padding: 14px 20px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 700; width: 220px; background: #f8fafc;">Application Version</td>
          <td style="padding: 14px 20px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace; font-weight: 700; font-size: 17px;">{}</td>
        </tr>
        <tr>
          <td style="padding: 14px 20px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 700; background: #f8fafc;">Git Commit SHA</td>
          <td style="padding: 14px 20px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace; font-weight: 700; font-size: 17px;"><span style="color: #0369a1;">{}</span> (<code>{}</code>)</td>
        </tr>
        <tr>
          <td style="padding: 14px 20px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 700; background: #f8fafc;">Git Branch</td>
          <td style="padding: 14px 20px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace; font-weight: 700; font-size: 17px;">{}</td>
        </tr>
        <tr>
          <td style="padding: 14px 20px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 700; background: #f8fafc;">Last Release</td>
          <td style="padding: 14px 20px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace; font-weight: 700; font-size: 17px;">{}</td>
        </tr>
        <tr>
          <td style="padding: 14px 20px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 700; background: #f8fafc;">Telemetry Database</td>
          <td style="padding: 14px 20px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace; font-size: 15px;">{}</td>
        </tr>
        <tr>
          <td style="padding: 14px 20px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 700; background: #f8fafc;">VM / Node Alias</td>
          <td style="padding: 14px 20px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace; font-weight: 700; font-size: 17px;">{}</td>
        </tr>
        <tr>
          <td style="padding: 14px 20px; border-bottom: 1px solid #f1f5f9; color: #475569; font-weight: 700; background: #f8fafc;">Local IPv4</td>
          <td style="padding: 14px 20px; border-bottom: 1px solid #f1f5f9; color: #0f172a; font-family: 'Ubuntu Mono', monospace; font-size: 17px;">{}</td>
        </tr>
        <tr>
          <td style="padding: 14px 20px; color: #475569; font-weight: 700; background: #f8fafc;">Dispatched At</td>
          <td style="padding: 14px 20px; color: #0f172a; font-family: 'Ubuntu Mono', monospace; font-size: 17px;">{}</td>
        </tr>
      </table>
      {}
    </div>
    <div style="background: #f8fafc; padding: 18px 32px; border-top: 1px solid #e2e8f0; font-size: 15px; color: #64748b; text-align: center;">
      Automated Remote Dispatcher &middot; Antigravity Manager {} ({}) &middot; Node {} ({})
    </div>
  </div>
</body>
</html>"#,
        pkg_ver,
        git_hash,
        git_branch,
        machine_name,
        machine_ip,
        escape_html_entities(title),
        pkg_ver,
        git_hash,
        git_full_hash,
        git_branch,
        last_release,
        escape_html_entities(&db_path),
        escape_html_entities(machine_name),
        escape_html_entities(machine_ip),
        now_str,
        details_section,
        pkg_ver,
        git_hash,
        machine_name,
        machine_ip
    )
}
