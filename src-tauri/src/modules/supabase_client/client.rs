use crate::error::AppError;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::Value;
use std::time::Duration;

use super::*;

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

    /// Classify a reqwest transport error into an actionable user-facing message
    /// so users see likely causes and next steps instead of a raw error string.
    pub(crate) fn classify_transport_error(&self, e: &reqwest::Error) -> String {
        if e.is_timeout() {
            format!(
                "Connection timed out reaching {}. The Supabase project may be paused (free-tier projects pause after inactivity) or the network may be blocking it. Verify the project is active in the Supabase dashboard, then press Test again.",
                self.base_url
            )
        } else if e.is_connect() {
            format!(
                "Cannot connect to {} (DNS/connection refused). The project URL may be wrong or the project deleted/paused. Verify the project URL in the Supabase dashboard.",
                self.base_url
            )
        } else {
            format!(
                "Network request to {} failed: {}. Check the URL, API key, and that no firewall/proxy blocks the connection.",
                self.base_url, e
            )
        }
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
                    message: self.classify_transport_error(&e),
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
