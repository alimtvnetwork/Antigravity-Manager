//! Public AI-protocol route table and final app assembly.
use super::app_state::AppState;
use crate::proxy::handlers;
use super::admin_oauth::handle_oauth_callback;
use crate::proxy::middleware::{
    auth_middleware, cors_layer, ip_filter_middleware, monitor_middleware,
    service_status_middleware,
};
use axum::{
    Json,
    extract::DefaultBodyLimit,
    http::StatusCode,
    response::Response,
    routing::{any, get, post},
    Router,
};
use axum::response::IntoResponse;

/// Health check handler
pub(crate) async fn health_check_handler() -> Response {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
    .into_response()
}

/// Silent success handler (for telemetry intercept etc.)
pub(crate) async fn silent_ok_handler() -> Response {
    StatusCode::OK.into_response()
}

pub(crate) fn build_proxy_routes(state: &AppState) -> Router<AppState> {
    // 1. Build primary AI proxy routes (honoring auth_mode configuration)
    let proxy_routes = Router::new()
        .route("/health", get(health_check_handler))
        .route("/healthz", get(health_check_handler))
        // OpenAI Protocol
        .route("/v1/models", get(handlers::openai::handle_list_models))
        .route(
            "/v1/chat/completions",
            post(handlers::openai::handle_chat_completions),
        )
        .route(
            "/v1/completions",
            post(handlers::openai::handle_completions),
        )
        .route(
            "/v1/responses",
            post(handlers::openai::handle_completions)
                .get(handlers::openai::handle_responses_websocket),
        ) // Compatible with Codex CLI
        .route("/responses", post(handlers::openai::handle_completions))
        .route(
            "/responses/compact",
            post(handlers::openai::handle_completions),
        )
        .route(
            "/v1/images/generations",
            post(handlers::openai::handle_images_generations),
        ) // Image generation API
        .route(
            "/v1/images/edits",
            post(handlers::openai::handle_images_edits),
        ) // Image edit API
        .route(
            "/v1/audio/transcriptions",
            post(handlers::audio::handle_audio_transcription),
        ) // Audio transcription API
        // Claude Protocol
        .route("/v1/messages", post(handlers::claude::handle_messages))
        .route(
            "/v1/messages/count_tokens",
            post(handlers::claude::handle_count_tokens),
        )
        .route(
            "/v1/models/claude",
            get(handlers::claude::handle_list_models),
        )
        // z.ai MCP (optional reverse-proxy)
        .route(
            "/mcp/web_search_prime/mcp",
            any(handlers::mcp::handle_web_search_prime),
        )
        .route("/mcp/web_reader/mcp", any(handlers::mcp::handle_web_reader))
        .route(
            "/mcp/zai-mcp-server/mcp",
            any(handlers::mcp::handle_zai_mcp_server),
        )
        // Gemini Protocol (Native)
        .route("/v1beta/models", get(handlers::gemini::handle_list_models))
        // Handle both GET (get info) and POST (generateContent with colon) at the same route
        .route(
            "/v1beta/models/:model",
            get(handlers::gemini::handle_get_model).post(handlers::gemini::handle_generate),
        )
        .route(
            "/v1beta/models/:model/countTokens",
            post(handlers::gemini::handle_count_tokens),
        ) // Specific route priority
        .route(
            "/v1/models/detect",
            post(handlers::common::handle_detect_model),
        )
        .route("/internal/warmup", post(handlers::warmup::handle_warmup)) // Internal warmup endpoint
        .route("/v1/api/event_logging/batch", post(silent_ok_handler))
        .route("/v1/api/event_logging", post(silent_ok_handler))
        .route(
            "/v1/thinking/end",
            post(handlers::thinking::handle_end_session),
        )
        .route(
            "/v1/thinking/sessions/:session_id",
            axum::routing::get(handlers::thinking::handle_session_stats)
                .delete(handlers::thinking::handle_delete_session),
        )
        // Apply AI service-specific middleware layers
        // Note: Axum layer execution order is bottom-up (onion model)
        // Inbound: ip_filter -> auth -> monitor -> handler
        // Outbound: handler -> monitor -> auth -> ip_filter
        // monitor executes after auth to access UserTokenIdentity
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            monitor_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            ip_filter_middleware,
        ));
    proxy_routes
}

pub(crate) fn assemble_app(state: &AppState, proxy_routes: Router<AppState>, admin_routes: Router<AppState>) -> Router<AppState> {
    // 3. Integrate and apply global middleware layers
    // Read body size limit from environment variable (default 50MB)
    let max_body_size: usize = std::env::var("ABV_MAX_BODY_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100 * 1024 * 1024); // Default 100MB
    tracing::info!(
        "Request body size limit: {} MB",
        max_body_size / 1024 / 1024
    );

    let app = Router::new()
        .nest("/api", admin_routes)
        .merge(proxy_routes)
        // Public routes (no authentication required)
        .route("/auth/callback", get(handle_oauth_callback))
        // Training REST API Endpoints (Guarded by training_api_enabled setting toggle)
        .route(
            "/api/v1/training",
            get(crate::modules::training_api::handle_training_telemetry),
        )
        .route(
            "/api/v1/status",
            get(crate::modules::training_api::handle_training_telemetry),
        )
        .route(
            "/api/v1/training/telemetry",
            get(crate::modules::training_api::handle_training_telemetry),
        )
        .route(
            "/api/v1/training/learn",
            post(crate::modules::training_api::handle_training_learn),
        )
        .route(
            "/api/v1/training/machines",
            post(crate::modules::training_api::handle_training_machines),
        )
        .route(
            "/api/v1/training/modify",
            post(crate::modules::training_api::handle_training_machines),
        )
        .route(
            "/api/v1/machines",
            post(crate::modules::training_api::handle_training_machines),
        )
        .route(
            "/training/telemetry",
            get(crate::modules::training_api::handle_training_telemetry),
        )
        .route(
            "/training/learn",
            post(crate::modules::training_api::handle_training_learn),
        )
        .route(
            "/training/machines",
            post(crate::modules::training_api::handle_training_machines),
        )
        .route(
            "/v1/training/telemetry",
            get(crate::modules::training_api::handle_training_telemetry),
        )
        .route(
            "/v1/remote/control",
            post(crate::modules::training_api::handle_training_machines),
        )
        .route(
            "/api/v1/remote/control",
            post(crate::modules::training_api::handle_training_machines),
        )
        // Apply global monitoring and status layer (outer)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            service_status_middleware,
        ))
        .layer(cors_layer())
        .layer(DefaultBodyLimit::max(max_body_size)) // Relax body size limit
        .with_state(state.clone());

    // Static file serving (for Headless/Docker mode)
    let dist_path = std::env::var("ABV_DIST_PATH").unwrap_or_else(|_| "dist".to_string());
    let app = if std::path::Path::new(&dist_path).exists() {
        tracing::info!("Serving static assets from: {}", dist_path);
        app.fallback_service(tower_http::services::ServeDir::new(&dist_path).fallback(
            tower_http::services::ServeFile::new(format!("{}/index.html", dist_path)),
        ))
    } else {
        app
    };

    // Bind address (uses socket2 with SO_REUSEADDR and dual-stack IPv4/IPv6 support)
    app
}
