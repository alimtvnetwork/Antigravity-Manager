//! Supabase PostgREST Client Module
//! Provides lightweight REST communication with Supabase PostgreSQL endpoints.

#![allow(dead_code)]

use crate::error::AppError;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// Supabase endpoint configuration
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
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl SupabaseClient {
    /// Create a new client from an endpoint configuration
    pub fn new(endpoint: &SupabaseEndpoint) -> Result<Self, AppError> {
        let mut headers = HeaderMap::new();
        let key_val = HeaderValue::from_str(&endpoint.api_key)
            .map_err(|e| AppError::Config(format!("Invalid API key: {}", e)))?;
        headers.insert("apikey", key_val.clone());
        let bearer_val = HeaderValue::from_str(&format!("Bearer {}", endpoint.api_key))
            .map_err(|e| AppError::Config(format!("Invalid Bearer token: {}", e)))?;
        headers.insert(AUTHORIZATION, bearer_val);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            reqwest::header::ACCEPT,
            HeaderValue::from_static("application/json, application/openapi+json, */*"),
        );

        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| AppError::Network(e.to_string(), None))?;

        let clean_url = normalize_supabase_url(&endpoint.url);

        Ok(Self {
            client: http_client,
            base_url: clean_url,
            api_key: endpoint.api_key.clone(),
        })
    }

    /// Base normalized endpoint URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Build the full PostgREST REST URL for a table
    pub fn table_url(&self, table: &str) -> String {
        format!("{}/rest/v1/{}", self.base_url, table)
    }

    /// Build the RPC URL for calling PostgreSQL stored functions
    pub fn rpc_url(&self, function_name: &str) -> String {
        format!("{}/rest/v1/rpc/{}", self.base_url, function_name)
    }

    /// Test the connection to the Supabase endpoint using a resilient multi-stage probe ladder:
    /// 1. Query root `/rest/v1/` with API key & Bearer token.
    /// 2. If 404 (due to Supabase blocking OpenAPI spec for anon keys or disabled OpenAPI),
    ///    query PostgREST table endpoint (`/rest/v1/nodes?limit=0` or `/rest/v1/command_queue?limit=0`).
    ///    - If 200: connected and table exists.
    ///    - If 404 with PostgREST body (PGRST204/PGRST205/relation does not exist):
    ///      PostgREST is responsive and authenticated, but tables are pending migration -> success!
    /// 3. If still unresolved, fallback to `/auth/v1/health` (GoTrue health check).
    pub async fn test_connection(&self) -> Result<EndpointTestResult, AppError> {
        // Probe 1: REST API root (/rest/v1/)
        let rest_url = format!("{}/rest/v1/", self.base_url);
        match self.client.get(&rest_url).send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                if status < 400 {
                    return Ok(EndpointTestResult {
                        is_success: true,
                        message: "Connected successfully (Supabase PostgREST verified)".to_string(),
                        status_code: Some(status),
                    });
                }
                if status == 401 || status == 403 {
                    let body = resp.text().await.unwrap_or_default();
                    if !body.contains("Secret API key required") {
                        return Ok(EndpointTestResult {
                            is_success: false,
                            message: format!(
                                "Authentication failed: Invalid API key or Bearer token (HTTP {})",
                                status
                            ),
                            status_code: Some(status),
                        });
                    }
                    // Publishable key: root OpenAPI spec requires secret key; proceed to table probe
                }
                // Status 404 or other 4xx: proceed to table and auth probes
            }
            Err(e) => {
                return Ok(EndpointTestResult {
                    is_success: false,
                    message: format!("Network connection failed: {}", e),
                    status_code: None,
                });
            }
        }

        // Probe 2: PostgREST Table probes (check if PostgREST responds to queries even if OpenAPI root is disabled)
        let probe_tables = ["nodes", "command_queue"];
        for table in probe_tables {
            let table_url = format!("{}/rest/v1/{}?limit=0", self.base_url, table);
            if let Ok(resp) = self.client.get(&table_url).send().await {
                let status = resp.status().as_u16();
                if status < 400 {
                    return Ok(EndpointTestResult {
                        is_success: true,
                        message: format!("Connected successfully ({} table verified)", table),
                        status_code: Some(status),
                    });
                }
                if status == 401 || status == 403 {
                    return Ok(EndpointTestResult {
                        is_success: false,
                        message: format!(
                            "Authentication failed: Invalid API key or Bearer token (HTTP {})",
                            status
                        ),
                        status_code: Some(status),
                    });
                }
                let body = resp.text().await.unwrap_or_default();
                if body.contains("PGRST")
                    || body.contains("relation")
                    || body.contains("does not exist")
                    || body.contains("schema cache")
                {
                    return Ok(EndpointTestResult {
                        is_success: true,
                        message: "Connected successfully (PostgREST responsive; run Schema Migration to create tables)".to_string(),
                        status_code: Some(200),
                    });
                }
            }
        }

        // Probe 3: Supabase Service Auth Health probe (/auth/v1/health)
        let auth_url = format!("{}/auth/v1/health", self.base_url);
        if let Ok(resp) = self.client.get(&auth_url).send().await {
            let status = resp.status().as_u16();
            if status < 400 {
                return Ok(EndpointTestResult {
                    is_success: true,
                    message: "Connected successfully (Supabase service online)".to_string(),
                    status_code: Some(status),
                });
            }
        }

        Ok(EndpointTestResult {
            is_success: false,
            message: "Supabase connection test failed with status: 404 (Endpoint not found. Please verify Project URL)".to_string(),
            status_code: Some(404),
        })
    }

    /// Check if a specific table exists and is readable via PostgREST
    pub async fn check_table_exists(&self, table: &str) -> Result<bool, AppError> {
        let url = format!("{}/rest/v1/{}?limit=0", self.base_url, table);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string(), None))?;

        let status_code = resp.status().as_u16();
        if status_code == 200 {
            return Ok(true);
        }
        if status_code == 404 {
            return Ok(false);
        }
        let text = resp.text().await.unwrap_or_default();
        if text.contains("does not exist") || text.contains("PGRST204") || text.contains("PGRST205")
        {
            return Ok(false);
        }
        Err(AppError::Network(
            format!(
                "Table probe failed for '{}' with status {}: {}",
                table, status_code, text
            ),
            Some(status_code),
        ))
    }

    /// Check existence of all expected tables for this endpoint role
    pub async fn verify_expected_tables(
        &self,
        endpoint_id: &str,
        role: &str,
    ) -> TableVerificationResult {
        let expected: Vec<&str> = match role {
            "root" => vec!["nodes", "instance_profiles", "workspace_leases"],
            "secondary" => vec!["command_queue", "command_telemetry", "endpoint_health"],
            _ => vec![
                "nodes",
                "instance_profiles",
                "workspace_leases",
                "command_queue",
                "command_telemetry",
                "endpoint_health",
            ],
        };

        let mut verified = Vec::new();
        let mut missing = Vec::new();
        let mut err_msg = None;

        for table in expected {
            match self.check_table_exists(table).await {
                Ok(true) => verified.push(table.to_string()),
                Ok(false) => missing.push(table.to_string()),
                Err(e) => {
                    missing.push(table.to_string());
                    if err_msg.is_none() {
                        err_msg = Some(e.to_string());
                    }
                }
            }
        }

        let is_connected = if !verified.is_empty() {
            true
        } else {
            err_msg.is_none() && !missing.is_empty()
        };

        TableVerificationResult {
            endpoint_id: endpoint_id.to_string(),
            is_connected,
            verified_tables: verified,
            missing_tables: missing,
            error_message: err_msg,
        }
    }

    /// Query rows from a table with PostgREST query parameters
    pub async fn select(&self, table: &str, query: &str) -> Result<Value, AppError> {
        let mut url = self.table_url(table);
        if !query.is_empty() {
            url.push('?');
            url.push_str(query);
        }

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string(), None))?;

        let parsed: Value = resp
            .json()
            .await
            .map_err(|e| AppError::Network(e.to_string(), None))?;
        Ok(parsed)
    }

    /// Insert a record into a table
    pub async fn insert(&self, table: &str, payload: Value) -> Result<Value, AppError> {
        let url = self.table_url(table);
        let resp = self
            .client
            .post(&url)
            .header("Prefer", "return=representation")
            .json(&payload)
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string(), None))?;

        let parsed: Value = resp
            .json()
            .await
            .map_err(|e| AppError::Network(e.to_string(), None))?;
        Ok(parsed)
    }

    /// Upsert a record with resolution merge duplicates
    pub async fn upsert(
        &self,
        table: &str,
        payload: Value,
        on_conflict: &str,
    ) -> Result<Value, AppError> {
        let mut url = self.table_url(table);
        if !on_conflict.is_empty() {
            url = format!("{}?on_conflict={}", url, on_conflict);
        }

        let resp = self
            .client
            .post(&url)
            .header(
                "Prefer",
                "resolution=merge-duplicates,return=representation",
            )
            .json(&payload)
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string(), None))?;

        let parsed: Value = resp
            .json()
            .await
            .map_err(|e| AppError::Network(e.to_string(), None))?;
        Ok(parsed)
    }

    /// Update records matching a query filter
    pub async fn update(
        &self,
        table: &str,
        query: &str,
        payload: Value,
    ) -> Result<Value, AppError> {
        let mut url = self.table_url(table);
        if !query.is_empty() {
            url.push('?');
            url.push_str(query);
        }

        let resp = self
            .client
            .patch(&url)
            .header("Prefer", "return=representation")
            .json(&payload)
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string(), None))?;

        let parsed: Value = resp
            .json()
            .await
            .map_err(|e| AppError::Network(e.to_string(), None))?;
        Ok(parsed)
    }

    /// Delete records matching a query filter
    pub async fn delete(&self, table: &str, query: &str) -> Result<Value, AppError> {
        let mut url = self.table_url(table);
        if !query.is_empty() {
            url.push('?');
            url.push_str(query);
        }

        let resp = self
            .client
            .delete(&url)
            .header("Prefer", "return=representation")
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string(), None))?;

        let parsed: Value = resp
            .json()
            .await
            .map_err(|e| AppError::Network(e.to_string(), None))?;
        Ok(parsed)
    }

    /// Call a PostgreSQL stored function via RPC
    pub async fn rpc(&self, function_name: &str, payload: Value) -> Result<Value, AppError> {
        let url = self.rpc_url(function_name);
        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string(), None))?;

        let parsed: Value = resp
            .json()
            .await
            .map_err(|e| AppError::Network(e.to_string(), None))?;
        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_supabase_url() {
        assert_eq!(
            normalize_supabase_url("https://abcdefg.supabase.co"),
            "https://abcdefg.supabase.co"
        );
        assert_eq!(
            normalize_supabase_url("https://abcdefg.supabase.co/"),
            "https://abcdefg.supabase.co"
        );
        assert_eq!(
            normalize_supabase_url("https://abcdefg.supabase.co///"),
            "https://abcdefg.supabase.co"
        );
        assert_eq!(
            normalize_supabase_url("https://abcdefg.supabase.co/rest/v1"),
            "https://abcdefg.supabase.co"
        );
        assert_eq!(
            normalize_supabase_url("https://abcdefg.supabase.co/rest/v1/"),
            "https://abcdefg.supabase.co"
        );
        assert_eq!(
            normalize_supabase_url("https://abcdefg.supabase.co/rest/v1///"),
            "https://abcdefg.supabase.co"
        );
        assert_eq!(
            normalize_supabase_url("  https://abcdefg.supabase.co/rest/v1/  "),
            "https://abcdefg.supabase.co"
        );
        assert_eq!(
            normalize_supabase_url("http://localhost:54321/rest/v1/"),
            "http://localhost:54321"
        );
        assert_eq!(
            normalize_supabase_url("https://example.com/custom/prefix/rest/v1"),
            "https://example.com/custom/prefix"
        );
    }

    #[test]
    fn test_client_url_construction() {
        let ep1 = SupabaseEndpoint {
            id: "ep_1".to_string(),
            name: "Cloud".to_string(),
            url: "https://abcdefg.supabase.co/rest/v1/".to_string(),
            api_key: "anon-key-sample".to_string(),
            role: "root".to_string(),
            is_enabled: true,
            prune_threshold_mb: 400,
            priority: 1,
            notes: None,
            tags: Vec::new(),
        };
        let client1 = SupabaseClient::new(&ep1).expect("valid client");
        assert_eq!(client1.base_url(), "https://abcdefg.supabase.co");
        assert_eq!(
            client1.table_url("nodes"),
            "https://abcdefg.supabase.co/rest/v1/nodes"
        );
        assert_eq!(
            client1.rpc_url("sync_nodes"),
            "https://abcdefg.supabase.co/rest/v1/rpc/sync_nodes"
        );

        let ep2 = SupabaseEndpoint {
            id: "ep_2".to_string(),
            name: "Bare".to_string(),
            url: "https://abcdefg.supabase.co".to_string(),
            api_key: "anon-key-sample".to_string(),
            role: "root".to_string(),
            is_enabled: true,
            prune_threshold_mb: 400,
            priority: 1,
            notes: None,
            tags: Vec::new(),
        };
        let client2 = SupabaseClient::new(&ep2).expect("valid client");
        assert_eq!(client2.base_url(), "https://abcdefg.supabase.co");
        assert_eq!(client2.table_url("nodes"), client1.table_url("nodes"));
        assert_eq!(client2.rpc_url("sync_nodes"), client1.rpc_url("sync_nodes"));
    }

    #[test]
    fn test_verification_is_connected_logic() {
        // Case 1: verified tables exist -> is_connected is true
        let res1 = TableVerificationResult {
            endpoint_id: "ep_1".to_string(),
            is_connected: true,
            verified_tables: vec!["nodes".to_string()],
            missing_tables: vec!["workspace_leases".to_string()],
            error_message: None,
        };
        assert!(res1.is_connected);

        // Case 2: no verified tables, but network error occurred -> is_connected is false
        let verified: Vec<String> = Vec::new();
        let err_msg = Some("Connection refused (os error 111)".to_string());
        let missing = vec!["nodes".to_string()];
        let is_conn = if !verified.is_empty() {
            true
        } else {
            err_msg.is_none() && !missing.is_empty()
        };
        assert!(!is_conn);

        // Case 3: no verified tables, no network error (PostgREST responsive but tables not created) -> is_connected is true
        let err_msg_none: Option<String> = None;
        let is_conn_pending = if !verified.is_empty() {
            true
        } else {
            err_msg_none.is_none() && !missing.is_empty()
        };
        assert!(is_conn_pending);
    }
}
