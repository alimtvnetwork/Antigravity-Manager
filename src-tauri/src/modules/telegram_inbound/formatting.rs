use crate::error::AppError;
use chrono::Utc;
use serde_json::{json, Value};
use std::time::Duration;

use super::*;

/// Helper to inspect currently unclosed HTML tags in an HTML snippet
pub(crate) fn get_unclosed_html_tags(s: &str) -> Vec<String> {
    let mut stack: Vec<String> = Vec::new();
    let mut in_tag = false;
    let mut current_tag = String::new();

    for c in s.chars() {
        if c == '<' {
            in_tag = true;
            current_tag.clear();
        } else if c == '>' {
            in_tag = false;
            let tag_content = current_tag.trim();
            if tag_content.starts_with('/') {
                let closing = tag_content[1..]
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_lowercase();
                if let Some(pos) = stack.iter().rposition(|t| {
                    t == &closing || t.split_whitespace().next().unwrap_or("") == closing.as_str()
                }) {
                    stack.remove(pos);
                }
            } else if !tag_content.is_empty() && !tag_content.ends_with('/') {
                let opening = tag_content
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_lowercase();
                if [
                    "b",
                    "strong",
                    "i",
                    "em",
                    "u",
                    "ins",
                    "s",
                    "strike",
                    "del",
                    "span",
                    "a",
                    "code",
                    "pre",
                    "blockquote",
                ]
                .contains(&opening.as_str())
                {
                    stack.push(opening);
                }
            }
        } else if in_tag {
            current_tag.push(c);
        }
    }
    stack
}

/// Chunk text into pieces <= max_chars splitting at newline boundaries where possible,
/// ensuring HTML tags are balanced across chunks to prevent Telegram parse errors.
pub fn chunk_telegram_text(text: &str, max_chars: usize) -> Vec<String> {
    let limit = if max_chars == 0 { 3800 } else { max_chars };
    if text.len() <= limit {
        return vec![text.to_string()];
    }

    let mut raw_chunks = Vec::new();
    let mut current_chunk = String::with_capacity(limit);

    for line in text.split_inclusive('\n') {
        if line.len() > limit {
            if !current_chunk.is_empty() {
                raw_chunks.push(current_chunk);
                current_chunk = String::with_capacity(limit);
            }
            let mut remaining = line;
            while remaining.len() > limit {
                let mut split_pos = limit;
                while split_pos > 0 && !remaining.is_char_boundary(split_pos) {
                    split_pos -= 1;
                }
                let sub = &remaining[..split_pos];
                if let Some(last_lt) = sub.rfind('<') {
                    if !sub[last_lt..].contains('>') && last_lt > 0 {
                        split_pos = last_lt;
                    }
                }
                let (slice, rest) = remaining.split_at(split_pos);
                raw_chunks.push(slice.to_string());
                remaining = rest;
            }
            if !remaining.is_empty() {
                current_chunk.push_str(remaining);
            }
        } else if current_chunk.len() + line.len() > limit {
            raw_chunks.push(current_chunk);
            current_chunk = String::with_capacity(limit);
            current_chunk.push_str(line);
        } else {
            current_chunk.push_str(line);
        }
    }

    if !current_chunk.is_empty() {
        raw_chunks.push(current_chunk);
    }

    if raw_chunks.is_empty() {
        return vec![text.to_string()];
    }

    let total = raw_chunks.len();
    let mut balanced_chunks = Vec::with_capacity(total);
    let mut carry_over_tags = Vec::new();

    for (idx, mut chunk) in raw_chunks.into_iter().enumerate() {
        if !carry_over_tags.is_empty() {
            let mut prefix = String::new();
            for tag in &carry_over_tags {
                if tag == "blockquote expandable" {
                    prefix.push_str("<blockquote expandable>");
                } else {
                    prefix.push_str(&format!("<{}>", tag));
                }
            }
            chunk = format!("{}{}", prefix, chunk);
        }

        let unclosed = get_unclosed_html_tags(&chunk);
        let has_expandable_blockquote = chunk.to_lowercase().contains("<blockquote expandable");

        carry_over_tags = unclosed
            .iter()
            .map(|t| {
                if t == "blockquote" && has_expandable_blockquote {
                    "blockquote expandable".to_string()
                } else {
                    t.clone()
                }
            })
            .collect();

        if !unclosed.is_empty() && idx + 1 < total {
            for tag in unclosed.iter().rev() {
                chunk.push_str(&format!("</{}>", tag));
            }
        }

        balanced_chunks.push(chunk);
    }

    balanced_chunks
}

