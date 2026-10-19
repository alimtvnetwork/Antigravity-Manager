use crate::error::AppError;
use chrono::Utc;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use std::path::PathBuf;

use super::*;

/// Normalize a `cmd:` inline-keyboard payload into the `/command …` text the
/// command pipeline expects. Both spellings exist in the wild (`cmd:/send`
/// from the report markups, `cmd:fleet` from the ping markup), so every
/// leading slash after `cmd:` is stripped. Non-`cmd:` payloads pass through
/// untouched.
pub(crate) fn normalize_cmd_callback_payload(data: &str) -> String {
    if let Some(stripped) = data.strip_prefix("cmd:") {
        format!("/{}", stripped.trim_start_matches('/'))
    } else {
        data.to_string()
    }
}

/// Handle `send:*` inline-keyboard callbacks for the `/send` two-step flow.
/// Returns true when the callback belonged to the flow (caller should `continue`).
async fn deliver_send_flow_callback(
    token: &str,
    chat_id: i64,
    cb_id: &str,
    data: &str,
    client: &reqwest::Client,
) -> bool {
    if !data.starts_with("send:") {
        return false;
    }
    let (reply, markup) = super::send_flow::handle_send_callback(chat_id, data).await;
    // Justification: the picker reply is already computed and flow state is
    // updated; a Telegram send failure is logged without changing the flow.
    // If reply is empty (e.g. this node dropped callback intended for another machine), do not send a message.
    if !reply.trim().is_empty() {
        crate::error::record_ignored(
            send_telegram_message_with_markup(token, chat_id, &reply, markup, false).await,
            "send /send picker reply",
        );
    }
    if !cb_id.is_empty() {
        let ans_url = format!("https://api.telegram.org/bot{}/answerCallbackQuery", token);
        // Justification: best-effort send; failure logged without changing control flow
        crate::error::record_ignored(
            client
                .post(&ans_url)
                .json(&json!({ "callback_query_id": cb_id }))
                .send()
                .await,
            "answerCallbackQuery",
        );
    }
    true
}

/// Detect if a Telegram command requests a CSV or JSON file attachment export
/// (e.g. `/export accounts csv`, `/accounts --csv`, `/instances --csv`, `/balance --csv`, `/status --json`)
pub(crate) fn detect_telegram_export_request(text_raw: &str) -> Option<(String, String)> {
    let trimmed = text_raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (cmd_raw, rest) = match trimmed.split_once(char::is_whitespace) {
        Some((c, r)) => (c, r.trim()),
        None => (trimmed, ""),
    };
    let cmd_no_slash = cmd_raw.strip_prefix('/').unwrap_or(cmd_raw);
    let cmd_base = cmd_no_slash
        .split('@')
        .next()
        .unwrap_or(cmd_no_slash)
        .to_lowercase();

    if cmd_base == "export" {
        let mut parts = rest.split_whitespace();
        let target = parts.next().unwrap_or("accounts").to_lowercase();
        let fmt = parts
            .next()
            .map(|s| s.trim_start_matches('-').to_lowercase())
            .unwrap_or_else(|| "csv".to_string());
        let normalized_fmt = if fmt == "json" { "json" } else { "csv" };
        return Some((target, normalized_fmt.to_string()));
    }

    let tokens: Vec<&str> = rest.split_whitespace().collect();
    let mut detected_fmt: Option<&str> = None;
    for (idx, tok) in tokens.iter().enumerate() {
        if tok.eq_ignore_ascii_case("--csv") || tok.eq_ignore_ascii_case("-csv") {
            detected_fmt = Some("csv");
            break;
        } else if tok.eq_ignore_ascii_case("--json") || tok.eq_ignore_ascii_case("-json") {
            detected_fmt = Some("json");
            break;
        } else if tok.eq_ignore_ascii_case("--format") {
            if let Some(next_tok) = tokens.get(idx + 1) {
                if next_tok.eq_ignore_ascii_case("json") {
                    detected_fmt = Some("json");
                } else {
                    detected_fmt = Some("csv");
                }
                break;
            }
        }
    }

    if let Some(fmt) = detected_fmt {
        let target = match cmd_base.as_str() {
            "accounts" | "acc" => Some("accounts"),
            "instances" | "inst" => Some("instances"),
            "balance" | "credits" | "quota" | "bal" => Some("balance"),
            "nodes" | "node" | "ssh" => Some("nodes"),
            "prompts" | "prompt_queue" => Some("prompts"),
            "status" | "observe" => Some("status"),
            _ => None,
        };
        if let Some(t) = target {
            return Some((t.to_string(), fmt.to_string()));
        }
    }

    None
}

