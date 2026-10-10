const MIN_SIGNATURE_LENGTH: usize = 32;

use super::*;

pub fn is_real_signature(sig: &str) -> bool {
    let s = sig.trim();
    if s == SENTINEL_SIGNATURE {
        return false;
    }
    if s.as_bytes().first() == Some(&0x12) && s.len() >= 16 {
        return true;
    }
    s.len() >= MIN_SIGNATURE_LENGTH
}

/// 判断签名是否符合 Google Gemini 原生 Protobuf 签名特征：
/// 1. 官方跳过验签哨兵 (skip_thought_signature_validator)；
/// 2. 或满足有效长度 (>= MIN_SIGNATURE_LENGTH)，且 Base64 解码后首字节为 Protobuf Tag 2 (0x12)
///    (单层 Base64 通常以 'E' 开头，双层 Base64 包装通常以 'R' 开头)
pub fn is_likely_gemini_signature(sig: &str) -> bool {
    let s = sig.trim();
    if s == SENTINEL_SIGNATURE {
        return true;
    }
    // Claude 签名绝不能被误判为 Gemini 签名
    if is_claude_signature(s) {
        return false;
    }
    // 兼容历史脏数据中被误解码为原始二进制 Protobuf (首字节 0x12) 的签名
    if s.as_bytes().first() == Some(&0x12) && s.len() >= 16 {
        return true;
    }
    if s.len() < MIN_SIGNATURE_LENGTH {
        return false;
    }
    if !s.starts_with('E') && !s.starts_with('R') {
        return false;
    }
    use base64::Engine;
    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(s) {
        if decoded.first() == Some(&0x12) {
            return true;
        }
        // 双层 Base64 包装支持（Google Vertex AI 格式）
        if let Ok(inner_str) = std::str::from_utf8(&decoded) {
            if inner_str.starts_with('E') {
                if let Ok(inner) = base64::engine::general_purpose::STANDARD.decode(inner_str) {
                    if inner.first() == Some(&0x12) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// 判断签名是否属于 Claude 家族的签名
pub fn is_claude_signature(sig: &str) -> bool {
    let s = sig.trim();
    if s.is_empty() || s == SENTINEL_SIGNATURE {
        return false;
    }
    use base64::Engine;
    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(s) {
        if decoded
            .windows(6)
            .any(|w| w.eq_ignore_ascii_case(b"claude"))
        {
            return true;
        }
        if let Ok(inner) = base64::engine::general_purpose::STANDARD.decode(&decoded) {
            if inner.windows(6).any(|w| w.eq_ignore_ascii_case(b"claude")) {
                return true;
            }
        }
    }
    false
}

/// 将 Claude 签名正规化为发送给 Google Vertex AI 接口所需的格式
/// Google 的 REST API 对 bytes 字段会自动执行 base64_decode，
/// 因此发往 Google 的 thoughtSignature 必须是 ASCII 签名字节的 Base64 编码 (即 "RXU4..." 格式)
pub fn ensure_google_claude_thought_signature(sig: &str) -> String {
    let s = sig.trim();
    if s.is_empty() || s == SENTINEL_SIGNATURE {
        return s.to_string();
    }
    use base64::Engine;
    // 如果已经由 Base64 包装过（即 base64 decode 出来能再解出 b"claude"），无需重复包装
    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(s) {
        if let Ok(inner) = base64::engine::general_purpose::STANDARD.decode(&decoded) {
            if inner.windows(6).any(|w| w.eq_ignore_ascii_case(b"claude")) {
                return s.to_string();
            }
        }
    }
    // 只有在当前签名确实是原始 Claude 客户端签名（解码一层后包含 b"claude"）时才进行一次 Base64 包装！
    // 严禁对非 Claude 签名或未知字符串无节制再包装，彻底阻断几何级膨胀死循环。
    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(s) {
        if decoded
            .windows(6)
            .any(|w| w.eq_ignore_ascii_case(b"claude"))
        {
            return base64::engine::general_purpose::STANDARD.encode(s.as_bytes());
        }
    }
    s.to_string()
}

/// 将 Claude 签名还原为客户端（Claude Code / Anthropic SDK）期望的原生格式 (Eu8...)
pub fn ensure_raw_claude_thought_signature(sig: &str) -> String {
    let s = sig.trim();
    if s.is_empty() || s == SENTINEL_SIGNATURE {
        return s.to_string();
    }
    use base64::Engine;
    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(s) {
        if let Ok(inner) = base64::engine::general_purpose::STANDARD.decode(&decoded) {
            if inner.windows(6).any(|w| w.eq_ignore_ascii_case(b"claude")) {
                if let Ok(raw_s) = String::from_utf8(decoded) {
                    return raw_s;
                }
            }
        }
    }
    s.to_string()
}

/// 用于 ThinkingStore / SignatureCache 内部的比对与哈希：
/// 统一归一化为原始客户端签名形式 (Eu8...)，使 "RXU4..." 与 "Eu8..." 判定为相同签名
pub fn normalize_signature_for_comparison(sig: &str) -> std::borrow::Cow<'_, str> {
    use base64::Engine;
    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(sig) {
        if let Ok(inner) = base64::engine::general_purpose::STANDARD.decode(&decoded) {
            if inner.windows(6).any(|w| w == b"claude") {
                if let Ok(s) = String::from_utf8(decoded) {
                    return std::borrow::Cow::Owned(s);
                }
            }
        }
    }
    std::borrow::Cow::Borrowed(sig)
}

pub fn signatures_match(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    normalize_signature_for_comparison(a) == normalize_signature_for_comparison(b)
}
