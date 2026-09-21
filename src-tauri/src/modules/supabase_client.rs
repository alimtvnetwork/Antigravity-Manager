//! Supabase PostgREST Client Module
//! Provides lightweight REST communication with Supabase PostgreSQL endpoints.

#![allow(dead_code)]

use crate::error::AppError;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// Supabase endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupabaseEndpoint {
    pub id: String,
    pub name: String,
    pub url: String,
    pub api_key: String,
    pub role: String, // "root" or "secondary"
    pub is_enabled: bool,
    pub prune_threshold_mb: u64,
    pub priority: u32,
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

        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| AppError::Network(e.to_string(), None))?;

        let clean_url = endpoint.url.trim_end_matches('/').to_string();

        Ok(Self {
            client: http_client,
            base_url: clean_url,
            api_key: endpoint.api_key.clone(),
        })
    }

    /// Build the full PostgREST REST URL for a table
    fn table_url(&self, table: &str) -> String {
        format!("{}/rest/v1/{}", self.base_url, table)
    }

    /// Build the RPC URL for calling PostgreSQL stored functions
    fn rpc_url(&self, function_name: &str) -> String {
        format!("{}/rest/v1/rpc/{}", self.base_url, function_name)
    }

    /// Test the connection to the Supabase endpoint
    pub async fn test_connection(&self) -> Result<bool, AppError> {
        let url = format!("{}/rest/v1/", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::Network(e.to_string(), None))?;

        let status_code = resp.status().as_u16();
        if status_code < 400 {
            return Ok(true);
        }
        Err(AppError::Network(
            format!(
                "Supabase connection test failed with status: {}",
                status_code
            ),
            Some(status_code),
        ))
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