/// Helper to clamp prompt and clipboard text safely on UTF-8 char boundaries (<= 3,900 chars)
pub fn clamp_telegram_copy_text(text: &str, max_chars: usize) -> String {
    let limit = if max_chars == 0 || max_chars > 3900 {
        3900
    } else {
        max_chars
    };
    if text.len() <= limit {
        text.to_string()
    } else {
        let mut end = limit.saturating_sub(3);
        while end > 0 && !text.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &text[..end])
    }
}

/// Sanitize unsupported HTML named entities (&nbsp;, &bull;) for Telegram HTML parse_mode.
/// Telegram HTML mode only supports &lt;, &gt;, &amp;, &quot; and numeric character references.
pub fn sanitize_telegram_html_entities(text: &str) -> String {
    text.replace("&nbsp;", "\u{2007}").replace("&bull;", "•")
}

/// Build rich inline keyboard markup for Account Rotation (Switch) Alerts
pub fn build_switch_alert_reply_markup(
    instance_name: &str,
    _selected_email: &str,
    target_4h: &str,
    target_weekly: &str,
    prompt_text: &str,
    asset_urls: &[String],
) -> serde_json::Value {
    let mut keyboard_rows: Vec<Vec<serde_json::Value>> = Vec::new();
    let inst_label = if instance_name.trim().is_empty() {
        "default"
    } else {
        instance_name.trim()
    };

    keyboard_rows.push(vec![json!({
        "text": format!("🟢 {} | 4h: {} | Wk: {}", inst_label, target_4h, target_weekly),
        "callback_data": "cmd:/balance"
    })]);

    keyboard_rows.push(vec![
        json!({
            "text": "💰 Check Balance",
            "callback_data": "cmd:/balance"
        }),
        json!({
            "text": "📥 Export CSV",
            "callback_data": "cmd:/export accounts csv"
        }),
    ]);

    keyboard_rows.push(vec![
        json!({
            "text": "🚀 Prompt Instance",
            "callback_data": "cmd:/send"
        }),
        json!({
            "text": "🔄 Refresh Status",
            "callback_data": "cmd:/status"
        }),
    ]);

    let clamped_text = clamp_telegram_copy_text(prompt_text, 3900);
    if !clamped_text.trim().is_empty() {
        keyboard_rows.push(vec![json!({
            "text": "📋 Copy Prompt",
            "copy_text": {
                "text": clamped_text
            }
        })]);

        if !asset_urls.is_empty() {
            let mut composite = clamped_text;
            composite.push_str("\n\n---\n### Attached Images:\n");
            for url in asset_urls {
                composite.push_str(&format!("![image]({})\n", url));
            }
            let clamped_composite = clamp_telegram_copy_text(&composite, 3900);
            keyboard_rows.push(vec![json!({
                "text": "🖼️ Copy Prompt + Images",
                "copy_text": {
                    "text": clamped_composite
                }
            })]);
        }
    }

    json!({
        "inline_keyboard": keyboard_rows
    })
}

/// Build rich inline keyboard markup for Post-Switch Liveness & Emergency Alerts
pub fn build_liveness_alert_reply_markup(
    instance_id: &str,
    verified_running: usize,
    backed_up_count: usize,
    prompt_text: &str,
    asset_urls: &[String],
) -> serde_json::Value {
    let mut keyboard_rows: Vec<Vec<serde_json::Value>> = Vec::new();
    let inst_label = if instance_id.trim().is_empty() {
        "default"
    } else {
        instance_id.trim()
    };

    keyboard_rows.push(vec![json!({
        "text": format!("🟢 {} | Running: {}/{}", inst_label, verified_running, backed_up_count),
        "callback_data": "cmd:/active"
    })]);

    keyboard_rows.push(vec![
        json!({
            "text": "💰 Check Balance",
            "callback_data": "cmd:/balance"
        }),
        json!({
            "text": "📥 Export CSV",
            "callback_data": "cmd:/export accounts csv"
        }),
    ]);

    keyboard_rows.push(vec![
        json!({
            "text": "🚀 Prompt Instance",
            "callback_data": "cmd:/send"
        }),
        json!({
            "text": "🔄 Refresh Status",
            "callback_data": "cmd:/status"
        }),
    ]);

    let clamped_text = clamp_telegram_copy_text(prompt_text, 3900);
    if !clamped_text.trim().is_empty() {
        keyboard_rows.push(vec![json!({
            "text": "📋 Copy Prompt",
            "copy_text": {
                "text": clamped_text
            }
        })]);

        if !asset_urls.is_empty() {
            let mut composite = clamped_text;
            composite.push_str("\n\n---\n### Attached Images:\n");
            for url in asset_urls {
                composite.push_str(&format!("![image]({})\n", url));
            }
            let clamped_composite = clamp_telegram_copy_text(&composite, 3900);
            keyboard_rows.push(vec![json!({
                "text": "🖼️ Copy Prompt + Images",
                "copy_text": {
                    "text": clamped_composite
                }
            })]);
        }
    }

    json!({
        "inline_keyboard": keyboard_rows
    })
}

