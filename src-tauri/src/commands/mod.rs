use crate::models::{Account, AppConfig, QuotaData};
use crate::modules;
use std::path::{Path, PathBuf};
use tauri::{Emitter, Manager};
use tauri_plugin_opener::OpenerExt;

// Export proxy command
pub mod proxy;
// Export autostart command
pub mod autostart;
// export cloudflared command
pub mod cloudflared;
// Export security command (IP monitoring)
pub mod security;
// Export proxy_pool command
pub mod proxy_pool;
// Export user_token command
pub mod user_token;
// Export patch command
pub mod patch;
pub use patch::*;
// Export instance command
pub mod instance;
pub use instance::*;
// export email command
pub mod email;
pub use email::*;
// export supabase command
pub mod supabase;
pub use supabase::*;
// Export telegram command
pub mod telegram;
pub use telegram::*;
// Export 100-improvements flywheel
pub mod improvements;
pub use improvements::*;
// Export toolchain command (Toolchain Installer UI backend)
pub mod toolchain;
pub use toolchain::*;
// Export fleet command (GitMap Fleet Sync)
pub mod fleet;
pub use fleet::*;
// Export ssh command (GitMap Multi-Machine & RSA SSH Vault)
pub mod ssh;
pub use ssh::*;
// Export doctor command (Imp-90: System Diagnostics)
pub mod doctor;
pub use doctor::*;

// Command groups extracted from the former monolithic mod.rs (each ≤500 lines)
mod cmd_accounts;
mod cmd_config;
mod cmd_data_dir;
mod cmd_device;
mod cmd_error_db;
mod cmd_files;
mod cmd_http_api;
mod cmd_import;
mod cmd_maintenance;
mod cmd_network;
mod cmd_oauth;
mod cmd_paths;
mod cmd_queue_resend;
pub(crate) mod cmd_quota;
mod cmd_running_up_next;
mod cmd_task_history;
mod cmd_token_stats;
mod cmd_training;
mod cmd_ui_trace;
mod cmd_updates;
mod cmd_window;

pub use cmd_accounts::*;
pub use cmd_config::*;
pub use cmd_data_dir::*;
pub use cmd_device::*;
pub use cmd_error_db::*;
pub use cmd_files::*;
pub use cmd_http_api::*;
pub use cmd_import::*;
pub use cmd_maintenance::*;
pub use cmd_network::*;
pub use cmd_oauth::*;
pub use cmd_paths::*;
pub use cmd_queue_resend::*;
pub use cmd_quota::*;
pub use cmd_running_up_next::*;
pub use cmd_task_history::*;
pub use cmd_token_stats::*;
pub use cmd_training::*;
pub use cmd_ui_trace::*;
pub use cmd_updates::*;
pub use cmd_window::*;
