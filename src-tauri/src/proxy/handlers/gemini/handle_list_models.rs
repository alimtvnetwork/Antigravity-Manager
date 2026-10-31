use super::*;

pub async fn handle_list_models(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    use crate::proxy::common::model_mapping::get_all_dynamic_models;

    // Get a list of all dynamic models (consistent with /v1/models)
    let only_raw = *state.only_raw_quota_models.read().await;
    let model_ids =
        get_all_dynamic_models(&state.custom_mapping, Some(&state.token_manager), only_raw).await;

    // Convert to Gemini API format
    let models: Vec<_> = model_ids
        .into_iter()
        .map(|id| {
            json!({
                "name": format!("models/{}", id),
                "version": "001",
                "displayName": id.clone(),
                "description": "",
                "inputTokenLimit": 128000,
                "outputTokenLimit": 8192,
                "supportedGenerationMethods": ["generateContent", "countTokens"],
                "temperature": 1.0,
                "topP": 0.95,
                "topK": 64
            })
        })
        .collect();

    Ok(Json(json!({ "models": models })))
}

pub async fn handle_get_model(Path(model_name): Path<String>) -> impl IntoResponse {
    Json(json!({
        "name": format!("models/{}", model_name),
        "displayName": model_name
    }))
}

pub async fn handle_count_tokens(
    State(state): State<AppState>,
    Path(model_name): Path<String>,
    Json(body): Json<Value>,
) -> Response {
    execute_count_tokens(state, model_name, body).await
}

/// Core countTokens implementation: transparent proxy to upstream v1internal:countTokens
///
/// Obtain a valid OAuth Token, wrap the standard Gemini request body into v1internal format and forward it.
/// Returns the true token count instead of the hardcoded 0
pub async fn execute_count_tokens(
    state: AppState,
    model_name: String,
    mut body: Value,
) -> Response {
    // [DEFENSE] Sanitize all inlineData in Gemini native request payload (filter or downgrade empty data/corrupted images)
    crate::proxy::mappers::common_utils::sanitize_gemini_payload_inline_data(&mut body);
    // 1. Model routing analysis
    let mapped_model = crate::proxy::common::model_mapping::resolve_model_route(
        &model_name,
        &*state.custom_mapping.read().await,
    );

    // 2. Parse the request configuration and obtain the Token
    let config = crate::proxy::mappers::common_utils::resolve_request_config(
        &model_name,
        &mapped_model,
        &None,
        None,
        None,
        None,
        Some(&body),
    );

    let session_id = SessionManager::extract_gemini_session_id(&body, &model_name);

    let (access_token, _project_id, email, account_id, _wait_ms) = match state
        .token_manager
        .get_token(
            &config.request_type,
            false,
            Some(&session_id),
            &config.final_model,
        )
        .await
    {
        Ok(t) => t,
        Err(e) => {
            let headers = build_token_error_headers(Some(mapped_model.as_str()), None, &e);
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                headers,
                Json(json!({ "error": format!("Token error: {}", e) })),
            )
                .into_response();
        }
    };

    // 3. Packaged in v1internal format
    // [Verified] countTokens is different from generateContent: only "request" keys are allowed at the top level,
    // Carrying model/project will be rejected by the upstream 400 (Unknown name "model"/"project");
    // safetySettings in request are also not accepted (aligned with CLIProxyAPI processing)
    let mut inner_body = body;
    if let Some(obj) = inner_body.as_object_mut() {
        obj.remove("safetySettings");
    }
    let wrapped_body = json!({
        "request": inner_body,
    });

    // 4. Call upstream v1internal:countTokens
    let call_result = match state
        .upstream
        .call_v1_internal_with_headers(
            "countTokens",
            &access_token,
            wrapped_body,
            None,
            std::collections::HashMap::new(),
            Some(account_id.as_str()),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": format!("Upstream call error: {}", e) })),
            )
                .into_response();
        }
    };

    let response = call_result.response;
    let status = response.status();

    if !status.is_success() {
        let err_text = response.text().await.unwrap_or_default();
        return (
            status,
            Json(json!({ "error": format!("Upstream countTokens error: {}", err_text) })),
        )
            .into_response();
    }

    let gemini_resp: Value = match response.json().await {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": format!("Parse error: {}", e) })),
            )
                .into_response();
        }
    };

    // 5. Extract totalTokens (compatible with wrapped / unwrapped two response formats)
    let total_tokens = gemini_resp
        .get("response")
        .and_then(|r| r.get("totalTokens"))
        .or_else(|| gemini_resp.get("totalTokens"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    // 6. Return standard Gemini REST response
    (
        StatusCode::OK,
        [
            ("X-Account-Email", email.as_str()),
            ("X-Mapped-Model", mapped_model.as_str()),
        ],
        Json(json!({ "totalTokens": total_tokens })),
    )
        .into_response()
}