async fn deliver_ui_or_command(
    token: &str,
    chat_id: i64,
    text_raw: &str,
    saved_media_opt: &Option<(PathBuf, Option<String>)>,
) -> Option<String> {
    // `/send` two-step flow: once the destination is locked, the next plain-text
    // message is the prompt. Commands (leading `/`) always pass through.
    if let Some(send) =
        super::send_flow::consume_send_prompt(token, chat_id, text_raw, saved_media_opt).await
    {
        // Justification: the send-flow reply is already computed; a Telegram
        // send failure is logged and does not change the dispatched prompt.
        crate::error::record_ignored(
            send_telegram_message_with_markup(token, chat_id, &send.reply, send.markup, false)
                .await,
            "send /send flow reply",
        );
        return Some(send.reply);
    }

    // Check if command requests CSV/JSON document attachment upload
    if let Some((export_target, export_fmt)) = detect_telegram_export_request(text_raw) {
        match reports_a::generate_export_document(&export_target, &export_fmt) {
            Ok(doc) => {
                if let Err(err) = send_telegram_document(
                    token,
                    chat_id,
                    &doc.file_name,
                    &doc.bytes,
                    &doc.caption,
                    doc.markup.clone(),
                )
                .await
                {
                    let fallback = format!(
                        "{}\n⚠️ <i>Document upload fallback:</i> <code>{}</code>",
                        doc.caption,
                        crate::modules::notification_hub::escape_telegram_html(&err)
                    );
                    // Justification: fallback message when document upload fails; logged via record_ignored
                    crate::error::record_ignored(
                        send_telegram_message_with_markup(
                            token, chat_id, &fallback, doc.markup, false,
                        )
                        .await,
                        "send export fallback message",
                    );
                }
                return Some(doc.caption);
            }
            Err(err) => {
                let err_msg = format!(
                    "❌ <b>Export Error:</b> <code>{}</code>",
                    crate::modules::notification_hub::escape_telegram_html(&err)
                );
                // Justification: best-effort error notification to Telegram chat
                crate::error::record_ignored(
                    send_telegram_message_with_markup(token, chat_id, &err_msg, None, false).await,
                    "send export error message",
                );
                return Some(err_msg);
            }
        }
    }

    if let Some(action) = super::ui_remote::prepare_local_ui_action(text_raw).await {
        if let Some(png) = action.png.as_deref() {
            if let Err(err) = send_telegram_photo(token, chat_id, png, &action.caption, None).await
            {
                let photo_err = format!(
                    "❌ <b>Screenshot was captured but Telegram rejected the photo:</b> <code>{}</code>",
                    crate::modules::notification_hub::escape_telegram_html(&err.to_string())
                );
                // Justification: the photo error is already included in the follow-up message; a second send failure is logged.
                crate::error::record_ignored(
                    send_telegram_message_with_markup(token, chat_id, &photo_err, None, false)
                        .await,
                    "send screenshot upload error",
                );
            }
        }
        // Justification: the command result is already computed; a Telegram send failure is logged and does not change the local action.
        crate::error::record_ignored(
            send_telegram_message_with_markup(token, chat_id, &action.reply, action.markup, false)
                .await,
            "send agm ui command reply",
        );
        return Some(action.reply);
    }

    let (reply_opt, reply_markup_opt) =
        resolve_telegram_command_reply_and_markup(text_raw, saved_media_opt).await;
    if let Some(reply) = reply_opt {
        // Justification: command text is already built; a Telegram send failure is logged and the caller still records the reply.
        crate::error::record_ignored(
            send_telegram_message_with_markup(token, chat_id, &reply, reply_markup_opt, false)
                .await,
            "send_telegram_message_with_markup",
        );
        return Some(reply);
    }
    None
}

