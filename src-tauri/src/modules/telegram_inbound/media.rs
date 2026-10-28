use chrono::Utc;
use rusqlite::params;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::*;

/// Compute SHA-256 hexadecimal hash string for binary data
pub fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Lookup public URL from local media_vault table by SHA-256 hash
pub fn lookup_media_vault_by_sha(sha: &str) -> Option<String> {
    let conn = crate::modules::repo_db::connect_db().ok()?;
    let mut stmt = conn
        .prepare(
            "SELECT public_url FROM media_vault 
             WHERE sha256 = ?1 AND public_url IS NOT NULL AND trim(public_url) != '' 
             LIMIT 1",
        )
        .ok()?;
    let mut rows = stmt.query(params![sha]).ok()?;
    if let Some(row) = rows.next().ok()? {
        let url: String = row.get(0).ok()?;
        if url.starts_with("http") {
            return Some(url);
        }
    }
    None
}

/// Record media metadata into media_vault table
pub fn record_media_vault(
    id: &str,
    sha: &str,
    local_path: Option<&str>,
    public_url: Option<&str>,
    file_id: Option<&str>,
    mime_type: Option<&str>,
) {
    if let Ok(conn) = crate::modules::repo_db::connect_db() {
        let now = Utc::now().timestamp();
        // Justification: best-effort SQLite cache insert; media remains usable even if caching fails
        crate::error::record_ignored(
            conn.execute(
                "INSERT INTO media_vault (id, sha256, local_path, public_url, telegram_file_id, mime_type, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(sha256) DO UPDATE SET
                    local_path = COALESCE(excluded.local_path, media_vault.local_path),
                    public_url = COALESCE(excluded.public_url, media_vault.public_url),
                    telegram_file_id = COALESCE(excluded.telegram_file_id, media_vault.telegram_file_id)",
                params![id, sha, local_path, public_url, file_id, mime_type, now],
            ),
            "upsert media_vault record",
        );
    }
}

/// Compound representation of a media asset across Telegram cloud, public CDN, and local filesystem
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompoundMediaUri {
    pub telegram_file_id: Option<String>,
    pub public_url: Option<String>,
    pub local_path: Option<String>,
}

impl CompoundMediaUri {
    pub fn parse(raw: &str) -> Self {
        let mut uri = Self::default();
        for segment in raw.split('|') {
            let seg = segment.trim();
            if let Some(val) = seg.strip_prefix("tg:") {
                uri.telegram_file_id = Some(val.to_string());
            } else if let Some(val) = seg.strip_prefix("catbox:") {
                uri.public_url = Some(val.to_string());
            } else if let Some(val) = seg.strip_prefix("local:") {
                uri.local_path = Some(val.to_string());
            } else if seg.starts_with("http://") || seg.starts_with("https://") {
                uri.public_url = Some(seg.to_string());
            }
        }
        uri
    }

    pub fn format(&self) -> String {
        let mut parts = Vec::new();
        if let Some(ref tg) = self.telegram_file_id {
            parts.push(format!("tg:{}", tg));
        }
        if let Some(ref url) = self.public_url {
            parts.push(format!("catbox:{}", url));
        }
        if let Some(ref loc) = self.local_path {
            parts.push(format!("local:{}", loc));
        }
        parts.join("|")
    }

    pub fn best_markdown_url(&self) -> Option<&str> {
        self.public_url.as_deref().or(self.local_path.as_deref())
    }
}

/// Send a photo directly to Telegram chat and return the permanent file_id from Telegram Cloud
pub async fn send_telegram_photo(
    bot_token: &str,
    chat_id: i64,
    img_bytes: &[u8],
    caption: &str,
    reply_markup: Option<serde_json::Value>,
) -> Result<String, crate::error::AppError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| {
            crate::error::AppError::Network(format!("HTTP client init failed: {}", e), None)
        })?;

    let part = reqwest::multipart::Part::bytes(img_bytes.to_vec())
        .file_name("image.png")
        .mime_str("image/png")
        .map_err(|e| {
            crate::error::AppError::Network(format!("Invalid multipart part: {}", e), None)
        })?;

    let mut form = reqwest::multipart::Form::new()
        .text("chat_id", chat_id.to_string())
        .part("photo", part);

    if !caption.is_empty() {
        form = form
            .text("caption", caption.to_string())
            .text("parse_mode", "HTML");
    }

    if let Some(markup) = reply_markup {
        let markup_str = serde_json::to_string(&markup).map_err(|e| {
            crate::error::AppError::Unknown(format!("Failed to serialize reply_markup: {}", e))
        })?;
        form = form.text("reply_markup", markup_str);
    }

    let send_photo_url = format!("https://api.telegram.org/bot{}/sendPhoto", bot_token);
    let resp = client.post(&send_photo_url).multipart(form).send().await?;

    let status = resp.status();
    if !status.is_success() {
        let err_text = resp.text().await.unwrap_or_default();
        return Err(crate::error::AppError::Network(
            format!("Telegram sendPhoto returned HTTP {}: {}", status, err_text),
            Some(status.as_u16()),
        ));
    }

    let body: Value = resp.json().await.map_err(|e| {
        crate::error::AppError::Network(
            format!("Failed to parse sendPhoto JSON response: {}", e),
            None,
        )
    })?;

    let photos = body["result"]["photo"].as_array().ok_or_else(|| {
        crate::error::AppError::Unknown("Missing photo array in sendPhoto response".to_string())
    })?;
    let best = photos.last().ok_or_else(|| {
        crate::error::AppError::Unknown("Empty photo array in sendPhoto response".to_string())
    })?;
    let file_id = best["file_id"].as_str().ok_or_else(|| {
        crate::error::AppError::Unknown("Missing file_id in sendPhoto response".to_string())
    })?;

    Ok(file_id.to_string())
}

