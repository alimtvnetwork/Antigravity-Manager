//! Deep JSON Payload Sanitizer
//! Sanitizes incoming JSON payloads by stripping unescaped control characters
//! and malformed delimiters before parsing and protocol forwarding.

pub struct JsonPayloadSanitizer;

impl JsonPayloadSanitizer {
    /// Strips invalid unescaped ASCII control characters (< 0x20 except \t, \n, \r)
    /// that cause standard JSON deserializers to fail with unescaped control character errors.
    pub fn sanitize_raw_json(raw: &str) -> String {
        let mut sanitized = String::with_capacity(raw.len());
        let mut in_string = false;
        let mut is_escaped = false;

        for ch in raw.chars() {
            if in_string {
                if is_escaped {
                    sanitized.push(ch);
                    is_escaped = false;
                } else if ch == '\\' {
                    sanitized.push(ch);
                    is_escaped = true;
                } else if ch == '"' {
                    sanitized.push(ch);
                    in_string = false;
                } else if (ch as u32) < 0x20 && ch != '\t' && ch != '\n' && ch != '\r' {
                    // Replace raw unescaped control byte with unicode escape sequence
                    sanitized.push_str(&format!("\\u{:04x}", ch as u32));
                } else {
                    sanitized.push(ch);
                }
            } else {
                if ch == '"' {
                    in_string = true;
                }
                sanitized.push(ch);
            }
        }

        sanitized
    }

    /// Strips trailing commas immediately preceding closing braces '}' or ']'
    pub fn strip_trailing_commas(raw: &str) -> String {
        let mut result = String::with_capacity(raw.len());
        let mut last_comma_idx: Option<usize> = None;
        let mut in_string = false;
        let mut is_escaped = false;

        for ch in raw.chars() {
            if in_string {
                if is_escaped {
                    is_escaped = false;
                } else if ch == '\\' {
                    is_escaped = true;
                } else if ch == '"' {
                    in_string = false;
                }
                result.push(ch);
            } else {
                if ch == '"' {
                    in_string = true;
                    last_comma_idx = None;
                    result.push(ch);
                } else if ch == ',' {
                    last_comma_idx = Some(result.len());
                    result.push(ch);
                } else if (ch == '}' || ch == ']') && last_comma_idx.is_some() {
                    // Remove the trailing comma
                    if let Some(comma_pos) = last_comma_idx {
                        result.remove(comma_pos);
                    }
                    last_comma_idx = None;
                    result.push(ch);
                } else {
                    if !ch.is_whitespace() {
                        last_comma_idx = None;
                    }
                    result.push(ch);
                }
            }
        }

        result
    }

    /// Full sanitation pipeline: control characters and trailing commas.
    pub fn sanitize_and_clean(raw: &str) -> String {
        let step1 = Self::sanitize_raw_json(raw);
        Self::strip_trailing_commas(&step1)
    }
}
