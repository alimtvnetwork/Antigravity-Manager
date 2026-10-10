use chrono::Utc;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use super::*;

/// Download photo/screenshot from Telegram and persist to disk for future reuse
pub async fn download_telegram_photo(bot_token: &str, msg: &Value) -> Option<PathBuf> {
    let photo_arr = msg["photo"].as_array()?;
    let best_photo = photo_arr.last()?;
    let file_id = best_photo["file_id"].as_str()?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .ok()?;

    let get_file_url = format!(
        "https://api.telegram.org/bot{}/getFile?file_id={}",
        bot_token, file_id
    );
    let resp: Value = client
        .get(&get_file_url)
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    let file_path = resp["result"]["file_path"].as_str()?;

    let download_url = format!(
        "https://api.telegram.org/file/bot{}/{}",
        bot_token, file_path
    );
    let img_bytes = client
        .get(&download_url)
        .send()
        .await
        .ok()?
        .bytes()
        .await
        .ok()?;

    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    let target_dir = PathBuf::from(home)
        .join(".antigravity_tools")
        .join("saved_images");
    // Justification: best-effort directory creation; later file ops fail loudly if the directory is actually needed
    crate::error::record_ignored(fs::create_dir_all(&target_dir), "create_dir_all");

    let ext = PathBuf::from(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_string();
    let filename = format!(
        "telegram_{}_{}.{}",
        chrono::Utc::now().format("%Y%m%d_%H%M%S"),
        &file_id[..file_id.len().min(8)],
        ext
    );
    let full_path = target_dir.join(&filename);
    if fs::write(&full_path, &img_bytes).is_ok() {
        Some(full_path)
    } else {
        None
    }
}
