use crate::proxy::server::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use std::collections::HashSet;
use tokio::time::{sleep, Duration};
use tracing::{debug, info};

mod handle_detect_model;
pub(crate) mod retrystrategy;

pub use handle_detect_model::build_dual_track_error;
pub use handle_detect_model::build_token_error_headers;
pub use handle_detect_model::extract_retry_after_seconds;
pub use handle_detect_model::handle_detect_model;
pub use handle_detect_model::is_model_not_found_error;
pub(crate) use handle_detect_model::map_status_code_to_gemini_status;
pub use handle_detect_model::parse_raw_upstream_error;
pub(crate) use handle_detect_model::retry_after_tests;
pub use retrystrategy::apply_retry_strategy;
pub use retrystrategy::calculate_max_retry_attempts;
pub use retrystrategy::determine_retry_strategy;
pub use retrystrategy::determine_retry_strategy_adaptive;
pub use retrystrategy::determine_retry_strategy_with_grace;
pub use retrystrategy::next_rotation_attempt;
pub use retrystrategy::should_rotate_account;
pub(crate) use retrystrategy::tests;
pub use retrystrategy::FailureStatusTracker;
pub use retrystrategy::RequestRetryState;
pub use retrystrategy::RetryStrategy;