/// Build Telegram Bot API 7.8+ inline keyboard markup with native copy_text button
pub fn build_copy_prompt_reply_markup(
    prompt_text: &str,
    asset_urls: &[String],
    dashboard_url: Option<&str>,
) -> serde_json::Value {
    let clamped_text = clamp_telegram_copy_text(prompt_text, 3900);
    let mut keyboard_rows: Vec<Vec<serde_json::Value>> = Vec::new();

    if !clamped_text.trim().is_empty() {
        keyboard_rows.push(vec![json!({
            "text": "📋 Copy Prompt",
            "copy_text": {
                "text": clamped_text
            }
        })]);

        if !asset_urls.is_empty() {
            let mut composite = clamped_text.clone();
            composite.push_str("\n\n---\n### Attached Images:\n");
            for url in asset_urls {
                composite.push_str(&format!("![image]({})\n", url));
            }
            let clamped_composite = clamp_telegram_copy_text(&composite, 3900);
            keyboard_rows.push(vec![json!({
                "text": "🖼️ Copy Prompt + Images",
                "copy_text": {
                    "text": clamped_composite
                }
            })]);
        }
    }

    let mut action_row: Vec<serde_json::Value> = Vec::new();
    if let Some(dash_url) = dashboard_url {
        let trimmed = dash_url.trim();
        if !trimmed.is_empty() {
            action_row.push(json!({
                "text": "📊 Open Dashboard",
                "url": trimmed
            }));
        }
    }

    if !action_row.is_empty() {
        keyboard_rows.push(action_row);
    }

    json!({
        "inline_keyboard": keyboard_rows
    })
}

