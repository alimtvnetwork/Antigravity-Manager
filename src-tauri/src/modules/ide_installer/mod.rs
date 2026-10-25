//! Multi-IDE Installer Subsystem Facade.
//!
//! Provides automated discovery and silent installation for Google Antigravity IDE,
//! Cursor AI, and Visual Studio Code across Windows, macOS, and Linux.
//!
//! Conforms to `02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements/03-migration-and-verification-spec.md`.

pub mod linux;
pub mod macos;
pub mod types;
pub mod verify;
pub mod windows;

pub use types::*;
pub use verify::*;

use crate::error::AppError;

/// Dispatches installation for the selected IDE target according to the host platform.
pub fn install_ide(target: IdeTarget, options: InstallOptions) -> Result<InstallResult, AppError> {
    #[cfg(target_os = "windows")]
    {
        windows::execute_windows_install(target, options)
    }
    #[cfg(target_os = "macos")]
    {
        macos::execute_macos_install(target, options)
    }
    #[cfg(target_os = "linux")]
    {
        linux::execute_linux_install(target, options)
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err(AppError::NotSupported(
            "IDE installation is not supported on this operating system".to_string(),
        ))
    }
}
