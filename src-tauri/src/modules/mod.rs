pub mod account;
pub mod account_service;
pub mod agy_cleaner;
pub mod auto_switcher;
pub mod cache;
pub mod cli;
pub mod cloudflared;
pub mod config;
pub mod db;
pub mod device;
pub mod email_inbound;
pub mod email_io;
pub mod email_sender;
pub mod email_vault_db;
pub mod email_watcher;
pub mod instance;

#[allow(dead_code)]
pub mod http_api;
pub mod i18n;
pub mod integration;
pub mod iterative_codec;
pub mod log_bridge;
pub mod logger;
pub mod migration;
pub mod notification_hub;
pub mod oauth;
pub mod oauth_server;
pub mod process;
pub mod proxy_db;
pub mod quota;
pub mod repo_db;
pub mod scheduler;
pub mod security_db;
pub mod supabase_client;
pub mod supabase_command_queue;
pub mod supabase_pruner;
pub mod supabase_schema;
pub mod supabase_sync;
pub mod telegram_inbound;
pub mod token_stats;
pub mod tray;
pub mod update_checker;
pub mod user_token_db;
pub mod version;
pub mod workspace_lease_manager;

use crate::models;

// Re-export commonly used functions to the top level of the modules namespace for easy external calling
pub use account::*;
pub use config::*;
#[allow(unused_imports)]
pub use logger::*;
#[allow(unused_imports)]
pub use quota::*;
// pub use device::*;

pub async fn fetch_quota(
    access_token: &str,
    email: &str,
    account_id: Option<&str>,
) -> crate::error::AppResult<(models::QuotaData, Option<String>)> {
    quota::fetch_quota(access_token, email, account_id).await
}
