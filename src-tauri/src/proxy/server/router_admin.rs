//! Authenticated admin REST API route table.

use super::app_state::AppState;
use super::router_admin_a::admin_route_table_a;
use super::router_admin_b::admin_route_table_b;
use crate::proxy::middleware::admin_auth_middleware;
use axum::Router;

pub(crate) fn build_admin_routes(state: &AppState) -> Router {
    Router::new()
        .merge(admin_route_table_a())
        .merge(admin_route_table_b())
        // Apply admin-specific auth layer (mandatory verification)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            admin_auth_middleware,
        ))
}
