//! OpenAPI v3.1 specification generator for Antigravity-Manager proxy endpoints.
//! Exposes structured schema metadata for reverse proxy routes and admin management APIs.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// API Endpoint definition for OpenAPI documentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpointDef {
    pub path: String,
    pub method: String,
    pub summary: String,
    pub description: String,
    pub tags: Vec<String>,
    pub request_body_description: Option<String>,
    pub response_codes: Vec<u16>,
}

/// Generates a comprehensive OpenAPI v3.1.0 specification Value.
pub fn generate_openapi_spec() -> Value {
    let endpoints = get_registered_endpoints();
    let mut paths_obj = json!({});

    for ep in endpoints {
        let method_lower = ep.method.to_lowercase();
        let mut responses_obj = json!({});
        for code in &ep.response_codes {
            responses_obj[code.to_string()] = json!({
                "description": match code {
                    200 => "Successful operation",
                    201 => "Resource created",
                    400 => "Bad request or validation error",
                    401 => "Unauthorized or invalid credentials",
                    404 => "Resource not found",
                    429 => "Upstream quota exceeded or rate limit hit",
                    500 => "Internal gateway server error",
                    _ => "Response code description",
                }
            });
        }

        let mut operation_obj = json!({
            "summary": ep.summary,
            "description": ep.description,
            "tags": ep.tags,
            "responses": responses_obj,
        });

        if let Some(req_desc) = ep.request_body_description {
            operation_obj["requestBody"] = json!({
                "description": req_desc,
                "required": true,
                "content": {
                    "application/json": {
                        "schema": {
                            "type": "object"
                        }
                    }
                }
            });
        }

        if paths_obj.get(&ep.path).is_none() {
            paths_obj[&ep.path] = json!({});
        }
        paths_obj[&ep.path][method_lower] = operation_obj;
    }

    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "Antigravity-Manager Gateway & Reverse Proxy API",
            "version": "4.250.0",
            "description": "Developer REST API Workbench and Reverse Proxy Gateway specification for Antigravity-Manager."
        },
        "servers": [
            {
                "url": "http://127.0.0.1:18188",
                "description": "Local Antigravity-Manager Gateway Loopback"
            }
        ],
        "paths": paths_obj,
        "components": {
            "securitySchemes": {
                "BearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT"
                }
            }
        }
    })
}

/// Generates the OpenAPI spec as a formatted JSON string.
pub fn generate_openapi_json() -> String {
    serde_json::to_string_pretty(&generate_openapi_spec()).unwrap_or_else(|_| "{}".to_string())
}

/// Returns the registry of all documented proxy and admin endpoints.
fn get_registered_endpoints() -> Vec<ApiEndpointDef> {
    vec![
        ApiEndpointDef {
            path: "/v1/models".to_string(),
            method: "GET".to_string(),
            summary: "List Available Models".to_string(),
            description: "Lists all models available across active Google and Anthropic providers.".to_string(),
            tags: vec!["Proxy".to_string(), "Models".to_string()],
            request_body_description: None,
            response_codes: vec![200, 401, 500],
        },
        ApiEndpointDef {
            path: "/v1/chat/completions".to_string(),
            method: "POST".to_string(),
            summary: "Create Chat Completion".to_string(),
            description: "OpenAI-compatible chat completion endpoint supporting streaming SSE and thinking backfill.".to_string(),
            tags: vec!["Proxy".to_string(), "Completions".to_string()],
            request_body_description: Some("OpenAI Chat Completion request payload".to_string()),
            response_codes: vec![200, 400, 401, 429, 500],
        },
        ApiEndpointDef {
            path: "/api/prompts".to_string(),
            method: "GET".to_string(),
            summary: "List Active Prompts".to_string(),
            description: "Queries active and recent prompts from repo_prompts.db with FIFO ordering.".to_string(),
            tags: vec!["Admin".to_string(), "Prompts".to_string()],
            request_body_description: None,
            response_codes: vec![200, 500],
        },
        ApiEndpointDef {
            path: "/api/prompts".to_string(),
            method: "POST".to_string(),
            summary: "Dispatch or Enqueue Prompt".to_string(),
            description: "Dispatches a new user prompt or enqueues it to the target instance queue.".to_string(),
            tags: vec!["Admin".to_string(), "Prompts".to_string()],
            request_body_description: Some("Prompt dispatch payload with instance_id and text".to_string()),
            response_codes: vec![200, 400, 500],
        },
        ApiEndpointDef {
            path: "/api/instances".to_string(),
            method: "GET".to_string(),
            summary: "List Running Instances".to_string(),
            description: "Returns all discovered Antigravity IDE instances, active PIDs, and workspace paths.".to_string(),
            tags: vec!["Admin".to_string(), "Instances".to_string()],
            request_body_description: None,
            response_codes: vec![200, 500],
        },
        ApiEndpointDef {
            path: "/api/accounts".to_string(),
            method: "GET".to_string(),
            summary: "List Registered Accounts".to_string(),
            description: "Returns managed Google accounts, quota telemetry (4h and weekly), and algorithm scores.".to_string(),
            tags: vec!["Admin".to_string(), "Accounts".to_string()],
            request_body_description: None,
            response_codes: vec![200, 500],
        },
        ApiEndpointDef {
            path: "/api/doctor".to_string(),
            method: "GET".to_string(),
            summary: "Run System Diagnostics".to_string(),
            description: "Performs full health checks on SQLite databases, process locks, and upstream connectivity.".to_string(),
            tags: vec!["Admin".to_string(), "Diagnostics".to_string()],
            request_body_description: None,
            response_codes: vec![200, 500],
        },
        ApiEndpointDef {
            path: "/api/rest/execute".to_string(),
            method: "POST".to_string(),
            summary: "Execute REST Try-It-Out Request".to_string(),
            description: "Secure loopback runner executing API requests with precise timing telemetry.".to_string(),
            tags: vec!["Workbench".to_string()],
            request_body_description: Some("Internal execution payload with method, path, headers, and body".to_string()),
            response_codes: vec![200, 400, 500],
        },
        ApiEndpointDef {
            path: "/api/env/info".to_string(),
            method: "GET".to_string(),
            summary: "Get Environment Info".to_string(),
            description: "Returns active gateway host, port, tunnel status, node identity, and OS details.".to_string(),
            tags: vec!["Workbench".to_string(), "Environment".to_string()],
            request_body_description: None,
            response_codes: vec![200],
        },
    ]
}
