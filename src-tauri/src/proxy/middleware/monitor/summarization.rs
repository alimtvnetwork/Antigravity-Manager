use super::*;

/// Request/response summarization helpers (multipart, images).
///
/// Pure code motion from `monitor.rs` — no logic changes.
pub(crate) fn summarize_multipart_request(
    bytes: &[u8],
    content_type: &str,
    uri: &str,
) -> Option<(String, Option<String>)> {
    let boundary = extract_boundary(content_type)?;
    let marker = format!("--{}", boundary).into_bytes();
    let mut cursor = find_subslice(bytes, &marker, 0)?;
    let mut fields = serde_json::Map::new();
    let mut files = Vec::new();
    let mut model = None;

    loop {
        cursor += marker.len();
        if cursor >= bytes.len() || bytes[cursor..].starts_with(b"--") {
            break;
        }
        if bytes[cursor..].starts_with(b"\r\n") {
            cursor += 2;
        } else if bytes[cursor..].starts_with(b"\n") {
            cursor += 1;
        }

        let Some(next_boundary) = find_subslice(bytes, &marker, cursor) else {
            break;
        };
        let part = trim_part_tail(&bytes[cursor..next_boundary]);
        cursor = next_boundary;

        let (headers, body) = if let Some(idx) = find_subslice(part, b"\r\n\r\n", 0) {
            (&part[..idx], &part[idx + 4..])
        } else if let Some(idx) = find_subslice(part, b"\n\n", 0) {
            (&part[..idx], &part[idx + 2..])
        } else {
            continue;
        };

        let headers_text = String::from_utf8_lossy(headers);
        let disposition = headers_text
            .lines()
            .find(|line| {
                line.to_ascii_lowercase()
                    .starts_with("content-disposition:")
            })
            .unwrap_or("");
        let content_type = headers_text
            .lines()
            .find(|line| line.to_ascii_lowercase().starts_with("content-type:"))
            .and_then(|line| {
                line.split_once(':')
                    .map(|(_, value)| value.trim().to_string())
            });
        let Some(name) = extract_quoted_param(disposition, "name") else {
            continue;
        };
        let filename = extract_quoted_param(disposition, "filename");

        if filename.is_some() || content_type.as_deref().unwrap_or("").starts_with("image/") {
            files.push(serde_json::json!({
                "field": name,
                "filename": filename,
                "content_type": content_type,
                "bytes": body.len()
            }));
        } else if let Ok(text) = std::str::from_utf8(body) {
            let value = truncate_for_log(text.trim(), MAX_LOGGED_FIELD_CHARS);
            if name == "model" {
                model = Some(value.clone());
            }
            fields.insert(name, Value::String(value));
        } else {
            fields.insert(
                name,
                Value::String(format!("[binary field: {} bytes]", body.len())),
            );
        }
    }

    let summary = serde_json::json!({
        "content_type": "multipart/form-data",
        "path": uri,
        "fields": fields,
        "files": files,
        "raw_bytes": bytes.len()
    });
    let rendered = serde_json::to_string_pretty(&summary).ok()?;
    Some((rendered, model))
}

pub(crate) fn image_mime_and_dimensions(bytes: &[u8]) -> (String, Option<(u32, u32)>) {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") && bytes.len() >= 24 {
        let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        return ("image/png".to_string(), Some((width, height)));
    }

    if bytes.starts_with(b"\xff\xd8") {
        let mut idx = 2;
        while idx + 9 < bytes.len() {
            if bytes[idx] != 0xff {
                idx += 1;
                continue;
            }
            let marker = bytes[idx + 1];
            if marker == 0xd9 || marker == 0xda {
                break;
            }
            if idx + 4 > bytes.len() {
                break;
            }
            let segment_len = u16::from_be_bytes([bytes[idx + 2], bytes[idx + 3]]) as usize;
            if segment_len < 2 || idx + 2 + segment_len > bytes.len() {
                break;
            }
            if matches!(
                marker,
                0xc0 | 0xc1
                    | 0xc2
                    | 0xc3
                    | 0xc5
                    | 0xc6
                    | 0xc7
                    | 0xc9
                    | 0xca
                    | 0xcb
                    | 0xcd
                    | 0xce
                    | 0xcf
            ) && segment_len >= 7
            {
                let height = u16::from_be_bytes([bytes[idx + 5], bytes[idx + 6]]) as u32;
                let width = u16::from_be_bytes([bytes[idx + 7], bytes[idx + 8]]) as u32;
                return ("image/jpeg".to_string(), Some((width, height)));
            }
            idx += 2 + segment_len;
        }
        return ("image/jpeg".to_string(), None);
    }

    ("application/octet-stream".to_string(), None)
}

pub(crate) fn summarize_image_json_response(json: &Value) -> Option<String> {
    let mut summary = json.clone();
    let data = summary.get_mut("data")?.as_array_mut()?;
    for item in data.iter_mut() {
        if let Some(obj) = item.as_object_mut() {
            if let Some(b64) = obj.get("b64_json").and_then(|v| v.as_str()) {
                let decoded = base64::engine::general_purpose::STANDARD.decode(b64).ok();
                let (mime_type, dimensions, bytes) = decoded
                    .as_deref()
                    .map(|bytes| {
                        let (mime, dims) = image_mime_and_dimensions(bytes);
                        (mime, dims, bytes.len())
                    })
                    .unwrap_or_else(|| ("unknown".to_string(), None, b64.len() * 3 / 4));
                obj.remove("b64_json");
                obj.insert(
                    "image".to_string(),
                    serde_json::json!({
                        "encoding": "base64",
                        "redacted": true,
                        "mime_type": mime_type,
                        "bytes": bytes,
                        "dimensions": dimensions.map(|(width, height)| serde_json::json!({
                            "width": width,
                            "height": height
                        }))
                    }),
                );
            } else if let Some(url) = obj.get("url").and_then(|v| v.as_str()) {
                if url.starts_with("data:image/") {
                    obj.insert(
                        "url".to_string(),
                        Value::String(format!("[data URL redacted: {} chars]", url.len())),
                    );
                }
            }
        }
    }
    serde_json::to_string_pretty(&summary).ok()
}
