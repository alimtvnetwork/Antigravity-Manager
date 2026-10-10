use crate::error::AppError;
use chrono::Utc;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use super::*;

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

            let msg = &update["message"];
            let chat_id = msg["chat"]["id"].as_i64().unwrap_or(0);
            let text_raw = msg["text"]
                .as_str()
                .or_else(|| msg["caption"].as_str())
                .unwrap_or("")
                .trim();

            let saved_photo_opt = download_telegram_photo(&clean_token, msg).await;

            if chat_id == 0 || (text_raw.is_empty() && saved_photo_opt.is_none()) {
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

            let reply_opt = if !text_raw.is_empty() {
                let base_reply = process_telegram_command_text(text_raw).await;
                if let Some(img_path) = saved_photo_opt.as_ref() {
                    let banner = format!(
                        "📸 <b>Image Saved:</b> <code>{}</code>\n\n",
                        img_path.display()
                    );
                    base_reply.map(|r| format!("{}{}", banner, r))
                } else {
                    base_reply
                }
            } else if let Some(img_path) = saved_photo_opt.as_ref() {
                Some(format!(
                    "📸 <b>Screenshot Saved Successfully</b>\n\n\
                    • <b>Path:</b> <code>{}</code>\n\n\
                    💡 To execute with instruction: <code>/prompt &lt;project&gt; \"instruction\"</code>",
                    img_path.display()
                ))
            } else {
                None
            };

            if let Some(reply) = reply_opt {
                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                crate::error::record_ignored(
                    send_telegram_message(&clean_token, chat_id, &reply).await,
                    "send_telegram_message",
                );
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
            }

            let mut config = match load_config() {
                Ok(c) => c,
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            let poll_url = format!(
                "https://api.telegram.org/bot{}/getUpdates?offset={}&timeout=20",
                config.bot_token.trim(),
                last_update_id + 1
            );

            let http_client = match reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
            {
                Ok(c) => c,
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
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

                                    let msg = &update["message"];
                                    let chat_id = msg["chat"]["id"].as_i64().unwrap_or(0);
                                    let text_raw = msg["text"]
                                        .as_str()
                                        .or_else(|| msg["caption"].as_str())
                                        .unwrap_or("")
                                        .trim();

                                    let saved_photo_opt =
                                        download_telegram_photo(&config.bot_token, msg).await;

                                    if chat_id == 0
                                        || (text_raw.is_empty() && saved_photo_opt.is_none())
                                    {
                                        continue;
                                    }

                                    // Auto-bind allowed_chat_id on first private message if not set
                                    if config.allowed_chat_id.is_none() {
                                        config.allowed_chat_id = Some(chat_id);
                                        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                                        crate::error::record_ignored(
                                            save_config(&config),
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

                                    let reply_opt = if !text_raw.is_empty() {
                                        let base_reply =
                                            process_telegram_command_text(text_raw).await;
                                        if let Some(img_path) = saved_photo_opt.as_ref() {
                                            let banner = format!(
                                                "📸 <b>Image Saved:</b> <code>{}</code>\n\n",
                                                img_path.display()
                                            );
                                            base_reply.map(|r| format!("{}{}", banner, r))
                                        } else {
                                            base_reply
                                        }
                                    } else if let Some(img_path) = saved_photo_opt.as_ref() {
                                        Some(format!(
                                            "📸 <b>Screenshot Saved Successfully</b>\n\n\
                                            • <b>Path:</b> <code>{}</code>\n\n\
                                            💡 To execute with instruction: <code>/prompt &lt;project&gt; \"instruction\"</code>",
                                            img_path.display()
                                        ))
                                    } else {
                                        None
                                    };

                                    if let Some(reply) = reply_opt {
                                        // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                                        crate::error::record_ignored(
                                            send_telegram_message(
                                                &config.bot_token,
                                                chat_id,
                                                &reply,
                                            )
                                            .await,
                                            "send_telegram_message",
                                        );
                                    }
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

            tokio::time::sleep(Duration::from_secs(config.poll_interval_secs.max(2))).await;
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
