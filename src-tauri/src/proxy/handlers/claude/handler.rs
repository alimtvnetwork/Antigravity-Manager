use super::*;
use crate::proxy::handlers::claude::error_handling::{build_exhaustion_error, handle_upstream_error};
use crate::proxy::handlers::claude::request_log::log_request_summary;
use crate::proxy::handlers::claude::response_handling::handle_nonstream_success;
use crate::proxy::handlers::claude::stream_handling::handle_stream_success;

pub async fn handle_messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    upstream_recorder: Option<
        axum::extract::Extension<crate::proxy::monitor::UpstreamRequestBodyHolder>,
    >,
    Json(body): Json<Value>,
) -> Response {
    let setup = match preprocess_request(&state, &headers, body, &upstream_recorder).await {
        Ok(s) => s,
        Err(resp) => return resp,
    };
    log_request_summary(&setup.request, &setup.trace_id);

    let mut st = AttemptState::from_setup(
        setup,
        state.token_manager,
        state.upstream,
        state.custom_mapping,
        headers,
        upstream_recorder,
    );

    for attempt in 0..st.max_attempts {
        match prepare_attempt(&mut st, attempt).await {
            PrepOutcome::Respond(resp) => return resp,
            PrepOutcome::Retry => continue,
            PrepOutcome::Proceed(call) => {
                if call.status.is_success() {
                    // [智能限流] 请求成功，重置该账号的连续失败计数
                    st.token_manager.mark_account_success(&call.email);
                    if call.actual_stream {
                        match handle_stream_success(&mut st, call).await {
                            StreamOutcome::Respond(resp) => return resp,
                            StreamOutcome::Retry => continue,
                        }
                    } else {
                        return handle_nonstream_success(&mut st, call).await;
                    }
                } else {
                    match handle_upstream_error(&mut st, call, attempt).await {
                        ErrorOutcome::Respond(resp) => return resp,
                        ErrorOutcome::Retry => continue,
                    }
                }
            }
        }
    }

    build_exhaustion_error(
        st.last_email,
        st.last_mapped_model,
        st.last_status,
        &st.last_error,
    )
}

pub async fn handle_list_models(State(state): State<AppState>) -> impl IntoResponse {
    use crate::proxy::common::model_mapping::get_all_dynamic_models;

    let only_raw = *state.only_raw_quota_models.read().await;
    let model_ids =
        get_all_dynamic_models(&state.custom_mapping, Some(&state.token_manager), only_raw).await;

    let data: Vec<_> = model_ids
        .into_iter()
        .map(|id| {
            json!({
                "id": id,
                "object": "model",
                "created": 1706745600,
                "owned_by": "antigravity"
            })
        })
        .collect();

    Json(json!({
        "object": "list",
        "data": data
    }))
}

/// 计算 tokens (占位符)
pub async fn handle_count_tokens(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    let zai = state.zai.read().await.clone();
    let zai_enabled =
        zai.enabled && !matches!(zai.dispatch_mode, crate::proxy::ZaiDispatchMode::Off);

    if zai_enabled {
        return crate::proxy::providers::zai_anthropic::forward_anthropic_json(
            &state,
            axum::http::Method::POST,
            "/v1/messages/count_tokens",
            &headers,
            body,
            0, // [NEW v4.0.0] Tokens count doesn't need rewind detection
        )
        .await;
    }

    Json(json!({
        "input_tokens": 0,
        "output_tokens": 0
    }))
    .into_response()
}
