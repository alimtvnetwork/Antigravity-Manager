use crate::models::{Account, AppConfig, QuotaData};
use crate::modules;
use std::path::{Path, PathBuf};
use tauri::{Emitter, Manager};
use tauri_plugin_opener::OpenerExt;

// 导出 proxy 命令
pub mod proxy;
// 导出 autostart 命令
pub mod autostart;
// 导出 cloudflared 命令
pub mod cloudflared;
// 导出 security 命令 (IP 监控)
pub mod security;
// 导出 proxy_pool 命令
pub mod proxy_pool;
// 导出 user_token 命令
pub mod user_token;
// 导出 patch 命令
pub mod patch;
pub use patch::*;
// 导出 instance 命令
pub mod instance;
pub use instance::*;
// 导出 email 命令
pub mod email;
pub use email::*;
// 导出 supabase 命令
pub mod supabase;
pub use supabase::*;
// 导出 telegram 命令
pub mod telegram;
pub use telegram::*;
// 导出 toolchain 命令 (Toolchain Installer UI backend)
pub mod toolchain;
pub use toolchain::*;
// 导出 fleet 命令 (GitMap Fleet Sync)
pub mod fleet;
pub use fleet::*;

// Command groups extracted from the former monolithic mod.rs (each ≤500 lines)
mod cmd_accounts;
mod cmd_config;
mod cmd_data_dir;
mod cmd_device;
mod cmd_files;
mod cmd_http_api;
mod cmd_import;
mod cmd_maintenance;
mod cmd_network;
mod cmd_oauth;
mod cmd_paths;
pub(crate) mod cmd_quota;
mod cmd_task_history;
mod cmd_token_stats;
mod cmd_training;
mod cmd_updates;
mod cmd_window;

pub use cmd_accounts::*;
pub use cmd_config::*;
pub use cmd_data_dir::*;
pub use cmd_device::*;
pub use cmd_files::*;
pub use cmd_http_api::*;
pub use cmd_import::*;
pub use cmd_maintenance::*;
pub use cmd_network::*;
pub use cmd_oauth::*;
pub use cmd_paths::*;
pub use cmd_quota::*;
pub use cmd_task_history::*;
pub use cmd_token_stats::*;
pub use cmd_training::*;
pub use cmd_updates::*;
pub use cmd_window::*;
