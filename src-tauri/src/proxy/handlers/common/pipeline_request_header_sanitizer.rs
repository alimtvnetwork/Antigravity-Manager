//! Pipeline request header sanitizer.
//! Sanitizes incoming request headers across protocol adapters within the pipeline stage.
//! Enforces filtering of hop-by-hop transport headers, internal telemetry leakage,
//! and untrusted client metadata while normalizing protocol casing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Category of header sanitization determining disposition in pipeline dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HeaderSanitizationCategory {
    HopByHopTransport,
    RiskyInternalTelemetry,
    UntrustedClientMetadata,
    AllowedProtocolHeader,
}

impl fmt::Display for HeaderSanitizationCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HopByHopTransport => write!(f, "HopByHopTransport"),
            Self::RiskyInternalTelemetry => write!(f, "RiskyInternalTelemetry"),
            Self::UntrustedClientMetadata => write!(f, "UntrustedClientMetadata"),
            Self::AllowedProtocolHeader => write!(f, "AllowedProtocolHeader"),
        }
    }
}

/// Action taken on an individual header during sanitization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SanitizedHeaderAction {
    Retained,
    Stripped {
        reason: String,
        category: HeaderSanitizationCategory,
    },
    NormalizedCasing {
        original: String,
        canonical: String,
    },
}

impl fmt::Display for SanitizedHeaderAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Retained => write!(f, "retained"),
            Self::Stripped { reason, category } => {
                write!(f, "stripped ({category}): {reason}")
            }
            Self::NormalizedCasing {
                original,
                canonical,
            } => {
                write!(f, "normalized casing: '{original}' -> '{canonical}'")
            }
        }
    }
}

/// Record of an action applied to a specific header key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderSanitizationRecord {
    pub name: String,
    pub action: SanitizedHeaderAction,
}

/// Complete report of the pipeline request header sanitization pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderSanitizationReport {
    pub is_clean: bool,
    pub total_input_headers: usize,
    pub retained_headers: HashMap<String, String>,
    pub stripped_headers: Vec<HeaderSanitizationRecord>,
    pub normalized_headers: Vec<HeaderSanitizationRecord>,
    pub has_violations: bool,
}

/// Standard hop-by-hop transport headers forbidden in pipeline dispatch.
pub const DISALLOWED_HOP_BY_HOP: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
    "upgrade",
    "host",
    "content-length",
];

/// Internal telemetry, proxy routing, or IP metadata headers forbidden from leaking upstream.
pub const DISALLOWED_INTERNAL_METADATA: &[&str] = &[
    "x-goog-user-project",
    "x-antigravity-internal",
    "cf-connecting-ip",
    "x-real-ip",
    "x-forwarded-for",
    "x-forwarded-proto",
];

/// Recognized mandatory upstream protocol header prefixes allowed in pipeline dispatch.
pub const MANDATORY_ALLOWED_PREFIXES: &[&str] = &[
    "x-goog-",
    "authorization",
    "content-type",
    "accept",
    "user-agent",
];

/// Checks if a lowercased header name is a prohibited hop-by-hop transport header.
pub fn is_disallowed_hop_by_hop(lower_name: &str) -> bool {
    DISALLOWED_HOP_BY_HOP.iter().any(|&item| item == lower_name)
}

/// Checks if a lowercased header name contains internal telemetry or routing secrets.
pub fn is_disallowed_internal(lower_name: &str) -> bool {
    DISALLOWED_INTERNAL_METADATA
        .iter()
        .any(|&item| item == lower_name)
        || lower_name.starts_with("x-agm-")
        || lower_name.starts_with("x-internal-")
}

/// Checks if a lowercased header name contains untrusted client metadata or cookies.
pub fn is_untrusted_client_metadata(lower_name: &str) -> bool {
    lower_name.starts_with("cookie") || lower_name.starts_with("sec-ch-")
}

/// Checks if a header name starts with any known mandatory protocol prefix.
pub fn is_mandatory_allowed_prefix(name: &str) -> bool {
    let lower = name.trim().to_ascii_lowercase();
    MANDATORY_ALLOWED_PREFIXES
        .iter()
        .any(|prefix| lower.starts_with(prefix))
}

/// Categorizes a header by its risk tier and protocol disposition.
pub fn categorize_header(name: &str) -> HeaderSanitizationCategory {
    let lower = name.trim().to_ascii_lowercase();
    if is_disallowed_hop_by_hop(&lower) {
        HeaderSanitizationCategory::HopByHopTransport
    } else if is_disallowed_internal(&lower) {
        HeaderSanitizationCategory::RiskyInternalTelemetry
    } else if is_untrusted_client_metadata(&lower) {
        HeaderSanitizationCategory::UntrustedClientMetadata
    } else {
        HeaderSanitizationCategory::AllowedProtocolHeader
    }
}

/// Generates the human-readable stripping explanation for a disallowed category.
fn strip_reason_for_category(category: HeaderSanitizationCategory) -> String {
    match category {
        HeaderSanitizationCategory::HopByHopTransport => {
            "Hop-by-hop transport header forbidden in pipeline dispatch".to_string()
        }
        HeaderSanitizationCategory::RiskyInternalTelemetry => {
            "Internal telemetry or IP forwarding header forbidden in pipeline".to_string()
        }
        HeaderSanitizationCategory::UntrustedClientMetadata => {
            "Untrusted client metadata or cookie header stripped".to_string()
        }
        HeaderSanitizationCategory::AllowedProtocolHeader => String::new(),
    }
}

/// Processes a single header, updating retained, stripped, and normalized collections.
fn process_single_header(
    raw_name: &str,
    val: &str,
    retained: &mut HashMap<String, String>,
    stripped: &mut Vec<HeaderSanitizationRecord>,
    normalized: &mut Vec<HeaderSanitizationRecord>,
) {
    let canonical = raw_name.trim().to_ascii_lowercase();
    let category = categorize_header(&canonical);

    match category {
        HeaderSanitizationCategory::AllowedProtocolHeader => {
            if raw_name != canonical {
                normalized.push(HeaderSanitizationRecord {
                    name: raw_name.to_string(),
                    action: SanitizedHeaderAction::NormalizedCasing {
                        original: raw_name.to_string(),
                        canonical: canonical.clone(),
                    },
                });
            }
            retained.insert(canonical, val.to_string());
        }
        _ => {
            let reason = strip_reason_for_category(category);
            stripped.push(HeaderSanitizationRecord {
                name: raw_name.to_string(),
                action: SanitizedHeaderAction::Stripped { reason, category },
            });
        }
    }
}

/// Sanitizes incoming pipeline request headers and returns a comprehensive report.
pub fn sanitize_pipeline_request_headers(
    headers: &HashMap<String, String>,
) -> HeaderSanitizationReport {
    let mut retained_headers = HashMap::new();
    let mut stripped_headers = Vec::new();
    let mut normalized_headers = Vec::new();

    let mut sorted_entries: Vec<(&String, &String)> = headers.iter().collect();
    sorted_entries.sort_by(|a, b| a.0.cmp(b.0));

    for (raw_name, val) in sorted_entries {
        process_single_header(
            raw_name,
            val,
            &mut retained_headers,
            &mut stripped_headers,
            &mut normalized_headers,
        );
    }

    let is_clean = stripped_headers.is_empty();
    let has_violations = !is_clean;

    HeaderSanitizationReport {
        is_clean,
        total_input_headers: headers.len(),
        retained_headers,
        stripped_headers,
        normalized_headers,
        has_violations,
    }
}
