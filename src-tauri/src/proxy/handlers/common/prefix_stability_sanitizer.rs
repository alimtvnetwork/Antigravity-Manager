//! Pipeline Prefix Stability & Header Sanitizer (`Imp-625`)
//!
//! Enforces prefix stability for prompt caching and sanitizes internal,
//! forbidden, or telemetry headers before upstream dispatch.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Internal proxy and telemetry headers strictly forbidden from upstream dispatch.
const FORBIDDEN_INTERNAL_HEADERS: &[&str] = &[
    "x-agm-internal",
    "x-antigravity-internal",
    "x-proxy-forwarded",
];

/// Disallowed Google SDK/API headers stripped for prefix stability.
const DISALLOWED_GOOG_HEADERS: &[&str] = &["x-goog-api-client", "x-goog-api-key"];

/// Policy controlling request header sanitization and deterministic formatting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrefixStabilityPolicy {
    /// Whether to strip disallowed Google headers (e.g. x-goog-api-client, x-goog-api-key).
    pub strip_disallowed_goog_headers: bool,
    /// Custom headers that are explicitly allowed even if matching general block patterns.
    pub allowed_custom_headers: Vec<String>,
    /// Whether JSON keys should be deterministically sorted.
    pub deterministic_key_order: bool,
}

impl Default for PrefixStabilityPolicy {
    fn default() -> Self {
        Self {
            strip_disallowed_goog_headers: true,
            allowed_custom_headers: Vec::new(),
            deterministic_key_order: true,
        }
    }
}

impl PrefixStabilityPolicy {
    /// Creates a new policy with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets allowed custom headers.
    pub fn with_allowed_headers(mut self, headers: Vec<String>) -> Self {
        self.allowed_custom_headers = headers;
        self
    }

    /// Sets whether to strip disallowed Google headers.
    pub fn with_strip_disallowed_goog_headers(mut self, strip: bool) -> Self {
        self.strip_disallowed_goog_headers = strip;
        self
    }
}

/// Report summarizing the outcome of header sanitization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrefixSanitizationReport {
    /// Indicates whether the header set required no strip modifications.
    pub is_stable: bool,
    /// List of header names that were stripped (sorted for determinism).
    pub stripped_headers: Vec<String>,
    /// Number of retained normalized headers.
    pub normalized_headers_count: usize,
    /// Positive flag indicating whether any headers were stripped.
    pub has_stripped_headers: bool,
}

impl PrefixSanitizationReport {
    /// Constructs a new report from stripped headers and normalized count.
    pub fn new(mut stripped_headers: Vec<String>, normalized_headers_count: usize) -> Self {
        stripped_headers.sort();
        let has_stripped_headers = !stripped_headers.is_empty();
        let is_stable = !has_stripped_headers;
        Self {
            is_stable,
            stripped_headers,
            normalized_headers_count,
            has_stripped_headers,
        }
    }
}

/// Checks if a lowercase header name matches internal forbidden patterns.
fn is_forbidden_internal_header(lower_name: &str) -> bool {
    FORBIDDEN_INTERNAL_HEADERS
        .iter()
        .any(|prefix| lower_name == *prefix || lower_name.starts_with(&format!("{prefix}-")))
}

/// Checks if a lowercase header name matches disallowed Google headers.
fn is_disallowed_goog_header(lower_name: &str, strip_goog: bool) -> bool {
    if !strip_goog {
        return false;
    }
    DISALLOWED_GOOG_HEADERS
        .iter()
        .any(|h| lower_name == *h || lower_name.starts_with(&format!("{h}-")))
}

/// Determines whether a specific header key should be stripped based on policy.
fn should_strip_header(lower_name: &str, policy: &PrefixStabilityPolicy) -> bool {
    let is_allowed = policy
        .allowed_custom_headers
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(lower_name));

    if is_allowed {
        return false;
    }

    is_forbidden_internal_header(lower_name)
        || is_disallowed_goog_header(lower_name, policy.strip_disallowed_goog_headers)
}

/// Sanitizes request headers by stripping internal and disallowed headers,
/// normalizing names to lowercase, and generating a stability report.
pub fn sanitize_request_headers(
    headers: &HashMap<String, String>,
    policy: &PrefixStabilityPolicy,
) -> (HashMap<String, String>, PrefixSanitizationReport) {
    let mut clean_headers = HashMap::new();
    let mut stripped_headers = Vec::new();

    for (key, val) in headers {
        let lower_key = key.trim().to_ascii_lowercase();
        if should_strip_header(&lower_key, policy) {
            stripped_headers.push(lower_key);
        } else {
            clean_headers.insert(lower_key, val.clone());
        }
    }

    let report = PrefixSanitizationReport::new(stripped_headers, clean_headers.len());
    (clean_headers, report)
}

/// Sorts object keys inside a JSON Map into deterministic alphabetical order.
fn sort_map_entries(
    map: &serde_json::Map<String, serde_json::Value>,
) -> serde_json::Map<String, serde_json::Value> {
    let mut sorted = serde_json::Map::new();
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();
    for key in keys {
        if let Some(v) = map.get(key) {
            sorted.insert(key.clone(), sort_json_keys(v));
        }
    }
    sorted
}

/// Recursively traverses a JSON value and orders all object keys.
pub fn sort_json_keys(val: &serde_json::Value) -> serde_json::Value {
    match val {
        serde_json::Value::Object(map) => serde_json::Value::Object(sort_map_entries(map)),
        serde_json::Value::Array(arr) => {
            let sorted_items = arr.iter().map(sort_json_keys).collect();
            serde_json::Value::Array(sorted_items)
        }
        primitive => primitive.clone(),
    }
}

/// Recursively formats JSON with sorted object keys to ensure identical byte prefixes for LLM prompt caching.
pub fn serialize_deterministic_json(val: &serde_json::Value) -> Result<String, String> {
    let sorted_val = sort_json_keys(val);
    serde_json::to_string(&sorted_val)
        .map_err(|e| format!("Deterministic JSON serialization error: {e}"))
}
