use modules::logger;
use std::sync::Arc;
use tauri::{Emitter, Manager};
use tracing::{error, info, warn};

mod commands;

pub mod constants;

pub mod error;

#[cfg(target_os = "linux")]
mod linux_graphics;

pub mod models;

pub mod modules;

mod proxy; // Proxy service module

pub mod utils;

mod appruntimeflags;
mod run;
mod setup_app;

pub(crate) use appruntimeflags::configure_linux_graphics;
pub(crate) use appruntimeflags::credential_state;
pub(crate) use appruntimeflags::env_flag_enabled;
pub use appruntimeflags::force_restore_and_focus_win32;
pub(crate) use appruntimeflags::greet;
pub(crate) use appruntimeflags::increase_nofile_limit;
pub(crate) use appruntimeflags::is_wayland_session;
pub(crate) use appruntimeflags::nvidia_proprietary_loaded;
pub use appruntimeflags::restore_and_focus_window;
pub(crate) use appruntimeflags::should_enable_tray;
#[cfg(target_os = "windows")]
pub(crate) use appruntimeflags::windows_api;
pub(crate) use appruntimeflags::AppRuntimeFlags;
pub use run::run;
pub(crate) use setup_app::handle_window_event;
pub(crate) use setup_app::run_headless;
pub(crate) use setup_app::init_databases;
pub(crate) use setup_app::setup_app;
