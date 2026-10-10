//! Axum proxy server: app state, router construction, and admin API handlers.
//! (Split from the former monolithic `server.rs`; all `crate::proxy::server::*`
//! paths keep resolving through the re-exports below.)

mod admin_accounts;
mod admin_accounts_ops;
mod admin_cloudflared;
mod admin_config;
mod admin_devices;
mod admin_import;
mod admin_instances;
mod admin_logs;
mod admin_oauth;
mod admin_prompts;
mod admin_proxy;
mod admin_security;
mod admin_stats;
mod admin_sync_external;
mod admin_sync_opencode;
mod admin_system;
mod admin_tokens;
mod app_state;
mod audit;
mod axum_server;
mod bind;
mod dto;
pub(crate) mod image_scheduler;
mod pending;
mod router_admin;
mod router_admin_a;
mod router_admin_b;
mod router_proxy;
mod start;

pub use crate::proxy::upstream::client::UpstreamClient;
pub use app_state::AppState;
pub use axum_server::AxumServer;
pub use image_scheduler::{ImagePermit, ImageScheduler};
pub use pending::{
    take_pending_delete_accounts, take_pending_reload_accounts, trigger_account_delete,
    trigger_account_reload,
};