/// Send a text message to a Telegram chat with optional reply markup (e.g. inline keyboard)
/// and optional link-preview suppression (for compact status notifications).
pub async fn send_telegram_message_with_markup(
    bot_token: &str,
    chat_id: i64,
    text: &str,
    reply_markup: Option<serde_json::Value>,
    disable_preview: bool,
) -> Result<(), AppError> {
    let clean_token = bot_token.trim();
    if clean_token.is_empty() {
        return Err(AppError::Config("Telegram bot token is empty".to_string()));
    }

    let sanitized_text = sanitize_telegram_html_entities(text);
    let chunks = chunk_telegram_text(&sanitized_text, 3800);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    let url = format!("https://api.telegram.org/bot{}/sendMessage", clean_token);
    let total = chunks.len();

    // Only attach reply_markup if it contains non-empty buttons
    let valid_markup = reply_markup.filter(|m| {
        m.get("inline_keyboard")
            .and_then(|k| k.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(true)
    });

    for (idx, chunk) in chunks.iter().enumerate() {
        if idx > 0 {
            tokio::time::sleep(Duration::from_millis(80)).await;
        }

        let is_last = idx + 1 == total;
        let mut payload_html = json!({
            "chat_id": chat_id,
            "text": chunk,
            "parse_mode": "HTML",
            "disable_web_page_preview": disable_preview
        });
        if is_last {
            if let Some(ref markup) = valid_markup {
                payload_html["reply_markup"] = markup.clone();
            }
        }

        let resp = client.post(&url).json(&payload_html).send().await;
        match resp {
            Ok(r) if r.status().is_success() => continue,
            Ok(r) if r.status().as_u16() == 400 => {
                // If HTML parse error occurred, strip HTML tags and retry sending chunk as clean plain text
                let clean_plain = strip_telegram_html_tags(chunk);
                let mut payload_plain = json!({
                    "chat_id": chat_id,
                    "text": clean_plain,
                    "disable_web_page_preview": disable_preview
                });
                if is_last {
                    if let Some(ref markup) = valid_markup {
                        payload_plain["reply_markup"] = markup.clone();
                    }
                }
                let retry_resp = client
                    .post(&url)
                    .json(&payload_plain)
                    .send()
                    .await
                    .map_err(|e| AppError::Network(e.to_string(), None))?;
                let status = retry_resp.status().as_u16();
                if !retry_resp.status().is_success() {
                    let err = retry_resp.text().await.unwrap_or_default();
                    return Err(AppError::Network(
                        format!("Telegram sendMessage failed: {}", err),
                        Some(status),
                    ));
                }
            }
            Ok(r) => {
                let status = r.status().as_u16();
                let err = r.text().await.unwrap_or_default();
                return Err(AppError::Network(
                    format!(
                        "Telegram sendMessage failed with status {}: {}",
                        status, err
                    ),
                    Some(status),
                ));
            }
            Err(e) => return Err(AppError::Network(e.to_string(), None)),
        }
    }

    Ok(())
}

/// Send a text message to a Telegram chat, automatically chunking messages longer than 3800 characters
pub async fn send_telegram_message(
    bot_token: &str,
    chat_id: i64,
    text: &str,
) -> Result<(), AppError> {
    send_telegram_message_with_markup(bot_token, chat_id, text, None, false).await
}

/// Send a text message with link previews suppressed (compact status notifications)
pub async fn send_telegram_message_no_preview(
    bot_token: &str,
    chat_id: i64,
    text: &str,
) -> Result<(), AppError> {
    send_telegram_message_with_markup(bot_token, chat_id, text, None, true).await
}

/// Convenience alias for send_telegram_message with automatic chunking
pub async fn send_telegram_message_chunked(
    bot_token: &str,
    chat_id: i64,
    text: &str,
) -> Result<(), AppError> {
    send_telegram_message(bot_token, chat_id, text).await
}

/// Strip ANSI escape codes and escape HTML entities for Telegram <pre> blocks
pub fn clean_for_telegram_html(input: &str, max_chars: usize) -> String {
    let mut no_ansi = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&nc) = chars.peek() {
                    chars.next();
                    if nc.is_ascii_alphabetic() {
                        break;
                    }
                }
                continue;
            }
        }
        no_ansi.push(c);
    }

    let truncated: String = if no_ansi.chars().count() > max_chars {
        let trimmed_slice: String = no_ansi.chars().take(max_chars).collect();
        let base = trimmed_slice.trim_end_matches('.');
        format!("{}...", base.trim_end())
    } else {
        no_ansi
    };

    truncated
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Look up an instance's recorded IDE executable path from the instance
/// registry — the same lookup the `/send` picker performs in
/// `send_flow::instance_entries`. Returns `None` when the registry is
/// unreadable or the instance id is unknown.
fn instance_executable_path(instance_id: &str) -> Option<String> {
    match crate::modules::instance::load_registry() {
        Ok(reg) => reg
            .instances
            .iter()
            .find(|i| i.id == instance_id)
            .and_then(|i| i.executable_path.clone()),
        Err(e) => {
            // Justification: the IDE line is best-effort receipt garnish; the
            // receipt must still render when the registry is unreadable, so
            // the line is simply omitted and the failure is tracked.
            crate::error::record_ignored(
                Err::<(), String>(e),
                "load instance registry for IDE exe line",
            );
            None
        }
    }
}

/// Build the standalone "IDE exe" line for a Telegram prompt receipt, or
/// `None` when the exe cannot be shown reliably.
///
/// The exe is shown only when `executable_path` is actually recorded in the
/// registry, or for the default instance. For other instances with no
/// recorded path the resolver would synthesize `Antigravity-{id}.exe`, which
/// is a guess and is deliberately omitted (commit af423861). Basename/ending
/// only — never full paths; the name is HTML-escaped for Telegram.
pub fn ide_exe_line(instance_id: &str, executable_path: Option<&str>) -> Option<String> {
    let is_default =
        instance_id == "default" || instance_id == "__default__" || instance_id.is_empty();
    let recorded = executable_path.filter(|p| !p.trim().is_empty());
    if !is_default && recorded.is_none() {
        return None;
    }
    let exe = crate::modules::instance::resolve_instance_exe_name(instance_id, recorded);
    Some(format!(
        "• <b>IDE:</b> <code>{}</code>\n",
        clean_for_telegram_html(&exe, 64)
    ))
}

