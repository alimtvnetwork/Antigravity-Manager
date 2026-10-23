//! Reproducible Curl Command Generator for Debugging Gateway Errors.
//! Synthesizes executable curl CLI invocations from rejected or failed incoming requests.

/// Generates a shell-ready curl command for failed or rejected HTTP requests.
pub fn generate_reproducible_curl(
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: Option<&[u8]>,
) -> String {
    let mut parts = Vec::new();
    parts.push(format!("curl -X {} \"{}\"", method.to_uppercase(), url));

    for (k, v) in headers {
        let key_lower = k.to_lowercase();
        if key_lower == "host" || key_lower == "content-length" {
            continue;
        }

        // Mask tokens partially if authorization header
        let val = if key_lower == "authorization" && v.len() > 16 {
            format!("{}...***", &v[..12])
        } else {
            v.clone()
        };

        parts.push(format!("  -H \"{}: {}\"", k, val.replace('"', "\\\"")));
    }

    if let Some(bytes) = body {
        if !bytes.is_empty() && method.to_uppercase() != "GET" && method.to_uppercase() != "HEAD" {
            if let Ok(utf8_str) = std::str::from_utf8(bytes) {
                let compact = utf8_str.replace('\n', "\\n").replace('"', "\\\"");
                parts.push(format!("  -d \"{}\"", compact));
            } else {
                parts.push(format!(
                    "  --data-binary \"<binary {} bytes>\"",
                    bytes.len()
                ));
            }
        }
    }

    parts.join(" \\\n")
}
