//! Delegated Out-of-Process Updater for Antigravity-Manager
//! Implements the 3-Stage Self-Delegating CLI Update Pipeline:
//!   Stage 1: Copy CLI to isolated `%TEMP%\agm-updater\agm-update-cli-<pid>.exe` & wait for UI exit
//!   Stage 2: Execute `<agm-update-cli> update --force --no-launch --install-dir <DIR>`
//!   Stage 3: Execute `<agm-update-cli> open-ui --target-exe <EXE> --install-dir <DIR>`

pub mod args;
pub mod help;
pub mod prepare;
pub mod process;
pub mod run;
pub mod tests;
pub mod types;
pub mod ui;

pub use args::*;
pub use help::*;
pub use prepare::*;
pub use process::*;
pub use run::*;
pub use tests::*;
pub use types::*;
pub use ui::*;