/// Convenience wrapper for receipts that only know the instance id: performs
/// the registry lookup and returns the IDE line, or `None` when unknown.
pub fn ide_exe_line_for(instance_id: &str) -> Option<String> {
    let path = instance_executable_path(instance_id);
    ide_exe_line(instance_id, path.as_deref())
}

/// Strip HTML tags and decode HTML entities for clean plain text fallback
pub fn strip_telegram_html_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for c in input.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            out.push(c);
        }
    }
    out.replace("&nbsp;", " ")
        .replace("&bull;", "•")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
}

/// Helper to abbreviate project names (e.g. Antigravity-Manager -> AGM)
pub fn shorten_project_name(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.contains("antigravity-manager") || lower.contains("antigravity_manager") {
        "AGM".to_string()
    } else if lower.contains("gitmap") {
        "GitMap".to_string()
    } else if lower == "repo" {
        "repo".to_string()
    } else {
        let trimmed = name.trim();
        if let Some(idx) = trimmed.rfind('-') {
            let suffix = &trimmed[idx + 1..];
            if suffix.len() >= 6 && suffix.chars().all(|c| c.is_ascii_hexdigit()) {
                return trimmed[..idx].to_string();
            }
        }
        trimmed.to_string()
    }
}

/// Helper to format running duration from a Unix timestamp in seconds
pub fn format_running_duration(created_at: i64) -> String {
    if created_at <= 0 {
        return String::new();
    }
    let now = chrono::Utc::now().timestamp();
    let elapsed = (now - created_at).max(0);
    if elapsed < 60 {
        format!("(running {}s)", elapsed)
    } else if elapsed < 3600 {
        let mins = elapsed / 60;
        let secs = elapsed % 60;
        format!("(running {}m {}s)", mins, secs)
    } else {
        let hours = elapsed / 3600;
        let mins = (elapsed % 3600) / 60;
        format!("(running {}h {}m)", hours, mins)
    }
}

#[cfg(test)]
mod formatting_tests {
    use super::*;

    #[test]
    fn test_ide_exe_line_caution() {
        // Unknown non-default instance: line omitted, never a guessed fallback.
        assert!(ide_exe_line("work", None).is_none());
        assert!(ide_exe_line("work", Some("   ")).is_none());
        // Default instance always shows the resolved exe name.
        let def = ide_exe_line("default", None).expect("default shows exe");
        assert!(def.starts_with("• <b>IDE:</b> <code>"));
        assert!(def.ends_with("</code>\n"));
        // Recorded path: basename only, HTML-escaped.
        let exe = ide_exe_line("work", Some("/opt/ide/A&g IDE")).expect("recorded path shows exe");
        assert!(!exe.contains("/opt/ide/"));
        assert!(exe.contains("A&amp;g IDE"));
    }

    #[test]
    fn test_unclosed_tags_with_blockquote_expandable() {
        let unclosed = get_unclosed_html_tags("<blockquote expandable>Hello world");
        assert_eq!(unclosed, vec!["blockquote"]);

        let closed = get_unclosed_html_tags("<blockquote expandable>Hello world</blockquote>");
        assert!(closed.is_empty());
    }

    #[test]
    fn test_clamp_telegram_copy_text() {
        let short = "Short prompt";
        assert_eq!(clamp_telegram_copy_text(short, 3900), short);

        let long = "a".repeat(4500);
        let clamped = clamp_telegram_copy_text(&long, 3900);
        assert!(clamped.len() <= 3900);
        assert!(clamped.ends_with("..."));
    }

    #[test]
    fn test_build_copy_prompt_reply_markup() {
        let markup = build_copy_prompt_reply_markup(
            "My active prompt",
            &["https://example.com/img1.png".to_string()],
            Some("https://dashboard.example.com"),
        );
        let rows = markup["inline_keyboard"].as_array().expect("rows array");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0][0]["text"], "📋 Copy Prompt");
        assert_eq!(rows[0][0]["copy_text"]["text"], "My active prompt");
        assert_eq!(rows[1][0]["text"], "🖼️ Copy Prompt + Images");
        assert_eq!(rows[2][0]["text"], "📊 Open Dashboard");
        assert_eq!(rows[2][0]["url"], "https://dashboard.example.com");
    }
}
