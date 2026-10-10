use super::*;
use axum::http::HeaderMap;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct EndSessionResult {
    pub session_id: String,
    pub deleted_turns: usize,
    pub deleted_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct SessionScope {
    pub client_id: String,
    pub store_key: String,
}

impl SessionScope {
    pub fn from_headers(headers: &HeaderMap, fallback: impl Into<String>) -> Self {
        Self::from_request_parts(headers, None, None, fallback)
    }

    pub fn from_headers_and_body(
        headers: &HeaderMap,
        body: Option<&Value>,
        fallback: impl Into<String>,
    ) -> Self {
        Self::from_request_parts(headers, body, None, fallback)
    }

    pub fn from_request_parts(
        headers: &HeaderMap,
        body: Option<&Value>,
        query: Option<&str>,
        fallback: impl Into<String>,
    ) -> Self {
        let fallback = fallback.into();
        let tenant = tenant_from_headers(headers);
        let session_headers = collect_session_semantic_headers(headers);
        let query_sid = extract_query_session_id(headers, query);
        let body_sid = extract_body_session_id(body);

        let client_id = derive_blended_session_id(
            &tenant,
            &session_headers,
            query_sid.as_deref(),
            body_sid.as_deref(),
            &fallback,
        );
        let store_key = format!("{}:{}", tenant, client_id);
        Self {
            client_id,
            store_key,
        }
    }
}