/// Resolve unified command response and interactive inline reply markup (copy buttons, navigation buttons)
pub(crate) async fn resolve_telegram_command_reply_and_markup(
    text_raw: &str,
    saved_media_opt: &Option<(PathBuf, Option<String>)>,
) -> (Option<String>, Option<serde_json::Value>) {
    let trimmed = text_raw.trim();

    // 1. If photo upload with empty caption
    if trimmed.is_empty() {
        if let Some((img_path, public_url_opt)) = saved_media_opt.as_ref() {
            if let Some(url) = public_url_opt {
                let msg_text = format!(
                    "📸 <b>Image Uploaded &amp; Ready for Reuse!</b>\n\
                    ━━━━━━━━━━━━━━━━━━━━━━━━\n\
                    🔗 <b>Direct CDN URL:</b> <a href=\"{}\">{}</a>\n\
                    📁 <b>Local File:</b> <code>{}</code>\n\n\
                    📝 <b>Markdown Embed:</b>\n\
                    <code>![image]({})</code>\n\n\
                    💡 Send <code>/prompt &lt;project&gt; \"instruction\"</code> to run tasks with this image.",
                    url,
                    url,
                    img_path.display(),
                    url
                );
                let markup = json!({
                    "inline_keyboard": [
                        [
                            {
                                "text": "📋 Copy Markdown",
                                "copy_text": {
                                    "text": format!("![image]({})", url)
                                }
                            },
                            {
                                "text": "🔗 Copy URL",
                                "copy_text": {
                                    "text": url
                                }
                            }
                        ]
                    ]
                });
                return (Some(msg_text), Some(markup));
            } else {
                let msg_text = format!(
                    "📸 <b>Screenshot Saved Successfully</b>\n\n\
                    • <b>Path:</b> <code>{}</code>\n\n\
                    💡 To execute with instruction: <code>/prompt &lt;project&gt; \"instruction\"</code>",
                    img_path.display()
                );
                return (Some(msg_text), None);
            }
        }
        return (None, None);
    }

    // 2. Command with text
    let base_reply = process_telegram_command_text(trimmed).await;
    let (cmd_raw, rest) = match trimmed.split_once(char::is_whitespace) {
        Some((c, r)) => (c, r.trim()),
        None => (trimmed, ""),
    };
    let cmd_no_slash = cmd_raw.strip_prefix('/').unwrap_or(cmd_raw);
    let cmd_base = cmd_no_slash
        .split('@')
        .next()
        .unwrap_or(cmd_no_slash)
        .to_lowercase();

    let mut markup_opt = match cmd_base.as_str() {
        "expand" | "exp" | "inspect" | "ins" | "show" | "view" => {
            reports_b::resolve_expand_prompt_markup(rest)
        }
        "history" | "hist" => Some(super::prompt_history::format_prompt_history(8).1),
        "fleet" | "whoami" => Some(json!({
            "inline_keyboard": [
                [
                    { "text": "🔄 Refresh Fleet", "callback_data": "cmd:/fleet" },
                    { "text": "💰 Account Balances", "callback_data": "cmd:/balance" }
                ],
                [
                    { "text": "🏷️ Instances", "callback_data": "cmd:/instances" },
                    { "text": "👤 Accounts", "callback_data": "cmd:/accounts" },
                    { "text": "🚀 Prompt Wizard (/send)", "callback_data": "cmd:/send" }
                ]
            ]
        })),
        "instances" | "inst" => Some(json!({
            "inline_keyboard": [
                [
                    { "text": "📥 Instances CSV", "callback_data": "cmd:/export instances csv" },
                    { "text": "💰 Account Balances", "callback_data": "cmd:/balance" }
                ],
                [
                    { "text": "🔄 Refresh", "callback_data": "cmd:/instances" },
                    { "text": "🚀 Prompt Wizard (/send)", "callback_data": "cmd:/send" }
                ]
            ]
        })),
        "accounts" | "acc" => Some(json!({
            "inline_keyboard": [
                [
                    { "text": "📥 Accounts CSV", "callback_data": "cmd:/export accounts csv" },
                    { "text": "💰 Account Balances", "callback_data": "cmd:/balance" }
                ],
                [
                    { "text": "🔄 Refresh", "callback_data": "cmd:/accounts" },
                    { "text": "🏷️ Instances", "callback_data": "cmd:/instances" }
                ]
            ]
        })),
        "balance" | "credits" | "quota" | "bal" => Some(reports_a::resolve_balance_markup()),
        "status" | "observe" => Some(json!({
            "inline_keyboard": [
                [
                    { "text": "💰 Account Balances", "callback_data": "cmd:/balance" },
                    { "text": "🚀 Prompt Wizard (/send)", "callback_data": "cmd:/send" }
                ],
                [
                    { "text": "📥 Accounts CSV", "callback_data": "cmd:/export accounts csv" },
                    { "text": "📥 Instances CSV", "callback_data": "cmd:/export instances csv" }
                ],
                [
                    { "text": "🖥️ SSH Nodes", "callback_data": "cmd:/nodes" },
                    { "text": "🔄 Refresh Status", "callback_data": "cmd:/status" }
                ]
            ]
        })),
        "nodes" | "node" | "ssh" => Some(json!({
            "inline_keyboard": [
                [
                    { "text": "📥 Nodes CSV", "callback_data": "cmd:/export nodes csv" },
                    { "text": "🔄 Import GitMap SSH", "callback_data": "cmd:/ssh import-gitmap" }
                ],
                [
                    { "text": "💰 Account Balances", "callback_data": "cmd:/balance" },
                    { "text": "🚀 Prompt Wizard (/send)", "callback_data": "cmd:/send" }
                ]
            ]
        })),
        "help" | "start" => Some(super::ui_remote::help_copy_markup()),
        "send" => {
            if rest.trim().eq_ignore_ascii_case("cancel") {
                None
            } else {
                Some(super::send_flow::send_entry_markup())
            }
        }
        "ping" => Some(json!({
            "inline_keyboard": [
                [
                    { "text": "🌐 Fleet Identity", "callback_data": "cmd:fleet" },
                    { "text": "📊 Live Observe", "callback_data": "cmd:observe" }
                ]
            ]
        })),
        _ => None,
    };

    // If media was uploaded along with this command
    if let Some((img_path, public_url_opt)) = saved_media_opt.as_ref() {
        let banner = if let Some(url) = public_url_opt {
            format!(
                "📸 <b>Image Uploaded:</b> <a href=\"{}\">{}</a>\n📁 <b>Local File:</b> <code>{}</code>\n\n",
                url, url, img_path.display()
            )
        } else {
            format!(
                "📸 <b>Image Saved:</b> <code>{}</code>\n\n",
                img_path.display()
            )
        };

        if markup_opt.is_none() {
            if let Some(url) = public_url_opt {
                markup_opt = Some(json!({
                    "inline_keyboard": [
                        [
                            {
                                "text": "📋 Copy Markdown",
                                "copy_text": {
                                    "text": format!("![image]({})", url)
                                }
                            },
                            {
                                "text": "🔗 Copy URL",
                                "copy_text": {
                                    "text": url.clone()
                                }
                            }
                        ]
                    ]
                }));
            }
        }
        return (base_reply.map(|r| format!("{}{}", banner, r)), markup_opt);
    }

    (base_reply, markup_opt)
}