/// Send a document (.csv, .json, .txt) to Telegram chat via multipart POST /sendDocument
pub async fn send_telegram_document(
    bot_token: &str,
    chat_id: i64,
    filename: &str,
    content_bytes: &[u8],
    caption_html: &str,
    reply_markup: Option<serde_json::Value>,
) -> Result<(), String> {
    let clean_token = bot_token.trim();
    if clean_token.is_empty() {
        return Err("Telegram bot token is empty".to_string());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("HTTP client init failed: {}", e))?;

    let safe_name = if filename.trim().is_empty() {
        "agm-export.csv"
    } else {
        filename.trim()
    };
    let lower = safe_name.to_lowercase();
    // Telegram clients often inline `text/csv` instead of offering a download.
    // `application/octet-stream` forces a real file attachment.
    let mime = if lower.ends_with(".json") {
        "application/json"
    } else if lower.ends_with(".txt") {
        "text/plain"
    } else {
        "application/octet-stream"
    };

    let send_once = |attach_markup: bool, use_html_caption: bool| {
        let part = reqwest::multipart::Part::bytes(content_bytes.to_vec())
            .file_name(safe_name.to_string())
            .mime_str(mime)
            .map_err(|e| format!("Invalid multipart document part: {}", e))?;

        let mut form = reqwest::multipart::Form::new()
            .text("chat_id", chat_id.to_string())
            .part("document", part);

        if !caption_html.trim().is_empty() {
            let sanitized_caption =
                super::formatting::sanitize_telegram_html_entities(caption_html.trim());
            let clamped_caption =
                super::formatting::clamp_telegram_copy_text(&sanitized_caption, 950);
            form = form.text("caption", clamped_caption);
            if use_html_caption {
                form = form.text("parse_mode", "HTML");
            }
        }

        if attach_markup {
            if let Some(ref markup) = reply_markup {
                let markup_str = serde_json::to_string(markup)
                    .map_err(|e| format!("Failed to serialize reply_markup: {}", e))?;
                form = form.text("reply_markup", markup_str);
            }
        }
        Ok::<_, String>(form)
    };

    let send_doc_url = format!("https://api.telegram.org/bot{}/sendDocument", clean_token);
    let attempts: [(bool, bool); 3] = [
        (false, true),
        (false, false),
        (reply_markup.is_some(), false),
    ];
    let mut last_err = String::from("sendDocument failed");
    for (attach_markup, use_html) in attempts {
        let form = match send_once(attach_markup, use_html) {
            Ok(f) => f,
            Err(e) => {
                last_err = e;
                continue;
            }
        };
        match client.post(&send_doc_url).multipart(form).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(body) = resp.json::<Value>().await {
                    if let Some(file_id) = body["result"]["document"]["file_id"].as_str() {
                        let sha = compute_sha256(content_bytes);
                        let id = format!("doc-{}", &sha[..sha.len().min(12)]);
                        record_media_vault(
                            &id,
                            &sha,
                            Some(safe_name),
                            None,
                            Some(file_id),
                            Some(mime),
                        );
                    }
                }
                return Ok(());
            }
            Ok(resp) => {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                last_err = format!("Telegram sendDocument returned HTTP {status}: {err_text}");
            }
            Err(e) => {
                last_err = format!("Telegram sendDocument request failed: {e}");
            }
        }
    }
    Err(last_err)
}

