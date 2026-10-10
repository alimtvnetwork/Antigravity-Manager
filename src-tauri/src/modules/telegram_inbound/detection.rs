use crate::error::AppError;
use serde_json::{json, Value};
use std::time::Duration;

use super::*;

/// Automatically detect Telegram Chat ID from recent bot updates (getUpdates)
pub async fn detect_telegram_chat_id(bot_token: &str) -> Result<TelegramDetectedChat, AppError> {
    let clean_token = bot_token.trim();
    if clean_token.is_empty() {
        return Err(AppError::Config("Bot token cannot be empty".to_string()));
    }

    let bot_username = test_telegram_connection(clean_token).await?;

    let url = format!("https://api.telegram.org/bot{}/getUpdates", clean_token);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    if !resp.status().is_success() {
        return Err(AppError::Network(
            format!("Telegram getUpdates failed with status {}", resp.status()),
            Some(resp.status().as_u16()),
        ));
    }

    let body: Value = resp
        .json()
        .await
        .map_err(|e| AppError::Network(e.to_string(), None))?;

    if let Some(updates) = body["result"].as_array() {
        for update in updates.iter().rev() {
            let chat_obj = if update["message"]["chat"]["id"].is_i64() {
                Some(&update["message"]["chat"])
            } else if update["edited_message"]["chat"]["id"].is_i64() {
                Some(&update["edited_message"]["chat"])
            } else if update["my_chat_member"]["chat"]["id"].is_i64() {
                Some(&update["my_chat_member"]["chat"])
            } else if update["channel_post"]["chat"]["id"].is_i64() {
                Some(&update["channel_post"]["chat"])
            } else {
                None
            };

            if let Some(chat) = chat_obj {
                if let Some(cid) = chat["id"].as_i64() {
                    let label = chat["username"]
                        .as_str()
                        .map(|u| format!("@{}", u))
                        .or_else(|| chat["title"].as_str().map(|t| t.to_string()))
                        .or_else(|| chat["first_name"].as_str().map(|f| f.to_string()))
                        .unwrap_or_else(|| cid.to_string());

                    return Ok(TelegramDetectedChat {
                        chat_id: cid,
                        chat_label: label,
                        bot_username,
                    });
                }
            }
        }
    }

    // Check if config on disk already has an allowed_chat_id
    if let Ok(existing) = load_config() {
        if let Some(cid) = existing.allowed_chat_id {
            return Ok(TelegramDetectedChat {
                chat_id: cid,
                chat_label: "Saved Config".to_string(),
                bot_username,
            });
        }
    }

    Err(AppError::Config(format!(
        "No recent messages found for @{}. Open https://t.me/{} in Telegram and send /ping or /start first, then click Auto-Detect Chat ID again.",
        bot_username, bot_username
    )))
}