/// Poll pending Telegram updates once, execute commands, reply, and advance update offset
pub async fn poll_telegram_updates_once() -> Result<Vec<(i64, String, String)>, AppError> {
    let mut config = load_config()?;
    let clean_token = config.bot_token.trim().to_string();
    if clean_token.is_empty() {
        return Err(AppError::Config("Telegram bot token is empty".to_string()));
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    let url = format!("https://api.telegram.org/bot{}/getUpdates", clean_token);
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    if !resp.status().is_success() {
        return Err(AppError::Network(
            format!("Telegram getUpdates returned {}", resp.status()),
            Some(resp.status().as_u16()),
        ));
    }

    let body: Value = resp
        .json()
        .await
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    let mut processed = Vec::new();
    let mut max_update_id: Option<i64> = None;

    if let Some(updates) = body["result"].as_array() {
        for update in updates {
            if let Some(up_id) = update["update_id"].as_i64() {
                max_update_id = Some(max_update_id.map_or(up_id, |m| m.max(up_id)));
            }

            // 1. Handle interactive inline keyboard callback queries
            if let Some(cb) = update.get("callback_query") {
                let cb_id = cb["id"].as_str().unwrap_or("");
                let chat_id = cb["message"]["chat"]["id"].as_i64().unwrap_or(0);
                let data = cb["data"].as_str().unwrap_or("").trim();

                if chat_id != 0 && !data.is_empty() {
                    if deliver_send_flow_callback(&clean_token, chat_id, cb_id, data, &client).await
                    {
                        continue;
                    }

                    if config.allowed_chat_id.is_none() {
                        config.allowed_chat_id = Some(chat_id);
                        config.is_enabled = true;
                        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                        crate::error::record_ignored(save_config(&config), "save_config");
                    }

                    if let Some(allowed) = config.allowed_chat_id {
                        if chat_id == allowed {
                            let cmd_str = normalize_cmd_callback_payload(data);

                            if let Some(reply) =
                                deliver_ui_or_command(&clean_token, chat_id, &cmd_str, &None).await
                            {
                                processed.push((chat_id, cmd_str, reply));
                            }

                            if !cb_id.is_empty() {
                                let ans_url = format!(
                                    "https://api.telegram.org/bot{}/answerCallbackQuery",
                                    clean_token
                                );
                                // Justification: best-effort send; failure logged without changing control flow
                                crate::error::record_ignored(
                                    client
                                        .post(&ans_url)
                                        .json(&json!({ "callback_query_id": cb_id }))
                                        .send()
                                        .await,
                                    "answerCallbackQuery",
                                );
                            }
                        }
                    }
                }
                continue;
            }

            // 2. Handle standard chat messages
            let msg = &update["message"];
            let chat_id = msg["chat"]["id"].as_i64().unwrap_or(0);
            let text_raw = msg["text"]
                .as_str()
                .or_else(|| msg["caption"].as_str())
                .unwrap_or("")
                .trim();

            let saved_media_opt = download_and_store_telegram_photo(&clean_token, msg).await;

            if chat_id == 0 && (text_raw.is_empty() && saved_media_opt.is_none()) {
                continue;
            }

            // Auto-bind allowed_chat_id if not yet configured
            if config.allowed_chat_id.is_none() {
                config.allowed_chat_id = Some(chat_id);
                config.is_enabled = true;
                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                crate::error::record_ignored(save_config(&config), "save_config");
            }

            if let Some(allowed) = config.allowed_chat_id {
                if chat_id != allowed {
                    continue;
                }
            }

            let reply =
                deliver_ui_or_command(&clean_token, chat_id, text_raw, &saved_media_opt).await;
            if let Some(reply) = reply {
                processed.push((chat_id, text_raw.to_string(), reply));
            }
        }
    }

    // Acknowledge processed updates so they are not re-processed on next call
    if let Some(last_id) = max_update_id {
        let ack_url = format!(
            "https://api.telegram.org/bot{}/getUpdates?offset={}&timeout=0",
            clean_token,
            last_id + 1
        );
        // Justification: best-effort send; failure logged without changing control flow
        crate::error::record_ignored(client.get(&ack_url).send().await, "send via get");
    }

    Ok(processed)
}

/// Start background Telegram inbound polling daemon
pub fn start_telegram_daemon() {
    let is_already_running = TELEGRAM_RUNNING.swap(true, Ordering::SeqCst);
    if is_already_running {
        return;
    }

    tauri::async_runtime::spawn(async move {
        // Enforce 60-second startup quiet period: nothing runs until 1 minute after launch
        tokio::time::sleep(Duration::from_secs(60)).await;
        let mut last_update_id = 0i64;

        loop {
            let is_enabled_flag = {
                let config = load_config().unwrap_or_default();
                config.is_enabled && !config.bot_token.trim().is_empty()
            };

            if !is_enabled_flag {
                {
                    let mut st = LAST_TELEGRAM_STATUS.write().await;
                    st.is_running = false;
                }
                tokio::time::sleep(Duration::from_secs(60)).await;
                continue;
            }

            {
                let mut st = LAST_TELEGRAM_STATUS.write().await;
                st.is_running = true;
                st.last_poll_at = Utc::now().timestamp();
                st.error_message = None;
            }

            let (config, clean_token) = {
                let cfg = load_config().unwrap_or_default();
                let tok = cfg.bot_token.trim().to_string();
                (cfg, tok)
            };

            if clean_token.is_empty() {
                tokio::time::sleep(Duration::from_secs(30)).await;
                continue;
            }

            let http_client = match reqwest::Client::builder()
                .timeout(Duration::from_secs(35))
                .build()
            {
                Ok(c) => c,
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    continue;
                }
            };

            // Long-poll getUpdates with 25-second server timeout
            let poll_url = if last_update_id > 0 {
                format!(
                    "https://api.telegram.org/bot{}/getUpdates?offset={}&timeout=25",
                    clean_token,
                    last_update_id + 1
                )
            } else {
                format!(
                    "https://api.telegram.org/bot{}/getUpdates?timeout=25",
                    clean_token
                )
            };

            let resp_result = http_client.get(&poll_url).send().await;
            match resp_result {
                Ok(resp) => {
                    if resp.status().is_success() {
                        if let Ok(body) = resp.json::<Value>().await {
                            if let Some(updates) = body["result"].as_array() {
                                for update in updates {
                                    if let Some(up_id) = update["update_id"].as_i64() {
                                        if up_id > last_update_id {
                                            last_update_id = up_id;
                                        }
                                    }

                                    // 1. Handle callback query in background loop
                                    if let Some(cb) = update.get("callback_query") {
                                        let cb_id = cb["id"].as_str().unwrap_or("");
                                        let chat_id =
                                            cb["message"]["chat"]["id"].as_i64().unwrap_or(0);
                                        let data = cb["data"].as_str().unwrap_or("").trim();

                                        if chat_id != 0 && !data.is_empty() {
                                            if deliver_send_flow_callback(
                                                &clean_token,
                                                chat_id,
                                                cb_id,
                                                data,
                                                &http_client,
                                            )
                                            .await
                                            {
                                                continue;
                                            }

                                            if config.allowed_chat_id.is_none() {
                                                let mut mutable_config = config.clone();
                                                mutable_config.allowed_chat_id = Some(chat_id);
                                                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                                                crate::error::record_ignored(
                                                    save_config(&mutable_config),
                                                    "save_config",
                                                );
                                            }

                                            if let Some(allowed) = config.allowed_chat_id {
                                                if chat_id == allowed {
                                                    // Same unified pipeline the one-shot poller
                                                    // uses, so `cmd:` buttons behave identically on
                                                    // both paths (exports, UI actions, and the
                                                    // unknown-action fallback included).
                                                    let cmd_str =
                                                        normalize_cmd_callback_payload(data);
                                                    let delivered = deliver_ui_or_command(
                                                        &clean_token,
                                                        chat_id,
                                                        &cmd_str,
                                                        &None,
                                                    )
                                                    .await;

                                                    if delivered.is_none()
                                                        && data.starts_with("cmd:")
                                                    {
                                                        // Justification: a tapped button must never
                                                        // die silently; an unrecognized payload is
                                                        // surfaced to the user and the failure is
                                                        // logged without changing control flow.
                                                        crate::error::record_ignored(
                                                            send_telegram_message_with_markup(
                                                                &clean_token,
                                                                chat_id,
                                                                "That button is no longer valid — send a fresh command to get new buttons.",
                                                                None,
                                                                false,
                                                            )
                                                            .await,
                                                            "send unknown callback action notice",
                                                        );
                                                    }

                                                    if !cb_id.is_empty() {
                                                        let ans_url = format!(
                                                            "https://api.telegram.org/bot{}/answerCallbackQuery",
                                                            clean_token
                                                        );
                                                        // Justification: best-effort send; failure logged without changing control flow
                                                        crate::error::record_ignored(
                                                            http_client
                                                                .post(&ans_url)
                                                                .json(&json!({ "callback_query_id": cb_id }))
                                                                .send()
                                                                .await,
                                                            "answerCallbackQuery",
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                        continue;
                                    }

                                    // 2. Handle standard message
                                    let msg = &update["message"];
                                    let chat_id = msg["chat"]["id"].as_i64().unwrap_or(0);
                                    let text_raw = msg["text"]
                                        .as_str()
                                        .or_else(|| msg["caption"].as_str())
                                        .unwrap_or("")
                                        .trim();

                                    let saved_media_opt =
                                        download_and_store_telegram_photo(&config.bot_token, msg)
                                            .await;

                                    if chat_id == 0
                                        || (text_raw.is_empty() && saved_media_opt.is_none())
                                    {
                                        continue;
                                    }

                                    // Auto-bind allowed_chat_id on first private message if not set
                                    if config.allowed_chat_id.is_none() {
                                        let mut mutable_config = config.clone();
                                        mutable_config.allowed_chat_id = Some(chat_id);
                                        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                                        crate::error::record_ignored(
                                            save_config(&mutable_config),
                                            "save_config",
                                        );
                                    }

                                    // Filter by allowed_chat_id if set
                                    if let Some(allowed) = config.allowed_chat_id {
                                        if chat_id != allowed {
                                            continue;
                                        }
                                    }

                                    {
                                        let mut st = LAST_TELEGRAM_STATUS.write().await;
                                        st.last_message_received = Some(text_raw.to_string());
                                        st.last_update_id = last_update_id;
                                    }

                                    let _ = deliver_ui_or_command(
                                        &config.bot_token,
                                        chat_id,
                                        text_raw,
                                        &saved_media_opt,
                                    )
                                    .await;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let mut st = LAST_TELEGRAM_STATUS.write().await;
                    st.error_message = Some(e.to_string());
                }
            }

            // Yield before next long-poll loop iteration
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });
}

/// Stop background Telegram polling daemon
pub fn stop_telegram_daemon() {
    TELEGRAM_RUNNING.store(false, Ordering::SeqCst);
}

/// Get latest Telegram watcher status
pub async fn get_telegram_status() -> TelegramWatcherStatus {
    LAST_TELEGRAM_STATUS.read().await.clone()
}
