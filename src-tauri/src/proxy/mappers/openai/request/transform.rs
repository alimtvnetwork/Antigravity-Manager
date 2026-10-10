// Request transform entry points (split from request.rs).
// OpenAI → Gemini 请求转换
use super::super::models::*;
use crate::proxy::model_specs;
use crate::proxy::token_manager::ProxyToken;

use serde_json::{json, Value};

use super::session::transform_openai_request_with_session;

pub fn transform_openai_request(
    request: &OpenAIRequest,
    project_id: &str,
    mapped_model: &str,
    token: Option<&ProxyToken>,
) -> (Value, String, usize, String) {
    let session_id =
        crate::proxy::session_manager::SessionManager::extract_openai_session_id(request);
    transform_openai_request_with_session(
        request,
        project_id,
        mapped_model,
        token,
        &session_id,
        Some(&session_id),
        false, // is_responses_api (Chat completions protocol)
    )
}

pub fn enforce_uppercase_types(value: &mut Value) {
    if let Value::Object(map) = value {
        if let Some(type_val) = map.get_mut("type") {
            if let Value::String(ref mut s) = type_val {
                *s = s.to_uppercase();
            }
        }
        if let Some(properties) = map.get_mut("properties") {
            if let Value::Object(ref mut props) = properties {
                for v in props.values_mut() {
                    enforce_uppercase_types(v);
                }
            }
        }
        if let Some(items) = map.get_mut("items") {
            enforce_uppercase_types(items);
        }
    } else if let Value::Array(arr) = value {
        for item in arr {
            enforce_uppercase_types(item);
        }
    }
}
