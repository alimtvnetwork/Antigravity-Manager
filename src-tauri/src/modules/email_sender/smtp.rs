use base64::prelude::*;
use std::io::{Read, Write};

use super::*;

/// Send single SMTP command line with optional credential redaction on error
pub(crate) fn send_smtp_cmd(
    stream: &mut EmailStream,
    cmd: &str,
    is_sensitive: bool,
) -> Result<(), String> {
    let line = format!("{}\r\n", cmd);
    stream.write_all(line.as_bytes()).map_err(|e| {
        if is_sensitive {
            format!("Failed to send SMTP command '<REDACTED>': {}", e)
        } else {
            format!("Failed to send SMTP command '{}': {}", cmd, e)
        }
    })
}

/// Read SMTP response and verify status code (< 400 is success)
pub(crate) fn read_smtp_response(stream: &mut EmailStream) -> Result<String, String> {
    let mut total_resp = String::new();
    let mut buf = [0u8; 1024];

    loop {
        let n = stream
            .read(&mut buf)
            .map_err(|e| format!("Failed to read SMTP response: {}", e))?;
        if n == 0 {
            break;
        }
        let chunk = String::from_utf8_lossy(&buf[0..n]);
        total_resp.push_str(&chunk);

        let mut is_done = false;
        for line in total_resp.lines() {
            let trimmed = line.trim_start();
            if trimmed.len() == 3 {
                let is_digits = trimmed.chars().all(|c| c.is_ascii_digit());
                if is_digits {
                    is_done = true;
                }
            }
            if trimmed.len() >= 4 {
                let code_part = &trimmed[0..3];
                let sep = &trimmed[3..4];
                let is_digits = code_part.chars().all(|c| c.is_ascii_digit());
                if is_digits {
                    if sep == " " {
                        is_done = true;
                    }
                }
            }
        }
        if is_done {
            break;
        }
    }

    if total_resp.is_empty() {
        return Err("Empty response from SMTP server".to_string());
    }

    let mut last_code = 200u16;
    for line in total_resp.lines() {
        let trimmed = line.trim_start();
        if trimmed.len() >= 3 {
            if let Ok(code) = trimmed[0..3].parse::<u16>() {
                last_code = code;
            }
        }
    }
    if last_code >= 400 {
        return Err(format!("SMTP server error: {}", total_resp.trim()));
    }
    Ok(total_resp)
}
