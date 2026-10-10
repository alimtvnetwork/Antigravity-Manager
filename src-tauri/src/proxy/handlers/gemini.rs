use crate::proxy::common::client_adapter::CLIENT_ADAPTERS;
use crate::proxy::debug_logger;
use crate::proxy::handlers::common::{
    apply_retry_strategy, build_token_error_headers, next_rotation_attempt, should_rotate_account,
    FailureStatusTracker, RequestRetryState, RetryStrategy,
};
use crate::proxy::mappers::gemini::{unwrap_response, wrap_request_v2};
use crate::proxy::server::AppState;
use crate::proxy::session_manager::SessionManager;
use crate::proxy::upstream::client::mask_email;
use axum::http::HeaderMap;
use axum::{
    body::Body,
    extract::State,
    extract::{Json, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::{json, Value};
use tracing::{debug, error, info, warn};

mod handle_generate;
mod handle_generate_error;
mod handle_list_models;
mod response_has_inline_image_data;

pub use handle_generate::handle_generate;
pub(crate) use handle_generate_error::handle_generate_error;
pub use handle_list_models::execute_count_tokens;
pub use handle_list_models::handle_count_tokens;
pub use handle_list_models::handle_get_model;
pub use handle_list_models::handle_list_models;
pub(crate) use response_has_inline_image_data::handle_generate_success;
pub(crate) use response_has_inline_image_data::image_success_tests;
pub(crate) use response_has_inline_image_data::response_has_inline_image_data;
pub(crate) use response_has_inline_image_data::ErrorOutcome;
pub(crate) use response_has_inline_image_data::HandleSuccessOutcome;
