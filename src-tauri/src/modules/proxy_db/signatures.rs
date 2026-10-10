use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rusqlite::{params, Connection, OpenFlags};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};

use super::*;

pub(crate) static LOG_WRITE_LOCK: Mutex<()> = Mutex::new(());

pub(crate) static TOOL_SIGNATURE_DB: OnceLock<Mutex<Option<(PathBuf, Connection)>>> =
    OnceLock::new();

pub(crate) const THOUGHT_RAW_MAGIC: &[u8] = b"RAW1";

pub(crate) const THOUGHT_GZIP_MAGIC: &[u8] = b"AGZ1";

pub(crate) const MIN_GZIP_THOUGHT: usize = 384;

pub const SENTINEL_SIGNATURE: &str = "skip_thought_signature_validator";

pub const MIN_REAL_SIGNATURE: usize = 32;

pub fn normalize_and_heal_signature(sig: &str) -> Option<String> {
    if sig.is_empty() || sig == SENTINEL_SIGNATURE {
        return None;
    }
    // 自愈防裂化：若签名被误传或脏存储为原始 Protobuf 二进制 (首字节 0x12)，自动纠正编码为标准 Base64
    let normalized = if sig.as_bytes().first() == Some(&0x12) {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(sig.as_bytes())
    } else {
        sig.to_string()
    };
    if normalized.len() >= MIN_REAL_SIGNATURE {
        Some(normalized)
    } else {
        None
    }
}

pub(crate) fn persist_signature(signature: Option<&str>) -> Option<String> {
    signature.and_then(normalize_and_heal_signature)
}

/// Tool turns match by tool_id at fill time — visible/tool_names are in the request JSON.
/// Only text-only turns keep visible so prefix matching still works after restart.
pub(crate) fn persist_visible<'a>(tool_ids: &[String], visible: &'a str) -> &'a str {
    if tool_ids.is_empty() {
        visible
    } else {
        ""
    }
}

pub(crate) fn pack_thought(s: &str) -> Vec<u8> {
    if s.len() >= MIN_GZIP_THOUGHT {
        let mut enc = GzEncoder::new(Vec::with_capacity(s.len() / 2), Compression::fast());
        if enc.write_all(s.as_bytes()).is_ok() {
            if let Ok(buf) = enc.finish() {
                if buf.len() + THOUGHT_GZIP_MAGIC.len() < s.len() {
                    let mut out = Vec::with_capacity(THOUGHT_GZIP_MAGIC.len() + buf.len());
                    out.extend_from_slice(THOUGHT_GZIP_MAGIC);
                    out.extend_from_slice(&buf);
                    return out;
                }
            }
        }
    }
    let mut out = Vec::with_capacity(THOUGHT_RAW_MAGIC.len() + s.len());
    out.extend_from_slice(THOUGHT_RAW_MAGIC);
    out.extend_from_slice(s.as_bytes());
    out
}

pub(crate) fn unpack_thought(bytes: &[u8]) -> String {
    if let Some(rest) = bytes.strip_prefix(THOUGHT_GZIP_MAGIC) {
        let mut decoder = GzDecoder::new(rest);
        let mut s = String::new();
        if decoder.read_to_string(&mut s).is_ok() {
            return s;
        }
    }
    if let Some(rest) = bytes.strip_prefix(THOUGHT_RAW_MAGIC) {
        return String::from_utf8_lossy(rest).into_owned();
    }
    String::from_utf8_lossy(bytes).into_owned()
}
