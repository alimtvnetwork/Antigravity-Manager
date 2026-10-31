//! TLS Server Name Indication (SNI) Hostname Sanitizer & IDN Normalizer
//!
//! Provides validation and normalization for upstream hostnames before TLS handshakes,
//! stripping port numbers, URL scheme prefixes, path fragments, and validating
//! RFC 1035 / RFC 5890 domain label constraints.

use tracing::debug;

/// Maximum allowed length for an FQDN in characters according to RFC 1035.
pub const MAX_HOSTNAME_LEN: usize = 253;

/// Maximum allowed length for a single DNS label.
pub const MAX_DNS_LABEL_LEN: usize = 63;

/// Result of TLS SNI hostname sanitization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizedSni {
    /// The normalized hostname suitable for TLS SNI.
    pub hostname: String,
    /// Extracted port if specified in the raw input.
    pub port: Option<u16>,
    /// Whether the hostname contains an internationalized domain name (Punycode / xn--).
    pub is_idn: bool,
    /// Whether the target is an IP literal rather than a domain name.
    pub is_ip_literal: bool,
}

/// Sanitizes a raw upstream endpoint string into a clean SNI hostname.
///
/// Strips `https://` / `http://` prefixes, trailing slashes, port suffixes,
/// validates DNS label characters, and normalizes ASCII casing.
pub fn sanitize_tls_sni(raw_host: &str) -> Result<SanitizedSni, String> {
    let trimmed = raw_host.trim();
    if trimmed.is_empty() {
        return Err("SNI hostname cannot be empty".to_string());
    }

    // Strip scheme prefix if present
    let host_without_scheme = if let Some(stripped) = trimmed.strip_prefix("https://") {
        stripped
    } else if let Some(stripped) = trimmed.strip_prefix("http://") {
        stripped
    } else {
        trimmed
    };

    // Strip any trailing path, query or fragment
    let host_only = host_without_scheme
        .split('/')
        .next()
        .unwrap_or(host_without_scheme)
        .split('?')
        .next()
        .unwrap_or(host_without_scheme);

    // Extract optional port
    let (hostname_raw, port) = if let Some((h, p_str)) = host_only.rsplit_once(':') {
        // Guard against IPv6 literals without brackets
        if h.contains(':') && !h.starts_with('[') {
            (host_only, None)
        } else {
            match p_str.parse::<u16>() {
                Ok(p) => (h, Some(p)),
                Err(_) => (host_only, None),
            }
        }
    } else {
        (host_only, None)
    };

    // Strip enclosing brackets for IPv6 literals
    let unbracketed = hostname_raw
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .unwrap_or(hostname_raw);

    // Check for IP literal (IPv4 or IPv6)
    let is_ip_literal = unbracketed.parse::<std::net::IpAddr>().is_ok();

    // Check length bounds
    if unbracketed.len() > MAX_HOSTNAME_LEN {
        return Err(format!(
            "Hostname exceeds maximum length of {} characters: {} bytes",
            MAX_HOSTNAME_LEN,
            unbracketed.len()
        ));
    }

    // Remove any trailing dot for FQDN normalization
    let normalized_fqdn = unbracketed.strip_suffix('.').unwrap_or(unbracketed);
    let lowercased = normalized_fqdn.to_ascii_lowercase();

    // Validate DNS labels if not an IP literal
    let mut is_idn = false;
    if !is_ip_literal {
        for label in lowercased.split('.') {
            if label.is_empty() {
                return Err("Hostname contains empty DNS label".to_string());
            }
            if label.len() > MAX_DNS_LABEL_LEN {
                return Err(format!(
                    "DNS label exceeds maximum length of {} characters: '{}'",
                    MAX_DNS_LABEL_LEN, label
                ));
            }
            if label.starts_with('-') || label.ends_with('-') {
                return Err(format!(
                    "DNS label cannot start or end with a hyphen: '{}'",
                    label
                ));
            }
            if label.starts_with("xn--") {
                is_idn = true;
            }
            for c in label.chars() {
                if !c.is_ascii_alphanumeric() && c != '-' && c != '_' {
                    return Err(format!(
                        "Invalid character '{}' in hostname label '{}'",
                        c, label
                    ));
                }
            }
        }
    }

    debug!(
        raw = raw_host,
        sanitized = %lowercased,
        port = ?port,
        is_idn = is_idn,
        is_ip = is_ip_literal,
        "Sanitized TLS SNI hostname"
    );

    Ok(SanitizedSni {
        hostname: lowercased,
        port,
        is_idn,
        is_ip_literal,
    })
}
