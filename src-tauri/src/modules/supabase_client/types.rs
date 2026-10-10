use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SupabaseEndpoint {
    pub id: String,
    pub name: String,
    pub url: String,
    pub api_key: String,
    pub role: String, // "root" or "secondary"
    pub is_enabled: bool,
    pub prune_threshold_mb: u64,
    pub priority: u32,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl SupabaseEndpoint {
    /// Construct a new SupabaseEndpoint with normalized URL and standard role defaults
    pub fn new_endpoint(
        id: impl Into<String>,
        name: impl Into<String>,
        url: impl Into<String>,
        api_key: impl Into<String>,
        role: impl Into<String>,
    ) -> Self {
        let role_str = role.into();
        let is_root = role_str.eq_ignore_ascii_case("root");
        let raw_url = url.into();
        let clean_url = normalize_supabase_url(&raw_url);
        Self {
            id: id.into(),
            name: name.into(),
            url: clean_url,
            api_key: api_key.into(),
            role: role_str.clone(),
            is_enabled: true,
            prune_threshold_mb: if is_root { 400 } else { 200 },
            priority: if is_root { 1 } else { 2 },
            notes: None,
            tags: vec![role_str],
        }
    }
}

/// Helper to construct a SupabaseEndpoint with standard defaults
pub fn create_endpoint(
    id: impl Into<String>,
    name: impl Into<String>,
    url: impl Into<String>,
    api_key: impl Into<String>,
    role: impl Into<String>,
) -> SupabaseEndpoint {
    SupabaseEndpoint::new_endpoint(id, name, url, api_key, role)
}

/// Table verification result for schema checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableVerificationResult {
    pub endpoint_id: String,
    pub is_connected: bool,
    pub verified_tables: Vec<String>,
    pub missing_tables: Vec<String>,
    pub error_message: Option<String>,
}

/// Endpoint connectivity test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointTestResult {
    pub is_success: bool,
    pub message: String,
    pub status_code: Option<u16>,
}

/// Normalize Supabase endpoint URL by trimming whitespace, trailing slashes, and redundant `/rest/v1` suffixes.
pub fn normalize_supabase_url(raw: &str) -> String {
    let mut trimmed = raw.trim();
    while trimmed.ends_with('/') {
        trimmed = &trimmed[..trimmed.len() - 1];
    }
    if let Some(stripped) = trimmed.strip_suffix("/rest/v1") {
        trimmed = stripped;
        while trimmed.ends_with('/') {
            trimmed = &trimmed[..trimmed.len() - 1];
        }
    }
    trimmed.to_string()
}

/// Lightweight PostgREST HTTP Client
pub struct SupabaseClient {
    pub(crate) client: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) api_key: String,
}
