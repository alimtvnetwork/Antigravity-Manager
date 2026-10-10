use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

mod get_core_gateway_models;
mod get_sync_status;
mod scan_windows_cli_paths;
mod sync_config;
#[cfg(test)]
mod tests;

pub use get_core_gateway_models::get_core_gateway_models;
pub use get_core_gateway_models::sync_jeikcode_toml_content;
pub use get_sync_status::get_sync_status;
pub use scan_windows_cli_paths::check_cli_installed;
pub(crate) use scan_windows_cli_paths::detect_fallback_binary;
pub(crate) use scan_windows_cli_paths::extract_version;
#[cfg(target_os = "windows")]
pub(crate) use scan_windows_cli_paths::is_cmd_file;
#[cfg(target_os = "windows")]
pub(crate) use scan_windows_cli_paths::is_safe_path;
#[cfg(target_os = "windows")]
pub(crate) use scan_windows_cli_paths::parse_where_output;
#[cfg(target_os = "windows")]
pub(crate) use scan_windows_cli_paths::run_version_command;
pub(crate) use scan_windows_cli_paths::scan_windows_cli_paths;
pub use scan_windows_cli_paths::CliApp;
pub use scan_windows_cli_paths::CliConfigFile;
pub use scan_windows_cli_paths::CliStatus;
pub use sync_config::execute_cli_restore;
pub use sync_config::execute_cli_sync;
pub use sync_config::get_cli_config_content;
pub use sync_config::get_cli_sync_status;
pub use sync_config::sync_config;
pub use sync_config::CoreModel;