/// Upload an image to Catbox for free, permanent public hosting
pub async fn upload_image_to_catbox(img_bytes: &[u8], filename: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let lower = filename.to_lowercase();
    let mime = if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".svg") {
        "image/svg+xml"
    } else {
        "image/png"
    };

    let part = reqwest::multipart::Part::bytes(img_bytes.to_vec())
        .file_name(filename.to_string())
        .mime_str(mime)
        .map_err(|e| format!("Invalid multipart part: {}", e))?;

    let form = reqwest::multipart::Form::new()
        .text("reqtype", "fileupload")
        .part("fileToUpload", part);

    let resp = client
        .post("https://catbox.moe/user/api.php")
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Catbox upload request failed: {}", e))?;

    if resp.status().is_success() {
        let url = resp.text().await.map_err(|e| e.to_string())?;
        let clean = url.trim().to_string();
        if clean.starts_with("http") {
            Ok(clean)
        } else {
            Err(format!("Catbox returned error: {}", clean))
        }
    } else {
        Err(format!("Catbox HTTP error {}", resp.status()))
    }
}

/// Upload an image to Litterbox (ephemeral fallback hosting if Catbox is unreachable)
pub async fn upload_image_to_litterbox(
    img_bytes: &[u8],
    filename: &str,
    time_retention: &str,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let lower = filename.to_lowercase();
    let mime = if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".svg") {
        "image/svg+xml"
    } else {
        "image/png"
    };

    let part = reqwest::multipart::Part::bytes(img_bytes.to_vec())
        .file_name(filename.to_string())
        .mime_str(mime)
        .map_err(|e| format!("Invalid multipart part: {}", e))?;

    let retention = if time_retention.is_empty() {
        "72h"
    } else {
        time_retention
    };

    let form = reqwest::multipart::Form::new()
        .text("reqtype", "fileupload")
        .text("time", retention.to_string())
        .part("fileToUpload", part);

    let resp = client
        .post("https://litterbox.catbox.moe/resources/internals/api.php")
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Litterbox upload request failed: {}", e))?;

    if resp.status().is_success() {
        let url = resp.text().await.map_err(|e| e.to_string())?;
        let clean = url.trim().to_string();
        if clean.starts_with("http") {
            Ok(clean)
        } else {
            Err(format!("Litterbox returned error: {}", clean))
        }
    } else {
        Err(format!("Litterbox HTTP error {}", resp.status()))
    }
}

/// Upload an image using Catbox first, falling back to Litterbox (ephemeral 72h) if Catbox fails
pub async fn upload_image_with_fallback(
    img_bytes: &[u8],
    filename: &str,
) -> Result<String, String> {
    match upload_image_to_catbox(img_bytes, filename).await {
        Ok(url) => Ok(url),
        Err(e) => {
            // Justification: Catbox upload failed; fallback to Litterbox ephemeral upload
            crate::error::record_ignored(
                Err::<(), _>(format!(
                    "Catbox upload failed: {}, falling back to Litterbox",
                    e
                )),
                "catbox_fallback_to_litterbox",
            );
            upload_image_to_litterbox(img_bytes, filename, "72h").await
        }
    }
}

/// Upload a local image file to Catbox CDN with SQLite deduplication caching
pub async fn upload_local_image_file(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Err(format!("File does not exist: {}", path.display()));
    }

    let bytes = fs::read(path).map_err(|e| format!("Failed to read image file: {}", e))?;
    let sha = compute_sha256(&bytes);

    if let Some(cached_url) = lookup_media_vault_by_sha(&sha) {
        return Ok(cached_url);
    }

    let fname = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "image.png".to_string());

    let public_url = upload_image_with_fallback(&bytes, &fname).await?;
    let id = format!("media-{}", &sha[..sha.len().min(12)]);
    let path_str = path.to_string_lossy().to_string();

    record_media_vault(
        &id,
        &sha,
        Some(&path_str),
        Some(&public_url),
        None,
        Some("image/png"),
    );

    Ok(public_url)
}

/// Download photo from Telegram, store locally, mirror to free CDN, and index in media_vault
pub async fn download_and_store_telegram_photo(
    bot_token: &str,
    msg: &Value,
) -> Option<(PathBuf, Option<String>)> {
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
    if fs::write(&full_path, &img_bytes).is_err() {
        return None;
    }

    let sha = compute_sha256(&img_bytes);
    let public_url = match lookup_media_vault_by_sha(&sha) {
        Some(url) => Some(url),
        None => upload_image_with_fallback(&img_bytes, &filename).await.ok(),
    };

    let id = format!("tg-{}", &file_id[..file_id.len().min(12)]);
    record_media_vault(
        &id,
        &sha,
        Some(&full_path.to_string_lossy()),
        public_url.as_deref(),
        Some(file_id),
        Some("image/png"),
    );

    Some((full_path, public_url))
}

/// Download photo/screenshot from Telegram and persist to disk for future reuse (backward-compatible)
pub async fn download_telegram_photo(bot_token: &str, msg: &Value) -> Option<PathBuf> {
    download_and_store_telegram_photo(bot_token, msg)
        .await
        .map(|(p, _)| p)
}
